#!/usr/bin/env python3
"""Validation and presentation for canonical task-completion contracts."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any


COMPLETION_KEYS = {
    "outcome", "execution_policy", "presentation_policy", "steps",
    "canonical_next_step_id", "post_completion",
}
EXECUTION_POLICY_KEYS = {
    "max_in_progress_steps", "dependency_independence_authorizes_parallelism",
}
PRESENTATION_POLICY_KEYS = {
    "mode", "preserve_step_order", "preserve_step_numbering",
    "allow_regrouping", "allow_supplementation", "allow_inferred_concurrency",
}
STEP_KEYS = {
    "id", "kind", "status", "action", "depends_on", "requirement_refs",
    "completion_evidence", "owner_input",
}
STEP_REQUIRED_KEYS = STEP_KEYS - {"owner_input"}
OWNER_INPUT_KEYS = {"response_format", "requests"}
OWNER_REQUEST_KEYS = {"id", "question", "recommended_response", "source_ref"}
EVIDENCE_KEYS = {"path", "marker"}
POST_KEYS = {
    "effects", "unblocks_issue_ids", "selection", "authorizes_successor",
    "selects_successor",
}
MARKER_RE = re.compile(r"<!-- REQ:([A-Za-z0-9-]+):([A-Za-z0-9-]+) -->")
ACCEPTANCE_ID_RE = re.compile(r"(?<![A-Za-z0-9-])([A-Za-z0-9-]+):")


def _text(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _closed_shape(
    value: Any, expected: set[str], required: set[str], label: str,
    failures: list[str],
) -> bool:
    if not isinstance(value, dict):
        failures.append(f"{label} must be an object")
        return False
    unknown = set(value) - expected
    missing = required - set(value)
    if unknown:
        failures.append(f"{label} has unknown keys: {', '.join(sorted(unknown))}")
    if missing:
        failures.append(f"{label} is missing keys: {', '.join(sorted(missing))}")
    return not missing


def validate_completion(
    root: Path, item: dict[str, Any], issues: dict[str, dict[str, Any]],
    governed: dict[str, Any],
) -> list[str]:
    """Validate one completion plan and its requirement/evidence coverage."""
    failures: list[str] = []
    key = item.get("key", "?")
    issue = issues.get(item.get("issue_id"), {})
    completion = item.get("completion")
    if not _closed_shape(
        completion, COMPLETION_KEYS, COMPLETION_KEYS, f"{key}: completion", failures
    ):
        return failures
    assert isinstance(completion, dict)
    if not _text(completion.get("outcome")):
        failures.append(f"{key}: completion.outcome must be non-empty")
    execution = completion.get("execution_policy")
    if _closed_shape(
        execution, EXECUTION_POLICY_KEYS, EXECUTION_POLICY_KEYS,
        f"{key}: execution_policy", failures,
    ):
        assert isinstance(execution, dict)
        if execution.get("max_in_progress_steps") != 1:
            failures.append(f"{key}: max_in_progress_steps must be 1")
        if execution.get("dependency_independence_authorizes_parallelism") is not False:
            failures.append(
                f"{key}: dependency independence must not authorize parallelism"
            )
    presentation = completion.get("presentation_policy")
    if _closed_shape(
        presentation, PRESENTATION_POLICY_KEYS, PRESENTATION_POLICY_KEYS,
        f"{key}: presentation_policy", failures,
    ):
        assert isinstance(presentation, dict)
        if presentation.get("mode") != "stdout_verbatim":
            failures.append(f"{key}: presentation mode must be stdout_verbatim")
        for name in ("preserve_step_order", "preserve_step_numbering"):
            if presentation.get(name) is not True:
                failures.append(f"{key}: presentation_policy.{name} must be true")
        for name in (
            "allow_regrouping", "allow_supplementation", "allow_inferred_concurrency",
        ):
            if presentation.get(name) is not False:
                failures.append(f"{key}: presentation_policy.{name} must be false")
    post = completion.get("post_completion")
    if _closed_shape(post, POST_KEYS, POST_KEYS, f"{key}: post_completion", failures):
        assert isinstance(post, dict)
        effects = post.get("effects")
        if not isinstance(effects, list) or not effects or any(not _text(x) for x in effects):
            failures.append(f"{key}: post_completion.effects must be a non-empty string list")
        unblocks = post.get("unblocks_issue_ids")
        if not isinstance(unblocks, list) or any(not _text(x) for x in unblocks):
            failures.append(f"{key}: post_completion.unblocks_issue_ids must be a string list")
        else:
            actual_unblocks = {
                target_id for target_id, target in issues.items()
                if target.get("status") != "closed" and any(
                    edge.get("type") == "blocks"
                    and edge.get("depends_on_id") == item.get("issue_id")
                    for edge in target.get("dependencies", []) if isinstance(edge, dict)
                )
            }
            for target_id in unblocks:
                target = issues.get(target_id)
                if target is None:
                    failures.append(f"{key}: post-completion target does not exist: {target_id}")
                elif not any(
                    edge.get("type") == "blocks"
                    and edge.get("depends_on_id") == item.get("issue_id")
                    for edge in target.get("dependencies", []) if isinstance(edge, dict)
                ):
                    failures.append(f"{key}: {target_id} is not blocked by {item.get('issue_id')}")
            if set(unblocks) != actual_unblocks:
                failures.append(
                    f"{key}: post-completion unblocks mismatch "
                    f"(declared={sorted(unblocks)}, tracker={sorted(actual_unblocks)})"
                )
            if set(item.get("unblocks", [])) != set(unblocks):
                failures.append(f"{key}: Frontier unblocks must equal completion issue IDs")
        if post.get("selection") != "explicit_frontier_update":
            failures.append(f"{key}: post_completion.selection must be explicit_frontier_update")
        for name in ("authorizes_successor", "selects_successor"):
            if post.get(name) is not False:
                failures.append(f"{key}: post_completion.{name} must be false")
    steps = completion.get("steps")
    if not isinstance(steps, list) or not steps:
        failures.append(f"{key}: completion.steps must be a non-empty list")
        return failures
    step_ids: list[str] = []
    covered: set[tuple[str, str]] = set()
    statuses: dict[str, str] = {}
    governing_docs = set(item.get("governing_docs", []))
    acceptance = issue.get("acceptance_criteria", "")
    acceptance_ids = ACCEPTANCE_ID_RE.findall(acceptance) if isinstance(acceptance, str) else []
    for index, step in enumerate(steps, 1):
        label = f"{key}: completion.steps[{index}]"
        if not _closed_shape(step, STEP_KEYS, STEP_REQUIRED_KEYS, label, failures):
            continue
        assert isinstance(step, dict)
        step_id = step.get("id")
        if not _text(step_id):
            failures.append(f"{label}.id must be non-empty")
            continue
        step_ids.append(step_id)
        if acceptance_ids.count(step_id) != 1:
            failures.append(
                f"{key}: bead acceptance criteria must contain {step_id} exactly once"
            )
        if not _text(step.get("action")):
            failures.append(f"{step_id}: action must be non-empty")
        kind = step.get("kind")
        if kind not in {"planning", "owner_decision", "governance", "execution"}:
            failures.append(f"{step_id}: invalid step kind {kind!r}")
        step_status = step.get("status")
        owner_input = step.get("owner_input")
        if kind == "owner_decision":
            _validate_owner_input(root, key, step_id, owner_input, governing_docs, failures)
        elif owner_input is not None:
            failures.append(f"{step_id}: owner_input is only allowed on owner_decision steps")
        if step_status not in {"pending", "in_progress", "complete"}:
            failures.append(f"{step_id}: invalid step status {step_status!r}")
        statuses[step_id] = step_status
        dependencies = step.get("depends_on")
        if not isinstance(dependencies, list) or any(not _text(x) for x in dependencies):
            failures.append(f"{step_id}: depends_on must be a string list")
        else:
            for dependency in dependencies:
                if dependency not in step_ids[:-1]:
                    failures.append(
                        f"{step_id}: dependency {dependency} must name an earlier step"
                    )
                elif step_status in {"in_progress", "complete"} and statuses.get(dependency) != "complete":
                    failures.append(
                        f"{step_id}: active/complete step requires completed dependency {dependency}"
                    )
        completion_evidence = step.get("completion_evidence")
        if not isinstance(completion_evidence, list):
            failures.append(f"{step_id}: completion_evidence must be a list")
        elif step_status == "complete" and not completion_evidence:
            failures.append(f"{step_id}: complete step requires completion evidence")
        elif isinstance(completion_evidence, list):
            proof_kinds: set[str] = set()
            for proof_index, proof in enumerate(completion_evidence, 1):
                proof_label = f"{step_id}: completion_evidence[{proof_index}]"
                if not isinstance(proof, dict):
                    failures.append(f"{proof_label} must be an object")
                    continue
                proof_kind = proof.get("kind")
                if not isinstance(proof_kind, str):
                    failures.append(f"{proof_label} has invalid kind {proof_kind!r}")
                    continue
                if proof_kind == "commit":
                    expected = {"kind", "revision"}
                    if set(proof) != expected:
                        failures.append(f"{proof_label} commit proof keys must be {sorted(expected)}")
                    revision = proof.get("revision")
                    if not _text(revision) or not _commit_exists(root, revision):
                        failures.append(f"{proof_label} commit does not resolve: {revision!r}")
                elif proof_kind in {"document", "review", "decision"}:
                    expected = {"kind", "path", "marker"}
                    if set(proof) != expected:
                        failures.append(f"{proof_label} document proof keys must be {sorted(expected)}")
                    path, marker = proof.get("path"), proof.get("marker")
                    if not _text(path) or not _text(marker):
                        failures.append(f"{proof_label} requires path and marker")
                        continue
                    try:
                        proof_source = (root / path).read_text(encoding="utf-8")
                    except OSError:
                        failures.append(f"{proof_label} document does not exist: {path}")
                        continue
                    if proof_source.count(marker) != 1:
                        failures.append(f"{proof_label} marker must resolve exactly once: {marker}")
                    if governed.get(path, {}).get("class") not in {"governed", "doctrine"}:
                        failures.append(f"{proof_label} document is not governed: {path}")
                    if proof_kind == "decision" and not path.startswith("docs/decisions/PRODUCT_MECHANICS_"):
                        failures.append(f"{proof_label} decision path is not numbered doctrine")
                else:
                    failures.append(f"{proof_label} has invalid kind {proof_kind!r}")
                    continue
                proof_kinds.add(proof_kind)
            if kind == "owner_decision" and step_status == "complete" and not (
                proof_kinds & {"review", "decision"}
            ):
                failures.append(f"{step_id}: completed owner decision requires review/decision evidence")
        evidence = step.get("requirement_refs")
        if not isinstance(evidence, list) or not evidence:
            failures.append(f"{step_id}: requirement_refs must be a non-empty list")
            continue
        for evidence_index, reference in enumerate(evidence, 1):
            evidence_label = f"{step_id}: requirement_refs[{evidence_index}]"
            if not _closed_shape(
                reference, EVIDENCE_KEYS, EVIDENCE_KEYS, evidence_label, failures
            ):
                continue
            assert isinstance(reference, dict)
            path, marker = reference.get("path"), reference.get("marker")
            if not _text(path) or not _text(marker):
                failures.append(f"{evidence_label} path and marker must be non-empty")
                continue
            if path not in governing_docs:
                failures.append(f"{step_id}: evidence path is not governing: {path}")
                continue
            if governed.get(path, {}).get("class") not in {"governed", "doctrine"}:
                failures.append(f"{step_id}: evidence path is not governed: {path}")
            try:
                source = (root / path).read_text(encoding="utf-8")
            except OSError as exc:
                failures.append(f"{step_id}: cannot read evidence path {path}: {exc}")
                continue
            token = f"<!-- REQ:{key}:{marker} -->"
            marker_count = source.count(token)
            if marker_count != 1:
                failures.append(
                    f"{step_id}: evidence marker must occur exactly once in {path}: {token}"
                )
            if marker != step_id:
                failures.append(f"{step_id}: evidence marker must equal the step id")
            if marker_count == 1 and marker == step_id:
                covered.add((path, marker))
    if len(step_ids) != len(set(step_ids)):
        failures.append(f"{key}: completion step ids must be unique")
    if acceptance_ids != step_ids:
        failures.append(
            f"{key}: bead acceptance step IDs must exactly match completion order "
            f"(acceptance={acceptance_ids}, completion={step_ids})"
        )
    active_steps = [step_id for step_id, value in statuses.items() if value == "in_progress"]
    maximum_active = (
        execution.get("max_in_progress_steps")
        if isinstance(execution, dict) and execution.get("max_in_progress_steps") == 1
        else 1
    )
    if len(active_steps) > maximum_active:
        failures.append(f"{key}: at most {maximum_active} completion step may be in_progress")
    canonical_step = completion.get("canonical_next_step_id")
    incomplete_steps = [step_id for step_id in step_ids if statuses.get(step_id) != "complete"]
    if not incomplete_steps:
        if canonical_step is not None:
            failures.append(f"{key}: completed plan requires canonical_next_step_id null")
        if item.get("canonical_next"):
            failures.append(f"{key}: canonical task requires an incomplete selected substep")
    elif not _text(canonical_step):
        failures.append(f"{key}: incomplete plan requires one canonical_next_step_id")
    elif canonical_step not in statuses:
        failures.append(f"{key}: canonical completion step does not exist: {canonical_step}")
    else:
        if statuses[canonical_step] not in {"pending", "in_progress"}:
            failures.append(f"{key}: canonical completion step must be pending or in_progress")
        selected = next(step for step in steps if step.get("id") == canonical_step)
        dependencies = selected.get("depends_on", [])
        if isinstance(dependencies, list):
            unresolved = [
                dependency for dependency in dependencies
                if statuses.get(dependency) != "complete"
            ]
            if unresolved:
                failures.append(
                    f"{key}: canonical completion step has incomplete dependencies: {unresolved}"
                )
        if active_steps and active_steps != [canonical_step]:
            failures.append(
                f"{key}: canonical_next_step_id must identify the in_progress step"
            )
        selected_kind = selected.get("kind")
        allowed_authorizations = {
            "planning": {"planning"},
            "governance": {"planning"},
            "owner_decision": {"owner_decision"},
            "execution": {"execution", "none"},
        }.get(selected_kind)
        if allowed_authorizations and item.get("authorization") not in allowed_authorizations:
            failures.append(
                f"{key}: selected {selected_kind} step requires item authorization "
                f"{' or '.join(sorted(allowed_authorizations))}"
            )
        if selected_kind == "owner_decision" and item.get("state") == "in_progress":
            failures.append(f"{key}: owner-decision boundary cannot carry an agent claim")
    if item.get("state") == "landed" and any(value != "complete" for value in statuses.values()):
        failures.append(f"{key}: landed item requires every completion step to be complete")
    for path in governing_docs:
        try:
            source = (root / path).read_text(encoding="utf-8")
        except OSError:
            continue
        for marker_key, marker in MARKER_RE.findall(source):
            if marker_key == key and (path, marker) not in covered:
                failures.append(f"{key}: uncovered governing requirement {path}#{marker}")
    return failures


def selected_completion_step(item: dict[str, Any]) -> dict[str, Any] | None:
    """Return the explicitly selected incomplete step, if any."""
    completion = item.get("completion", {})
    step_id = completion.get("canonical_next_step_id")
    return next(
        (step for step in completion.get("steps", []) if step.get("id") == step_id),
        None,
    )


def _validate_owner_input(
    root: Path, key: str, step_id: str, owner_input: Any,
    governing_docs: set[str], failures: list[str],
) -> None:
    label = f"{step_id}: owner_input"
    if not _closed_shape(owner_input, OWNER_INPUT_KEYS, OWNER_INPUT_KEYS, label, failures):
        return
    assert isinstance(owner_input, dict)
    if not _text(owner_input.get("response_format")):
        failures.append(f"{label}.response_format must be non-empty")
    requests = owner_input.get("requests")
    if not isinstance(requests, list) or not requests:
        failures.append(f"{label}.requests must be a non-empty list")
        return
    request_ids: list[str] = []
    for index, request in enumerate(requests, 1):
        request_label = f"{label}.requests[{index}]"
        if not _closed_shape(
            request, OWNER_REQUEST_KEYS, OWNER_REQUEST_KEYS, request_label, failures
        ):
            continue
        assert isinstance(request, dict)
        request_id = request.get("id")
        if not _text(request_id):
            failures.append(f"{request_label}.id must be non-empty")
        else:
            request_ids.append(request_id)
        for field in ("question", "recommended_response"):
            if not _text(request.get(field)):
                failures.append(f"{request_label}.{field} must be non-empty")
        source_ref = request.get("source_ref")
        if not _closed_shape(
            source_ref, EVIDENCE_KEYS, EVIDENCE_KEYS,
            f"{request_label}.source_ref", failures,
        ):
            continue
        assert isinstance(source_ref, dict)
        path, marker = source_ref.get("path"), source_ref.get("marker")
        if path not in governing_docs:
            failures.append(f"{request_label}.source_ref path is not governing: {path}")
            continue
        try:
            source = (root / path).read_text(encoding="utf-8")
        except OSError:
            failures.append(f"{request_label}.source_ref document does not exist: {path}")
            continue
        token = f"<!-- OWNER:{key}:{step_id}:{marker} -->"
        if source.count(token) != 1:
            failures.append(f"{request_label}.source_ref must resolve exactly once: {token}")
    if len(request_ids) != len(set(request_ids)):
        failures.append(f"{label} request IDs must be unique")


def claim_instruction(item: dict[str, Any], issue: dict[str, Any]) -> str:
    """Return claim-safe work-start guidance from validated operational state."""
    selected = selected_completion_step(item)
    if selected and selected.get("kind") == "owner_decision":
        return (
            "OWNER BOUNDARY REACHED: project-owner input is required. Code agents "
            "must not claim, edit, choose dispositions, or advance to another step."
        )
    if item.get("state") == "in_progress":
        claim = item.get("claim", {})
        return (
            f"Active claim: {claim.get('agent')} owns the declared scope; other agents "
            "must stand down or coordinate an explicit handoff."
        )
    return (
        "Before editing, atomically claim this task by adding the Frontier lease and "
        "setting beads in_progress/assignee in the same synchronized change."
    )


def _commit_exists(root: Path, revision: str) -> bool:
    import subprocess

    return subprocess.run(
        ["git", "cat-file", "-e", f"{revision}^{{commit}}"], cwd=root,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
    ).returncode == 0


def completion_view(item: dict[str, Any], issue: dict[str, Any]) -> dict[str, Any]:
    """Build the stable JSON view used by both text and JSON presentations."""
    selected = selected_completion_step(item)
    return {
        "key": item["key"],
        "title": item["title"],
        "issue_id": item["issue_id"],
        "state": item["state"],
        "authorization": item["authorization"],
        "tracker_status": issue.get("status"),
        "assignee": issue.get("assignee"),
        "live_claim": item.get("state") == "in_progress",
        "work_start": claim_instruction(item, issue),
        "outcome": item["completion"]["outcome"],
        "canonical_next_step_id": item["completion"]["canonical_next_step_id"],
        "owner_input_required": bool(selected and selected.get("kind") == "owner_decision"),
        "owner_input": selected.get("owner_input") if selected else None,
        "execution_policy": item["completion"]["execution_policy"],
        "presentation_policy": item["completion"]["presentation_policy"],
        "steps": item["completion"]["steps"],
        "post_completion": item["completion"]["post_completion"],
    }


def render_completion(view: dict[str, Any]) -> str:
    """Render a completion view without inventing or reordering content."""
    assignee = view["assignee"] or "unassigned"
    claim = "active" if view["live_claim"] else "none"
    lines = [
        "Presentation contract: return this stdout byte-for-byte; do not preface, "
        "summarize, regroup, renumber, or supplement it.",
        "Execution policy: at most one completion step may be in_progress; "
        "dependency independence does not authorize parallel work.",
        f"Task details: {view['title']} ({view['issue_id']})",
        f"State: {view['state']}; authorization: {view['authorization']}",
        f"Tracker: {view['tracker_status']}; assignee: {assignee}; live claim: {claim}",
        f"Work start: {view['work_start']}",
        f"Completion outcome: {view['outcome']}",
    ]
    canonical_step = view["canonical_next_step_id"]
    if canonical_step is None:
        lines.append("Next completion step: none; every completion step is complete.")
    else:
        selected = next(step for step in view["steps"] if step["id"] == canonical_step)
        lines.append(f"Next completion step: [{canonical_step}] {selected['action']}")
    if view["owner_input_required"]:
        lines.append(
            "Owner boundary: INPUT REQUIRED; code agents must stop and request the "
            "project owner's decisions before any claim or edit."
        )
        owner_input = view["owner_input"]
        lines.append(f"Owner response format: {owner_input['response_format']}")
        lines.append("Owner decisions requested:")
        for request in owner_input["requests"]:
            lines.append(f"- [{request['id']}] {request['question']}")
            lines.append(f"  Recommended: {request['recommended_response']}")
    lines.append("Steps:")
    for index, step in enumerate(view["steps"], 1):
        dependencies = ", ".join(step["depends_on"]) or "none"
        refs = ", ".join(
            f"{ref['path']}#{ref['marker']}" for ref in step["requirement_refs"]
        )
        proof = ", ".join(
            item.get("revision") or f"{item.get('path')}#{item.get('marker')}"
            for item in step["completion_evidence"]
        ) or "none yet"
        lines.append(
            f"{index}. [{step['id']}] ({step['kind']}; {step['status']}) {step['action']}"
        )
        lines.append(
            f"   Depends on: {dependencies}. Requirements: {refs}. Completion evidence: {proof}."
        )
    lines.append("Post-completion effects:")
    lines.extend(f"- {effect}" for effect in view["post_completion"]["effects"])
    post = view["post_completion"]
    unblocks = ", ".join(post["unblocks_issue_ids"]) or "none"
    lines.append(f"Unblocks issue IDs: {unblocks}")
    lines.append(
        "Successor policy: explicit Frontier update; "
        f"authorizes successor: {str(post['authorizes_successor']).lower()}; "
        f"selects successor: {str(post['selects_successor']).lower()}."
    )
    return "\n".join(lines)
