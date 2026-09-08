"""Retained synthetic readiness INPUT, never actual readiness or owner approval."""

from copy import deepcopy
import json

from project_status import render_block
from workflow_delivery_io import canonical_json

READY_VALID = "INFRA-S03-01.ready-valid"
READINESS_CASES = {
    READY_VALID: (None, None, None),
    "INFRA-S03-01.no-declaration": ("NEXT", "WDQ-COVERAGE", "requires a delivery declaration"),
    "INFRA-S03-02.missing-marker": ("docs/authority.md", "WDQ-AUTHORITY", "marker must occur exactly once: missing-reference"),
    "INFRA-S03-03.owner-question": ("MISSING", "WDQ-AUTHORITY", "mandatory owner question unresolved"),
    "INFRA-S03-03.foundation-answer": ("next.contract.json.foundation_answers", "WDQ-COVERAGE", "missing=['units_precision']"),
}


def prepare_readiness(f, manifest, policy, variant):
    if variant not in READINESS_CASES:
        raise ValueError("explicit implemented readiness fixture required")
    item = manifest["frontier"][1]
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
    elif variant.endswith(".owner-question"):
        contract["open_decisions"] = [{"id": "MISSING", "question": "Unresolved authority",
            "required_for_scenarios": [contract["scenarios"][0]["id"]], "disposition_ref": None}]
    elif variant.endswith(".foundation-answer"):
        del contract["foundation_answers"]["units_precision"]
    next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "NEXT")["category"] = "infrastructure"
    f.save("next.contract.json", contract)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
