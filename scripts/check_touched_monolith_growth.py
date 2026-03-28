#!/usr/bin/env python3
"""Require burn-down on known monolith files when they are touched.

Policy intent:
- feature work may continue in parallel
- if a known monolith file is touched, it must shrink in the current branch
- monolith shrink must come from structural extraction into companion shards,
  not pure deletion
- decomposition baselines must trend downward over time (ratchet-down)
"""

from __future__ import annotations

import os
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


def find_base_ref() -> str | None:
    env_base = os.environ.get("GITHUB_BASE_REF")
    candidates: list[str] = []
    if env_base:
        candidates.extend([f"origin/{env_base}", env_base])
    candidates.extend(["origin/main", "main", "origin/master", "master"])
    seen: set[str] = set()
    for ref in candidates:
        if ref in seen:
            continue
        seen.add(ref)
        code, _ = run(["git", "rev-parse", "--verify", ref])
        if code == 0:
            return ref
    return None


def merge_base_commit() -> str | None:
    base_ref = find_base_ref()
    if base_ref:
        code, out = run(["git", "merge-base", "HEAD", base_ref])
        if code == 0 and out:
            return out
    code, out = run(["git", "rev-parse", "HEAD~1"])
    if code == 0 and out:
        return out
    return None


def blob_line_count(rev: str, rel_path: str) -> int | None:
    code, _ = run(["git", "cat-file", "-e", f"{rev}:{rel_path}"])
    if code != 0:
        return None
    code, out = run(["git", "show", f"{rev}:{rel_path}"])
    if code != 0:
        return None
    return len(out.splitlines())


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


def diff_numstat(base: str) -> dict[str, tuple[int, int]]:
    code, out = run(["git", "diff", "--numstat", base, "--"])
    if code != 0:
        return {}
    stats: dict[str, tuple[int, int]] = {}
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) != 3:
            continue
        added_raw, deleted_raw, path = parts
        if added_raw == "-" or deleted_raw == "-":
            continue
        try:
            added = int(added_raw)
            deleted = int(deleted_raw)
        except ValueError:
            continue
        stats[path] = (added, deleted)
    return stats


def untracked_files() -> set[str]:
    code, out = run(["git", "ls-files", "--others", "--exclude-standard"])
    if code != 0:
        return set()
    return {line.strip() for line in out.splitlines() if line.strip()}


def companion_paths(monolith_rel: str) -> list[str]:
    monolith_path = Path(monolith_rel)
    parent = monolith_path.parent.as_posix()
    name = monolith_path.name

    # `*/mod.rs` decomposition typically extracts into sibling files under the
    # same module directory.
    if name == "mod.rs":
        return [
            p.relative_to(ROOT).as_posix()
            for p in (ROOT / parent).glob("*.rs")
            if p.relative_to(ROOT).as_posix() != monolith_rel and p.name != "mod.rs"
        ]

    # Flat monolith decomposition typically extracts into same-dir stem shards
    # such as `command_project_*.rs`, `main_*.rs`, `cli_args_*.rs`.
    stem = monolith_path.stem
    prefix = f"{parent}/{stem}_"
    return [
        p.relative_to(ROOT).as_posix()
        for p in (ROOT / parent).glob(f"{stem}_*.rs")
        if p.relative_to(ROOT).as_posix() != monolith_rel
        and p.relative_to(ROOT).as_posix().startswith(prefix)
        and p.name.endswith(".rs")
    ]


def companion_added_lines(
    monolith_rel: str,
    stats: dict[str, tuple[int, int]],
    untracked: set[str],
) -> int:
    total = 0
    for path in companion_paths(monolith_rel):
        if path in stats:
            total += stats[path][0]
            continue
        if path in untracked:
            full = ROOT / path
            if full.exists():
                total += line_count(full)
    return total


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def main() -> int:
    touched = changed_files()
    base = merge_base_commit()
    stats = diff_numstat(base) if base else {}
    untracked = untracked_files()
    violations: list[str] = []
    checked = 0

    if not base:
        print(
            "Touched monolith burn-down check failed: unable to resolve base commit.",
            file=sys.stderr,
        )
        return 1

    for rel, baseline in MONOLITH_BASELINES.items():
        if rel not in touched:
            continue
        checked += 1
        path = ROOT / rel
        if not path.exists():
            violations.append(f"{rel}: missing file (baseline {baseline})")
            continue

        before = blob_line_count(base, rel)
        if before is None:
            violations.append(
                f"{rel}: not present at base commit {base[:12]} (cannot evaluate burn-down)"
            )
            continue

        current = line_count(path)
        if current >= before:
            violations.append(
                f"{rel}: no burn-down vs base ({current} lines >= base {before})"
            )
        else:
            reduction = before - current
            extracted = companion_added_lines(rel, stats, untracked)
            if extracted <= 0:
                violations.append(
                    f"{rel}: burn-down lacks structural extraction evidence "
                    f"(reduced {reduction}, companion shard additions {extracted})"
                )
            elif extracted * 100 < reduction * 70:
                violations.append(
                    f"{rel}: burn-down appears compression-heavy; "
                    f"companion extraction {extracted} < 70% of reduced {reduction}"
                )
        if current > baseline:
            violations.append(f"{rel}: {current} lines > baseline {baseline}")

    if violations:
        print("Touched monolith burn-down check failed:", file=sys.stderr)
        print(
            "  Touched monolith files must shrink versus branch base and show "
            "companion shard extraction, not pure deletion.",
            file=sys.stderr,
        )
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        return 1

    print(
        "Touched monolith burn-down check passed "
        f"({checked} touched monolith files checked vs base {base[:12]})."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
