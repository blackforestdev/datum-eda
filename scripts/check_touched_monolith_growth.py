#!/usr/bin/env python3
"""Block growth of known monolith files when they are touched.

Policy intent:
- feature work may continue in parallel
- if a known monolith file is touched, it must not grow
- decomposition PRs should reduce these baselines over time (ratchet-down)
"""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]

# Baseline totals captured at policy enactment. These values must trend
# downward as decomposition PRs land.
MONOLITH_BASELINES: dict[str, int] = {
    "crates/cli/src/command_project.rs": 9245,
    "crates/cli/src/main.rs": 3032,
    "crates/cli/src/command_exec.rs": 2595,
    "crates/cli/src/cli_args.rs": 2262,
    "crates/engine/src/export/mod.rs": 930,
    "crates/engine/src/board/mod.rs": 724,
    "crates/test-harness/src/bin/m3_sidecar_roundtrip_fidelity.rs": 911,
}


def run(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout.strip()


def changed_files() -> set[str]:
    changed: set[str] = set()
    commands = (
        ["git", "diff", "--name-only"],
        ["git", "diff", "--cached", "--name-only"],
        ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"],
    )
    for cmd in commands:
        code, out = run(cmd)
        if code != 0:
            continue
        for line in out.splitlines():
            path = line.strip()
            if path:
                changed.add(path)
    return changed


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def main() -> int:
    touched = changed_files()
    violations: list[str] = []
    checked = 0

    for rel, baseline in MONOLITH_BASELINES.items():
        if rel not in touched:
            continue
        checked += 1
        path = ROOT / rel
        if not path.exists():
            violations.append(f"{rel}: missing file (baseline {baseline})")
            continue
        current = line_count(path)
        if current > baseline:
            violations.append(f"{rel}: {current} lines > baseline {baseline}")

    if violations:
        print("Touched monolith growth check failed:", file=sys.stderr)
        print(
            "  Touched monolith files must not grow. Split or move logic into shards.",
            file=sys.stderr,
        )
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        return 1

    print(
        "Touched monolith growth check passed "
        f"({checked} touched monolith files checked)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
