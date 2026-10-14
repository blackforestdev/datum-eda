"""Locate historical source pins after an owner-approved artifact-only rewrite.

The map changes where retained source can be found. It never changes the
historical evidence's source ID, acceptance status, or missing raw proof.
"""

from pathlib import Path
import json
import re
import subprocess


def _exists(root: Path, revision: str) -> bool:
    return subprocess.run(
        ["git", "cat-file", "-e", f"{revision}^{{commit}}"], cwd=root,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
    ).returncode == 0


def locate_commit(root: Path, revision: str) -> str | None:
    if _exists(root, revision):
        return revision
    if not re.fullmatch(r"[0-9a-f]{7,40}", revision):
        return None
    try:
        record = json.loads((root / "specs/history_commit_map.json").read_text())
        if not isinstance(record, dict) or record.get("schema_version") != 1:
            return None
        entries = record["commits"]
        if not isinstance(entries, dict) or any(
            not re.fullmatch(r"[0-9a-f]{40}", old)
            or not isinstance(new, str) or not re.fullmatch(r"[0-9a-f]{40}", new)
            for old, new in entries.items()
        ):
            return None
        candidates = [new for old, new in entries.items() if old.startswith(revision)]
        if len(candidates) != 1 or not _exists(root, candidates[0]):
            return None
        return candidates[0]
    except (OSError, ValueError, TypeError, KeyError):
        return None


def commit_exists(root: Path, revision: str) -> bool:
    return locate_commit(root, revision) is not None
