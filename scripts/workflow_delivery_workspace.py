"""Classify local workspace state; never choose authority or permit source edits.

Callers must supply an owner-pinned policy and establish source-only startup
before repository imports. Every tracked path and every unrecognized/changed
file remains an input. Classification does not remove files from state capture.
"""

import copy
import hashlib
import importlib.util
import marshal
import os
from pathlib import Path
import re
import stat
import sys

from workflow_delivery_io import normalized_path
from workflow_delivery_shapes import closed, require


MAX_CACHE_BYTES = 8 * 1024 * 1024
BEADS_FILES = frozenset({
    ".beads/beads.db", ".beads/beads.db-wal", ".beads/beads.db-shm",
    ".beads/.sync.lock", ".beads/.write.lock", ".beads/last-touched",
})
HISTORY = re.compile(r"\.beads/\.br_history/issues\.[0-9]{8}_[0-9]{6}_[0-9]+\.jsonl(?:\.meta\.json)?\Z")
CACHE = re.compile(r"[^/]+\.cpython-[0-9]+(?:\.opt-[12])?\.pyc\Z")


def workspace_policy_shape(value):
    closed(value, "schema_version kind local_files python_caches beads_runtime", "workspace policy")
    require(type(value["schema_version"]) is int and value["schema_version"] == 1
            and value["kind"] == "datum.workflow-delivery.workspace-inputs",
            "workspace policy", "workspace-inputs version 1 required", "WDQ-WORKSPACE")
    require(value["python_caches"] in ("none", "verified-source-v1")
            and value["beads_runtime"] in ("none", "standard-v1"),
            "workspace policy", "explicit known runtime classification required", "WDQ-WORKSPACE")
    require(type(value["local_files"]) is list, "workspace policy", "local_files must be a list", "WDQ-WORKSPACE")
    seen = set()
    for row in value["local_files"]:
        closed(row, "path sha256 size mode category", "workspace local file")
        path = row["path"]
        normalized_path(path)
        require(".git" not in path.split("/") and not path.startswith(".beads/")
                and path not in seen, path, "unique non-metadata local file required", "WDQ-WORKSPACE")
        require(type(row["sha256"]) is str and re.fullmatch(r"[a-f0-9]{64}", row["sha256"])
                and type(row["size"]) is int and row["size"] >= 0
                and type(row["mode"]) is int and 0 <= row["mode"] <= 0o777,
                path, "exact SHA-256, size and permission mode required", "WDQ-WORKSPACE")
        require(row["category"] in ("owner_local", "legacy_python_cache"),
                path, "known local-file category required", "WDQ-WORKSPACE")
        if row["category"] == "legacy_python_cache":
            require(cache_source_name(path) is not None and not row["mode"] & 0o111,
                    path, "non-executable standard Python cache path required", "WDQ-WORKSPACE")
        seen.add(path)
    return value


def cache_source_name(path):
    value = Path(path)
    if value.parent.name != "__pycache__" or CACHE.fullmatch(value.name) is None:
        return None
    try:
        return importlib.util.source_from_cache(path)
    except ValueError:
        return None


def regular_file(root, name):
    normalized_path(name)
    if ".git" in name.split("/"):
        return None
    path = root / name
    try:
        # Refuse both final-component and directory redirects, even within root.
        if path.resolve(strict=True) != path:
            return None
        info = path.lstat()
        return (path, info) if stat.S_ISREG(info.st_mode) else None
    except (OSError, RuntimeError):
        return None


def read_stable(file, *, limit):
    path, before = file
    if before.st_size > limit:
        return None
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, "rb") as stream:
            opened = os.fstat(stream.fileno())
            raw = stream.read(limit + 1)
            after = os.fstat(stream.fileno())
        identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_mode)
        if identity(before) != identity(opened) or identity(opened) != identity(after):
            return None
        if path.resolve(strict=True) != path or identity(path.lstat()) != identity(after):
            return None
        return raw if len(raw) == before.st_size and len(raw) <= limit else None
    except (OSError, RuntimeError):
        return None


def genuinely_absent(root, name):
    path = root / name
    try:
        path.lstat()
    except FileNotFoundError:
        try:
            return path.resolve(strict=False) == path
        except (OSError, RuntimeError):
            return False
    except (OSError, RuntimeError):
        pass
    return False


class WorkspaceInputs:
    def __init__(self, root, policy, tracked_paths, *, cache_isolated=False):
        self.root = Path(root).resolve(strict=True)
        self.policy = copy.deepcopy(workspace_policy_shape(policy))
        self.tracked = frozenset(tracked_paths)
        self.local = {row["path"]: row for row in self.policy["local_files"]}
        self.cache_isolated = cache_isolated is True

    def input_paths(self, names, *, root, tracked_paths, explicit_paths=(), captured_files=None,
                    proof=False):
        """Filter indirect enumeration only, against the caller's captured view."""
        require(Path(root).resolve(strict=True) == self.root
                and set(tracked_paths) <= self.tracked,
                "workspace", "classifier must cover this root and every tracked input", "WDQ-WORKSPACE")
        self.require_pinned_presence(captured_files=captured_files)
        names = set(names)
        explicit = set(explicit_paths)
        inputs = set()
        for name in names:
            category = self.classify(name, captured_files=captured_files)
            if (name in explicit or category == "input" or (proof and (
                    category != "python_cache" or cache_source_name(name) not in names))):
                inputs.add(name)
        return inputs

    def require_pinned_presence(self, *, captured_files=None):
        # Enumeration omits deleted files. An exact local baseline must not
        # silently become optional merely because a pin no longer appears.
        for name, row in self.local.items():
            if row["category"] == "legacy_python_cache" and genuinely_absent(self.root, name):
                continue
            require(regular_file(self.root, name) is not None, name,
                    "required pinned local file is missing or redirected", "WDQ-WORKSPACE")
            if captured_files is not None:
                require(captured_files.get(name, {}).get("kind") == "file", name,
                        "required pinned local file is absent from captured state", "WDQ-WORKSPACE")

    def _captured(self, name, file, raw, captured_files):
        if captured_files is None:
            return True
        return raw is not None and captured_files.get(name) == {
            "kind": "file", "mode": stat.S_IMODE(file[1].st_mode),
            "size": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}

    def classify(self, name, *, captured_files=None):
        """Return input/runtime category; never grant mutation or suppress capture."""
        if name in self.tracked:
            return "input"
        file = regular_file(self.root, name)
        if file is None:
            return "input"
        mode = stat.S_IMODE(file[1].st_mode)
        if self.policy["beads_runtime"] == "standard-v1" and (
                name in BEADS_FILES or HISTORY.fullmatch(name)):
            matches = captured_files is None or self._captured(
                name, file, read_stable(file, limit=file[1].st_size), captured_files)
            return "beads_runtime" if not mode & 0o111 and matches else "input"
        row = self.local.get(name)
        if row is not None:
            if row["category"] == "legacy_python_cache" and not self.cache_isolated:
                return "input"
            raw = read_stable(file, limit=row["size"])
            if (raw is not None and len(raw) == row["size"] and mode == row["mode"]
                    and self._captured(name, file, raw, captured_files)
                    and hashlib.sha256(raw).hexdigest() == row["sha256"]):
                return row["category"]
            if (row["category"] == "legacy_python_cache" and not mode & 0o111
                    and self.policy["python_caches"] == "verified-source-v1"
                    and self._verified_cache(name, file, captured_files)):
                return "python_cache"
            return "input"  # Owner files never acquire a derived-cache exception.
        if (self.cache_isolated and self.policy["python_caches"] == "verified-source-v1"
                and not mode & 0o111 and self._verified_cache(name, file, captured_files)):
            return "python_cache"
        return "input"

    def _verified_cache(self, name, file, captured_files):
        source_name = cache_source_name(name)
        if source_name is None or source_name not in self.tracked:
            return False
        prefix = Path(source_name).stem + "." + sys.implementation.cache_tag
        if Path(name).name not in {prefix + suffix for suffix in (".pyc", ".opt-1.pyc", ".opt-2.pyc")}:
            return False
        source = regular_file(self.root, source_name)
        if source is None:
            return False
        raw = read_stable(file, limit=MAX_CACHE_BYTES)
        content = read_stable(source, limit=MAX_CACHE_BYTES)
        if (raw is None or content is None or len(raw) < 16
                or not self._captured(name, file, raw, captured_files)
                or not self._captured(source_name, source, content, captured_files)
                or raw[:4] != importlib.util.MAGIC_NUMBER
                or int.from_bytes(raw[4:8], "little") not in (0, 1, 3)):
            return False
        optimize = 2 if ".opt-2." in name else 1 if ".opt-1." in name else 0
        try:
            # Compile only declared source. Never marshal.loads or execute a cache.
            for filename in (str(source[0]), source_name, Path(source_name).name):
                code = compile(content, filename, "exec", dont_inherit=True, optimize=optimize)
                if raw[16:] == marshal.dumps(code):
                    return True
        except (ValueError, SyntaxError, MemoryError, RecursionError):
            pass
        return False
