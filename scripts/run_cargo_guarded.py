#!/usr/bin/env python3
"""Run Cargo under Datum's cross-session resource and serialization policy."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import tomllib
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Iterator, Sequence


GIB = 1024**3
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_POLICY = Path(__file__).with_name("cargo_resource_policy.json")
ACTIVE_GUARD = "DATUM_CARGO_GUARD_ACTIVE"


class ResourcePolicyError(RuntimeError):
    """A Cargo invocation would violate the repository resource policy."""


@dataclass(frozen=True)
class ResourcePolicy:
    lock_path: Path
    lock_timeout_seconds: int
    minimum_tmp_free_bytes: int
    minimum_target_filesystem_free_bytes: int
    target_soft_limit_bytes: int
    target_high_watermark_bytes: int
    forbid_tmp_target: bool
    proof_incremental: bool


@dataclass(frozen=True)
class ResourceSnapshot:
    target_dir: Path
    target_bytes: int
    target_filesystem_free_bytes: int
    tmp_free_bytes: int
    retained_cache_bytes: int = 0


def _gib(value: object, field: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ResourcePolicyError(f"{field} must be a non-negative integer")
    return value * GIB


def load_policy(path: Path = DEFAULT_POLICY) -> ResourcePolicy:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ResourcePolicyError(f"cannot load Cargo resource policy {path}: {exc}") from exc
    if document.get("schema_version") != 1:
        raise ResourcePolicyError("unsupported Cargo resource policy schema")
    lock_path = document.get("lock_path")
    timeout = document.get("lock_timeout_seconds")
    if not isinstance(lock_path, str) or not lock_path.startswith("/"):
        raise ResourcePolicyError("lock_path must be an absolute path")
    if not isinstance(timeout, int) or isinstance(timeout, bool) or timeout <= 0:
        raise ResourcePolicyError("lock_timeout_seconds must be a positive integer")
    return ResourcePolicy(
        lock_path=Path(lock_path),
        lock_timeout_seconds=timeout,
        minimum_tmp_free_bytes=_gib(
            document.get("minimum_tmp_free_gib"), "minimum_tmp_free_gib"
        ),
        minimum_target_filesystem_free_bytes=_gib(
            document.get("minimum_target_filesystem_free_gib"),
            "minimum_target_filesystem_free_gib",
        ),
        target_soft_limit_bytes=_gib(
            document.get("target_soft_limit_gib"), "target_soft_limit_gib"
        ),
        target_high_watermark_bytes=_gib(
            document.get("target_high_watermark_gib"), "target_high_watermark_gib"
        ),
        forbid_tmp_target=document.get("forbid_tmp_target") is True,
        proof_incremental=document.get("proof_incremental") is True,
    )


def _tree_size(path: Path) -> int:
    if not path.exists():
        return 0
    completed = subprocess.run(
        ["du", "-sx", "--block-size=1", str(path)],
        check=True,
        capture_output=True,
        text=True,
    )
    return int(completed.stdout.split()[0])


def snapshot_resources(target_dir: Path) -> ResourceSnapshot:
    target_probe = target_dir if target_dir.exists() else target_dir.parent
    while not target_probe.exists() and target_probe != target_probe.parent:
        target_probe = target_probe.parent
    return ResourceSnapshot(
        target_dir=target_dir.resolve(),
        target_bytes=_tree_size(target_dir),
        target_filesystem_free_bytes=shutil.disk_usage(target_probe).free,
        tmp_free_bytes=shutil.disk_usage("/tmp").free,
        retained_cache_bytes=retained_cache_size(ROOT, target_dir),
    )


def retained_cache_paths(root: Path, target_dir: Path) -> list[Path]:
    """Count shared, alternate and archived build trees without double-counting."""
    candidates = [root / "target", target_dir]
    candidates.extend((root / ".datum").glob("agent-cargo-archive-*"))
    paths: list[Path] = []
    for path in sorted({p.resolve() for p in candidates}, key=lambda p: len(p.parts)):
        if not any(path == parent or parent in path.parents for parent in paths):
            paths.append(path)
    return paths


def retained_cache_size(root: Path, target_dir: Path) -> int:
    return sum(_tree_size(path) for path in retained_cache_paths(root, target_dir))


def _is_within(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
    except ValueError:
        return False
    return True


def evaluate_resources(
    policy: ResourcePolicy, snapshot: ResourceSnapshot, workload: str
) -> list[str]:
    errors: list[str] = []
    if workload == "proof" and policy.forbid_tmp_target:
        if _is_within(snapshot.target_dir, Path("/tmp")):
            errors.append(
                f"proof Cargo target must not reside on /tmp: {snapshot.target_dir}"
            )
    if snapshot.tmp_free_bytes < policy.minimum_tmp_free_bytes:
        errors.append(
            "insufficient /tmp reserve: "
            f"{format_gib(snapshot.tmp_free_bytes)} free; "
            f"requires {format_gib(policy.minimum_tmp_free_bytes)}"
        )
    if (
        snapshot.target_filesystem_free_bytes
        < policy.minimum_target_filesystem_free_bytes
    ):
        errors.append(
            "insufficient target-filesystem reserve: "
            f"{format_gib(snapshot.target_filesystem_free_bytes)} free; "
            f"requires {format_gib(policy.minimum_target_filesystem_free_bytes)}"
        )
    return errors


def resource_warnings(
    policy: ResourcePolicy, snapshot: ResourceSnapshot
) -> list[str]:
    cache_bytes = max(snapshot.target_bytes, snapshot.retained_cache_bytes)
    if cache_bytes <= policy.target_soft_limit_bytes:
        return []
    # Retention targets guide maintenance; actual filesystem reserves remain
    # blocking above. A cache target alone must not strand safe development.
    if cache_bytes > policy.target_high_watermark_bytes:
        return [
            "retained Cargo caches exceed advisory high watermark: "
            f"{format_gib(cache_bytes)} used; "
            f"target {format_gib(policy.target_high_watermark_bytes)}; "
            "build may proceed while filesystem reserves are sufficient; "
            "inspect with python3 scripts/manage_cargo_cache.py"
        ]
    return [
        "retained Cargo caches exceed soft target: "
        f"{format_gib(cache_bytes)} used; "
        f"target {format_gib(policy.target_soft_limit_bytes)}; "
        "inspect with python3 scripts/manage_cargo_cache.py"
    ]


def format_gib(byte_count: int) -> str:
    return f"{byte_count / GIB:.1f} GiB"


def command_environment(
    base: dict[str, str], workload: str, target_dir: Path, policy: ResourcePolicy,
    *, build_dir: Path | None = None,
) -> dict[str, str]:
    environment = dict(base)
    environment[ACTIVE_GUARD] = str(os.getpid())
    environment["CARGO_TARGET_DIR"] = str(target_dir)
    environment["CARGO_BUILD_TARGET_DIR"] = str(target_dir)
    environment["CARGO_BUILD_BUILD_DIR"] = str(build_dir if build_dir is not None else target_dir)
    if workload == "proof":
        environment["CARGO_INCREMENTAL"] = "1" if policy.proof_incremental else "0"
    return environment


@contextmanager
def acquire_lock(lock_path: Path, timeout_seconds: float) -> Iterator[None]:
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open("a+", encoding="utf-8") as lock_file:
        deadline = time.monotonic() + timeout_seconds
        while True:
            try:
                fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise ResourcePolicyError(
                        f"timed out waiting for Cargo resource lock {lock_path}"
                    )
                time.sleep(min(0.25, max(0.0, deadline - time.monotonic())))
        try:
            yield
        finally:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)


def _directory(value: str, base: Path) -> Path:
    if not isinstance(value, str) or not value:
        raise ResourcePolicyError("Cargo output directory must be a nonempty path")
    try:
        path = Path(value)
        return (path if path.is_absolute() else base / path).resolve()
    except (OSError, ValueError, RuntimeError):
        raise ResourcePolicyError("cannot resolve Cargo output directory") from None


def _target_from_arguments(explicit: str | None) -> Path:
    configured = (explicit or os.environ.get("CARGO_TARGET_DIR")
                  or os.environ.get("CARGO_BUILD_TARGET_DIR"))
    return _directory(configured, Path.cwd()) if configured else (ROOT / "target").resolve()


def _document_targets(document: dict, base: Path) -> dict:
    build = document.get("build", {})
    if not isinstance(build, dict):
        raise ResourcePolicyError("Cargo build configuration must be a table")
    return {name: (build[name], base) for name in ("target-dir", "build-dir") if name in build}


def _included_targets(document: dict, directory: Path, active: tuple[Path, ...]) -> dict:
    includes = document.get("include", [])
    if not isinstance(includes, list):
        raise ResourcePolicyError("Cargo configuration includes must be an array")
    targets = {}
    for entry in includes:
        optional = False
        if isinstance(entry, dict):
            optional = entry.get("optional", False)
            entry = entry.get("path")
        if not isinstance(entry, str) or not isinstance(optional, bool):
            raise ResourcePolicyError("invalid Cargo configuration include")
        path = directory / entry
        if path.suffix != ".toml":
            raise ResourcePolicyError("Cargo configuration includes must name TOML files")
        targets.update(_file_targets(path, active, optional=optional))
    return targets


def _file_targets(path: Path, active: tuple[Path, ...] = (), *, optional: bool = False) -> dict:
    # Cargo resolves include names beside the including file, but output paths
    # two levels above the file, including files explicitly passed via --config.
    try:
        identity = path.resolve()
    except (OSError, ValueError, RuntimeError):
        raise ResourcePolicyError("cannot resolve Cargo configuration file") from None
    if identity in active or len(active) >= 32:
        raise ResourcePolicyError("cyclic or excessively nested Cargo configuration includes")
    try:
        with path.open("rb") as source:
            document = tomllib.load(source)
    except FileNotFoundError:
        if optional:
            return {}
        raise ResourcePolicyError("Cargo configuration file is missing") from None
    except (OSError, ValueError):
        raise ResourcePolicyError("cannot read valid Cargo TOML configuration") from None
    targets = _included_targets(document, path.parent, (*active, identity))
    targets.update(_document_targets(document, path.parent.parent))
    return targets


def _config_targets(value: str, cwd: Path) -> dict:
    if "=" not in value:
        path = Path(value)
        return _file_targets(path if path.is_absolute() else cwd / path)
    try:
        document = tomllib.loads(value)
    except ValueError:
        raise ResourcePolicyError("invalid inline Cargo TOML configuration") from None
    targets = _included_targets(document, cwd, ())
    targets.update(_document_targets(document, cwd))
    return targets


def validate_output_directories(command: Sequence[str], target: Path) -> Path:
    """Bind explicit Cargo overrides before resource inspection or child launch.

    The guard is not a sandbox for aliases/plugins or arbitrary subprocesses.
    Canonical environment paths override ordinary discovered configuration;
    explicit --config values override that environment and need validation here.
    """
    cwd = Path.cwd()
    targets = {}
    direct = None
    index = 1
    while index < len(command):
        argument = command[index]
        if argument == "--":
            break  # Everything remaining belongs to the test/program/compiler.
        if argument == "-C" or argument.startswith("-C"):
            raise ResourcePolicyError("Cargo -C is unsupported; run the guard from the desired directory")
        option, separator, value = argument.partition("=")
        if option in {"--target-dir", "--config"}:
            if not separator:
                index += 1
                if index >= len(command) or command[index] == "--":
                    raise ResourcePolicyError("Cargo target/config option requires a value")
                value = command[index]
            if option == "--target-dir":
                direct = (value, cwd)
            else:
                targets.update(_config_targets(value, cwd))
        index += 1
    effective = direct if direct is not None else targets.get("target-dir")
    if effective is not None and _directory(*effective) != target:
        raise ResourcePolicyError("Cargo target override differs from guarded target; use the same --target-dir")
    build = targets.get("build-dir")
    if build is None and os.environ.get("CARGO_BUILD_BUILD_DIR"):
        build = (os.environ["CARGO_BUILD_BUILD_DIR"], cwd)
    if build is not None:
        if not isinstance(build[0], str) or "{" in build[0] or "}" in build[0]:
            raise ResourcePolicyError("Cargo build-dir needs a concrete guarded path; expand templates explicitly")
        build_dir = _directory(*build)
        if not build_dir.is_relative_to(target):
            raise ResourcePolicyError("Cargo build-dir escapes guarded target; use a concrete directory inside it")
        return build_dir
    return target


def _parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workload", choices=("interactive", "proof"), required=True)
    parser.add_argument("--target-dir")
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.check_only:
        if not args.command or Path(args.command[0]).name != "cargo":
            parser.error("the guarded command must begin with cargo")
    return args


def refuse_native_projects(target: Path) -> None:
    """Cargo cleanup must never own persistent native fixture/user inputs."""
    if target.exists():
        marker = next(target.rglob("project.json"), None)
        if marker is not None:
            raise ResourcePolicyError(
                f"native project found in disposable Cargo output: {marker}; "
                "preserve/migrate it with scripts/relocate_target_workspaces.py first"
            )


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(sys.argv[1:] if argv is None else argv)
    try:
        if os.environ.get(ACTIVE_GUARD):
            raise ResourcePolicyError(
                "nested Cargo would wait on its parent guard; build matching "
                "CLI/daemon companions before starting the test run"
            )
        target_dir = _target_from_arguments(args.target_dir).resolve()
        build_dir = validate_output_directories(args.command, target_dir)
        policy = load_policy(args.policy)
        with acquire_lock(policy.lock_path, policy.lock_timeout_seconds):
            refuse_native_projects(ROOT / "target")
            if target_dir != ROOT / "target":
                refuse_native_projects(target_dir)
            snapshot = snapshot_resources(target_dir)
            print(
                "Cargo resource preflight: "
                f"target={target_dir} size={format_gib(snapshot.target_bytes)} "
                f"retained_caches={format_gib(snapshot.retained_cache_bytes)} "
                f"target_free={format_gib(snapshot.target_filesystem_free_bytes)} "
                f"tmp_free={format_gib(snapshot.tmp_free_bytes)}",
                file=sys.stderr,
            )
            for warning in resource_warnings(policy, snapshot):
                print(f"warning: {warning}", file=sys.stderr)
            errors = evaluate_resources(policy, snapshot, args.workload)
            if errors:
                raise ResourcePolicyError("; ".join(errors))
            if args.check_only:
                return 0
            environment = command_environment(
                os.environ, args.workload, target_dir, policy, build_dir=build_dir
            )
            return subprocess.run(args.command, env=environment, check=False).returncode
    except (OSError, subprocess.SubprocessError, ResourcePolicyError) as exc:
        print(f"Cargo resource guard refused execution: {exc}", file=sys.stderr)
        return 75


if __name__ == "__main__":
    raise SystemExit(main())
