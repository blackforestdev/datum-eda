#!/usr/bin/env python3
"""Reject temporary proof images and compressed capture archives in Git."""

from __future__ import annotations

import argparse
import json
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


def contains_raw_samples(value: object, keys: tuple[str, ...] = ("drm_samples",)) -> bool:
    """Raw observer arrays are temporary capture payloads."""
    if isinstance(value, dict):
        if any(isinstance(value.get(key), list) and value[key] for key in keys):
            return True
        return any(contains_raw_samples(child, keys) for child in value.values())
    if isinstance(value, list):
        return any(contains_raw_samples(child, keys) for child in value)
    return False


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
    for path in paths:
        if not path.lower().startswith("docs/reviews/") or not path.lower().endswith(".json"):
            continue
        payload = subprocess.run(["git", "show", f":{path}"], capture_output=True, check=False)
        if payload.returncode:
            print(f"temporary-artifact gate failed: cannot read indexed record {path!r}")
            return 2
        try:
            value = json.loads(payload.stdout)
        except (ValueError, UnicodeDecodeError):
            # This gate checks capture storage; schema validity belongs to the
            # owning record reader. Non-JSON fixtures remain outside this rule.
            continue
        if contains_raw_samples(value):
            forbidden.append(path)
        elif contains_raw_samples(value, ("raw_counters",)):
            # Preserve unchanged source-pinned historical receipts. New or
            # changed records must retain summaries rather than raw captures.
            original = subprocess.run(["git", "show", f"HEAD:{path}"],
                                      capture_output=True, check=False)
            try:
                unchanged = original.returncode == 0 and json.loads(original.stdout) == value
            except (ValueError, UnicodeDecodeError):
                unchanged = False
            if not unchanged:
                forbidden.append(path)
    if forbidden:
        print("temporary-artifact gate refused tracked proof output:")
        for path in forbidden:
            print(f"  {path!r}")
        print("Keep captures in external development workspaces; retain concise text findings in Git.")
        print("See docs/reviews/README.md. Forced adds do not override this policy.")
        return 1
    print("temporary-artifact gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
