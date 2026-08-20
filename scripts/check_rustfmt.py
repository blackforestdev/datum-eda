#!/usr/bin/env python3
"""Drift gate: the workspace must be rustfmt-clean.

Runs `cargo fmt --all -- --check` and fails on any diff, except for files listed
in `specs/rustfmt_exemption_manifest.json`. That list exists only for modules
whose rustfmt reflow would exceed an exact source-health ceiling (decision 022)
and is downward-only: an exempted file that is actually rustfmt-clean is a
stale exemption and fails the gate until its row is removed.

`--staged` restricts failures to files staged for commit (for the pre-commit
hook): another agent's dirty-but-unstaged files never block a commit, and the
stale-exemption check is skipped because it is a handoff concern, not a
per-commit one. Staged mode runs rustfmt directly on the staged files rather
than through `cargo fmt`, so a new file not yet reachable from a crate root is
still checked. rustfmt inspects the working tree, so a file staged with
further unstaged edits is judged by its working-tree content.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "specs" / "rustfmt_exemption_manifest.json"
DIFF_LINE = re.compile(r"^Diff in (?P<path>.+?)(?: at line \d+|:\d+):$")


def load_exemptions() -> dict[str, dict]:
    data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if data.get("schema_version") != 1:
        raise SystemExit(f"{MANIFEST}: unsupported schema_version {data.get('schema_version')!r}")
    exemptions = data.get("exemptions")
    if not isinstance(exemptions, dict):
        raise SystemExit(f"{MANIFEST}: 'exemptions' must be an object")
    for rel, row in exemptions.items():
        if not (ROOT / rel).is_file():
            raise SystemExit(f"{MANIFEST}: exempted file does not exist: {rel}")
        if not isinstance(row, dict) or not row.get("reason"):
            raise SystemExit(f"{MANIFEST}: exemption for {rel} needs a 'reason'")
    return exemptions


def rustfmt_dirty_files(files: list[str] | None = None) -> set[str]:
    """Whole-workspace check via cargo fmt, or a direct rustfmt check of `files`."""
    if files is not None:
        if not files:
            return set()
        cmd = ["rustfmt", "--edition", "2024", "--check", *files]
    else:
        cmd = ["cargo", "fmt", "--all", "--", "--check"]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    dirty: set[str] = set()
    for line in proc.stdout.splitlines():
        match = DIFF_LINE.match(line)
        if match:
            path = Path(match.group("path"))
            try:
                rel = path.resolve().relative_to(ROOT)
            except ValueError:
                rel = path
            dirty.add(rel.as_posix())
    if proc.returncode != 0 and not dirty:
        # rustfmt failed for a reason other than formatting diffs (parse error, etc).
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise SystemExit(f"rustfmt gate: `{' '.join(cmd[:4])} ...` failed")
    return dirty


def staged_rust_files() -> set[str]:
    proc = subprocess.run(
        ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR", "--", "*.rs"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return {line.strip() for line in proc.stdout.splitlines() if line.strip()}


def select_violations(
    dirty: set[str], exemptions: set[str], staged: set[str] | None
) -> tuple[list[str], list[str]]:
    """Return (violations, stale_exemptions). staged=None means whole-tree mode."""
    violations = dirty - exemptions
    if staged is not None:
        return sorted(violations & staged), []
    return sorted(violations), sorted(exemptions - dirty)


def main() -> int:
    staged_only = "--staged" in sys.argv[1:]
    exemptions = load_exemptions()
    staged = staged_rust_files() if staged_only else None
    dirty = rustfmt_dirty_files(sorted(staged) if staged is not None else None)

    violations, stale = select_violations(dirty, set(exemptions), staged)

    if violations:
        scope = "staged files are" if staged_only else "files are"
        print(f"rustfmt gate FAILED: the following {scope} not rustfmt-clean:")
        for rel in violations:
            print(f"  - {rel}")
        print("Run `cargo fmt --all` (or rustfmt on the listed files) and re-run.")
    if stale:
        print("rustfmt gate FAILED: stale exemptions (file is now rustfmt-clean; remove its row):")
        for rel in stale:
            print(f"  - {rel}")
    if violations or stale:
        return 1

    scope = "staged files" if staged_only else "workspace"
    print(
        f"rustfmt gate passed for {scope} ({len(exemptions)} ceiling-bound "
        f"exemption(s) in {MANIFEST.relative_to(ROOT)})."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
