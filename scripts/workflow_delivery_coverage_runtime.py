"""Apply owner-pinned schema-2 coverage in the actual delivery entry points."""

from workflow_delivery_coverage import validate_classifications
from workflow_delivery_categories import validate_categories
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_contract import load_contract
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_proof import _issues
from workflow_delivery_source_scopes import authorize_source_paths
from workflow_delivery_specification import validate_specification_outputs
from workflow_delivery_transaction import changed_production_paths
from workflow_delivery_shapes import require


def validate_coverage(tree, manifest, trust, *, environments=None):
    validate_coverage_state(tree, manifest, trust, environments=environments)
    if trust.policy["schema_version"] == 1:
        return []
    coverage = trust.policy["coverage"]
    # Ordinary transactions always require current execution claims. Promotion
    # inspection separately validates its complete history and owner boundary.
    base = trust.base.revision if tree.revision else tree.resolve("HEAD")
    changed = changed_production_paths(tree, coverage["production_roots"], base_ref=base,
                                      workspace=tree.workspace)
    return authorize_source_paths(changed, coverage, manifest, _issues(tree), trust.enrolled)


def validate_coverage_state(tree, manifest, trust, *, environments=None):
    """Shared classification/readiness checks; never source authorization."""
    if trust.policy["schema_version"] == 1:
        return []
    items, readiness = validate_coverage_structure(tree, manifest, trust)
    for key in readiness:
        # Ordinary readiness continues to escalate changed enrolled inputs to
        # proof. Source-history review uses a separate non-authorizing caller.
        validate_delivery(tree, items[key], phase="ready",
                          trust=trust if key in trust.enrolled else None,
                          environment=(environments or {}).get(key))
    return items


def validate_coverage_structure(tree, manifest, trust):
    """Classification and contract structure only; not readiness or permission."""
    coverage = trust.policy["coverage"]
    items = validate_classifications(tree, coverage, manifest, authority=trust.authority)
    readiness = validate_categories(items, coverage, trust.enrolled, trust.authority)
    validate_specification_outputs(tree, items, coverage, trust.authority)
    for row in coverage["rows"]:
        if row["category"] not in ("product", "infrastructure"):
            continue
        item = items[row["frontier_key"]]
        declaration = item["completion"].get("delivery")
        if declaration is not None:
            try:
                contract = load_contract(tree, declaration["contract_path"],
                                         frontier_key=item["key"], issue_id=item["issue_id"])
                require(contract["category"] == row["category"], item["key"],
                        "delivery contract category must match owner-promoted classification",
                        "WDQ-COVERAGE")
            except DeliveryInputError as error:
                error.key = item["key"]
                error.step = item["completion"]["canonical_next_step_id"]
                raise
    return items, readiness
