#!/usr/bin/env python3
"""Fail when MCP test modules exceed the configured line-count limit."""

from __future__ import annotations

import argparse
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Enforce max line count for mcp-server test modules."
    )
    parser.add_argument(
        "--max-lines",
        type=int,
        default=700,
        help="Maximum allowed line count per test file (default: 700).",
    )
    args = parser.parse_args()

    test_dir = ROOT / "mcp-server"
    test_files = sorted(test_dir.glob("test_*.py"))

    violations: list[tuple[Path, int]] = []
    for path in test_files:
        line_count = len(path.read_text().splitlines())
        if line_count > args.max_lines:
            violations.append((path, line_count))

    if violations:
        print(
            f"Test file size check failed: {len(violations)} file(s) exceed {args.max_lines} lines.",
            file=sys.stderr,
        )
        for path, line_count in violations:
            rel = path.relative_to(ROOT)
            print(f"  - {rel}: {line_count} lines", file=sys.stderr)
        return 1

    print(
        f"Test file size check passed ({len(test_files)} files, max {args.max_lines} lines)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
