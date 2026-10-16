"""Frontier holds, work purpose, explicit selection and bounded PM059 routing.

These checks verify durable provenance structure, not the truth of a purported
owner receipt. Actual appointment authority still requires independent review.
"""
from __future__ import annotations

import json
from pathlib import Path

PURPOSES = {"product_delivery", "qualification", "maintenance", "planning"}
DECISIONS = {"review_provisioning", "review_receipt", "continuation", "objective_acceptance"}


def text(value):
    return isinstance(value, str) and bool(value.strip())


def reference_text(root: Path, ref, governed: dict) -> str | None:
    """Resolve one governed repository-local literal marker, refusing aliases out."""
    if not isinstance(ref, dict) or set(ref) != {"path", "marker"}:
        return None
    path, marker = ref["path"], ref["marker"]
    if not text(path) or not text(marker):
        return None
    rel = Path(path)
    if rel.is_absolute() or ".." in rel.parts:
        return None
    try:
        resolved = (root / rel).resolve()
        if not resolved.is_relative_to(root.resolve()):
            return None
    except (OSError, ValueError, RuntimeError):
        return None
    entry = governed.get(path)
    if not isinstance(entry, dict) or entry.get("class") not in {"governed", "doctrine"}:
        return None
    try:
        source = resolved.read_text()
    except (OSError, UnicodeError):
        return None
    if source.count(marker) != 1:
        return None
    tail = source.split(marker, 1)[1]
    return tail if tail.strip() else None


def frontier_items(manifest: dict) -> list[dict]:
    items = manifest.get("frontier")
    return [i for i in items if isinstance(i, dict)] if isinstance(items, list) else []


def completion_steps(item: dict) -> list[dict]:
    completion = item.get("completion")
    steps = completion.get("steps") if isinstance(completion, dict) else None
    return [s for s in steps if isinstance(s, dict)] if isinstance(steps, list) else []


def decision_classification(root: Path, item: dict, step: dict, governed: dict) -> tuple[str, bool]:
    """A route request never classifies its own decision; absent evidence reserves it."""
    authority = step.get("decision_authority")
    if "decision_authority" not in step:
        return "owner_reserved", True
    if (not isinstance(authority, dict) or set(authority) != {"kind", "source_ref"}
            or not isinstance(authority.get("kind"), str)
            or authority["kind"] not in DECISIONS | {"owner_reserved"}):
        return "owner_reserved", False
    tail = reference_text(root, authority["source_ref"], governed)
    try:
        witness, _ = json.JSONDecoder().raw_decode((tail or "").lstrip())
    except ValueError:
        witness = None
    expected = {"task_key": item.get("key"), "step_id": step.get("id"),
                "decision_kind": authority["kind"]}
    return (authority["kind"], True) if witness == expected else ("owner_reserved", False)


def validate_item_authority(root: Path, item: dict, governed: dict, version: int) -> list[str]:
    failures = []
    key = item.get("key", "?")
    purpose = item.get("work_purpose")
    if purpose is None:
        if item.get("state") != "landed":
            failures.append(f"{key}: unfinished work requires explicit work_purpose")
    elif (not isinstance(purpose, dict) or set(purpose) != {"kind", "rationale"}
          or not isinstance(purpose.get("kind"), str) or purpose.get("kind") not in PURPOSES or not text(purpose.get("rationale"))):
        failures.append(f"{key}: work_purpose requires a valid kind and concrete rationale")
    hold = item.get("hold")
    if hold is not None:
        if (not isinstance(hold, dict) or set(hold) != {"reason", "source_ref", "resume_requires", "completion_checkpoint"}
                or not text(hold.get("reason")) or not text(hold.get("resume_requires"))
                or reference_text(root, hold.get("source_ref"), governed) is None):
            failures.append(f"{key}: hold needs reason, governed provenance and resumption boundary")
        steps = completion_steps(item)
        completion = item.get("completion")
        current = {"canonical_next_step_id": completion.get("canonical_next_step_id") if isinstance(completion, dict) else None,
                   "step_statuses": {s.get("id"): s.get("status") for s in steps if isinstance(s.get("id"), str)}}
        if isinstance(hold, dict) and hold.get("completion_checkpoint") != current:
            failures.append(f"{key}: held completion cannot advance before authorized resumption")
        if (item.get("state") in {"in_progress", "landed"} or item.get("claim") is not None
                or any(s.get("status") == "in_progress" for s in steps if isinstance(s, dict))):
            failures.append(f"{key}: held work cannot carry a claim, active step or completed lifecycle")
    return failures


def validate_selection(root: Path, manifest: dict, selected: list, governed: dict) -> list[str]:
    selection = manifest.get("selection")
    if manifest.get("schema_version") != 7 and selection is None:
        return [] if len(selected) == 1 else [f"exactly one canonical_next is required, found {len(selected)}"]
    if not isinstance(selection, dict):
        return ["explicit selection is required; absent selection does not mean idle"]
    if selection == {"mode": "selected"}:
        return [] if len(selected) == 1 else [f"selected mode requires exactly one canonical_next, found {len(selected)}"]
    if (set(selection) != {"mode", "reason", "source_ref"} or selection.get("mode") != "idle"
            or not text(selection.get("reason"))
            or reference_text(root, selection.get("source_ref"), governed) is None):
        return ["idle selection requires explicit mode, reason and governed disposition provenance"]
    failures = []
    if selected:
        failures.append("idle selection cannot also select a canonical task")
    if any(t.get("state") == "in_progress" or t.get("claim") is not None
           for t in frontier_items(manifest)):
        failures.append("idle selection cannot hide active or unreconciled ownership")
    return failures


def validate_appointments(root: Path, manifest: dict, governed: dict) -> list[str]:
    appointments = manifest.get("management_appointments", {})
    if not isinstance(appointments, dict):
        return ["management_appointments must be an object"]
    failures = []
    tasks = {i.get("key"): i for i in frontier_items(manifest) if isinstance(i.get("key"), str)}
    fields = {"session", "task_keys", "decision_kinds", "source_ref"}
    for name, appointment in appointments.items():
        label = f"appointment {name}"
        if not text(name) or not isinstance(appointment, dict) or set(appointment) != fields:
            failures.append(f"{label}: invalid appointment shape")
            continue
        kinds, keys = appointment.get("decision_kinds"), appointment.get("task_keys")
        if (not text(appointment.get("session")) or not isinstance(kinds, list) or not kinds
                or any(not isinstance(k, str) or k not in DECISIONS for k in kinds)
                or len(kinds) != len(set(kinds)) or not isinstance(keys, list) or not keys
                or any(not isinstance(k, str) or k not in tasks for k in keys)
                or len(keys) != len(set(keys))):
            failures.append(f"{label}: needs named session, exact tasks and only delegated decision kinds")
            continue
        tail = reference_text(root, appointment.get("source_ref"), governed)
        try:
            receipt, _ = json.JSONDecoder().raw_decode((tail or "").lstrip())
        except ValueError:
            receipt = None
        expected = {"owner_appointed": True, "session": appointment["session"],
                    "task_keys": keys, "decision_kinds": kinds}
        if not isinstance(receipt, dict) or receipt.get("owner_appointed") is not True or receipt != expected:
            failures.append(f"{label}: exact owner appointment/scope receipt does not resolve")
    for key, item in tasks.items():
        routes = item.get("decision_routing", {})
        if not isinstance(routes, dict):
            failures.append(f"{key}: decision_routing must be an object")
            continue
        steps = {s.get("id"): s for s in completion_steps(item) if isinstance(s.get("id"), str)}
        for step in steps.values():
            kind, valid = decision_classification(root, item, step, governed)
            if not valid or ("decision_authority" in step and step.get("kind") != "owner_decision"):
                failures.append(f"{key}/{step.get('id')}: invalid governed decision classification")
        for step_id, route in routes.items():
            if (step_id not in steps or steps[step_id].get("kind") != "owner_decision"
                    or not isinstance(route, dict) or set(route) != {"appointment", "decision_kind"}):
                failures.append(f"{key}: decision route must name an existing owner_decision and appointment/kind")
                continue
            if not text(route["appointment"]) or not text(route["decision_kind"]):
                failures.append(f"{key}/{step_id}: decision route needs nonempty appointment and kind")
            # Missing/out-of-scope/unsupported routes conservatively remain owner
            # boundaries. They never broaden a verified appointment.
    return failures


def decision_view(item: dict, appointments: dict) -> dict:
    step_id = item.get("completion", {}).get("canonical_next_step_id")
    selected = next((s for s in completion_steps(item) if s.get("id") == step_id), {})
    boundary = selected.get("kind") == "owner_decision"
    route = item.get("decision_routing", {}).get(step_id, {})
    appointment = appointments.get(route.get("appointment"), {})
    authority = selected.get("decision_authority")
    classification = authority.get("kind") if isinstance(authority, dict) else "owner_reserved"
    delegated = (boundary and classification == route.get("decision_kind")
                 and route.get("decision_kind") in DECISIONS
                 and item.get("key") in appointment.get("task_keys", [])
                 and route.get("decision_kind") in appointment.get("decision_kinds", []))
    return {"decision_input_required": boundary,
            "decision_routing_status": "delegated" if delegated else "owner_reserved_or_unverified" if boundary else "not_required",
            "owner_input_required": boundary and not delegated,
            "decision_maker": appointment.get("session") if delegated else "project_owner" if boundary else None,
            "decision_authority": appointment if delegated else None}


def work_start(item: dict, issue: dict, decision: dict, claim_state: str) -> str:
    if item.get("hold"):
        return "STOP: held work. " + item["hold"]["resume_requires"]
    if claim_state != "none":
        claim = item.get("claim", {})
        return f"Active claim: {claim.get('agent')} owns the declared scope; stand down or coordinate an explicit handoff. Expired or orphaned ownership requires explicit reconciliation."
    if item.get("state") == "landed" or issue.get("status") == "closed":
        return "STOP: completed work; no claim or successor authorization follows closure."
    if item.get("state") in {"blocked", "deferred"}:
        return f"STOP: {item['state']} work cannot be claimed; reconcile its recorded blockers or disposition."
    if not item.get("canonical_next"):
        return "STOP: unselected work; readiness is not selection or authorization."
    if decision["decision_input_required"]:
        return ("OWNER BOUNDARY REACHED: project-owner input is required. " if decision["owner_input_required"] else f"DECISION BOUNDARY REACHED: input from {decision['decision_maker']} is required. ") + "Do not claim, edit, choose dispositions or advance."
    if item.get("state") not in {"ready", "specified", "planned"} or item.get("authorization") not in {"execution", "planning", "none"} or (item.get("state") == "planned" and item.get("authorization") == "none"):
        return "STOP: current lifecycle/authorization does not permit ordinary work start."
    return "Before editing, atomically claim this task by adding the Frontier lease and setting beads in_progress/assignee in the same synchronized change."


def authority_lines(view: dict) -> list[str]:
    lines = []
    if view.get("hold"):
        h = view["hold"]
        lines += ["Hold: " + h["reason"], "Hold provenance: " + str(h["source_ref"]),
                  "Resume requires: " + h["resume_requires"]]
    if view.get("work_purpose"):
        p = view["work_purpose"]
        lines.append(f"Work purpose: {p['kind']}; {p['rationale']}")
    if view.get("decision_input_required"):
        if view.get("owner_input_required"):
            lines.append("Owner boundary: INPUT REQUIRED; code agents must stop and request the project owner's decisions before any claim or edit.")
        else:
            lines.append(f"Delegated boundary: INPUT REQUIRED; route to appointed manager {view['decision_maker']}; implementers must stop before any claim or edit.")
            lines.append("Appointment provenance: " + str(view["decision_authority"]["source_ref"]))
    return lines


def idle_view(manifest: dict) -> dict:
    return {"selection": manifest["selection"], "work_start": "STOP: explicit idle; no task or successor is selected or authorized."}


def render_idle(view: dict) -> str:
    return ("Presentation contract: return this stdout byte-for-byte; do not supplement it.\n"
            "Next task: none; explicit idle.\n" + view["work_start"] + "\nIdle reason: "
            + view["selection"]["reason"] + "\nIdle provenance: " + str(view["selection"]["source_ref"]))
