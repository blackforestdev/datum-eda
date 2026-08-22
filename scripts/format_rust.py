#!/usr/bin/env python3
"""Format exactly the Rust source files named on the command line.

Unlike ``cargo fmt -- <paths>``, this command never gives Cargo or rustfmt a
workspace path to discover. Each source is formatted through stdin and only the
corresponding explicitly requested file can be written back.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys
from collections.abc import Callable


ROOT = Path(__file__).resolve().parent.parent


class FormatError(Exception):
    """The requested exact-file formatting operation is invalid or failed."""


def resolve_files(arguments: list[str], root: Path = ROOT) -> list[Path]:
    if not arguments:
        raise FormatError("name at least one Rust source file")

    root = root.resolve()
    resolved: list[Path] = []
    seen: set[Path] = set()
    for argument in arguments:
        candidate = Path(argument)
        if not candidate.is_absolute():
            candidate = root / candidate
        candidate = candidate.resolve()
        try:
            candidate.relative_to(root)
        except ValueError as exc:
            raise FormatError(f"path is outside the repository: {argument}") from exc
        if candidate.suffix != ".rs":
            raise FormatError(f"not a Rust source file: {argument}")
        if not candidate.is_file():
            raise FormatError(f"Rust source file does not exist: {argument}")
        if candidate not in seen:
            seen.add(candidate)
            resolved.append(candidate)
    return resolved


def rustfmt_source(source: bytes) -> bytes:
    process = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=source,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if process.returncode != 0:
        detail = process.stderr.decode("utf-8", errors="replace").strip()
        raise FormatError(f"rustfmt failed: {detail or 'no diagnostic'}")
    return process.stdout


def format_files(
    paths: list[Path],
    *,
    check: bool,
    formatter: Callable[[bytes], bytes] = rustfmt_source,
) -> list[Path]:
    formatted: list[tuple[Path, bytes]] = []
    changed: list[Path] = []
    for path in paths:
        source = path.read_bytes()
        output = formatter(source)
        if output != source:
            changed.append(path)
            formatted.append((path, output))

    if not check:
        for path, output in formatted:
            path.write_bytes(output)
    return changed


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Format only the explicitly named Rust source files."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="report files that need formatting without changing them",
    )
    parser.add_argument("files", nargs="*", help="repository-relative .rs paths")
    args = parser.parse_args()

    try:
        paths = resolve_files(args.files)
        changed = format_files(paths, check=args.check)
    except FormatError as exc:
        print(f"exact-file rustfmt: {exc}", file=sys.stderr)
        return 2

    if args.check and changed:
        print("exact-file rustfmt check FAILED:")
        for path in changed:
            print(f"  - {path.relative_to(ROOT)}")
        return 1
    if args.check:
        print(f"exact-file rustfmt check passed ({len(paths)} file(s))")
    else:
        print(f"exact-file rustfmt formatted {len(changed)} of {len(paths)} file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
