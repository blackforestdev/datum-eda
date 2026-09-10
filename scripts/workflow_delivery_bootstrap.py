#!/usr/bin/env python3
"""Standard-library-only support verification before importing the pinned runner.

The owner hook must authenticate this file against Git before executing it.
No activation, repairs, downloads or candidate-module imports happen here.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys

HOOK_SOURCE = "scripts/workflow_delivery_owner_hook.sh"
FIXED_PYTHON = {"scripts/check_workflow_delivery.py", "scripts/project_status.py",
                "scripts/project_task_details.py", "scripts/check_evidence_traceability.py",
                "scripts/check_file_lane_ownership.py", "scripts/check_rustfmt.py"}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def git(root, *args):
    env = {"PATH": os.environ.get("PATH", os.defpath), "LC_ALL": "C",
           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "GIT_GRAFT_FILE": os.devnull}
    result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks", *args],
                            cwd=root, env=env, capture_output=True, check=False)
    require(result.returncode == 0, "pinned Git read failed: " + result.stderr.decode("utf-8", "replace"))
    return result.stdout


def runtime_paths(entries):
    return FIXED_PYTHON | {p for p in entries if p.startswith("scripts/workflow_delivery_")
                          and p.endswith(".py") and not p.endswith("_test_support.py")}


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode()


def pinned_commit(root, value):
    require(type(value) is str and re.fullmatch(r"[a-f0-9]{40}|[a-f0-9]{64}", value),
            "full pinned commit ID required")
    require(git(root, "rev-parse", "--verify", value + "^{commit}").decode().strip() == value,
            "exact commit object required")


def bundle_manifest(root, authority):
    pinned_commit(root, authority)
    entries = {}
    for row in git(root, "ls-tree", "-rz", authority).split(b"\0"):
        if row:
            meta, name = row.split(b"\t", 1)
            mode, kind, oid = meta.decode().split()
            entries[name.decode()] = (mode, kind, oid)
    sources, records = {}, []
    for source in sorted(runtime_paths(entries) | {HOOK_SOURCE}):
        require(source in entries, "authority lacks required runtime module: " + source)
        mode, kind, oid = entries[source]
        require(mode in ("100644", "100755") and kind == "blob", "regular source Git blob required")
        require(source == "scripts/" + Path(source).name, "flat runtime source required")
        destination = "owner-hooks/pre-commit" if source == HOOK_SOURCE else source
        raw = git(root, "cat-file", "blob", oid)
        sources[destination] = raw
        records.append({"path": destination, "source_path": source, "source_blob": oid,
                        "sha256": hashlib.sha256(raw).hexdigest(),
                        "mode": "0555" if source == HOOK_SOURCE else "0444"})
    return {"schema_version": 2, "kind": "datum.workflow-delivery.support-bundle",
            "authority": authority, "files": records, "activation_asserted": False,
            "scope": "pinned runtime and hook bytes; preparation is not activation"}, sources


def verify_support(root, authority):
    root = Path(root).resolve(strict=True)
    require(git(root, "rev-parse", "--show-toplevel").decode().strip() == str(root), "exact worktree root required")
    manifest, sources = bundle_manifest(root, authority)
    common = Path(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()).resolve(strict=True)
    directory = common / "datum-wdq/trusted" / authority
    require(directory.is_dir(), "support bundle absent or incomplete")
    require(directory.resolve() == directory, "support paths cannot traverse symlink redirections")
    require({p.name for p in directory.iterdir()} == {"scripts", "owner-hooks", "manifest.json"},
            "support bundle incomplete or has unexpected entries")
    for folder in ("scripts", "owner-hooks"):
        require((directory / folder).is_dir() and (directory / folder).resolve() == directory / folder,
                "support paths cannot traverse symlink redirections")
        expected = {Path(name).name for name in sources if name.startswith(folder + "/")}
        require({p.name for p in (directory / folder).iterdir()} == expected,
                "runtime module set differs; caches and unpinned imports are forbidden")
    modes = {row["path"]: int(row["mode"], 8) for row in manifest["files"]}
    for name, raw in {**sources, "manifest.json": canonical(manifest)}.items():
        path = directory / name
        require(path.is_file() and not path.is_symlink(), "regular nonredirected bundle file required")
        require(stat.S_IMODE(path.stat().st_mode) == modes.get(name, 0o444), "bundle files must remain read-only")
        require(path.read_bytes() == raw, "runner bytes differ from pinned Git authority: " + name)
    return manifest, directory


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("root", "authority-ref", "base-ref", "environment-path", "runner"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args(argv)
    try:
        require(sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode,
                "bootstrap requires -I -S -B")
        manifest, directory = verify_support(args.root, args.authority_ref)
        pinned_commit(args.root, args.base_ref)
        runner = directory / "scripts/check_workflow_delivery.py"
        require(Path(args.runner) == runner, "exact owner-pinned runner location required")
        # Preserve the existing earlier gates and their candidate-root semantics,
        # but authenticate their bytes before executing them. Preserve the actual
        # Git-selected index and any legitimate visual-lane context untouched.
        for source in ("scripts/check_file_lane_ownership.py", "scripts/check_rustfmt.py"):
            local = Path(args.root) / source
            require(local.is_file() and not local.is_symlink()
                    and local.read_bytes() == (directory / source).read_bytes(),
                    "candidate prerequisite gate differs from pinned authority: " + source)
            status = subprocess.run([sys.executable, "-I", "-S", "-B", str(local), "--staged"],
                                    cwd=args.root, check=False).returncode
            if status:
                return status
        sys.path.insert(0, str(directory / "scripts"))
        from check_workflow_delivery import main as run
        return run(["--root", args.root, "--enforce", "--staged", "--authority-ref", args.authority_ref,
                    "--base-ref", args.base_ref, "--environment-path", args.environment_path])
    except (ValueError, OSError, KeyError) as error:
        print("WDQ-TRUST: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
