"""Observe interrupted publication state without repairing it or approving it.

Matching Git/configuration bytes are not evidence that hooks ran successfully.
This observer deliberately never calls a state activated or accepted.
"""

import os
from pathlib import Path

from workflow_delivery_activation_preflight import REDIRECTS, clean_inputs, require
from workflow_delivery_bootstrap import git, pinned_commit
from workflow_delivery_capture_state import TRUST_KEYS, protected_state
from workflow_delivery_io import canonical_json


def inspect_activation_state(root, *, base, candidate, input_roots, prior_trust, proposed_trust):
    require(not any(name in os.environ for name in REDIRECTS), "repository overrides are not observation inputs")
    require(type(input_roots) is list and bool(input_roots)
            and all(type(path) is str for path in input_roots)
            and len(set(input_roots)) == len(input_roots), "explicit unique input roots required")
    for trust in (prior_trust, proposed_trust):
        require(type(trust) is dict and set(trust) == set(TRUST_KEYS)
                and all(type(values) is list and all(type(value) is str for value in values)
                        for values in trust.values()), "exact prior and proposed trust maps required")
    require(prior_trust != proposed_trust, "distinct prior and proposed trust required")
    root = Path(root).resolve(strict=True)
    pinned_commit(root, base)
    pinned_commit(root, candidate)
    require(base != candidate and git(root, "merge-base", base, candidate).decode().strip() == base,
            "distinct candidate descending from exact base required")
    before = protected_state(root, input_roots)
    actual = before["local_trust"]
    trust_states = {key: ("unchanged" if actual[key] == prior_trust[key] == proposed_trust[key]
                         else "prior" if actual[key] == prior_trust[key]
                         else "proposed" if actual[key] == proposed_trust[key]
                         else "unexpected") for key in TRUST_KEYS}
    findings = []
    if before["symbolic_head"] != "refs/heads/main":
        findings.append("checkout is not attached main")
    head_state = "base" if before["head"] == base else "candidate" if before["head"] == candidate else "unexpected"
    if head_state == "unexpected":
        findings.append("HEAD matches neither exact publication pin")
    if "unexpected" in trust_states.values():
        findings.append("local trust contains values outside the exact prior/proposed maps")
    if git(root, "status", "--porcelain=v1", "--untracked-files=all"):
        findings.append("index/worktree is dirty or contains untracked files")
    try:
        clean_inputs(root, before, before["head"])
    except ValueError as error:
        findings.append(str(error))
    after = protected_state(root, input_roots)
    if canonical_json(before) != canonical_json(after):
        findings.append("protected state changed during observation")
    if findings:
        state = "diverged"
    elif head_state == "base" and actual == prior_trust:
        state = "not_started"
    elif head_state == "candidate" and actual == proposed_trust:
        state = "candidate_and_configuration_match_unverified"
    else:
        state = "partial"
    return {"schema_version": 1, "kind": "datum.workflow-delivery.activation-state",
            "base": base, "candidate": candidate, "state": state,
            "head_state": head_state, "trust_states": trust_states, "findings": findings,
            "before": before, "after": after, "repair_performed": False,
            "publication_authorized": False, "activation_asserted": False,
            "scope": "Read-only observed pins, source and local trust; not support integrity, hook verification, owner approval or recovery authorization."}
