"""Validate completed delivery at an owner-pinned revision, not today's source.

An archive pin preserves an acceptance statement; it grants no current source
permission and is never evidence that a later product revision has passed.
"""

from copy import copy

from workflow_delivery_evidence_shapes import proof_shape
from workflow_delivery_io import parse_json
from workflow_delivery_proof import read_blob
from workflow_delivery_shapes import require
from workflow_delivery_tree import Tree


def historical_delivery(tree, item, trust):
    """Return a labeled historical result, or None for ordinary live validation."""
    row = trust.enrolled.get(item["key"], {})
    revision = row.get("historical_ref")
    if revision is None:
        return None
    archive = Tree(tree.root, revision=revision)
    # A candidate cannot invent a completed checkpoint or select an unrelated
    # history. The pin itself is protected by Trust's exact policy comparison.
    trust.authority.git("merge-base", "--is-ancestor", revision, trust.authority.revision)
    previous = next((i for i in archive.json("specs/active_frontier.json")["frontier"]
                     if i["key"] == item["key"]), None)
    require(previous is not None, item["key"], "historical item absent", "WDQ-TRANSITION")
    for value in (previous, item):
        require(value.get("state") == "landed" and value.get("authorization") == "none"
                and not value.get("claim")
                and value.get("completion", {}).get("canonical_next_step_id") is None,
                item["key"], "historical delivery requires unchanged terminal work",
                "WDQ-TRANSITION")
    require(all(item.get(k) == previous.get(k) for k in ("issue_id", "landing_commit")),
            item["key"], "historical identity changed", "WDQ-TRANSITION")
    for field in ("steps", "delivery", "outcome"):
        require(item["completion"].get(field) == previous["completion"].get(field),
                item["key"], "historical completion changed", "WDQ-TRANSITION")
    archived_trust = copy(trust)
    archived_trust.authority = archived_trust.base = archive
    archived_trust.policy = archive.json("specs/workflow_delivery_policy.json")
    archived_trust.enrolled = {r["frontier_key"]: r for r in archived_trust.policy["enrolled"]}
    require(item["key"] in archived_trust.enrolled
            and "historical_ref" not in archived_trust.enrolled[item["key"]],
            item["key"], "pin the original delivery, not another archive", "WDQ-POLICY")
    # Reuse all original proof/replay/receipt checks. Do not execute archived
    # Python, relabel its evidence, or copy its source into the working tree.
    from workflow_delivery_checkpoints import _validate_delivery

    contract = archive.json(previous["completion"]["delivery"]["contract_path"])
    proof = proof_shape(archive.json(contract["proof_path"]), contract["proof_path"])
    environment = parse_json(read_blob(archive, proof["environment"]),
                             proof["environment"]["path"])
    phase = _validate_delivery(archive, previous, trust=archived_trust,
                               environment=environment)
    return f"historical:{phase}@{revision}"
