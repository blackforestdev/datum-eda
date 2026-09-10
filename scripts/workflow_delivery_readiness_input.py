"""Retained synthetic readiness INPUT, never actual readiness or owner approval."""

from copy import deepcopy
import json

from project_status import render_block
from workflow_delivery_io import canonical_json
from workflow_delivery_product_readiness_input import PRODUCT_READINESS_CASES, PRODUCT_READY, product_readiness

READY_VALID = "INFRA-S03-01.ready-valid"
PENDING_CATEGORIES = {"INFRA-S03-11.pending-product": "product",
                      "INFRA-S03-11.pending-infrastructure": "infrastructure"}
ENROLLMENT_BASELINES = {"INFRA-S02-07.product": "INFRA-S03-11.pending-product",
                       "INFRA-S02-07.infrastructure": "INFRA-S03-11.pending-infrastructure"}
READINESS_CASES = {
    READY_VALID: (None, None, None),
    "INFRA-S03-01.no-declaration": ("NEXT", "WDQ-COVERAGE", "requires a delivery declaration"),
    "INFRA-S03-02.missing-marker": ("docs/authority.md", "WDQ-AUTHORITY", "marker must occur exactly once: missing-reference"),
    "INFRA-S03-02.stale-route": ("next-readiness-input", "WDQ-AUTHORITY", "owning route review is stale; return to owning lane"),
    "INFRA-S03-03.owner-question": ("MISSING", "WDQ-AUTHORITY", "mandatory owner question unresolved"),
    "INFRA-S03-03.foundation-answer": ("next.contract.json.foundation_answers", "WDQ-COVERAGE", "missing=['units_precision']"),
}
READINESS_CASES.update(PRODUCT_READINESS_CASES)
READINESS_CASES.update({key: (None, None, None) for key in PENDING_CATEGORIES})
READINESS_CASES.update({key: ("NEXT", "WDQ-COVERAGE", "execution requires owner-promoted enrollment")
                       for key in ENROLLMENT_BASELINES})


def prepare_readiness(f, manifest, policy, variant):
    if variant not in READINESS_CASES:
        raise ValueError("explicit implemented readiness fixture required")
    item = manifest["frontier"][1]
    if variant in PENDING_CATEGORIES:
        next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "NEXT")["category"] = PENDING_CATEGORIES[variant]
        return  # Pending, unassigned planning needs no invented readiness or proof.
    doc = item["governing_docs"][0]
    ready = deepcopy(item["completion"]["steps"][0])
    owner, verify = deepcopy(ready), deepcopy(ready)
    ready.update(status="complete", completion_evidence=[
        {"kind": "document", "path": doc, "marker": "NEXT-READY-EVIDENCE"}])
    owner.update(id="NEXT-C02", kind="owner_decision", depends_on=["NEXT-C01"],
                 requirement_refs=[{"path": doc, "marker": "NEXT-C02"}], owner_input={
                     "response_format": "Fixture authorization only", "requests": [{"id": "AUTHORIZE",
                         "question": "Authorize fixture?", "recommended_response": "Fixture only",
                         "source_ref": {"path": doc, "marker": "AUTHORIZE"}}]})
    verify.update(id="NEXT-C03", kind="execution", depends_on=["NEXT-C02"],
                  requirement_refs=[{"path": doc, "marker": "NEXT-C03"}])
    item.update(state="specified", authorization="owner_decision")
    item["completion"].update(steps=[ready, owner, verify], canonical_next_step_id="NEXT-C02",
        delivery={"contract_path": "next.contract.json", "checkpoints": {
            "ready": "NEXT-C01", "activate": None, "verify": "NEXT-C03", "accept": None}})
    f.write(doc, (f.root / doc).read_bytes() + (
        "\n<!-- NEXT-READY-EVIDENCE -->\n<!-- REQ:NEXT:NEXT-C02 -->\n"
        "<!-- OWNER:NEXT:NEXT-C02:AUTHORIZE -->\n<!-- REQ:NEXT:NEXT-C03 -->\n").encode())
    issues = [json.loads(line) for line in (f.root / ".beads/issues.jsonl").read_bytes().splitlines()]
    next(row for row in issues if row["id"] == "dat-next")["acceptance_criteria"] = (
        "NEXT-C01: Prepare\nNEXT-C02: Authorize\nNEXT-C03: Verify")
    f.write(".beads/issues.jsonl", b"".join(canonical_json(row) for row in issues))
    contract = deepcopy(f.contract)
    contract.update(id="next-contract", frontier_key="NEXT", issue_id="dat-next", category="infrastructure",
                    proof_path="missing-future-proof.json", review_path="missing-future-review.json")
    for scenario in contract["scenarios"]:
        scenario["method"] = "infrastructure"
    if variant.endswith(".no-declaration"):
        del item["completion"]["delivery"]
    elif variant.endswith(".missing-marker"):
        contract["authority_refs"][0]["marker"] = "missing-reference"
    elif variant.endswith(".stale-route"):
        # Keep accepted TASK evidence intact. Only NEXT's readiness consumes
        # this deliberately stale synthetic route; never refresh its digest.
        source = "research/next-readiness-input.md"
        f.write(source, b"Unreconciled synthetic readiness source.\n")
        routes = json.loads((f.root / "specs/evidence_traceability_manifest.json").read_bytes())
        routes["routes"].append({"id": "next-readiness-input", "sources": [source],
                                 "consumers": [], "reviewed_digest": "0" * 64})
        f.save("specs/evidence_traceability_manifest.json", routes)
        contract["route_ids"].append("next-readiness-input")
    elif variant.endswith(".owner-question"):
        contract["open_decisions"] = [{"id": "MISSING", "question": "Unresolved authority",
            "required_for_scenarios": [contract["scenarios"][0]["id"]], "disposition_ref": None}]
    elif variant.endswith(".foundation-answer"):
        del contract["foundation_answers"]["units_precision"]
    if variant in PRODUCT_READINESS_CASES or variant == "INFRA-S02-07.product":
        additions = product_readiness(item, contract, PRODUCT_READY if variant == "INFRA-S02-07.product" else variant)
        f.write(doc, (f.root / doc).read_bytes() + additions.encode())
        next(row for row in issues if row["id"] == "dat-next")["acceptance_criteria"] += (
            "\nNEXT-C04: Verify\nNEXT-C05: Independent review\nNEXT-C06: Owner acceptance")
        f.write(".beads/issues.jsonl", b"".join(canonical_json(row) for row in issues))
    if variant in ENROLLMENT_BASELINES:
        # Synthetic owner input only: keep PM025 prerequisites coherent so the
        # actual enrollment refusal is reached, never fabricate real approval.
        item["completion"]["steps"][1].update(status="complete",
            completion_evidence=[{"kind": "review", **f.ref}])
        item["completion"]["canonical_next_step_id"] = "NEXT-C03"
        item.update(state="ready", authorization="execution")
    next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "NEXT")["category"] = contract["category"]
    f.save("next.contract.json", contract)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
