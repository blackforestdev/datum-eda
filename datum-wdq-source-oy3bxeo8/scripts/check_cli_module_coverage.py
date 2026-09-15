#!/usr/bin/env python3
"""Orphaned-source guard for the CLI crate.

Every .rs file under crates/cli/src must be reachable from the crate's
module graph. A file whose mod declaration is dropped (e.g. during a
directory reorganization) silently stops compiling — the build stays
green while its code and tests vanish. This gate compares the on-disk
source set against cargo's dep-info from the most recent test
compilation (which includes #[cfg(test)] modules).

Requires a prior `cargo build`/`cargo test` of datum-eda-cli; in the
drift-gate runner it executes after the migration proof gates, which
build the test binary.
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crates/cli/src"
DEPS = ROOT / "target/debug/deps"
BIN_DEP = ROOT / "target/debug/datum-eda.d"


def dep_sources(dep_file: Path) -> set[str]:
    text = dep_file.read_text()
    return {tok for tok in text.replace("\\\n", " ").split() if tok.endswith(".rs")}


def newest_test_dep_info() -> Path | None:
    candidates = [
        p
        for p in DEPS.glob("datum_eda-*.d")
        if "main_tests" in p.read_text()
    ]
    if not candidates:
        return None
    return max(candidates, key=lambda p: p.stat().st_mtime)


def main() -> int:
    if not SRC.exists():
        print("missing crates/cli/src", file=sys.stderr)
        return 1

    compiled: set[str] = set()
    test_dep = newest_test_dep_info()
    for dep_file in filter(None, [test_dep, BIN_DEP if BIN_DEP.exists() else None]):
        for tok in dep_sources(dep_file):
            path = (ROOT / tok) if not Path(tok).is_absolute() else Path(tok)
            try:
                compiled.add(str(path.resolve().relative_to(ROOT)))
            except ValueError:
                continue

    if not compiled:
        print(
            "CLI module coverage: no dep-info found under target/debug; "
            "run `cargo test -p datum-eda-cli --no-run` first",
            file=sys.stderr,
        )
        return 1

    on_disk = {
        str(p.relative_to(ROOT))
        for p in SRC.rglob("*.rs")
    }
    orphans = sorted(on_disk - compiled)
    if orphans:
        print(
            "CLI module coverage FAILED — source files not compiled into the "
            "crate (missing mod declaration?):",
            file=sys.stderr,
        )
        for orphan in orphans:
            print(f"  {orphan}", file=sys.stderr)
        return 1

    print(
        f"CLI module coverage passed ({len(on_disk)} source files, "
        f"all present in dep-info)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
