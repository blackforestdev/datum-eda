#!/usr/bin/env python3
"""Preview or prune released development output while preserving project fixtures."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import manage_cargo_cache as cache
import run_cargo_guarded as guard


ROOT = guard.ROOT
BINS = ("datum-gui", "datum-eda", "eda-engine-daemon")


def is_user_service_manager(command: list[bytes]) -> bool:
    return (
        len(command) >= 2
        and command[0] in {b"/usr/lib/systemd/systemd", b"/lib/systemd/systemd"}
        and command[1] == b"--user"
        and all(arg.startswith(b"--deserialize=") and arg.split(b"=", 1)[1].isdigit()
                for arg in command[2:])
    )


def is_user_service_process(process: Path, command: list[bytes]) -> bool:
    if is_user_service_manager(command):
        return True
    if command != [b"(sd-pam)"]:
        return False
    parent = next(line.split()[1] for line in (process / "status").read_text().splitlines()
                  if line.startswith("PPid:"))
    parent_command = (process.parent / parent / "cmdline").read_bytes().rstrip(b"\0").split(b"\0")
    return is_user_service_manager(parent_command)


def project_roots(root: Path) -> list[Path]:
    candidates: list[Path] = []
    for base in (root / "target", root / ".datum"):
        for marker in base.rglob("project.json"):
            candidates.append(marker.parent)
    preserved: list[Path] = []
    for path in sorted(set(candidates), key=lambda p: len(p.parts)):
        if not any(parent == path or parent in path.parents for parent in preserved):
            preserved.append(path)
    return preserved


def removable_entries(base: Path, preserve: list[Path]) -> list[Path]:
    """Select whole subtrees without traversing symlinks or protected projects."""
    if not base.is_dir() or base.is_symlink():
        raise guard.ResourcePolicyError(f"unsafe development root: {base}")
    protected = set(preserve)
    ancestors = {parent for kept in preserve for parent in kept.parents}

    def collect(directory: Path) -> list[Path]:
        entries: list[Path] = []
        for path in directory.iterdir():
            if path in protected:
                continue
            if path in ancestors:
                if path.is_symlink() or not path.is_dir():
                    raise guard.ResourcePolicyError(f"unsafe preservation ancestor: {path}")
                entries.extend(collect(path))
            else:
                entries.append(path)
        return entries

    return collect(base)


def refuse_open_output(roots: list[Path]) -> None:
    active = cache.active_processes()
    if active:
        raise guard.ResourcePolicyError("build/application is active: " + ", ".join(active))
    for process in Path("/proc").iterdir():
        if not process.name.isdigit():
            continue
        command: list[bytes] = []
        try:
            if process.stat().st_uid != os.getuid():
                continue
            # The user's nondumpable systemd supervisor cannot be inspected
            # through cwd/exe. It is a service manager, not a workspace worker;
            # every child is still checked independently below. Do not skip
            # other uninspectable programs by name.
            command = (process / "cmdline").read_bytes().rstrip(b"\0").split(b"\0")
            status = (process / "status").read_text()
            if any(line.startswith("State:") and "Z (zombie)" in line for line in status.splitlines()):
                continue
            if is_user_service_process(process, command):
                continue
            references = [process / "cwd", process / "exe", *(process / "fd").iterdir()]
            for reference in references:
                try:
                    opened = Path(os.readlink(reference))
                except FileNotFoundError:
                    continue
                if any(opened == path or path in opened.parents for path in roots):
                    raise guard.ResourcePolicyError(f"active output reference: PID{process.name} {opened}")
        except PermissionError:
            # These nondumpable desktop/security services are not workspace
            # workers. Match installed executable paths, not arbitrary names;
            # all inspectable services and their children are still inspected.
            installed_services = {
                b"/usr/bin/ssh-agent", b"/usr/bin/kwin_wayland",
                b"/usr/lib/x86_64-linux-gnu/libexec/polkit-kde-authentication-agent-1",
                b"/usr/lib/x86_64-linux-gnu/libexec/org_kde_powerdevil",
            }
            if command and command[0] in installed_services:
                print(f"Limited process visibility: PID{process.name} {command[0].decode()} (system service)", file=sys.stderr)
                continue
            raise
        except FileNotFoundError:
            continue


def validate_destinations(destinations: list[Path]) -> None:
    for destination in destinations:
        parent = destination.parent
        if not parent.is_dir() or parent.is_symlink() or parent.resolve() != parent:
            raise guard.ResourcePolicyError(f"unsafe runtime destination directory: {parent}")
        if destination.is_symlink() or (destination.exists() and not destination.is_file()):
            raise guard.ResourcePolicyError(f"unsafe runtime destination: {destination}")
        legacy_temp = destination.with_name(destination.name + ".maintenance-new")
        if legacy_temp.exists() or legacy_temp.is_symlink():
            raise guard.ResourcePolicyError(f"unexpected runtime temporary path: {legacy_temp}")


def copy_runtime(source: Path, destination: Path) -> None:
    validate_destinations([destination])
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=destination.parent, prefix=".maintenance-", delete=False) as output:
            temporary = Path(output.name)
            with source.open("rb") as stream:
                shutil.copyfileobj(stream, output)
        shutil.copymode(source, temporary)
        with source.open("rb") as src, temporary.open("rb") as dst:
            if hashlib.file_digest(src, "sha256").digest() != hashlib.file_digest(dst, "sha256").digest():
                raise guard.ResourcePolicyError("runtime copy verification failed")
        temporary.replace(destination)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def deletion_plan(roots: list[Path], preserve: list[Path]) -> list[Path]:
    entries = [entry for base in roots[:2] for entry in removable_entries(base, preserve)]
    if roots[2].exists():
        if roots[2].is_symlink():
            raise guard.ResourcePolicyError("unexpected WDQ cache symlink")
        entries.append(roots[2])
    return sorted(entries)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--released", action="store_true", help="confirm task ownership of pruned output is released")
    parser.add_argument("--runtime-from", default="target/debug", metavar="PATH")
    args = parser.parse_args()
    if args.apply and not args.released:
        parser.error("--apply requires explicit --released ownership confirmation")
    try:
        runtime = ROOT / args.runtime_from
        if runtime.is_symlink() or runtime.resolve() != runtime or not runtime.is_relative_to(ROOT / "target"):
            raise guard.ResourcePolicyError("runtime source must be inside target/ without symlink parents")
        binaries = [runtime / name for name in BINS]
        if any(not path.is_file() or path.is_symlink() for path in binaries):
            raise guard.ResourcePolicyError("all three runtime binaries must exist before cleanup")
        for path in binaries:
            with path.open("rb") as stream:
                if stream.read(4) != b"\x7fELF":
                    raise guard.ResourcePolicyError(f"not a built runtime binary: {path}")
        preserved = project_roots(ROOT)
        history_backup = ROOT / ".datum/history-cleanup"
        if history_backup.exists():
            preserved.append(history_backup)
        destinations = [ROOT / "target/debug" / name for name in BINS]
        validate_destinations(destinations)
        roots = [ROOT / "target", ROOT / ".datum", ROOT / ".git/datum-wdq"]
        plan = deletion_plan(roots, preserved + destinations)
        for path in plan:
            print(f"REMOVE {path.relative_to(ROOT)}")
        print(f"Preserve {len(preserved)} native project roots and one runtime binary set.")
        print("Remove other released target/.datum output and retired WDQ cache.")
        print("Tracker backups are pruned separately with br history prune.")
        if not args.apply:
            print("Preview only; use --apply --released after reviewing contents and ownership.")
            return 0
        with guard.acquire_lock(guard.load_policy().lock_path, 0):
            validate_destinations(destinations)
            if any(path.is_symlink() for path in roots):
                raise guard.ResourcePolicyError("development roots cannot be symlinks")
            refuse_open_output(roots)
            tracked = subprocess.check_output(["git", "ls-files", "-z", "--", "target/", ".datum/"], cwd=ROOT)
            if tracked:
                raise guard.ResourcePolicyError("development output contains tracked files")
            for source, destination in zip(binaries, destinations):
                if source != destination:
                    copy_runtime(source, destination)
            deleted = 0
            for base in roots[:2]:
                for path in removable_entries(base, preserved + destinations):
                    if path.is_symlink() or not path.is_dir():
                        path.unlink()
                    else:
                        shutil.rmtree(path)
                    deleted += 1
            retired = roots[2]
            if retired.is_symlink():
                raise guard.ResourcePolicyError("unexpected WDQ cache symlink")
            if retired.is_dir():
                shutil.rmtree(retired)
            print(f"Removed {deleted} released output entries; preserved project contents and runtime binaries.")
        return 0
    except (OSError, subprocess.SubprocessError, guard.ResourcePolicyError) as exc:
        print(f"Development prune refused: {exc}", file=sys.stderr)
        return 75


if __name__ == "__main__":
    raise SystemExit(main())
