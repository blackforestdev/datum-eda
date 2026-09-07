"""Read-only preparation diagnostics, never trusted acceptance or promotion.

Catch incomplete defect accounting before producing owner commands. The actual
enforcement validator still checks these records against owner-selected trust.
No source receipt, reviewer identity or resolution gains authority here.
"""

from workflow_delivery_authority import authority_sha256, resolve_ref
from workflow_delivery_contract import load_contract
from workflow_delivery_evidence_shapes import review_sha256, review_shape
from workflow_delivery_native import validate_correlations, validate_environment
from workflow_delivery_proof import _issues, packet_sha256, read_blob, validate_proof
from workflow_delivery_review import receipt_section
from workflow_delivery_shapes import require


def check_defect_accounting(tree, proof, replay, review, path):
    """Require explicit accounting even when a producer defect is already closed."""
    findings = {finding["issue_id"]: finding for finding in review["findings"]}
    defects = {d for run in (proof, replay) for result in run["results"] for d in result["defects"]}
    missing = sorted(defects - findings.keys())
    require(not missing, path, f"proof defects missing review disposition: {missing}", "WDQ-DEFECT")
    issues = _issues(tree)
    for issue_id, finding in findings.items():
        require(issue_id in issues, path, f"unknown defect {issue_id}", "WDQ-DEFECT")
        reference = finding["disposition_ref"]
        require(reference is not None, path, f"undisposed defect {issue_id}", "WDQ-DEFECT")
        section = receipt_section(resolve_ref(tree, reference), reference["marker"], reference["path"])
        if finding["severity"] == "blocking":
            require(issues[issue_id].get("status") == "closed", path,
                    f"blocking defect still open: {issue_id}", "WDQ-DEFECT")
            required = [f"RESOLVED {issue_id}", f"REPLAY {review['replay']['sha256']}"]
        else:
            required = [f"DEFER {issue_id}"]
        require(all(line in section.splitlines() for line in required), reference["path"],
                f"missing exact defect/replay disposition for {issue_id}", "WDQ-DEFECT")
    return sorted(defects)


def check_review(tree, contract_path, environment_path):
    contract = load_contract(tree, contract_path)
    proof = validate_proof(tree, contract)
    path = contract["review_path"]
    review = review_shape(tree.json(path), path)
    packet = packet_sha256(contract, proof, authority_sha256(tree, contract))
    require(packet == review["packet_sha256"], path, "review packet is stale", "WDQ-STALE")
    require(review["disposition"] == "approve", path, "independent review has not approved", "WDQ-REVIEW")
    read_blob(tree, review["replay"])
    replay = validate_proof(tree, contract, review["replay"]["path"])
    environment = tree.json(environment_path)
    for run in (proof, replay):
        validate_environment(tree, run, environment)
        validate_correlations(tree, contract, run)
    defects = check_defect_accounting(tree, proof, replay, review, path)
    return {"mode": "preparation-review-only", "packet_sha256": packet,
            "review_sha256": review_sha256(review), "defects_accounted": defects,
            "owner_acceptance_checked": False, "trust_checked": False,
            "promotion_performed": False}
