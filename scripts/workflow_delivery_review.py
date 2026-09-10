"""Independent replay and exact trusted owner-receipt consistency checks."""

import re

from workflow_delivery_io import parse_json
from workflow_delivery_authority import resolve_ref
from workflow_delivery_evidence_shapes import review_sha256, review_shape
from workflow_delivery_native import event_identities, validate_correlations, validate_environment
from workflow_delivery_proof import _issues, packet_sha256, read_blob, validate_proof
from workflow_delivery_shapes import require
from workflow_delivery_enabled import validate_required_activation


def receipt_section(raw, marker, path):
    text = raw.decode("utf-8")
    require(text.count(marker) == 1, path, "unique receipt section marker required", "WDQ-RECEIPT")
    _, section = text.split(marker, 1)
    # A marker introduces its receipt section. End at the next heading or
    # evidence marker after an optional immediately following section heading.
    lines = section.splitlines()
    selected = []
    content = False
    heading_seen = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("<!--") or (stripped.startswith("#") and (heading_seen or content)):
            break
        if stripped.startswith("#"):
            heading_seen = True
        elif stripped:
            content = True
        selected.append(line)
    return "\n".join(selected)


def validate_independent_review(tree, contract, proof, authority_digest, trust, item, environment):
    """Validate replay and defect accounting, without asserting owner disposition.

    The caller supplies validated producer proof and owner-selected enrollment.
    Findings may await owner disposition; acceptance must use validate_review.
    """
    path = contract["review_path"]
    review = review_shape(tree.json(path, committed=True), path)
    packet = packet_sha256(contract, proof, authority_digest)
    require(review["packet_sha256"] == packet, path, "review packet is stale", "WDQ-STALE")
    require(review["disposition"] == "approve", path, "independent review has not approved", "WDQ-REVIEW")
    sessions = set(trust.enrolled[item["key"]]["implementation_sessions"]) | {proof["producer_session"]}
    require(sessions <= set(review["independent_of"])
            and review["reviewer_session"] not in set(review["independent_of"]), path,
            "reviewer not independent of implementation/original proof", "WDQ-REVIEW")
    read_blob(tree, review["replay"])
    replay = validate_proof(tree, contract, review["replay"]["path"])
    require(replay["producer_session"] == review["reviewer_session"]
            and replay["producer_session"] not in sessions
            and replay["input_manifest"]["sha256"] == proof["input_manifest"]["sha256"]
            and replay["fixture"]["sha256"] == proof["fixture"]["sha256"], path,
            "independent replay session/input identity mismatch", "WDQ-REVIEW")
    original_build = parse_json(read_blob(tree, proof["build"]["receipt"]), path)
    replay_build = parse_json(read_blob(tree, replay["build"]["receipt"]), path)
    require(original_build["binary_sha256"] == replay_build["binary_sha256"]
            and proof["build"]["command"] == replay["build"]["command"]
            and proof["build"]["toolchain"] == replay["build"]["toolchain"], path,
            "independent replay must exercise the reviewed executable/build identity", "WDQ-REVIEW")
    validate_environment(tree, replay, environment, contract=contract)
    original_events = validate_correlations(tree, contract, proof)
    replay_events = event_identities(tree, replay)
    require(not original_events & replay_events, path, "copied event artifact is not independent replay",
            "WDQ-REVIEW")
    validate_correlations(tree, contract, replay)
    if item["completion"]["delivery"].get("schema_version") == 2:
        validate_required_activation(tree, contract, proof)
        validate_required_activation(tree, contract, replay)
    findings = {f["issue_id"]: f for f in review["findings"]}
    defects = {d for p in (proof, replay) for r in p["results"] for d in r["defects"]}
    require(defects <= set(findings), path, "every proof defect requires review disposition", "WDQ-DEFECT")
    issues = _issues(tree)
    for issue_id in findings:
        require(issue_id in issues, path, f"unknown defect {issue_id}", "WDQ-DEFECT")
    return review


def validate_defect_dispositions(tree, review, path, trust):
    """Require exact dispositions after independent review, before acceptance/publication."""
    findings = {f["issue_id"]: f for f in review["findings"]}
    issues = _issues(tree)
    for issue_id, finding in findings.items():
        reference = finding["disposition_ref"]
        require(reference is not None, path, f"undisposed defect {issue_id}", "WDQ-DEFECT")
        approved = resolve_ref(trust.authority, reference)
        require(tree.read(reference["path"]) == approved, path,
                "defect disposition changed after owner promotion", "WDQ-DEFECT")
        disposition = receipt_section(approved, reference["marker"], reference["path"])
        token = "RESOLVED" if finding["severity"] == "blocking" else "DEFER"
        require(f"{token} {issue_id}" in disposition.splitlines(), reference["path"],
                "trusted disposition must explicitly bind this defect", "WDQ-DEFECT")
        if finding["severity"] == "blocking":
            require(issues[issue_id].get("status") == "closed", path,
                    f"blocking defect still open: {issue_id}", "WDQ-DEFECT")
            require(f"REPLAY {review['replay']['sha256']}" in disposition.splitlines(),
                    reference["path"], "resolution must bind this exact passing replay", "WDQ-DEFECT")


def validate_review(tree, contract, proof, authority_digest, trust, item, environment):
    """Legacy acceptance wrapper: independent replay plus exact owner authority."""
    review = validate_independent_review(
        tree, contract, proof, authority_digest, trust, item, environment)
    path = contract["review_path"]
    packet = packet_sha256(contract, proof, authority_digest)
    validate_defect_dispositions(tree, review, path, trust)
    receipt = review["owner_receipt"]
    require(receipt is not None, path, "exact owner receipt required", "WDQ-RECEIPT")
    approved = resolve_ref(trust.authority, receipt)
    require(tree.read(receipt["path"], committed=True) == approved, receipt["path"],
            "candidate receipt differs from promoted authority", "WDQ-RECEIPT")
    section = receipt_section(approved, receipt["marker"], receipt["path"])
    accept_step = item["completion"]["delivery"]["checkpoints"]["accept"]
    exact = f"ACCEPT {item['key']}/{accept_step} {packet} {review_sha256(review)}"
    require(section.splitlines().count(exact) == 1, receipt["path"],
            "exact owner ACCEPT response missing from referenced section", "WDQ-RECEIPT")
    require(re.search(r"(?m)^Source: \S.*$", section) is not None
            and re.search(r"(?m)^Date: \d{4}-\d{2}-\d{2}(?:[ T].*)?$", section) is not None,
            receipt["path"], "receipt requires response source and recorded date", "WDQ-RECEIPT")
    trust.acceptance_present(item)
    return review
