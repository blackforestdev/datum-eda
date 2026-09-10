"""Non-authorizing source/history review before the final evidence packet exists.

Activation never calls this inspector. Its result cannot replace final exact
candidate proof, independent review, owner response, or activation preflight.
"""

from copy import deepcopy
from datetime import datetime, timezone

from workflow_delivery_activation_preflight import (
    ROLLOUT, initial_mapping_migration, publication_scope, require,
)
from workflow_delivery_checkpoints import validate_delivery, validate_transition
from workflow_delivery_contract import load_contract
from workflow_delivery_coverage import index_items
from workflow_delivery_coverage_runtime import validate_coverage_structure
from workflow_delivery_environments import load_environments
from workflow_delivery_frontier import validate_frontier
from workflow_delivery_proof import _issues
from workflow_delivery_publication_delta import publication_delta, verify_publication_review
from workflow_delivery_tree import Tree
from workflow_delivery_trust import POLICY_PATH, Trust


def review_boundary(manifest, baseline, candidate_issues, base_issues, *, migration):
    """Require the same synchronized live repair/recheck state in both views."""
    from project_status import validate_claim

    current, old = index_items(manifest), index_items(baseline)
    require(set(current) == set(old) and ROLLOUT in current,
            "review inspection requires unchanged Frontier identities")
    require(current[ROLLOUT]["completion"].get("delivery") == {
        "schema_version": 2, "contract_path": "specs/workflow_delivery/rollout.contract.json",
        "checkpoints": {"ready": "WDQ-READY", "activate": None, "verify": "WDQ-COMPAT",
                        "review": "WDQ-RECHECK", "accept": None}},
        "review inspection requires the exact renewed rollout mapping")
    for key in current:
        if key != ROLLOUT:
            require(current[key] == old[key], "review inspection changes another Frontier lane: " + key)
    comparison = deepcopy(current[ROLLOUT])
    if migration:
        del comparison["completion"]["delivery"]
    require(comparison == old[ROLLOUT],
            "review inspection cannot advance or rewrite the rollout lifecycle")
    selected = current[ROLLOUT]["completion"]["canonical_next_step_id"]
    require(selected in ("WDQ-COMPAT", "WDQ-RECHECK"),
            "review inspection requires active WDQ-COMPAT or WDQ-RECHECK")
    for view, issues in ((manifest, candidate_issues), (baseline, base_issues)):
        item = index_items(view)[ROLLOUT]
        require(item.get("issue_id") == "dat-wdq-rollout-implementation-ffy"
                and item.get("state") == "in_progress" and item.get("authorization") == "execution",
                "review inspection requires live rollout execution")
        steps = {s["id"]: s for s in item["completion"]["steps"]}
        require(steps[selected]["kind"] == "execution" and steps[selected]["status"] == "in_progress",
                "review inspection selected step must be in progress")
        for sid in ("WDQ-TOOLS", "WDQ-READY", "WDQ-I03", "WDQ-REVIEW"):
            require(steps.get(sid, {}).get("status") == "complete", "review inspection requires completed " + sid)
        require(steps["WDQ-COMPAT"]["depends_on"] == ["WDQ-REVIEW"]
                and steps["WDQ-RECHECK"]["depends_on"] == ["WDQ-COMPAT"]
                and steps["WDQ-I04"]["depends_on"] == ["WDQ-RECHECK"],
                "review inspection requires the exact renewal dependency chain")
        require(steps["WDQ-I04"]["kind"] == "owner_decision" and steps["WDQ-I04"]["status"] == "pending",
                "review inspection cannot claim activation completion")
        require(steps["WDQ-RECHECK"]["status"] == ("pending" if selected == "WDQ-COMPAT" else "in_progress")
                and (selected == "WDQ-COMPAT" or steps["WDQ-COMPAT"]["status"] == "complete"),
                "review inspection requires honest repair/recheck sequencing")
        failures = []
        validate_claim(item, issues.get(item["issue_id"], {}), view["claim_ttl_hours"],
                       datetime.now(timezone.utc), failures)
        require(not failures, "review inspection requires a synchronized live claim: " + "; ".join(failures))
    return selected


def inspect_review_candidate(root, *, base, candidate, authority, environment_path, publication_review):
    """Inspect immutable history, scope and prerequisites, not future proof."""
    require(authority == candidate, "review authority must be the exact candidate")
    delta = publication_delta(root, base=base, candidate=candidate)
    verify_publication_review(delta, publication_review)
    tree = Tree(root, revision=candidate)
    manifest = validate_frontier(tree)
    trust = Trust(tree, authority, base)
    require(trust.policy["schema_version"] == 2, "review inspection requires schema-2 coverage")
    baseline = validate_frontier(trust.base)
    migration = initial_mapping_migration(manifest, baseline, trust.policy, trust.base.json(POLICY_PATH))
    selected = review_boundary(manifest, baseline, _issues(tree), _issues(trust.base), migration=migration)
    paths = publication_scope(trust.policy["coverage"], delta["touched_paths"])
    require(ROLLOUT in trust.enrolled, "review inspection requires rollout enrollment")
    environments = load_environments(tree, environment_path, trust.policy, authority=trust.authority)
    items, readiness = validate_coverage_structure(tree, manifest, trust)
    item = items[ROLLOUT]
    contract = load_contract(tree, item["completion"]["delivery"]["contract_path"],
                             frontier_key=ROLLOUT, issue_id=item["issue_id"])
    # Explicitly authenticate contract and transitions before the standalone
    # readiness checks. Ordinary trusted delivery keeps changed-input escalation.
    trust.contract(tree, item, contract)
    validate_transition(item, index_items(baseline)[ROLLOUT])
    validate_delivery(tree, item, phase="ready")
    for key in readiness:
        if key != ROLLOUT:
            validate_delivery(tree, items[key], phase="ready",
                              trust=trust if key in trust.enrolled else None,
                              environment=environments.get(key))
    checked = []
    for key in trust.enrolled:
        require(key in items, "enrolled review item missing: " + key)
        if key != ROLLOUT:
            phase = validate_delivery(tree, items[key], trust=trust, environment=environments.get(key))
            checked.append(f"{key}: {phase}")
    return {"kind": "datum.workflow-delivery.review-inspection", "selected_step": selected,
            "base": base, "candidate": candidate, "authority": authority,
            "checks": checked, "production_paths": paths, "evidence_validated": False,
            "publication_authorized": False, "activation_asserted": False,
            "scope": "Source/history and prerequisites only; later evidence creates a distinct candidate requiring full final inspection."}
