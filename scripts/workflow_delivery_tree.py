"""Read-only candidate views. Git index/tree reads never fall back to worktree."""

import os
from pathlib import Path, PurePosixPath
import subprocess

from workflow_delivery_io import (
    DeliveryInputError, normalized_path, parse_json, read_worktree_bytes, sha256,
    worktree_path,
)
from workflow_delivery_shapes import commit_id, require


class Tree:
    def __init__(self, root, *, staged=False, revision=None):
        self.root = Path(root).resolve()
        require(not (staged and revision), "candidate", "choose index or revision", "WDQ-TRUST")
        self.staged = staged
        self.revision = self.resolve(revision) if revision else None
        selected_index = os.environ.get("GIT_INDEX_FILE")
        self.index_path = None
        if self.revision is None and selected_index is not None:
            require(bool(selected_index), "GIT_INDEX_FILE", "empty selected index path", "WDQ-INDEX")
            self.index_path = str((self.root / selected_index).resolve())
        self.entries = self._entries()
        self.workspace = None

    def git(self, *args, index_path=None):
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
                           GIT_GRAFT_FILE=os.devnull)
        if index_path is not None:
            environment["GIT_INDEX_FILE"] = index_path
        result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks", *args],
                                cwd=self.root, env=environment, capture_output=True, check=False)
        require(result.returncode == 0, "git", result.stderr.decode("utf-8", "replace").strip(),
                "WDQ-TRUST")
        return result.stdout

    def resolve(self, revision):
        require(type(revision) is str and revision and not revision.startswith("-"),
                "revision", "explicit Git revision required", "WDQ-TRUST")
        return self.git("rev-parse", "--verify", "--end-of-options",
                        revision + "^{commit}").decode().strip()

    def has_commit(self, revision):
        commit_id(revision, "source_commit")
        require(self.resolve(revision) == revision, "source_commit",
                "full resolving commit required", "WDQ-STALE")

    def _entries(self):
        entries = {}
        if self.revision:
            raw = self.git("ls-tree", "-rz", self.revision)
            for record in raw.split(b"\0"):
                if record:
                    meta, path = record.split(b"\t", 1)
                    mode, kind, oid = meta.decode().split()
                    entries[path.decode("utf-8")] = (mode, oid)
        else:
            # Honor the actual selected index only for index enumeration. It
            # cannot redirect pinned authority/history reads or later replace
            # the already captured entry-to-blob mapping.
            for record in self.git("ls-files", "--stage", "-z", index_path=self.index_path).split(b"\0"):
                if record:
                    meta, path = record.split(b"\t", 1)
                    mode, oid, stage = meta.decode().split()
                    require(stage == "0", path.decode("utf-8"),
                            "unmerged index cannot identify candidate", "WDQ-INDEX")
                    entries[path.decode("utf-8")] = (mode, oid)
        return entries

    def read(self, path, *, committed=False):
        normalized_path(path)
        if committed:
            require(path in self.entries, path, "required artifact is not in candidate Git tree",
                    "WDQ-INDEX" if self.staged else "WDQ-ARTIFACT")
        if not self.staged and not self.revision:
            return read_worktree_bytes(self.root, path)
        return self._read_git(path, set())

    def _read_git(self, path, seen):
        require(path not in seen, path, "cyclic symlink", "WDQ-ARTIFACT")
        seen.add(path)
        require(path in self.entries, path, "reference absent from candidate tree",
                "WDQ-INDEX" if self.staged else "WDQ-ARTIFACT")
        mode, oid = self.entries[path]
        require(mode in ("100644", "100755", "120000"), path,
                "regular file or contained symlink required", "WDQ-ARTIFACT")
        data = self.git("cat-file", "blob", oid)
        if mode != "120000":
            return data
        try:
            target = data.decode("utf-8")
        except UnicodeError as error:
            raise DeliveryInputError("WDQ-ARTIFACT", path, "invalid symlink encoding") from error
        require(not PurePosixPath(target).is_absolute(), path,
                "absolute symlink refused", "WDQ-ARTIFACT")
        parts = list(PurePosixPath(path).parent.parts)
        for part in target.split("/"):
            if part == "..":
                require(bool(parts), path, "symlink escapes repository", "WDQ-ARTIFACT")
                parts.pop()
            elif part not in ("", "."):
                parts.append(part)
        resolved = "/".join(parts)
        normalized_path(resolved)
        return self._read_git(resolved, seen)

    def json(self, path, *, committed=False):
        return parse_json(self.read(path, committed=committed), path)

    def manifest(self, roots, *, workspace=None):
        workspace = self.workspace if workspace is None else workspace
        paths = set()
        for root in roots:
            normalized_path(root)
            if self.staged or self.revision:
                members = {p for p in self.entries if p == root or p.startswith(root + "/")}
                require(bool(members), root, "input root absent from candidate", "WDQ-STALE")
                paths.update(members)
            else:
                location = worktree_path(self.root, root)
                if location.is_file():
                    paths.add(root)
                else:
                    # Include ignored/untracked generated inputs too. Never walk .git
                    # or follow directory symlinks into unreviewed trees silently.
                    def inaccessible(error):
                        raise DeliveryInputError("WDQ-ARTIFACT", root, str(error)) from error
                    for directory, dirs, files in os.walk(location, followlinks=False, onerror=inaccessible):
                        require(".git" not in dirs, root, "Git metadata is not a build input",
                                "WDQ-ARTIFACT")
                        for name in dirs:
                            require(not (Path(directory) / name).is_symlink(), root,
                                    "directory symlink requires explicit resolved input root",
                                    "WDQ-ARTIFACT")
                        paths.update((Path(directory) / name).relative_to(self.root).as_posix()
                                     for name in files)
        if workspace is not None and not (self.staged or self.revision):
            paths = workspace.input_paths(paths, root=self.root, tracked_paths=self.entries,
                                          explicit_paths=roots, proof=True)
        return [{"path": p, "sha256": sha256(self.read(p))} for p in sorted(paths)]
