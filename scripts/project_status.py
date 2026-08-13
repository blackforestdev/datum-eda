#!/usr/bin/env python3
"""Validate and report Datum's canonical project-management state.

The structured Active Frontier is the roadmap authority.  This tool joins it
to the canonical beads JSONL export and governed-document inventory, renders
the human projection in ``specs/PROGRESS.md``, and returns the one explicitly
selected next task.  It deliberately uses only the Python standard library so
the same checks run in a clean CI checkout without a beads database.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any

from project_task_details import completion_view, render_completion, validate_completion


START_MARKER = "<!-- ACTIVE FRONTIER:START -->"
END_MARKER = "<!-- ACTIVE FRONTIER:END -->"
VALID_STATES = {
    "planned", "specified", "ready", "in_progress", "blocked", "deferred", "landed"
}
VALID_AUTHORIZATIONS = {"planning", "execution", "owner_decision", "none"}
STATE_AUTHORIZATIONS = {
    "planned": {"planning", "execution", "owner_decision", "none"},
    "specified": {"planning", "owner_decision"},
    "ready": {"execution", "none"},
    "in_progress": {"execution", "planning", "owner_decision"},
    "blocked": {"execution", "planning", "owner_decision", "none"},
    "deferred": {"none"},
    "landed": {"none"},
}
ROOT_KEYS = {"schema_version", "policy_decision", "claim_ttl_hours", "frontier"}
ITEM_KEYS = {
    "key", "order", "title", "issue_id", "related_issue_ids", "governing_docs",
    "state", "authorization", "canonical_next", "parallel", "summary",
    "dependencies", "unblocks", "landing_commit", "claim", "completion",
}
REQUIRED_ITEM_KEYS = {
    "key", "order", "title", "issue_id", "governing_docs", "state",
    "authorization", "canonical_next", "parallel", "summary", "dependencies", "unblocks",
}
CLAIM_KEYS = {
    "agent", "harness", "session", "worktree", "head", "claimed_at", "heartbeat_at",
    "expires_at", "scope",
}
ROADMAP_LABELS = {"roadmap:frontier", "roadmap:intake", "roadmap:deferred"}


class StatusError(ValueError):
    """Invalid or inconsistent project-management state."""


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise StatusError(f"cannot read valid JSON from {path}: {exc}") from exc


def nonempty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def timestamp(value: Any, label: str, failures: list[str]) -> datetime | None:
    if not nonempty(value):
        failures.append(f"{label} must be a non-empty ISO-8601 timestamp")
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        failures.append(f"{label} is not a valid ISO-8601 timestamp: {value!r}")
        return None
    if parsed.tzinfo is None:
        failures.append(f"{label} must include a timezone")
        return None
    return parsed.astimezone(timezone.utc)


def load_issues(path: Path) -> tuple[dict[str, dict[str, Any]], list[str]]:
    issues: dict[str, dict[str, Any]] = {}
    failures: list[str] = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        return {}, [f"cannot read tracker {path}: {exc}"]
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            issue = json.loads(line)
        except json.JSONDecodeError as exc:
            failures.append(f"{path}:{number}: invalid JSON: {exc}")
            continue
        issue_id = issue.get("id") if isinstance(issue, dict) else None
        if not nonempty(issue_id):
            failures.append(f"{path}:{number}: issue id must be non-empty")
        elif issue_id in issues:
            failures.append(f"duplicate tracker issue id: {issue_id}")
        else:
            issues[issue_id] = issue
    return issues, failures


def hard_blockers(issue: dict[str, Any], issues: dict[str, dict[str, Any]]) -> set[str]:
    blockers: set[str] = set()
    for edge in issue.get("dependencies", []):
        if not isinstance(edge, dict) or edge.get("type") != "blocks":
            continue
        dependency = edge.get("depends_on_id")
        if nonempty(dependency) and issues.get(dependency, {}).get("status") != "closed":
            blockers.add(dependency)
    return blockers


def validate_claim(
    item: dict[str, Any], issue: dict[str, Any], ttl_hours: int, now: datetime,
    failures: list[str],
) -> None:
    key = item.get("key", "?")
    claim = item.get("claim")
    if item.get("state") != "in_progress":
        if claim is not None:
            failures.append(f"{key}: claim is only allowed for in_progress state")
        return
    if not isinstance(claim, dict):
        failures.append(f"{key}: in_progress item requires a claim object")
        return
    unknown = set(claim) - CLAIM_KEYS
    missing = CLAIM_KEYS - set(claim)
    if unknown:
        failures.append(f"{key}: unknown claim keys: {', '.join(sorted(unknown))}")
    if missing:
        failures.append(f"{key}: missing claim keys: {', '.join(sorted(missing))}")
    for name in ("agent", "harness", "session", "worktree", "head"):
        if not nonempty(claim.get(name)):
            failures.append(f"{key}: claim.{name} must be non-empty")
    scope = claim.get("scope")
    if not isinstance(scope, list) or not scope or any(not nonempty(value) for value in scope):
        failures.append(f"{key}: claim.scope must be a non-empty list of strings")
    claimed = timestamp(claim.get("claimed_at"), f"{key}: claim.claimed_at", failures)
    heartbeat = timestamp(claim.get("heartbeat_at"), f"{key}: claim.heartbeat_at", failures)
    expires = timestamp(claim.get("expires_at"), f"{key}: claim.expires_at", failures)
    if claimed and heartbeat and claimed > heartbeat:
        failures.append(f"{key}: claim heartbeat precedes claim time")
    if heartbeat and expires:
        if heartbeat >= expires:
            failures.append(f"{key}: claim expiry must follow heartbeat")
        if (expires - heartbeat).total_seconds() > ttl_hours * 3600:
            failures.append(f"{key}: claim lease exceeds claim_ttl_hours={ttl_hours}")
    if expires and expires <= now:
        failures.append(f"{key}: claim expired at {claim.get('expires_at')}")
    if heartbeat and heartbeat > now:
        failures.append(f"{key}: claim heartbeat is in the future")
    if issue.get("assignee") != claim.get("agent"):
        failures.append(
            f"{key}: tracker assignee {issue.get('assignee')!r} does not match "
            f"claim agent {claim.get('agent')!r}"
        )


def git_commit_exists(root: Path, revision: str) -> bool:
    result = subprocess.run(
        ["git", "cat-file", "-e", f"{revision}^{{commit}}"], cwd=root,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
    )
    return result.returncode == 0


def validate(root: Path, now: datetime | None = None) -> tuple[list[str], dict[str, Any]]:
    """Return all consistency failures and the loaded state."""
    manifest_path = root / "specs/active_frontier.json"
    try:
        manifest = read_json(manifest_path)
    except StatusError as exc:
        return [str(exc)], {}
    failures: list[str] = []
    if not isinstance(manifest, dict):
        return ["active frontier root must be an object"], {}
    unknown_root = set(manifest) - ROOT_KEYS
    missing_root = ROOT_KEYS - set(manifest)
    if unknown_root:
        failures.append(f"unknown active-frontier keys: {', '.join(sorted(unknown_root))}")
    if missing_root:
        failures.append(f"missing active-frontier keys: {', '.join(sorted(missing_root))}")
    if manifest.get("schema_version") != 3:
        failures.append("schema_version must be 3")
    if not nonempty(manifest.get("policy_decision")):
        failures.append("policy_decision must be non-empty")
    ttl = manifest.get("claim_ttl_hours")
    if isinstance(ttl, bool) or not isinstance(ttl, int) or ttl <= 0:
        failures.append("claim_ttl_hours must be an integer > 0")
        ttl = 1
    frontier = manifest.get("frontier")
    if not isinstance(frontier, list) or not frontier:
        failures.append("frontier must be a non-empty list")
        frontier = []
    issues, issue_failures = load_issues(root / ".beads/issues.jsonl")
    failures.extend(issue_failures)
    try:
        governance = read_json(root / "specs/spec_governance_manifest.json")
        governed = governance.get("entries", {}) if isinstance(governance, dict) else {}
    except StatusError as exc:
        failures.append(str(exc))
        governed = {}
    policy_path = manifest.get("policy_decision")
    if nonempty(policy_path):
        policy_parts = PurePosixPath(policy_path)
        if (
            policy_parts.is_absolute() or ".." in policy_parts.parts
            or not policy_path.startswith("docs/decisions/") or policy_parts.suffix != ".md"
        ):
            failures.append("policy_decision must be a canonical docs/decisions/*.md path")
        policy_entry = governed.get(policy_path, {})
        if not (root / policy_path).is_file():
            failures.append(f"policy decision does not exist: {policy_path}")
        if policy_entry.get("class") != "doctrine" or not policy_entry.get("controlling"):
            failures.append(f"policy decision must be controlling doctrine: {policy_path}")
    keys: set[str] = set()
    orders: set[int] = set()
    primary_ids: set[str] = set()
    next_items: list[dict[str, Any]] = []
    active_claims: list[tuple[str, dict[str, Any]]] = []
    moment = (now or datetime.now(timezone.utc)).astimezone(timezone.utc)
    for index, raw in enumerate(frontier):
        if not isinstance(raw, dict):
            failures.append(f"frontier[{index}] must be an object")
            continue
        item = raw
        label = item.get("key", f"frontier[{index}]")
        unknown = set(item) - ITEM_KEYS
        missing = REQUIRED_ITEM_KEYS - set(item)
        if unknown:
            failures.append(f"{label}: unknown keys: {', '.join(sorted(unknown))}")
        if missing:
            failures.append(f"{label}: missing keys: {', '.join(sorted(missing))}")
        key = item.get("key")
        if not nonempty(key):
            failures.append(f"frontier[{index}]: key must be non-empty")
        elif key in keys:
            failures.append(f"duplicate frontier key: {key}")
        else:
            keys.add(key)
        order = item.get("order")
        if isinstance(order, bool) or not isinstance(order, int) or order < 0:
            failures.append(f"{label}: order must be an integer >= 0")
        elif order in orders:
            failures.append(f"duplicate frontier order: {order}")
        else:
            orders.add(order)
        for name in ("title", "issue_id", "summary"):
            if not nonempty(item.get(name)):
                failures.append(f"{label}: {name} must be non-empty")
        state = item.get("state")
        authorization = item.get("authorization")
        if state not in VALID_STATES:
            failures.append(f"{label}: invalid state {state!r}")
        if authorization not in VALID_AUTHORIZATIONS:
            failures.append(f"{label}: invalid authorization {authorization!r}")
        elif state in STATE_AUTHORIZATIONS and authorization not in STATE_AUTHORIZATIONS[state]:
            failures.append(f"{label}: authorization {authorization} is invalid for state {state}")
        for name in ("canonical_next", "parallel"):
            if not isinstance(item.get(name), bool):
                failures.append(f"{label}: {name} must be boolean")
        for name in ("related_issue_ids", "governing_docs", "dependencies", "unblocks"):
            value = item.get(name, [])
            if not isinstance(value, list) or any(not nonempty(x) for x in value):
                failures.append(f"{label}: {name} must be a list of non-empty strings")
            elif len(value) != len(set(value)):
                failures.append(f"{label}: {name} must not contain duplicates")
        docs = item.get("governing_docs", [])
        if isinstance(docs, list) and not docs:
            failures.append(f"{label}: governing_docs must not be empty")
        for doc in docs if isinstance(docs, list) else []:
            if not (root / doc).is_file():
                failures.append(f"{label}: governing document does not exist: {doc}")
                continue
            classification = governed.get(doc, {}).get("class")
            if classification not in {"governed", "doctrine", "pending"}:
                failures.append(f"{label}: governing document is not active/classified: {doc}")
            if authorization == "execution" and classification == "pending":
                failures.append(f"{label}: execution cannot rely on pending document: {doc}")
        issue_id = item.get("issue_id")
        issue = issues.get(issue_id)
        if nonempty(issue_id):
            if issue_id in primary_ids:
                failures.append(f"duplicate primary frontier issue: {issue_id}")
            primary_ids.add(issue_id)
        if issue is None:
            failures.append(f"{label}: tracker issue does not exist: {issue_id}")
            continue
        for related in item.get("related_issue_ids", []):
            if related not in issues:
                failures.append(f"{label}: related tracker issue does not exist: {related}")
        declared = set(item.get("dependencies", [])) if isinstance(item.get("dependencies"), list) else set()
        missing_dependencies = declared - set(issues)
        for dependency in sorted(missing_dependencies):
            failures.append(f"{label}: dependency issue does not exist: {dependency}")
        tracker_hard = {
            edge.get("depends_on_id") for edge in issue.get("dependencies", [])
            if isinstance(edge, dict) and edge.get("type") == "blocks" and nonempty(edge.get("depends_on_id"))
        }
        if declared != tracker_hard:
            failures.append(
                f"{label}: dependency mismatch (frontier={sorted(declared)}, "
                f"tracker hard blocks={sorted(tracker_hard)})"
            )
        unresolved = hard_blockers(issue, issues)
        expected_status = "closed" if state == "landed" else "deferred" if state == "deferred" else "in_progress" if state == "in_progress" else "open"
        if state in VALID_STATES and issue.get("status") != expected_status:
            failures.append(
                f"{label}: state {state} requires tracker status {expected_status}, "
                f"found {issue.get('status')!r}"
            )
        if state == "blocked" and not unresolved:
            failures.append(f"{label}: blocked state has no unresolved hard blocker")
        if state in {"ready", "in_progress"} and unresolved:
            failures.append(f"{label}: state {state} has unresolved hard blockers: {sorted(unresolved)}")
        if state == "deferred" and item.get("canonical_next"):
            failures.append(f"{label}: deferred item cannot be canonical next")
        if item.get("canonical_next"):
            next_items.append(item)
            if "completion" not in item:
                failures.append(f"{label}: canonical next requires a completion plan")
            if state in {"blocked", "deferred", "landed"} or unresolved:
                failures.append(f"{label}: canonical next is not actionable")
            if state == "planned" and authorization not in {"planning", "owner_decision"}:
                failures.append(
                    f"{label}: planned canonical next requires planning or owner_decision authorization"
                )
        landing = item.get("landing_commit")
        if state == "landed":
            if not nonempty(landing):
                failures.append(f"{label}: landed item requires landing_commit")
            elif not git_commit_exists(root, landing):
                failures.append(f"{label}: landing commit does not resolve: {landing}")
        elif landing is not None:
            failures.append(f"{label}: landing_commit is only allowed for landed state")
        validate_claim(item, issue, ttl, moment, failures)
        if "completion" in item:
            failures.extend(validate_completion(root, item, issues, governed))
        claim = item.get("claim")
        if isinstance(claim, dict) and nonempty(claim.get("head")):
            if not git_commit_exists(root, claim["head"]):
                failures.append(f"{label}: claimed head does not resolve: {claim['head']}")
        if state == "in_progress" and isinstance(claim, dict):
            active_claims.append((str(label), claim))
    if len(next_items) != 1:
        failures.append(f"exactly one canonical_next is required, found {len(next_items)}")
    if orders and orders != set(range(len(orders))):
        failures.append("frontier order values must be contiguous from 0")
    for index, (left_key, left) in enumerate(active_claims):
        left_scope = set(left.get("scope", [])) if isinstance(left.get("scope"), list) else set()
        for right_key, right in active_claims[index + 1:]:
            right_scope = set(right.get("scope", [])) if isinstance(right.get("scope"), list) else set()
            overlap = sorted(left_scope & right_scope)
            if left.get("worktree") == right.get("worktree") and overlap:
                failures.append(
                    f"overlapping active claims: {left_key} and {right_key} share {overlap}"
                )
    for issue_id, issue in issues.items():
        labels = set(issue.get("labels", [])) if isinstance(issue.get("labels", []), list) else set()
        controlled = labels & ROADMAP_LABELS
        if issue.get("status") != "closed" and len(controlled) != 1:
            failures.append(
                f"non-closed tracker issue must have exactly one roadmap label: "
                f"{issue_id} has {sorted(controlled)}"
            )
        is_deferred = issue.get("status") == "deferred"
        labeled_deferred = "roadmap:deferred" in controlled
        if is_deferred != labeled_deferred:
            failures.append(
                f"tracker deferred status/label mismatch: {issue_id} "
                f"status={issue.get('status')!r} labels={sorted(controlled)}"
            )
        if "roadmap:frontier" in labels and issue_id not in primary_ids:
            failures.append(f"scheduled tracker issue is absent from frontier: {issue_id}")
        if "roadmap:intake" in labels and issue_id in primary_ids:
            failures.append(f"intake-only tracker issue cannot be a frontier item: {issue_id}")
    state = {"manifest": manifest, "issues": issues, "next": next_items[0] if len(next_items) == 1 else None}
    return sorted(set(failures)), state


def render_block(manifest: dict[str, Any]) -> str:
    records = sorted(manifest["frontier"], key=lambda item: item["order"])
    lines = [
        START_MARKER,
        "> Generated from `specs/active_frontier.json` by `scripts/project_status.py`; do not hand-edit.",
        "",
    ]
    for item in records:
        flags = [f"state `{item['state']}`", f"authorization `{item['authorization']}`"]
        if item["canonical_next"]:
            flags.append("**CANONICAL NEXT**")
        if item["parallel"]:
            flags.append("parallel lane")
        lines.append(f"- **{item['title']}** (`{item['key']}`; `{item['issue_id']}`).")
        lines.append(f"   {item['summary']} *{'; '.join(flags)}.*")
        dependencies = ", ".join(f"`{value}`" for value in item["dependencies"]) or "none"
        unblocks = ", ".join(item["unblocks"]) or "none"
        docs = ", ".join(f"`{value}`" for value in item["governing_docs"])
        lines.append(f"   *Dependencies:* {dependencies}. *Unblocks:* {unblocks}. *Governing:* {docs}.")
        if item["canonical_next"]:
            lines.append("   *Completion plan:* `python3 scripts/project_status.py details`.")
    lines.extend([END_MARKER, ""])
    return "\n".join(lines)


def replace_block(progress: str, block: str) -> str:
    if progress.count(START_MARKER) != 1 or progress.count(END_MARKER) != 1:
        raise StatusError("specs/PROGRESS.md must contain exactly one ordered Active Frontier marker pair")
    start = progress.index(START_MARKER)
    end = progress.index(END_MARKER, start) + len(END_MARKER)
    if end <= start:
        raise StatusError("Active Frontier markers are out of order")
    return progress[:start] + block.rstrip("\n") + progress[end:]


def render_status(root: Path, write: bool) -> tuple[bool, str]:
    manifest = read_json(root / "specs/active_frontier.json")
    progress_path = root / "specs/PROGRESS.md"
    progress = progress_path.read_text(encoding="utf-8")
    expected = replace_block(progress, render_block(manifest))
    current = progress == expected
    if write and not current:
        progress_path.write_text(expected, encoding="utf-8")
    return current, "current" if current else "updated" if write else "stale"


def emit(payload: dict[str, Any], as_json: bool, human: str, error: bool = False) -> None:
    if as_json:
        print(json.dumps(payload, sort_keys=True, separators=(",", ":")))
    else:
        print(human, file=sys.stderr if error else sys.stdout)


def next_view(item: dict[str, Any], issue: dict[str, Any]) -> dict[str, Any]:
    """Join roadmap selection with the operational facts agents must report."""
    view = dict(item)
    view["tracker_status"] = issue.get("status")
    view["assignee"] = issue.get("assignee")
    view["live_claim"] = item.get("state") == "in_progress" and isinstance(
        item.get("claim"), dict
    )
    return view


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--json", action="store_true", dest="json_global")
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("check", "next", "details", "render", "check-render"):
        sub = commands.add_parser(name)
        sub.add_argument("--json", action="store_true", dest="json_command")
        if name == "details":
            sub.add_argument("target", nargs="?", help="Frontier key or issue id; defaults to next")
    args = parser.parse_args(argv)
    as_json = args.json_global or args.json_command
    root = args.root.resolve()
    if args.command in {"check", "next", "details"}:
        failures, state = validate(root)
        if not failures:
            try:
                current, _ = render_status(root, False)
                if not current:
                    failures.append("generated Active Frontier block is stale")
            except (OSError, StatusError, KeyError, TypeError) as exc:
                failures.append(str(exc))
        if failures:
            emit({"ok": False, "failures": failures}, as_json,
                 "Project status check failed:\n- " + "\n- ".join(failures), True)
            return 1
        if args.command == "check":
            emit({"ok": True, "frontier_count": len(state["manifest"]["frontier"])},
                 as_json, f"Project status check passed ({len(state['manifest']['frontier'])} frontier items).")
        elif args.command == "next":
            item = state["next"]
            view = next_view(item, state["issues"][item["issue_id"]])
            assignee = view["assignee"] or "unassigned"
            live_claim = "active" if view["live_claim"] else "none"
            emit({"ok": True, "next": view}, as_json,
                 f"Next task: {item['title']} ({item['issue_id']})\n"
                 f"State: {item['state']}; authorization: {item['authorization']}\n"
                 f"Tracker: {view['tracker_status']}; assignee: {assignee}; "
                 f"live claim: {live_claim}\n{item['summary']}")
        else:
            item = state["next"]
            if args.target:
                matches = [
                    candidate for candidate in state["manifest"]["frontier"]
                    if args.target in {candidate["key"], candidate["issue_id"]}
                ]
                if len(matches) != 1:
                    emit(
                        {"ok": False, "failures": [f"unknown or ambiguous Frontier target: {args.target}"]},
                        as_json, f"Project task details failed: unknown or ambiguous Frontier target: {args.target}", True,
                    )
                    return 1
                item = matches[0]
            if "completion" not in item:
                emit(
                    {"ok": False, "failures": [f"{item['key']}: no completion plan"]},
                    as_json, f"Project task details failed: {item['key']} has no completion plan", True,
                )
                return 1
            view = completion_view(item, state["issues"][item["issue_id"]])
            emit({"ok": True, "details": view}, as_json, render_completion(view))
        return 0
    if args.command == "render":
        failures, _ = validate(root)
        if failures:
            emit({"ok": False, "failures": failures}, as_json,
                 "Project status render refused invalid state:\n- " + "\n- ".join(failures), True)
            return 1
    try:
        current, result = render_status(root, args.command == "render")
    except (OSError, StatusError, KeyError, TypeError) as exc:
        emit({"ok": False, "failures": [str(exc)]}, as_json, f"Project status render failed: {exc}", True)
        return 1
    ok = current or args.command == "render"
    emit({"ok": ok, "render": result}, as_json,
         f"Active Frontier projection is {result}.", not ok)
    return int(not ok)


if __name__ == "__main__":
    raise SystemExit(main())
