"""Terminal cancellation validation; historical proof is never fabricated."""
from __future__ import annotations

import json
import subprocess
from pathlib import Path


def validate_cancellation(root: Path, item: dict, governed: dict) -> list[str]:
    from project_work_authority import reference_text

    record = item.get("cancellation")
    canceled = item.get("state") == "canceled"
    key = item.get("key", "?")
    if not canceled:
        return [f"{key}: cancellation requires terminal canceled state"] if record is not None else []
    failures = []
    if (not isinstance(record, dict) or set(record) != {"reason", "source_ref"}
            or not isinstance(record.get("reason"), str) or not record["reason"].strip()
            or reference_text(root, record.get("source_ref"), governed) is None):
        failures.append(f"{key}: canceled work requires reason and governed owner disposition")
    if item.get("canonical_next") or item.get("authorization") != "none":
        failures.append(f"{key}: canceled work cannot be selected or authorized")
    if item.get("claim") is not None or item.get("hold") is not None:
        failures.append(f"{key}: canceled work cannot retain a claim or hold")
    return failures


def validate_cancellation_history(root: Path, manifest: dict) -> list[str]:
    """A committed cancellation cannot disappear or silently become active again."""
    current = {}
    current_items = manifest.get("frontier")
    if not isinstance(current_items, list):
        return ["cannot verify cancellation history without a current frontier list"]
    for item in current_items:
        if isinstance(item, dict) and isinstance(item.get("issue_id"), str):
            current[item["issue_id"]] = item
    failures = []
    for revision in ("HEAD", "HEAD^"):
        result = subprocess.run(
            ["git", "show", f"{revision}:specs/active_frontier.json"], cwd=root,
            text=True, capture_output=True,
        )
        if result.returncode:
            if revision == "HEAD":
                failures.append("cannot verify committed cancellation history")
            continue
        try:
            previous = json.loads(result.stdout)
        except ValueError:
            previous = None
        if not isinstance(previous, dict) or not isinstance(previous.get("frontier"), list):
            failures.append("cannot parse committed cancellation history")
            continue
        for old in previous["frontier"]:
            if not isinstance(old, dict) or not isinstance(old.get("issue_id"), str):
                failures.append("invalid committed cancellation history identity")
                continue
            if old.get("state") != "canceled":
                continue
            new = current.get(old["issue_id"], {})
            if new.get("state") != "canceled" or new.get("cancellation") != old.get("cancellation"):
                failures.append(
                    f"{old.get('key', '?')}: committed cancellation must be preserved; "
                    "renewed owner-authorized work requires a separate task"
                )
    return failures
