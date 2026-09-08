"""Apply owner-pinned schema-2 coverage in the actual delivery entry points."""

from workflow_delivery_coverage import validate_classifications
from workflow_delivery_categories import validate_categories
from workflow_delivery_proof import _issues
from workflow_delivery_source_scopes import authorize_source_paths
from workflow_delivery_transaction import changed_production_paths


def validate_coverage(tree, manifest, trust):
    if trust.policy["schema_version"] == 1:
        return []
    coverage = trust.policy["coverage"]
    items = validate_classifications(tree, coverage, manifest, authority=trust.authority)
    validate_categories(items, coverage, trust.enrolled, trust.authority)
    # For an ordinary staged/worktree check HEAD is the transaction's parent,
    # not a substitute trust anchor. Candidate-ref review uses the explicit
    # owner-selected base and therefore covers the entire proposed source delta.
    base = trust.base.revision if tree.revision else tree.resolve("HEAD")
    changed = changed_production_paths(tree, coverage["production_roots"], base_ref=base)
    return authorize_source_paths(changed, coverage, manifest, _issues(tree), trust.enrolled)
