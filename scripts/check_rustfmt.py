#!/usr/bin/env python3
"""Drift gate: the workspace must be rustfmt-clean.

Runs `cargo fmt --all -- --check` and fails on any diff, except for files listed
in `specs/rustfmt_exemption_manifest.json`. That list exists only for modules
whose rustfmt reflow would exceed an exact source-health ceiling (decision 022)
and is downward-only: an exempted file that is actually rustfmt-clean is a
stale exemption and fails the gate until its row is removed.
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


def rustfmt_dirty_files() -> set[str]:
    proc = subprocess.run(
        ["cargo", "fmt", "--all", "--", "--check"],
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
        raise SystemExit("rustfmt gate: `cargo fmt --all -- --check` failed")
    return dirty


def main() -> int:
    exemptions = load_exemptions()
    dirty = rustfmt_dirty_files()

    violations = sorted(dirty - set(exemptions))
    stale = sorted(set(exemptions) - dirty)

    if violations:
        print("rustfmt gate FAILED: the following files are not rustfmt-clean:")
        for rel in violations:
            print(f"  - {rel}")
        print("Run `cargo fmt --all` (or rustfmt on the listed files) and re-run.")
    if stale:
        print("rustfmt gate FAILED: stale exemptions (file is now rustfmt-clean; remove its row):")
        for rel in stale:
            print(f"  - {rel}")
    if violations or stale:
        return 1

    print(
        f"rustfmt gate passed ({len(exemptions)} ceiling-bound exemption(s) "
        f"in {MANIFEST.relative_to(ROOT)})."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
