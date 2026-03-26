#!/usr/bin/env python3
"""Fail when test modules or inline test blocks exceed configured limits."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]

# Temporary freeze caps for known large inline-test tails. These values should
# only move downward as files are decomposed.
INLINE_TEST_TAIL_ALLOWLIST: dict[str, int] = {
    "crates/engine/src/api/mod.rs": 25,
    "crates/engine-daemon/src/main.rs": 53,
    "crates/engine/src/erc/mod.rs": 8,
    "crates/engine/src/board/mod.rs": 8,
    "crates/engine/src/connectivity/mod.rs": 8,
    "crates/engine/src/import/kicad/mod.rs": 33,
    "crates/engine/src/import/eagle/mod.rs": 39,
    "crates/engine/src/import/mod.rs": 3,
    "crates/engine/src/import/ids_sidecar.rs": 3,
    "crates/engine/src/drc/mod.rs": 48,
    "crates/engine/src/schematic/mod.rs": 7,
    "crates/engine/src/pool/mod.rs": 3,
    "crates/cli/src/main.rs": 2,
}

# Temporary freeze caps for known oversized dedicated Rust test modules.
# These values should only move downward as files are decomposed.
RUST_TEST_FILE_ALLOWLIST: dict[str, int] = {}


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def inline_test_tail_lines(path: Path) -> int:
    for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.strip().startswith("#[cfg(test)]"):
            return line_count(path) - line_no + 1
    return 0


def rust_test_module_candidate(path: Path) -> bool:
    name = path.name.lower()
    return "test" in name or "tests" in path.parts


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Enforce max line count for test modules and inline-test tails "
            "across mcp-server and Rust crates."
        )
    )
    parser.add_argument(
        "--max-lines",
        type=int,
        default=700,
        help="Maximum allowed line count per dedicated test file (default: 700).",
    )
    parser.add_argument(
        "--inline-test-max-lines",
        type=int,
        default=350,
        help=(
            "Maximum allowed inline-test tail lines (from #[cfg(test)] to EOF) "
            "for non-allowlisted Rust files (default: 350)."
        ),
    )
    args = parser.parse_args()

    mcp_test_files = sorted((ROOT / "mcp-server").glob("test_*.py"))
    rust_test_files = sorted(
        path
        for path in (ROOT / "crates").glob("**/*.rs")
        if rust_test_module_candidate(path)
    )
    rust_sources = sorted((ROOT / "crates").glob("**/*.rs"))

    test_file_violations: list[tuple[Path, int, int, str]] = []
    for path in mcp_test_files + rust_test_files:
        lines = line_count(path)
        rel = str(path.relative_to(ROOT))
        rust_allow_budget = RUST_TEST_FILE_ALLOWLIST.get(rel)
        if lines > args.max_lines:
            category = "mcp_test_file" if path.suffix == ".py" else "rust_test_file"
            if path.suffix == ".rs" and rust_allow_budget is not None and lines <= rust_allow_budget:
                continue
            budget = rust_allow_budget if rust_allow_budget is not None else args.max_lines
            test_file_violations.append((path, lines, budget, category))

    inline_tail_violations: list[tuple[Path, int, int]] = []
    for path in rust_sources:
        rel = str(path.relative_to(ROOT))
        tail_lines = inline_test_tail_lines(path)
        if tail_lines == 0:
            continue
        budget = INLINE_TEST_TAIL_ALLOWLIST.get(rel, args.inline_test_max_lines)
        if tail_lines > budget:
            inline_tail_violations.append((path, tail_lines, budget))

    if test_file_violations or inline_tail_violations:
        print("Test size guardrail check failed.", file=sys.stderr)
        if test_file_violations:
            print(
                f"  Dedicated test files over {args.max_lines} lines: "
                f"{len(test_file_violations)}",
                file=sys.stderr,
            )
            for path, lines, budget, category in test_file_violations:
                rel = path.relative_to(ROOT)
                print(
                    f"    - [{category}] {rel}: {lines} lines > budget {budget}",
                    file=sys.stderr,
                )
        if inline_tail_violations:
            print(
                "  Inline Rust test tails over budget: "
                f"{len(inline_tail_violations)}",
                file=sys.stderr,
            )
            for path, tail, budget in inline_tail_violations:
                rel = path.relative_to(ROOT)
                print(
                    f"    - [inline_test_tail] {rel}: {tail} lines > budget {budget}",
                    file=sys.stderr,
                )
        return 1

    print(
        "Test file size check passed "
        f"({len(mcp_test_files)} mcp test files, {len(rust_test_files)} rust test files, "
        f"{len(RUST_TEST_FILE_ALLOWLIST)} rust test-file freezes, "
        f"{len(INLINE_TEST_TAIL_ALLOWLIST)} inline-tail freezes)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
