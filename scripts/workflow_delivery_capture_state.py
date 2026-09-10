"""Read protected fixture state without repairing files, index, refs or trust."""

import os
from pathlib import Path
import stat
import subprocess

from workflow_delivery_io import normalized_path, sha256

TRUST_KEYS = ("core.hooksPath", "datum.workflowDeliveryAuthorityRef",
              "datum.workflowDeliveryBaseRef", "datum.workflowDeliveryEnvironmentPath",
              "datum.workflowDeliveryRunnerPath")


def git(root, *args, allowed=(0,)):
    environment = {"PATH": os.environ.get("PATH", os.defpath), "LC_ALL": "C",
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                   "GIT_GRAFT_FILE": os.devnull}
    result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks", *args], cwd=root,
                            env=environment, capture_output=True, check=False)
    if result.returncode not in allowed:
        raise RuntimeError(f"state capture git {args[0]} failed: {result.returncode}")
    return result.stdout


def file_state(path):
    try:
        info = path.lstat()
    except FileNotFoundError:
        return {"kind": "missing"}
    mode = stat.S_IMODE(info.st_mode)
    if stat.S_ISLNK(info.st_mode):
        return {"kind": "symlink", "mode": mode, "target": os.readlink(path)}
    if stat.S_ISREG(info.st_mode):
        raw = path.read_bytes()
        return {"kind": "file", "mode": mode, "size": len(raw), "sha256": sha256(raw)}
    if stat.S_ISDIR(info.st_mode):
        return {"kind": "directory", "mode": mode}
    raise ValueError(f"unsupported special file in protected state: {path}")


def protected_path(root, name):
    normalized_path(name)
    if ".git" in Path(name).parts:
        raise ValueError("Git metadata is not a protected source root")
    path = root / name
    for parent in path.parents:
        if parent == root:
            break
        if parent.is_symlink():
            raise ValueError(f"protected path traverses a directory symlink: {name}")
    return path


def protected_state(root, untracked_roots):
    root = Path(root).resolve(strict=True)
    if git(root, "rev-parse", "--show-toplevel").decode().strip() != str(root):
        raise ValueError("capture requires the exact fixture repository root")
    entries = git(root, "ls-files", "--stage", "-z")
    names = {row.split(b"\t", 1)[1].decode() for row in entries.split(b"\0") if row}
    for name in untracked_roots:
        location = protected_path(root, name)
        names.add(name)
        if location.is_dir() and not location.is_symlink():
            def inaccessible(error):
                raise error
            for directory, dirs, files in os.walk(location, followlinks=False, onerror=inaccessible):
                if ".git" in dirs or ".git" in files:
                    raise ValueError("nested Git metadata is not a protected source input")
                for child in dirs + files:
                    names.add((Path(directory) / child).relative_to(root).as_posix())
    for name in names:
        # Never follow a directory symlink into host files or another worktree.
        protected_path(root, name)
    index_path = Path(git(root, "rev-parse", "--path-format=absolute", "--git-path", "index").decode().strip())
    exclude_path = Path(git(root, "rev-parse", "--path-format=absolute", "--git-path", "info/exclude").decode().strip())
    return {
        "schema_version": 1, "root": str(root),
        "files": {name: file_state(root / name) for name in sorted(names)},
        "untracked_roots": sorted(untracked_roots),
        "index": file_state(index_path),
        "git_info_exclude": file_state(exclude_path),
        "index_entries_hex": entries.hex(),
        "head": git(root, "rev-parse", "--verify", "HEAD").decode().strip(),
        "symbolic_head": git(root, "symbolic-ref", "-q", "HEAD", allowed=(0, 1)).decode().strip(),
        "refs": git(root, "for-each-ref", "--format=%(refname) %(objectname)").decode().splitlines(),
        "local_trust": {key: git(root, "config", "--local", "--no-includes", "--null",
                                  "--get-all", key, allowed=(0, 1)).decode().split("\0")[:-1]
                        for key in TRUST_KEYS},
        "scope": "tracked paths, explicit untracked roots, index, refs, local Git exclusions and named local trust only",
    }
