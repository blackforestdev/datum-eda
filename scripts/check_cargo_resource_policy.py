#!/usr/bin/env python3
"""Reject unguarded Cargo compilation in repository proof and gate scripts."""

from __future__ import annotations

from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
GUARD = "run_cargo_guarded.py"
COMPILE_COMMAND = re.compile(
    r"(?:^|[;&|\s])cargo\s+(?:build|check|test|clippy|run|rustc)(?:\s|$)"
)
REQUIRED_GUARDED_RUNNERS = (
    "run_agent_launch_pty_proof.sh",
    "run_drift_gates.sh",
    "run_gui_compositor_smoke.sh",
    "run_migration_proof_gates.sh",
    "run_terminal_compatibility_proof.sh",
    "run_terminal_transport_proof_gates.sh",
)


def unguarded_cargo_lines(path: Path) -> list[tuple[int, str]]:
    violations: list[tuple[int, str]] = []
    for line_number, line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        stripped = line.lstrip()
        if (
            stripped.startswith("#")
            or GUARD in line
            or '"${cargo_guard[@]}"' in line
        ):
            continue
        if COMPILE_COMMAND.search(line):
            violations.append((line_number, line.strip()))
    return violations


def check(root: Path = ROOT) -> list[str]:
    scripts = root / "scripts"
    errors: list[str] = []
    for name in REQUIRED_GUARDED_RUNNERS:
        path = scripts / name
        if not path.is_file():
            errors.append(f"missing required Cargo runner: scripts/{name}")
            continue
        text = path.read_text(encoding="utf-8")
        if GUARD not in text:
            errors.append(f"scripts/{name} does not invoke {GUARD}")
    for path in sorted(scripts.glob("*.sh")):
        for line_number, line in unguarded_cargo_lines(path):
            errors.append(
                f"{path.relative_to(root)}:{line_number}: unguarded Cargo command: {line}"
            )
    return errors


def main() -> int:
    errors = check()
    if errors:
        print("Cargo resource policy violations:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("Cargo resource policy: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
