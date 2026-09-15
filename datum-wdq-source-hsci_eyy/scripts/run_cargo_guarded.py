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
from contextlib import contextmanager
from dataclasses import dataclass
from typing import Iterator, Sequence


GIB = 1024**3
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_POLICY = Path(__file__).with_name("cargo_resource_policy.json")


class ResourcePolicyError(RuntimeError):
    """A Cargo invocation would violate the repository resource policy."""


@dataclass(frozen=True)
class ResourcePolicy:
    lock_path: Path
    lock_timeout_seconds: int
    minimum_tmp_free_bytes: int
    minimum_target_filesystem_free_bytes: int
    target_soft_limit_bytes: int
    target_hard_limit_bytes: int
    forbid_tmp_target: bool
    proof_incremental: bool


@dataclass(frozen=True)
class ResourceSnapshot:
    target_dir: Path
    target_bytes: int
    target_filesystem_free_bytes: int
    tmp_free_bytes: int


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
        target_hard_limit_bytes=_gib(
            document.get("target_hard_limit_gib"), "target_hard_limit_gib"
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
    )


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
    if snapshot.target_bytes > policy.target_hard_limit_bytes:
        errors.append(
            "Cargo target exceeds hard limit: "
            f"{format_gib(snapshot.target_bytes)} used; "
            f"limit {format_gib(policy.target_hard_limit_bytes)}"
        )
    return errors


def resource_warnings(
    policy: ResourcePolicy, snapshot: ResourceSnapshot
) -> list[str]:
    if snapshot.target_bytes <= policy.target_soft_limit_bytes:
        return []
    return [
        "Cargo target exceeds soft limit: "
        f"{format_gib(snapshot.target_bytes)} used; "
        f"soft limit {format_gib(policy.target_soft_limit_bytes)}"
    ]


def format_gib(byte_count: int) -> str:
    return f"{byte_count / GIB:.1f} GiB"


def command_environment(
    base: dict[str, str], workload: str, target_dir: Path, policy: ResourcePolicy
) -> dict[str, str]:
    environment = dict(base)
    environment["CARGO_TARGET_DIR"] = str(target_dir)
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


def _target_from_arguments(explicit: str | None) -> Path:
    configured = explicit or os.environ.get("CARGO_TARGET_DIR")
    if configured:
        path = Path(configured)
        return path if path.is_absolute() else (Path.cwd() / path)
    return ROOT / "target"


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


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(sys.argv[1:] if argv is None else argv)
    try:
        policy = load_policy(args.policy)
        target_dir = _target_from_arguments(args.target_dir).resolve()
        with acquire_lock(policy.lock_path, policy.lock_timeout_seconds):
            snapshot = snapshot_resources(target_dir)
            print(
                "Cargo resource preflight: "
                f"target={target_dir} size={format_gib(snapshot.target_bytes)} "
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
                os.environ, args.workload, target_dir, policy
            )
            return subprocess.run(args.command, env=environment, check=False).returncode
    except (OSError, subprocess.SubprocessError, ResourcePolicyError) as exc:
        print(f"Cargo resource guard refused execution: {exc}", file=sys.stderr)
        return 75


if __name__ == "__main__":
    raise SystemExit(main())
