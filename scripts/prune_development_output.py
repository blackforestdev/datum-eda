#!/usr/bin/env python3
"""Preview or prune released development output while preserving project fixtures."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
from collections import Counter
from contextlib import ExitStack
import subprocess
import sys
import tempfile

import manage_cargo_cache as cache
import run_cargo_guarded as guard
import cargo_qa_companions as bound
import cargo_cache_metadata as compiler_metadata


ROOT = guard.ROOT
BINS = ("datum-gui", "datum-eda", "eda-engine-daemon")
COMPILER_ROOTS = tuple(f"target/{profile}/{name}" for profile in ("debug", "release")
                       for name in ("deps", "build", ".fingerprint", "incremental"))


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
    if base in protected:
        return []
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


def refuse_open_output(roots: list[Path], *, check_mappings: bool = False, builds_only: bool = False,
                       admitted_fds: frozenset[int] = frozenset()) -> None:
    selected = set(roots)
    def selected_path(path):
        return path in selected or any(parent in selected for parent in path.parents)
    active = cache.active_processes(builds_only=builds_only)
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
                if (process.name == str(os.getpid()) and reference.parent == process / "fd"
                        and reference.name.isdigit() and int(reference.name) in admitted_fds):
                    continue  # Exact writable/read descriptors held by this admitted batch.
                try:
                    opened = Path(os.readlink(reference))
                except FileNotFoundError:
                    continue
                if selected_path(opened):
                    raise guard.ResourcePolicyError(f"active output reference: PID{process.name} {opened}")
            if check_mappings:
                for line in (process / "maps").read_text().splitlines():
                    fields = line.split(maxsplit=5)
                    if len(fields) == 6 and "x" in fields[1] and fields[5].startswith("/"):
                        opened = Path(fields[5].removesuffix(" (deleted)"))
                        if selected_path(opened):
                            raise guard.ResourcePolicyError(f"active output mapping: PID{process.name} {opened}")
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


def retained_dependency_records(runtime: Path, destinations: list[Path]) -> list[Path]:
    # Alternate runtime copies invalidate destination metadata. Let its removal
    # force a fresh companion build rather than pairing new binaries with old .d.
    return [path.with_suffix(".d") for path in destinations
            if runtime / path.name == path
            and path.with_suffix(".d").is_file()
            and not path.with_suffix(".d").is_symlink()]


def compiler_path(root: Path, relative: str) -> Path:
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts or not any(
            path.is_relative_to(Path(base)) and path != Path(base) for base in COMPILER_ROOTS):
        raise guard.ResourcePolicyError("compiler selection escapes the fixed cache roots")
    return root / path


def compiler_kind(fd: int, relative: str) -> str | None:
    """Admit generated compiler data, never source, logs or arbitrary executables."""
    path = Path(relative)
    head = os.pread(fd, 64, 0)
    if path.suffix in (".rlib", ".a") and head[:8] in (b"!<arch>\n", b"!<thin>\n"):
        return "archive"
    if path.suffix == ".rmeta" and head[:8] == b"rust\x00\x00\x00\x0a":
        return "metadata"
    if path.suffix in (".bc", ".o") and head[:4] in (b"BC\xc0\xde", b"\xde\xc0\x17\x0b"):
        return "bitcode"
    if len(head) < 20 or head[:4] != b"\x7fELF" or head[5] not in (1, 2):
        return None
    endian = "little" if head[5] == 1 else "big"
    kind = int.from_bytes(head[16:18], endian)
    if path.suffix == ".o" and compiler_metadata.relocatable_header(head):
        return "relocatable"
    # Shared libraries remain available to live proc-macro/analysis workers.
    cargo_name = re.fullmatch(r"[A-Za-z0-9_]+-[0-9a-f]{16}", path.name)
    if not (cargo_name or path.name == "build-script-build") or kind not in (2, 3):
        return None
    if (len(head) != 64 or head[4] != 2 or head[6] != 1
            or int.from_bytes(head[20:24], endian) != 1
            or int.from_bytes(head[52:54], endian) != 64):
        return None
    offset = int.from_bytes(head[32:40], endian)
    size = int.from_bytes(head[54:56], endian)
    count = int.from_bytes(head[56:58], endian)
    if size < 4 or offset + size * count > os.fstat(fd).st_size:
        return None
    if any(int.from_bytes(os.pread(fd, 4, offset + size * i), endian) == 3 for i in range(count)):
        return "program"
    return None


def compiler_plan(root: Path, keep: list[str]) -> dict:
    root = root.resolve()
    protected = {}
    for relative in [f"target/{profile}/{name}" for profile in ("debug", "release") for name in BINS] + keep:
        path = root / relative
        if Path(relative).is_absolute() or ".." in Path(relative).parts or not path.is_relative_to(root / "target"):
            raise guard.ResourcePolicyError("protected runtime must be inside target")
        if path.is_symlink() or path.resolve() != path or not path.is_file():
            raise guard.ResourcePolicyError(f"unsafe protected runtime: {relative}")
        with path.open("rb") as source:
            if source.read(4) != b"\x7fELF":
                raise guard.ResourcePolicyError(f"protected runtime is not ELF: {relative}")
            source.seek(0)
            protected[relative] = {**bound.info(path.stat()), "sha256": hashlib.file_digest(source, "sha256").hexdigest()}
    held = {(v["device"], v["inode"]) for v in protected.values()}
    candidates, preserved = [], []
    with guard.acquire_lock(guard.load_policy().lock_path, 0):
        if cache.active_processes(builds_only=True):
            raise guard.ResourcePolicyError("Cargo/rustc must be idle before compiler-cache planning")
        for base in COMPILER_ROOTS:
            directory = root / base
            if not directory.exists():
                continue
            if directory.is_symlink() or directory.resolve() != directory:
                raise guard.ResourcePolicyError(f"unsafe compiler root: {base}")
            projects = []
            if any((ancestor / "project.json").exists() for ancestor in (directory, directory.parent, root / "target")):
                projects.append(directory)
            for parent, dirs, files in os.walk(directory, followlinks=False):
                parent = Path(parent)
                if "project.json" in files:
                    projects.append(parent)
                for name in dirs:
                    if (parent / name).is_symlink():
                        raise guard.ResourcePolicyError("compiler cache contains a symlink directory")
                for name in files:
                    path = parent / name
                    relative = str(path.relative_to(root))
                    info = path.lstat()
                    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid():
                        raise guard.ResourcePolicyError(f"unknown compiler cache entry: {relative}")
                    record = {"path": relative, **bound.info(info), "allocated": info.st_blocks * 512}
                    with path.open("rb") as source:
                        kind = compiler_kind(source.fileno(), relative)
                    native = any(path.is_relative_to(project) for project in projects)
                    if name.endswith(".d") or native or (info.st_dev, info.st_ino) in held or kind is None:
                        preserved.append(record)
                    else:
                        candidates.append({**record, "kind": kind})
        retained_inodes = held | {(x["device"], x["inode"]) for x in preserved}
        selected = [x for x in candidates if (x["device"], x["inode"]) not in retained_inodes]
        preserved += [x for x in candidates if (x["device"], x["inode"]) in retained_inodes]
        links = Counter((x["device"], x["inode"]) for x in selected)
        unique = {(x["device"], x["inode"]): x for x in selected}
        reclaimed = sum(x["allocated"] for key, x in unique.items() if links[key] == x["nlink"])
        before = guard.retained_cache_size(root, root / "target")
    return {"operation": "compiler_cache", "root": str(root), "roots": list(COMPILER_ROOTS),
            "protected": protected, "preserved": preserved, "selected": selected,
            "expected_reclaimed_bytes": reclaimed, "retained_before_bytes": before,
            "expected_retained_after_bytes": before - reclaimed}


def apply_compiler_plan(root: Path, plan: dict) -> dict:
    root = root.resolve()
    if plan.get("operation") != "compiler_cache" or plan.get("root") != str(root) or plan.get("roots") != list(COMPILER_ROOTS):
        raise guard.ResourcePolicyError("not this repository's fixed compiler-cache plan")
    receipt = {"operation": "compiler_cache", "completed": [], "status": "preflight"}
    def verify(path, expected):
        if bound.info(path.lstat()) != {key: expected[key] for key in bound.FIELDS}:
            raise guard.ResourcePolicyError(f"compiler plan input changed: {path}")
    def protected():
        for relative, info in plan["protected"].items():
            path = root / relative
            if path.is_symlink() or path.resolve() != path:
                raise guard.ResourcePolicyError("protected runtime path changed")
            verify(path, info)
            with path.open("rb") as source:
                if hashlib.file_digest(source, "sha256").hexdigest() != info["sha256"]:
                    raise guard.ResourcePolicyError("protected runtime content changed")
    try:
        with guard.acquire_lock(guard.load_policy().lock_path, 0), ExitStack() as stack:
            if cache.active_processes(builds_only=True):
                raise guard.ResourcePolicyError("Cargo/rustc must be idle before compiler-cache cleanup")
            selected = plan["selected"]
            paths = [compiler_path(root, x["path"]) for x in selected]
            if len(set(paths)) != len(paths):
                raise guard.ResourcePolicyError("duplicate compiler-cache selection")
            required = {f"target/{profile}/{name}" for profile in ("debug", "release") for name in BINS}
            if not required.issubset(plan["protected"]):
                raise guard.ResourcePolicyError("plan omits current normal runtimes")
            kept_inodes = {(x["device"], x["inode"]) for x in [*plan["protected"].values(), *plan["preserved"]]}
            for path, info in zip(paths, selected):
                if (info["device"], info["inode"]) in kept_inodes or path.name.endswith(".d"):
                    raise guard.ResourcePolicyError("plan selects protected runtime/evidence")
                if any((parent / "project.json").exists() for parent in path.parents if parent.is_relative_to(root / "target")):
                    raise guard.ResourcePolicyError("plan selects native project data")
            tracked = subprocess.check_output(["git", "ls-files", "-z", "--", *COMPILER_ROOTS], cwd=root)
            if tracked:
                raise guard.ResourcePolicyError("compiler cache contains tracked input")
            protected()
            for info in plan["preserved"]:
                path = compiler_path(root, info["path"])
                if path.is_symlink() or path.resolve() != path:
                    raise guard.ResourcePolicyError("preserved compiler input path changed")
                verify(path, info)
            records, aliases = [], {}
            for path, info in zip(paths, selected):
                parent, chain = bound.directory(root, path.relative_to(root).parts[:-1], stack)
                descriptor = os.open(path.name, bound.WRITE if info["kind"] == "program" else bound.READ, dir_fd=parent)
                stack.callback(os.close, descriptor)
                verify(path, info)
                if bound.info(os.fstat(descriptor)) != {key: info[key] for key in bound.FIELDS} or compiler_kind(descriptor, info["path"]) != info["kind"]:
                    raise guard.ResourcePolicyError("selected compiler identity/type changed")
                records.append((path, info.copy(), parent, chain, descriptor))
                aliases.setdefault((info["device"], info["inode"]), []).append((records[-1][1], descriptor))
            refuse_open_output(paths, check_mappings=True, builds_only=True,
                               admitted_fds=frozenset(record[4] for record in records))
            if cache.active_processes(builds_only=True):
                raise guard.ResourcePolicyError("build appeared during compiler-cache preflight")
            for path, info, parent, chain, descriptor in records:
                with ExitStack() as check:
                    _, current = bound.directory(root, path.relative_to(root).parts[:-1], check)
                    if current != chain:
                        raise guard.ResourcePolicyError("compiler parent identity changed")
                verify(path, info)
                before = bound.info(os.fstat(descriptor))
                os.unlink(path.name, dir_fd=parent)
                receipt["completed"].append(info["path"])
                after = bound.info(os.fstat(descriptor))
                if after != dict(before, nlink=before["nlink"] - 1, ctime_ns=after["ctime_ns"]):
                    raise guard.ResourcePolicyError("compiler inode changed after unlink")
                for alias, fd in aliases[(info["device"], info["inode"])]:
                    alias.update(bound.info(os.fstat(fd)))
            protected()
            for info in plan["preserved"]:
                verify(compiler_path(root, info["path"]), info)
        receipt["status"] = "complete_current_runtime_and_evidence_retained"
        receipt["retained_after_bytes"] = guard.retained_cache_size(root, root / "target")
        return receipt
    except (OSError, guard.ResourcePolicyError):
        receipt["status"] = "partial_failure" if receipt["completed"] else "refused"
        print(json.dumps(receipt, sort_keys=True))
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--released", action="store_true", help="confirm task ownership of pruned output is released")
    parser.add_argument("--build-only", action="store_true", help="retain all .datum task output; prune only released target output")
    parser.add_argument("--runtime-from", default="target/debug", metavar="PATH")
    parser.add_argument("--compiler-cache", action="store_true", help="plan only generated files in the eight compiler-cache roots")
    parser.add_argument("--keep-runtime", action="append", default=[], metavar="PATH")
    parser.add_argument("--plan-file", type=Path)
    parser.add_argument("--plan-sha256")
    args = parser.parse_args()
    if args.apply and not args.released:
        parser.error("--apply requires explicit --released ownership confirmation")
    try:
        if args.compiler_cache:
            if args.apply:
                if not args.plan_file or not args.plan_sha256 or args.keep_runtime:
                    raise guard.ResourcePolicyError("compiler apply requires the reviewed plan file/SHA and no new selection")
                payload = args.plan_file.read_bytes()
                if hashlib.sha256(payload).hexdigest() != args.plan_sha256:
                    raise guard.ResourcePolicyError("compiler plan SHA256 differs from reviewed plan")
                print(json.dumps(apply_compiler_plan(ROOT, json.loads(payload)), sort_keys=True))
            else:
                if args.plan_file or args.plan_sha256:
                    raise guard.ResourcePolicyError("redirect the compiler preview JSON to its frozen plan file")
                print(json.dumps(compiler_plan(ROOT, args.keep_runtime), sort_keys=True))
            return 0
        if args.keep_runtime or args.plan_file or args.plan_sha256:
            raise guard.ResourcePolicyError("compiler options require --compiler-cache")
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
        native_project_count = len(preserved)
        if args.build_only:
            preserved.append(ROOT / ".datum")
        history_backup = ROOT / ".datum/history-cleanup"
        if history_backup.exists():
            preserved.append(history_backup)
        destinations = [ROOT / "target/debug" / name for name in BINS]
        validate_destinations(destinations)
        # Companion freshness uses Cargo's dependency record. Keeping it avoids
        # rebuilding unchanged companions merely because cleanup removed .d.
        dependencies = retained_dependency_records(runtime, destinations)
        roots = [ROOT / "target", ROOT / ".datum", ROOT / ".git/datum-wdq"]
        plan = deletion_plan(roots, preserved + destinations + dependencies)
        for path in plan:
            print(f"REMOVE {path.relative_to(ROOT)}")
        print(f"Preserve {native_project_count} native project roots and one runtime binary set.")
        print("Retain .datum output." if args.build_only else "Remove other released .datum output.")
        print("Remove released build output and retired WDQ cache; retain companion dependency records.")
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
                for path in removable_entries(base, preserved + destinations + dependencies):
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
