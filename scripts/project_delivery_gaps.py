"""Resolve required delivery corrections through existing tracker blockers."""
from __future__ import annotations

import re

REQUIRED_LABEL = "delivery:required"
TERMINAL_STATES = {"landed", "canceled"}


def required(issue: dict) -> bool:
    labels = issue.get("labels", [])
    return isinstance(labels, list) and REQUIRED_LABEL in labels


def blocker_ids(issue: dict) -> set[str]:
    return {edge["depends_on_id"] for edge in issue.get("dependencies", [])
            if isinstance(edge, dict) and edge.get("type") == "blocks"
            and isinstance(edge.get("depends_on_id"), str)}


def unresolved(ids: set[str], issues: dict, canceled: set[str]) -> list[str]:
    return sorted(key for key in ids
                  if key in canceled or issues.get(key, {}).get("status") != "closed")


def checklist_errors(issue: dict) -> list[str]:
    checklist = issue.get("acceptance_criteria") or ""
    if not isinstance(checklist, str):
        checklist = ""
    boxes = re.findall(r"^\s*[-*]\s+\[([ xX])\]\s+\S", checklist, re.M)
    if not boxes:
        return [f"{issue['id']}: required delivery correction needs a closure checklist"]
    if issue.get("status") == "closed":
        if any(box == " " for box in boxes):
            return [f"{issue['id']}: cannot close required correction with unchecked criteria"]
        if not str(issue.get("close_reason") or "").strip():
            return [f"{issue['id']}: required correction needs landing/disposition evidence in close_reason"]
    return []


def criterion_text(issue: dict) -> list[str]:
    value = issue.get("acceptance_criteria") or ""
    if not isinstance(value, str):
        return []
    return [" ".join(text.split()) for text in
            re.findall(r"^\s*[-*]\s+\[[ xX]\]\s+(.+)$", value, re.M)]


def validate_delivery_gaps(manifest: dict, issues: dict) -> list[str]:
    failures = []
    covered = {dependency for item in manifest.get("frontier", [])
               if item.get("state") not in TERMINAL_STATES
               for dependency in item.get("dependencies", [])}
    canceled = {item["issue_id"] for item in manifest.get("frontier", [])
                if item.get("state") == "canceled"}
    for key, issue in issues.items():
        if not required(issue):
            continue
        failures.extend(checklist_errors(issue))
        if issue.get("status") != "closed" and key not in covered:
            failures.append(f"{key}: required delivery correction is not a hard prerequisite of a nonterminal Frontier item")
    for item in manifest.get("frontier", []):
        if item.get("state") == "landed":
            required_ids = {key for key in item.get("dependencies", [])
                            if required(issues.get(key, {}))}
            pending = unresolved(required_ids, issues, canceled)
            if pending:
                failures.append(f"{item['key']}: landed delivery has unresolved required corrections: {', '.join(pending)}")
        contract = item.get("feature_delivery")
        if not isinstance(contract, dict) or not contract.get("development_ready"):
            continue
        pending = unresolved(set(item.get("dependencies", [])), issues, canceled)
        if pending:
            failures.append(f"{item['key']}: development availability has unresolved prerequisites: {', '.join(pending)}")
    return failures


def gap_view(item: dict, issues: dict) -> list[dict]:
    return [{"issue_id": key, "title": issues.get(key, {}).get("title", key),
             "status": issues.get(key, {}).get("status", "missing")}
            for key in sorted(item.get("dependencies", []))
            if required(issues.get(key, {})) and issues[key].get("status") != "closed"]


def gap_lines(view: dict, *, full: bool = False) -> list[str]:
    gaps = view.get("delivery_gaps", [])
    if not gaps:
        return []
    lines = [f"Required delivery corrections: {len(gaps)} unresolved (hard prerequisites)."]
    lines += [f"- {gap['issue_id']}: {gap['title']} [{gap['status']}]"
              for gap in (gaps if full else gaps[:3])]
    if not full and len(gaps) > 3:
        lines.append("Remaining corrections are listed in the full task record.")
    return lines


def transition_errors(before: dict, after: dict, old_issues: dict, issues: dict) -> list[str]:
    """Validate newly admitted claims, preserving existing historical closures."""
    failures = []
    previous = {item["issue_id"]: item for item in before.get("frontier", [])}
    candidates = {item["issue_id"]: item for item in after.get("frontier", [])}
    canceled = {item["issue_id"] for item in after.get("frontier", [])
                if item.get("state") == "canceled"}
    for key, old in old_issues.items():
        if required(old) and old.get("status") != "closed":
            candidate = issues.get(key)
            if candidate is None or not required(candidate):
                failures.append(f"{key}: unresolved required correction cannot be deleted or unmarked")
            elif criterion_text(old) != criterion_text(candidate):
                failures.append(f"{key}: recorded correction criteria cannot be replaced while unresolved")
    for key, old_item in previous.items():
        if old_item.get("state") in TERMINAL_STATES:
            continue
        candidate_ids = set(candidates.get(key, {}).get("dependencies", []))
        for dependency in set(old_item.get("dependencies", [])) - candidate_ids:
            if required(old_issues.get(dependency, {})) and unresolved({dependency}, issues, canceled):
                failures.append(f"{old_item['key']}: unresolved required prerequisite cannot be detached: {dependency}")
    for key, issue in issues.items():
        if issue.get("status") != "closed" or old_issues.get(key, {}).get("status") == "closed":
            continue
        old = old_issues.get(key, {})
        pending = unresolved(blocker_ids(old) | blocker_ids(issue), issues, canceled)
        if pending:
            failures.append(f"{key}: new closure has unresolved prerequisites: {', '.join(pending)}")
        if required(issue) or required(old):
            failures.extend(checklist_errors(issue))
    for item in after.get("frontier", []):
        old = previous.get(item["issue_id"], {})
        admitted = ((item.get("canonical_next") and not old.get("canonical_next"))
                    or (item.get("state") in {"ready", "in_progress", "landed"}
                        and item.get("state") != old.get("state"))
                    or ((item.get("feature_delivery") or {}).get("development_ready")
                        and not (old.get("feature_delivery") or {}).get("development_ready")))
        if admitted:
            pending = unresolved(set(old.get("dependencies", [])) | set(item.get("dependencies", [])), issues, canceled)
            if pending:
                failures.append(f"{item['key']}: new delivery admission has unresolved prerequisites: {', '.join(pending)}")
    return sorted(set(failures))
