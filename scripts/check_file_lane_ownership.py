#!/usr/bin/env python3
"""Refuse staged changes outside the committing session's owned file lane."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from dataclasses import dataclass


VISUAL_TRUTH_MARKER = "DATUM_VISUAL_TRUTH_LANE"
VISUAL_TRUTH_VALUE = "claude"
VISUAL_TRUTH_PREFIX = b"docs/gui/prototypes/"


class StagedPathError(ValueError):
    """The staged Git record stream was unavailable or malformed."""


@dataclass(frozen=True)
class StagedChange:
    status: str
    paths: tuple[bytes, ...]


def parse_name_status_z(raw: bytes) -> list[StagedChange]:
    """Parse ``git diff --name-status -z`` without losing unusual filenames."""
    if not raw:
        return []
    if not raw.endswith(b"\0"):
        raise StagedPathError("staged Git record stream is not NUL-terminated")

    fields = raw[:-1].split(b"\0")
    changes: list[StagedChange] = []
    index = 0
    while index < len(fields):
        status_raw = fields[index]
        index += 1
        try:
            status = status_raw.decode("ascii")
        except UnicodeDecodeError as exc:
            raise StagedPathError("staged Git status is not ASCII") from exc
        if not status or status[0] not in "ACMRD":
            raise StagedPathError(f"unexpected staged Git status {status!r}")

        path_count = 2 if status[0] in "CR" else 1
        if index + path_count > len(fields):
            raise StagedPathError(f"status {status!r} has incomplete path records")
        paths = tuple(fields[index : index + path_count])
        if any(not path for path in paths):
            raise StagedPathError(f"status {status!r} has an empty path record")
        index += path_count
        changes.append(StagedChange(status=status, paths=paths))
    return changes


def staged_changes() -> list[StagedChange]:
    command = [
        "git",
        "diff",
        "--cached",
        "--name-status",
        "-z",
        "--find-renames",
        "--diff-filter=ACMRD",
        "--",
    ]
    try:
        result = subprocess.run(command, check=False, capture_output=True)
    except OSError as exc:
        raise StagedPathError(f"could not execute Git: {exc}") from exc
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise StagedPathError(
            f"Git could not inspect the staged index (exit {result.returncode}): {detail}"
        )
    return parse_name_status_z(result.stdout)


def is_visual_truth_path(path: bytes) -> bool:
    return path.startswith(VISUAL_TRUTH_PREFIX) and path.lower().endswith(b".html")


def protected_paths(changes: list[StagedChange]) -> list[bytes]:
    return sorted({path for change in changes for path in change.paths if is_visual_truth_path(path)})


def display_path(path: bytes) -> str:
    return repr(os.fsdecode(path))


def check_staged(marker: str | None = None) -> int:
    try:
        paths = protected_paths(staged_changes())
    except StagedPathError as exc:
        print(f"visual-truth file-lane gate failed closed: {exc}", file=sys.stderr)
        return 2

    if not paths:
        print("visual-truth file-lane gate passed (no protected staged paths).")
        return 0

    active_marker = os.environ.get(VISUAL_TRUTH_MARKER) if marker is None else marker
    rendered_paths = "\n".join(f"  - {display_path(path)}" for path in paths)
    if active_marker != VISUAL_TRUTH_VALUE:
        print(
            "visual-truth file-lane gate refused this commit.\n"
            "docs/gui/prototypes/**/*.html is the Claude-owned visual-truth lane.\n"
            f"Protected staged paths:\n{rendered_paths}\n"
            "Unstage or revert only your own changes and send Claude a bounded "
            "reconciliation list. Codex and other lanes must not set or forward "
            f"{VISUAL_TRUTH_MARKER}.",
            file=sys.stderr,
        )
        return 1

    print(
        "visual-truth file-lane gate passed for the marked Claude session:\n"
        f"{rendered_paths}"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--staged",
        action="store_true",
        required=True,
        help="inspect only paths in the staged Git index",
    )
    parser.parse_args()
    return check_staged()


if __name__ == "__main__":
    raise SystemExit(main())
