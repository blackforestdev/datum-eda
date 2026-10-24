"""Positive, descriptor-bound compiler-metadata inventory for incremental cleanup.

This proves that the selected files are non-runnable compiler metadata. It does
not establish that an uninspectable process is idle. Callers retain the shared
build lock, active build/application checks, and normal path/tracking checks.
"""
from contextlib import ExitStack
import hashlib
import os
from pathlib import Path, PurePosixPath
import re
import stat

import run_cargo_guarded as guard

ALLOWED = {"target/debug/incremental", "target/release/incremental"}
BINS = {"query-cache.bin", "dep-graph.bin", "dep-graph.part.bin", "work-products.bin"}
CRATE = re.compile(r"[A-Za-z0-9_]+-[a-z0-9]+\Z")
SESSION = re.compile(r"s-[a-z0-9]+-[a-z0-9]+-[a-z0-9]+\Z")
LOCK = re.compile(r"s-[a-z0-9]+-[a-z0-9]+\.lock\Z")
FLAGS = os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC


def identity(value: os.stat_result) -> tuple:
    return (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid,
            value.st_nlink, value.st_size, value.st_mtime_ns, value.st_ctime_ns)


def core(value: os.stat_result) -> tuple:
    return identity(value)[:5]


def refuse(detail: str) -> None:
    raise guard.ResourcePolicyError(f"incremental metadata-only cleanup refused: {detail}")


class MetadataTree:
    """Keep the validated directory descriptors open through inventory/removal."""
    def __init__(self, root: Path, relative: str):
        if relative not in ALLOWED:
            refuse(f"mode permits only exact incremental paths: {relative}")
        self.root, self.relative = root.resolve(), relative
        self.stack = ExitStack()
        self.chain = []
        try:
            descriptor = self._open(str(self.root), FLAGS | os.O_DIRECTORY)
            self.chain.append(core(os.fstat(descriptor)))
            for part in Path(relative).parts:
                self.parent, self.name = descriptor, part
                descriptor = self._open(part, FLAGS | os.O_DIRECTORY, dir_fd=descriptor)
                self.chain.append(core(os.fstat(descriptor)))
            self.descriptor = descriptor
        except BaseException:
            self.stack.close()
            raise

    def _open(self, *args, **kwargs) -> int:
        descriptor = os.open(*args, **kwargs)
        self.stack.callback(os.close, descriptor)
        return descriptor

    def __enter__(self):
        return self

    def __exit__(self, *exception):
        self.stack.close()

    def verify_named_root(self) -> None:
        # Reopen every component without following links. The original open
        # descriptor alone would not notice a renamed/replaced lexical tree.
        with MetadataTree(self.root, self.relative) as current:
            if current.chain != self.chain:
                refuse("selected path/directory identity changed")

    @staticmethod
    def _file(directory: int, name: str, relative: str) -> tuple:
        before = os.stat(name, dir_fd=directory, follow_symlinks=False)
        if not stat.S_ISREG(before.st_mode) or before.st_mode & 0o111:
            refuse(f"not a non-executable regular metadata file: {relative}")
        parts = PurePosixPath(relative).parts
        lock = len(parts) == 2 and LOCK.fullmatch(name)
        metadata = len(parts) == 3 and name == "metadata.rmeta"
        binary = len(parts) == 3 and name in BINS
        if not (lock or metadata or binary):
            refuse(f"unknown compiler metadata filename: {relative}")
        descriptor = os.open(name, FLAGS | os.O_NONBLOCK, dir_fd=directory)
        with os.fdopen(descriptor, "rb") as source:
            if identity(os.fstat(source.fileno())) != identity(before):
                refuse(f"metadata file changed while opening: {relative}")
            magic = source.read(8)
            if lock:
                valid = before.st_size == 0 and not magic
            elif metadata:
                valid = magic == b"rust\x00\x00\x00\x0a"
            else:
                valid = magic.startswith(b"RSIC")
            if not valid:
                refuse(f"compiler metadata header/empty-lock mismatch: {relative}")
            source.seek(0)
            digest = hashlib.file_digest(source, "sha256").hexdigest()
            if identity(os.fstat(source.fileno())) != identity(before):
                refuse(f"metadata file changed during inspection: {relative}")
        if identity(os.stat(name, dir_fd=directory, follow_symlinks=False)) != identity(before):
            refuse(f"metadata path changed during inspection: {relative}")
        return (identity(before), digest)

    def capture(self) -> dict[str, tuple]:
        inventory = {}

        def walk(directory: int, prefix: str) -> None:
            before = os.fstat(directory)
            inventory[prefix] = (identity(before), None)
            names = sorted(os.listdir(directory))
            for name in names:
                relative = f"{prefix}/{name}" if prefix else name
                entry = os.stat(name, dir_fd=directory, follow_symlinks=False)
                if stat.S_ISDIR(entry.st_mode):
                    parts = PurePosixPath(relative).parts
                    valid = (len(parts) == 1 and CRATE.fullmatch(name)) or (len(parts) == 2 and SESSION.fullmatch(name))
                    if not valid:
                        refuse(f"unknown incremental directory: {relative}")
                    child = os.open(name, FLAGS | os.O_DIRECTORY, dir_fd=directory)
                    try:
                        if identity(os.fstat(child)) != identity(entry):
                            refuse(f"directory changed while opening: {relative}")
                        walk(child, relative)
                    finally:
                        os.close(child)
                else:
                    inventory[relative] = self._file(directory, name, relative)
            if sorted(os.listdir(directory)) != names or identity(os.fstat(directory)) != identity(before):
                refuse(f"directory inventory changed during inspection: {prefix}")

        walk(self.descriptor, "")
        return inventory

    def remove_known(self, inventory: dict[str, tuple], *, link_updates: dict | None = None) -> None:
        """Unlink only inventoried entries through their bound parent descriptors.

        A late unexpected entry is never swept up by recursive tree deletion.
        Cooperating Cargo writers are excluded by the caller's build lock.
        """
        self.verify_named_root()
        link_updates = {} if link_updates is None else link_updates

        def remove(directory: int, prefix: str) -> None:
            if core(os.fstat(directory)) != inventory[prefix][0][:5]:
                refuse(f"directory identity changed before removal: {prefix}")
            children = {PurePosixPath(path).name: path for path in inventory
                        if path and str(PurePosixPath(path).parent) == (prefix or ".")}
            if sorted(os.listdir(directory)) != sorted(children):
                refuse(f"directory entries changed before removal: {prefix}")
            for name, relative in sorted(children.items()):
                expected, digest = inventory[relative]
                if digest is None:
                    child = os.open(name, FLAGS | os.O_DIRECTORY, dir_fd=directory)
                    try:
                        if identity(os.fstat(child)) != expected:
                            refuse(f"directory changed before removal: {relative}")
                        remove(child, relative)
                    finally:
                        os.close(child)
                    if core(os.stat(name, dir_fd=directory, follow_symlinks=False)) != expected[:5]:
                        refuse(f"directory path replaced before removal: {relative}")
                    os.rmdir(name, dir_fd=directory)
                else:
                    inode = expected[:2]
                    current = link_updates.get(inode, expected)
                    if self._file(directory, name, relative) != (current, digest):
                        refuse(f"metadata changed before removal: {relative}")
                    descriptor = os.open(name, FLAGS | os.O_NONBLOCK, dir_fd=directory)
                    try:
                        if identity(os.fstat(descriptor)) != current:
                            refuse(f"metadata changed immediately before unlink: {relative}")
                        os.unlink(name, dir_fd=directory)
                        after = identity(os.fstat(descriptor))
                        # Cargo legitimately hard-links unchanged metadata across
                        # sessions. Only our unlink's link-count/ctime change may
                        # carry forward to another inventoried link to that inode.
                        if after[:5] != current[:5] or after[6:8] != current[6:8] or after[5] != current[5] - 1:
                            refuse(f"metadata inode changed during unlink: {relative}")
                        link_updates[inode] = after
                    finally:
                        os.close(descriptor)
        remove(self.descriptor, "")
        if core(os.stat(self.name, dir_fd=self.parent, follow_symlinks=False)) != inventory[""][0][:5]:
            refuse("selected directory replaced before removal")
        os.rmdir(self.name, dir_fd=self.parent)
