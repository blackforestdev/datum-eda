"""Descriptor-bound, byte-preserving deduplication of approved old QA companions.

This opt-in mode proves that the selected ELF inodes cannot execute while held
open for writing. It never writes their contents, truncates, chmods, retires a
QA path, or infers that an uninspectable process is idle.
"""
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import secrets
import stat
import subprocess

import run_cargo_guarded as guard

COHORTS = {
    "startup-0a8e8b5d", "initial-prefix-050cdf8c", "manifest-open-94d59290",
    "manifest-open-ea04ce38", "terminal-handoff-ec2cd61b",
    "terminal-handoff-4b6ff272", "file-actions-fdef8fde",
}
ROLES = {"datum-eda", "eda-engine-daemon"}
READ = os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
WRITE = os.O_RDWR | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
FIELDS = ("device", "inode", "nlink", "size", "uid", "gid", "mode", "mtime_ns", "ctime_ns")


def refuse(message):
    raise guard.ResourcePolicyError(f"QA companion dedup refused: {message}")


class Failure(guard.ResourcePolicyError):
    def __init__(self, message, receipt):
        super().__init__(message)
        self.receipt = receipt


def security(descriptor):
    return {name: hashlib.sha256(os.getxattr(descriptor, name)).hexdigest()
            for name in sorted(os.listxattr(descriptor))}


def info(value):
    return dict(zip(FIELDS, (value.st_dev, value.st_ino, value.st_nlink,
        value.st_size, value.st_uid, value.st_gid, stat.S_IMODE(value.st_mode),
        value.st_mtime_ns, value.st_ctime_ns)))


def digest(descriptor):
    result, offset = hashlib.sha256(), 0
    while block := os.pread(descriptor, 1024 * 1024, offset):
        result.update(block)
        offset += len(block)
    return result.hexdigest()


def snapshot(descriptor):
    before = os.fstat(descriptor)
    value = info(before)
    value.update(sha256=digest(descriptor), xattrs_sha256=security(descriptor))
    if info(os.fstat(descriptor)) != info(before):
        refuse("file changed during inspection")
    return value


def role(path):
    parts = PurePosixPath(path).parts
    if (len(parts) != 5 or parts[:2] != ("target", "qa")
            or parts[2] not in COHORTS or parts[3] != "debug" or parts[4] not in ROLES
            or str(PurePosixPath(path)) != path):
        refuse(f"not an approved immutable QA companion path: {path}")
    return parts


def load_manifest(path, expected_sha):
    raw = Path(path).read_bytes()
    if len(expected_sha) != 64 or hashlib.sha256(raw).hexdigest() != expected_sha:
        refuse("exact manifest SHA mismatch")
    value = json.loads(raw)
    if value.get("schema_version") != 1 or value.get("operation") != "deduplicate_immutable_qa_binaries":
        refuse("unknown manifest operation")
    groups = value.get("groups")
    if not isinstance(groups, list) or not 1 <= len(groups) <= 2:
        refuse("one or two exact companion groups are required")
    paths, roles = set(), set()
    for group in groups:
        anchor, duplicates = group["anchor"], group["duplicates"]
        if not isinstance(duplicates, list) or not duplicates:
            refuse("empty duplicate group")
        name = role(anchor["path"])[-1]
        if name in roles:
            refuse("duplicate companion group")
        roles.add(name)
        for entry in [anchor, *duplicates]:
            if role(entry["path"])[-1] != name or entry["path"] in paths:
                refuse("mixed roles or repeated selected paths")
            paths.add(entry["path"])
            if any(key not in entry for key in [*FIELDS, "sha256", "xattrs_sha256"]):
                refuse("incomplete positive file metadata")
            if any(entry[key] != anchor[key] for key in
                    ("device", "size", "uid", "gid", "mode", "mtime_ns", "sha256", "xattrs_sha256")):
                refuse("group bytes, filesystem, timestamps or security metadata differ")
    return groups, paths


def directory(root, parts, stack):
    descriptor = os.open(root, READ | os.O_DIRECTORY)
    stack.callback(os.close, descriptor)
    chain = []
    for part in [None, *parts]:
        if part is not None:
            descriptor = os.open(part, READ | os.O_DIRECTORY, dir_fd=descriptor)
            stack.callback(os.close, descriptor)
        value = os.fstat(descriptor)
        chain.append((value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid))
    return descriptor, chain


class Bound:
    def __init__(self, root, entry, stack):
        self.root, self.expected = root, entry
        self.parts = role(entry["path"])
        self.parent, self.chain = directory(root, self.parts[:-1], stack)
        self.name = self.parts[-1]
        before = os.stat(self.name, dir_fd=self.parent, follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or before.st_uid != os.getuid() or not before.st_mode & 0o111:
            refuse("selected file is not an owned executable regular file")
        # ETXTBSY rejects an executing image, including a renamed image whose
        # /proc executable cannot be inspected. Retain every descriptor to keep
        # subsequent execve blocked until commit and postverification finish.
        self.descriptor = os.open(self.name, WRITE, dir_fd=self.parent)
        stack.callback(os.close, self.descriptor)
        if info(os.fstat(self.descriptor)) != info(before):
            refuse("selected inode changed while opening")
        header = os.pread(self.descriptor, 18, 0)
        if (len(header) != 18 or header[:4] != b"\x7fELF" or header[4] not in (1, 2)
                or header[5] not in (1, 2) or int.from_bytes(header[16:18],
                    "little" if header[5] == 1 else "big") not in (2, 3)):
            refuse("selected image is not an ELF executable")
        self.links, self.ctime = entry["nlink"], entry["ctime_ns"]
        self.verify()

    def verify_parent(self):
        with ExitStack() as check:
            _, chain = directory(self.root, self.parts[:-1], check)
            if chain != self.chain:
                refuse("named root/parent directory identity changed")

    def verify(self, *, named=True):
        self.verify_parent()
        actual = snapshot(self.descriptor)
        expected = {key: self.expected[key] for key in [*FIELDS, "sha256", "xattrs_sha256"]}
        expected.update(nlink=self.links, ctime_ns=self.ctime)
        if actual != expected:
            refuse(f"selected metadata or bytes changed: {self.expected['path']}")
        if named and info(os.stat(self.name, dir_fd=self.parent, follow_symlinks=False)) != info(os.fstat(self.descriptor)):
            refuse("selected named inode changed")
        return actual

    def link_changed(self, delta):
        value = os.fstat(self.descriptor)
        if value.st_nlink != self.links + delta:
            refuse("unexpected selected inode link count")
        self.links += delta
        self.ctime = value.st_ctime_ns
        self.verify(named=False)


def nonselected_inventory(root, cohorts, excluded):
    result = {}

    def walk(descriptor, prefix):
        value = os.fstat(descriptor)
        result[prefix] = (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid)
        for name in sorted(os.listdir(descriptor)):
            path = f"{prefix}/{name}"
            if path in excluded:
                continue
            value = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
            if stat.S_ISDIR(value.st_mode):
                child = os.open(name, READ | os.O_DIRECTORY, dir_fd=descriptor)
                try:
                    walk(child, path)
                finally:
                    os.close(child)
            elif stat.S_ISREG(value.st_mode):
                child = os.open(name, READ, dir_fd=descriptor)
                try:
                    result[path] = snapshot(child)
                finally:
                    os.close(child)
            elif stat.S_ISLNK(value.st_mode):
                result[path] = (info(value), os.readlink(name, dir_fd=descriptor))
            else:
                refuse("unknown nonselected QA file type")

    with ExitStack() as stack:
        for cohort in sorted(cohorts):
            parts = ("target", "qa", cohort)
            descriptor, _ = directory(root, parts, stack)
            walk(descriptor, "/".join(parts))
    return result


def inventory_sha(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def deduplicate(root, manifest, manifest_sha, lock_path, active_check, *, apply=False):
    try:
        groups, selected = load_manifest(manifest, manifest_sha)
    except (ValueError, KeyError, TypeError) as error:
        refuse(f"malformed manifest: {error}")
    root = Path(root).resolve()
    cohorts = {role(path)[2] for path in selected}
    receipt = {"manifest_sha256": manifest_sha, "apply": apply, "completed": [],
               "status": "preflight", "kernel_proof": "selected ELF write descriptors; no process-idle assertion"}
    bound = {}
    with guard.acquire_lock(lock_path, 0), ExitStack() as stack:
        temporary = []
        try:
            def idle():
                if active := active_check():
                    refuse("active Cargo/Datum process: " + ", ".join(active))

            idle()
            tracked = subprocess.run(["git", "ls-files", "-z", "--", *sorted(selected)],
                cwd=root, capture_output=True, check=False)
            if tracked.returncode or tracked.stdout:
                refuse("tracked or unverifiable selected path")
            # Validate and hold the complete batch before creating any link.
            bound = {entry["path"]: Bound(root, entry, stack) for group in groups
                     for entry in [group["anchor"], *group["duplicates"]]}
            if len({(item.expected["device"], item.expected["inode"]) for item in bound.values()}) != len(bound):
                refuse("manifest must select distinct pre-deduplication inodes")
            receipt["before"] = {path: item.verify() for path, item in bound.items()}
            before = nonselected_inventory(root, cohorts, selected)
            receipt["nonselected_before_sha256"] = inventory_sha(before)
            receipt["nonselected_entries"] = len(before)
            idle()
            if apply:
                receipt["phase"] = "staging"
                for group in groups:
                    anchor = bound[group["anchor"]["path"]]
                    for entry in group["duplicates"]:
                        target = bound[entry["path"]]
                        anchor.verify(); target.verify()
                        name = ".datum-qa-dedup-" + secrets.token_hex(16)
                        os.link(anchor.name, name, src_dir_fd=anchor.parent,
                            dst_dir_fd=target.parent, follow_symlinks=False)
                        temporary.append((target, name, anchor))
                        anchor.link_changed(1)
                # Stage every replacement, then prove all names and exact
                # anchor inodes again before the first atomic path swap.
                for target, name, anchor in temporary:
                    target.verify(); anchor.verify()
                    if info(os.stat(name, dir_fd=target.parent, follow_symlinks=False)) != info(os.fstat(anchor.descriptor)):
                        refuse("staged hardlink inode differs from held anchor")
                idle()
                receipt["phase"] = "atomic_path_replacement"
                for target, name, anchor in temporary:
                    target.verify(); anchor.verify()
                    if info(os.stat(name, dir_fd=target.parent, follow_symlinks=False)) != info(os.fstat(anchor.descriptor)):
                        refuse("staged hardlink changed before replacement")
                    os.replace(name, target.name, src_dir_fd=target.parent, dst_dir_fd=target.parent)
                    receipt["completed"].append(target.expected["path"])
                    # Renaming a hardlink can update the shared anchor's ctime
                    # even though its link count and contents stay unchanged.
                    anchor.link_changed(0)
                    target.link_changed(-1)
                receipt["status"] = "paths_replaced_pending_verification"
            else:
                receipt["status"] = "dry_run_validated_no_mutation"
            receipt["phase"] = "postverification"
            after = {}
            for group in groups:
                anchor = bound[group["anchor"]["path"]]
                for entry in [group["anchor"], *group["duplicates"]]:
                    target = bound[entry["path"]]
                    target.verify_parent()
                    descriptor = os.open(target.name, READ, dir_fd=target.parent)
                    try:
                        actual = snapshot(descriptor)
                    finally:
                        os.close(descriptor)
                    wanted = anchor.verify() if apply else target.verify()
                    if actual != wanted:
                        refuse("postverification differs from exact held inode/bytes")
                    after[entry["path"]] = actual
            receipt["after"] = after
            unchanged = nonselected_inventory(root, cohorts, selected)
            receipt["nonselected_after_sha256"] = inventory_sha(unchanged)
            if unchanged != before:
                refuse("nonselected QA inventory changed")
            if apply:
                receipt["status"] = "complete_all_paths_and_bytes_retained"
        except (OSError, ValueError, KeyError, TypeError, guard.ResourcePolicyError) as error:
            receipt.update(status="refused_or_partial", error=str(error))
            # An atomic replacement can succeed even when a later syscall or
            # injected failure prevents its acknowledgement. Record actual
            # named identities as well as the completed-call list.
            observed = {}
            known = {(item.expected["device"], item.expected["inode"]) for item in bound.values()}
            for path, item in bound.items():
                try:
                    descriptor = os.open(item.name, READ, dir_fd=item.parent)
                    try:
                        value = os.fstat(descriptor)
                        observed[path] = (snapshot(descriptor) if (value.st_dev, value.st_ino) in known
                                          else {"unexpected_inode": info(value)})
                    finally:
                        os.close(descriptor)
                except (OSError, guard.ResourcePolicyError) as failure:
                    observed[path] = {"inspection_error": str(failure)}
            receipt["observed_after_error"] = observed
            raise Failure(str(error), receipt) from error
        finally:
            # Only our positively identified temporary links may be removed.
            # A committed replacement has no temporary name and is retained.
            for target, name, anchor in temporary:
                try:
                    value = os.stat(name, dir_fd=target.parent, follow_symlinks=False)
                    held = os.fstat(anchor.descriptor)
                    if (value.st_dev, value.st_ino) == (held.st_dev, held.st_ino):
                        os.unlink(name, dir_fd=target.parent)
                        anchor.link_changed(-1)
                    else:
                        receipt.setdefault("unremoved_foreign_temporary_paths", []).append(name)
                except FileNotFoundError:
                    pass
                except (OSError, guard.ResourcePolicyError) as error:
                    receipt.setdefault("cleanup_errors", []).append(str(error))
    if receipt.get("cleanup_errors") or receipt.get("unremoved_foreign_temporary_paths"):
        receipt["status"] = "refused_or_partial"
        raise Failure("temporary link cleanup requires inspection", receipt)
    return receipt
