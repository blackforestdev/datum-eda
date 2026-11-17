"""Exact released compiler-output retirement; never shared tree cleanup."""
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import PurePosixPath
import re
import subprocess

import cargo_cache_metadata as metadata
import cargo_qa_companions as q
import cargo_qa_retention as retention
import run_cargo_guarded as guard

OBJECT = re.compile(r"[A-Za-z0-9_]+-[0-9a-f]{16}\.[A-Za-z0-9_]+\.[0-9a-f]{16}-cgu\.[0-9]+\.rcgu\.o\Z")
EXECUTABLE = re.compile(r"[A-Za-z0-9_]+-[0-9a-f]{16}\Z")
ARCHIVE = re.compile(r"lib[A-Za-z0-9_]+-[0-9a-f]{16}\.rlib\Z")


def retire(root, manifest, expected_sha, lock_path, active_check, *, apply=False, executable=False, archive=False):
    receipt = {"manifest_sha256": expected_sha, "apply": apply, "attempted": [], "completed": []}
    try:
        if executable and archive:
            raise guard.ResourcePolicyError("select exactly one compiler artifact kind")
        raw = manifest.read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected_sha:
            raise guard.ResourcePolicyError("interrupted-codegen manifest SHA mismatch")
        value = json.loads(raw)
        operation = ("retire_released_compiler_archive_v1" if archive else
                     "retire_released_compiler_executable_v1" if executable else "retire_interrupted_codegen_v1")
        if (not isinstance(value, dict) or value.get("operation") != operation
                or value.get("released") is not True
                or not all(isinstance(value.get(key), str) and value[key].strip()
                           for key in ("owner", "session", "issue", "reason"))
                or not isinstance(value.get("files"), list) or not value["files"]):
            raise guard.ResourcePolicyError("explicit interrupted-build ownership release required")
        paths = [entry["path"] for entry in value["files"]]
        if len(set(paths)) != len(paths):
            raise guard.ResourcePolicyError("duplicate interrupted-codegen selection")
        with guard.acquire_lock(lock_path, 1), ExitStack() as stack:
            if active_check():
                raise guard.ResourcePolicyError("active Cargo/rustc prevents codegen retirement")
            held = []
            for entry in value["files"]:
                path = entry["path"]
                parts = PurePosixPath(path).parts
                if (len(parts) != 4 or parts[:3] not in (("target", "debug", "deps"), ("target", "release", "deps"))
                        or str(PurePosixPath(path)) != path
                        or not (ARCHIVE if archive else EXECUTABLE if executable else OBJECT).fullmatch(parts[-1])):
                    raise guard.ResourcePolicyError("only exact dependency codegen object paths are admitted")
                tracked = subprocess.run(["git", "ls-files", "-z", "--", path], cwd=root, capture_output=True)
                if tracked.returncode or tracked.stdout:
                    raise guard.ResourcePolicyError("tracked/unverifiable codegen selection")
                # Writable ELF admission rejects ETXTBSY even when /proc cannot
                # identify an executing process, and prevents exec until close.
                item = retention.Bound(root, path, stack, executable=executable)
                if (item.expected != {key: entry[key] for key in (*q.FIELDS, "sha256", "xattrs_sha256")}
                        or item.expected["nlink"] != 1
                        or (not executable and item.expected["mode"] & 0o111)):
                    raise guard.ResourcePolicyError("codegen bytes/identity/release link count changed")
                header = os.pread(item.fd, 64, 0)
                if archive and header[:8] != b"!<arch>\n":
                    raise guard.ResourcePolicyError("not a regular non-runnable compiler archive")
                if not executable and not archive and item.expected["size"] != 0 and not metadata.relocatable_header(header):
                    raise guard.ResourcePolicyError("not an empty interrupted or non-runnable ELF relocatable object")
                held.append(item)
            # Validate the whole selection before the first mutation. The common
            # lock and positive artifact/execution proof preserve current QA.
            for item in held:
                item.verify()
            if active_check():
                raise guard.ResourcePolicyError("build started during codegen admission")
            if apply:
                for item in held:
                    item.verify()
                    receipt["attempted"].append(item.relative)
                    os.unlink(item.name, dir_fd=item.parent)
                    item.accept_unlink(item.post_unlink())
                    receipt["completed"].append(item.relative)
            receipt["selected_bytes"] = sum(item.expected["size"] for item in held)
            receipt["status"] = "retired_exact_released_objects" if apply else "dry_run_validated_no_mutation"
        return receipt
    except (OSError, KeyError, TypeError, ValueError, guard.ResourcePolicyError) as error:
        receipt["status"] = "refused_or_partial"
        raise retention.Failure(str(error), receipt) from error
