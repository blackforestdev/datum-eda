#!/usr/bin/env python3
"""Reject temporary proof images and compressed capture archives in Git."""

from __future__ import annotations

import argparse
import os
import subprocess


ARCHIVES = (".gz", ".xz", ".zip", ".tar", ".tgz", ".7z", ".bz2", ".zst", ".zstd", ".rar")
CAPTURES = (
    ".png", ".jpg", ".jpeg", ".webp", ".gif", ".avif", ".bmp", ".tif", ".tiff",
    ".apng", ".heic", ".mp4", ".webm", ".mov", ".mkv", ".avi",
)
DESIGN_REFERENCES = {
    "docs/gui/reference/board-editor.png",
    "docs/gui/reference/selection-study.png",
}
RAW_PROOF = (".log", ".stdout", ".stderr", ".jsonl")


def is_temporary_artifact(path: str) -> bool:
    lower = path.lower()
    if lower.endswith(ARCHIVES):
        return True
    if lower.startswith("docs/reviews/") and lower.endswith(RAW_PROOF):
        return True
    if lower.startswith("docs/reviews/") and (
        lower.endswith(("-atspi.json", "-windows.json"))
        or (lower.startswith("docs/reviews/workflow-delivery-pilot/") and lower.endswith("-state.json"))
    ):
        return True
    if not lower.endswith(CAPTURES):
        return False
    if path in DESIGN_REFERENCES:
        return False
    if path.startswith("crates/gui-render/testdata/golden/") and path.endswith(".golden.png"):
        return False
    # Authored product images have an explicit asset home; arbitrary evidence
    # directories (including new ones) are not an image storage exemption.
    if path.startswith("crates/") and "/assets/" in path:
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staged", action="store_true", help="check staged additions and modifications")
    args = parser.parse_args()
    command = (
        ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z", "--"]
        if args.staged
        else ["git", "ls-files", "-z"]
    )
    result = subprocess.run(command, capture_output=True, check=False)
    if result.returncode:
        print("temporary-artifact gate failed: cannot inspect Git index")
        return 2
    paths = [os.fsdecode(path) for path in result.stdout.split(b"\0") if path]
    forbidden = sorted(path for path in paths if is_temporary_artifact(path))
    if forbidden:
        print("temporary-artifact gate refused tracked proof output:")
        for path in forbidden:
            print(f"  {path!r}")
        print("Keep captures in ignored target/ output; retain concise text findings in Git.")
        print("See docs/reviews/README.md. Forced adds do not override this policy.")
        return 1
    print("temporary-artifact gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
