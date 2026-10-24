#!/usr/bin/env python3
"""Report Cargo cache usage; remove explicitly selected disposable build trees."""

from __future__ import annotations

import argparse
from contextlib import ExitStack
import cargo_cache_metadata as metadata
import os
from pathlib import Path
import shutil
import subprocess
import sys

import run_cargo_guarded as guard


ROOT = guard.ROOT
ACTIVE_NAMES = {"cargo", "rustc", "datum-gui", "datum-gui-app", "datum-eda", "eda-engine-daem", "eda-engine-daemon"}


def active_processes(proc_root: Path = Path("/proc")) -> list[str]:
    active: list[str] = []
    for process in proc_root.iterdir():
        if not process.name.isdigit():
            continue
        try:
            name = (process / "comm").read_text().strip()
        except FileNotFoundError:
            continue  # Process exited during enumeration.
        if name in ACTIVE_NAMES:
            active.append(f"{process.name} ({name})")
    return active


def validate_cleanup_path(root: Path, relative: str) -> Path:
    root = root.resolve()
    lexical = Path(relative)
    if lexical.is_absolute() or ".." in lexical.parts:
        raise guard.ResourcePolicyError(f"expected a repository-relative cache path: {relative}")
    path = root / lexical
    if path.is_symlink() or not path.is_dir() or path.resolve() != path:
        raise guard.ResourcePolicyError(f"cache must be an existing directory without symlink parents: {relative}")
    archived = path.parent == root / ".datum" and path.name.startswith("agent-cargo-archive-")
    incremental = path in {
        root / "target/debug/incremental", root / "target/release/incremental",
    }
    separate_target = (
        path.parent == root / "target"
        and path.name not in {"debug", "release"}
        and (path / ".rustc_info.json").is_file()
        and any((path / profile / ".cargo-lock").is_file() for profile in ("debug", "release"))
    )
    if not (archived or separate_target or incremental):
        raise guard.ResourcePolicyError(
            f"not an archived or separate Cargo build tree: {relative}; shared target/, "
            "debug/, release/, fixtures and evidence are protected; their incremental/ "
            "subdirectories may be explicitly selected"
        )
    if separate_target:
        cargo_layout = {"debug", "release", "tmp", ".rustc_info.json", "CACHEDIR.TAG"}
        if any(entry.name not in cargo_layout or entry.is_symlink() for entry in path.iterdir()):
            raise guard.ResourcePolicyError(f"mixed-content target requires separate ownership review: {relative}")
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", relative], cwd=root, capture_output=True, check=False,
    )
    if result.returncode or result.stdout:
        raise guard.ResourcePolicyError(f"cannot clean tracked or unverifiable path: {relative}")
    return path


def active_cache_users(paths: list[Path], proc_root: Path, *, uninspectable: list[str] | None = None) -> list[str]:
    """Also protect a selected tree running an executable with an unknown name."""
    users: list[str] = []
    for process in proc_root.iterdir():
        if not process.name.isdigit():
            continue
        try:
            executable = (process / "exe").readlink()
        except FileNotFoundError:
            continue
        except PermissionError:
            # Other system users cannot be our development sessions; same-user
            # inspection failures refuse deletion instead of assuming inactivity.
            if process.stat().st_uid != os.getuid():
                continue
            if uninspectable is not None:
                uninspectable.append(process.name)
                continue
            raise
        if any(executable == path or path in executable.parents for path in paths):
            users.append(f"{process.name} ({executable})")
    return users


def clean_caches(
    root: Path, relative_paths: list[str], lock_path: Path, proc_root: Path = Path("/proc"),
    *, metadata_only: bool = False, apply: bool = True,
) -> None:
    # Share the build guard's lock and fail immediately rather than waiting to
    # delete a cache whose ownership may have changed while this command waited.
    with guard.acquire_lock(lock_path, 0):
        active = active_processes(proc_root)
        if active:
            raise guard.ResourcePolicyError("cleanup refused while builds/apps are active: " + ", ".join(active))
        paths = [validate_cleanup_path(root, relative) for relative in relative_paths]
        if metadata_only:
            clean_incremental_metadata(root, relative_paths, paths, proc_root, apply=apply)
            return
        users = active_cache_users(paths, proc_root)
        if users:
            raise guard.ResourcePolicyError("cleanup refused for active cache executables: " + ", ".join(users))
        for path in dict.fromkeys(paths):
            shutil.rmtree(path)
            print(f"Removed {path.relative_to(root.resolve())}")


def clean_incremental_metadata(root: Path, relatives: list[str], paths: list[Path], proc_root: Path, *, apply: bool) -> None:
    # Called only under the same build lock and normal path/tracking checks.
    # Validate the complete batch before considering any process exception.
    if not relatives or any(relative not in metadata.ALLOWED for relative in relatives):
        raise guard.ResourcePolicyError("metadata-only mode requires exclusively exact incremental paths")
    with ExitStack() as stack:
        trees = [stack.enter_context(metadata.MetadataTree(root, relative)) for relative in dict.fromkeys(relatives)]
        snapshots = [tree.capture() for tree in trees]
        uninspectable = []
        users = active_cache_users(paths, proc_root, uninspectable=uninspectable)
        if users:
            raise guard.ResourcePolicyError("cleanup refused for active cache executables: " + ", ".join(users))
        for tree, snapshot in zip(trees, snapshots):
            tree.verify_named_root()
            if tree.capture() != snapshot:
                raise guard.ResourcePolicyError("incremental metadata inventory changed before deletion")
        active = active_processes(proc_root)
        if active:
            raise guard.ResourcePolicyError("cleanup refused while builds/apps are active: " + ", ".join(active))
        files = sum(sum(digest is not None for _, digest in snapshot.values()) for snapshot in snapshots)
        print(f"Validated {files} non-runnable compiler metadata files; {len(uninspectable)} process executables uninspectable (no process-idle assertion).")
        link_updates = {}
        for tree, snapshot in zip(trees, snapshots):
            if apply:
                tree.remove_known(snapshot, link_updates=link_updates)
                print(f"Removed {tree.relative} (explicit metadata-only mode)")
            else:
                print(f"Would remove {tree.relative} (explicit metadata-only mode)")


def report(root: Path) -> None:
    rows: list[tuple[int, str, str]] = []
    logs: list[tuple[int, str]] = []
    for path in sorted((root / "target").iterdir()) if (root / "target").is_dir() else []:
        if not path.is_dir() or path.is_symlink():
            continue
        relative = str(path.relative_to(root))
        try:
            validate_cleanup_path(root, relative)
            kind = "separate Cargo target; remove only after releasing task ownership"
        except guard.ResourcePolicyError:
            kind = "shared cache / fixtures / evidence; retained"
        rows.append((guard._tree_size(path), relative, kind))
        if path.name not in {"debug", "release"}:
            for output in path.glob("*.log"):
                if output.is_file() and not output.is_symlink():
                    logs.append((output.stat().st_size, str(output.relative_to(root))))
    for path in (root / ".datum").glob("agent-cargo-archive-*"):
        if path.is_dir() and not path.is_symlink():
            rows.append((guard._tree_size(path), str(path.relative_to(root)), "archived build output"))
    print(f"Retained target/build-archive total: {guard.format_gib(guard.retained_cache_size(root, root / 'target'))}")
    for size, path, kind in sorted(rows, reverse=True):
        print(f"{guard.format_gib(size):>10}  {path}  [{kind}]")
    if logs:
        print("Largest direct proof logs (included in target usage, not Cargo caches):")
        for size, path in sorted(logs, reverse=True)[:10]:
            print(f"{guard.format_gib(size):>10}  {path}")
    for profile in ("debug", "release"):
        incremental = root / "target" / profile / "incremental"
        if incremental.is_dir() and not incremental.is_symlink():
            print(f"Rebuildable shared incremental cache: {incremental.relative_to(root)} "
                  f"({guard.format_gib(guard._tree_size(incremental))}; included above)")
    print("Read-only report. To remove a released disposable build tree:")
    print("  python3 scripts/manage_cargo_cache.py --remove <reported-relative-path> --apply")
    print("Cleanup never archives or automatically evicts another task's cache.")
    print("Unmarked proof/fixture directories are retained. Selecting a cache deletes its entire tree;")
    print("review its contents and release ownership first, including any output inside it.")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--remove", action="append", default=[], metavar="PATH")
    parser.add_argument("--apply", action="store_true", help="delete the explicitly selected disposable caches")
    parser.add_argument("--metadata-only", action="store_true", help="opt in to positive non-runnable compiler-metadata proof, only for exact target/{debug,release}/incremental; does not assert unknown processes are idle")
    args = parser.parse_args()
    if args.metadata_only and not args.remove:
        parser.error("--metadata-only requires explicit --remove incremental paths")
    if args.apply and not args.remove:
        parser.error("--apply requires at least one --remove path")
    try:
        if not args.remove:
            report(ROOT)
        elif args.metadata_only:
            clean_caches(ROOT, args.remove, guard.load_policy().lock_path, metadata_only=True, apply=args.apply)
        elif args.apply:
            clean_caches(ROOT, args.remove, guard.load_policy().lock_path)
        else:
            for relative in args.remove:
                path = validate_cleanup_path(ROOT, relative)
                print(f"Would remove {relative}: {guard.format_gib(guard._tree_size(path))}")
            print("Dry run only; append --apply to remove these explicitly selected caches.")
        return 0
    except (OSError, subprocess.SubprocessError, guard.ResourcePolicyError) as exc:
        print(f"Cargo cache maintenance refused: {exc}", file=sys.stderr)
        return 75


if __name__ == "__main__":
    raise SystemExit(main())
