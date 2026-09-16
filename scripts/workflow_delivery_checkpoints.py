"""Thin Frontier delivery extension: validation only, never a selector or writer."""

from workflow_delivery_authority import authority_sha256
from workflow_delivery_contract import load_contract, validate_handler_files
from workflow_delivery_io import DeliveryInputError, normalized_path
from workflow_delivery_native import validate_correlations, validate_environment
from workflow_delivery_proof import _issues, validate_proof
from workflow_delivery_review import validate_independent_review, validate_review
from workflow_delivery_mapping import LEGACY_PHASES, mapping_phases
from workflow_delivery_enabled import required_normal_consumers, validate_required_activation
from workflow_delivery_shapes import closed, require


PHASES = LEGACY_PHASES


def preparation_bootstrap_structure(tree, item, trust, contract):
    """Keep the installed rollout repair structural while its own inputs are exact.

    Execution still requires the original I05 owner. Owner-reviewed closeout
    may follow without building products or replaying unchanged infrastructure.
    Any changed rollout input ends this continuation. Product gates are separate.
    """
    from workflow_delivery_activation_preflight import S5A_PREPARATION_SCOPE

    coverage = trust.policy.get("coverage", {})
    preparation = coverage.get("preparation_scopes", [])
    rollout_scopes = [row for row in coverage.get("source_scopes", [])
                      if row.get("frontier_key") == "WORKFLOW-DELIVERY-IMPLEMENTATION"]
    selected = item.get("completion", {}).get("canonical_next_step_id")
    claim = item.get("claim")
    if not (item.get("key") == "WORKFLOW-DELIVERY-IMPLEMENTATION"
            and preparation == [S5A_PREPARATION_SCOPE]
            and len(rollout_scopes) == 1
            and "WDQ-I05" in rollout_scopes[0].get("step_ids", [])):
        return False
    authority_items = trust.authority.json("specs/active_frontier.json")["frontier"]
    authority_item = next((row for row in authority_items if row.get("key") == item["key"]), None)
    if authority_item is None:
        return False
    executing = (item.get("state") == "in_progress"
                 and item.get("authorization") == "execution"
                 and selected == "WDQ-I05" and type(claim) is dict
                 and same_claim_identity(claim, authority_item.get("claim")))
    if not executing and not infrastructure_closeout(item, authority_item):
        return False
    return (tree.manifest(contract["input_roots"])
            == trust.authority.manifest(contract["input_roots"]))


def infrastructure_closeout(item, approved):
    """Permit only evidence-backed I05->I06->landed, never product acceptance.

    PM025 independently validates evidence references and tracker/lifecycle state.
    This predicate grants no source permission and supplies no owner receipt.
    """
    if item.get("claim") is not None:
        return False
    current, prior = item.get("completion", {}), approved.get("completion", {})
    if ({k: v for k, v in current.items() if k not in ("steps", "canonical_next_step_id")}
            != {k: v for k, v in prior.items() if k not in ("steps", "canonical_next_step_id")}):
        return False
    steps, old_steps = current.get("steps", []), prior.get("steps", [])
    if len(steps) != 11 or [s.get("id") for s in steps] != [s.get("id") for s in old_steps]:
        return False
    for step, old in zip(steps, old_steps):
        if step.get("id") not in ("WDQ-I05", "WDQ-I06"):
            if step != old or step.get("status") != "complete":
                return False
        elif ({k: v for k, v in step.items() if k not in ("status", "completion_evidence")}
              != {k: v for k, v in old.items() if k not in ("status", "completion_evidence")}):
            return False
    by_id = {step["id"]: step for step in steps}
    if not {"WDQ-I05", "WDQ-I06"} <= by_id.keys():
        return False
    delivery, owner = by_id["WDQ-I05"], by_id["WDQ-I06"]
    if delivery.get("status") != "complete" or not delivery.get("completion_evidence"):
        return False
    awaiting = (item.get("state") == "specified" and item.get("authorization") == "owner_decision"
                and current.get("canonical_next_step_id") == "WDQ-I06"
                and owner.get("status") == "pending")
    accepted = (item.get("state") == "landed" and item.get("authorization") == "none"
                and current.get("canonical_next_step_id") is None
                and owner.get("status") == "complete" and bool(owner.get("completion_evidence")))
    return awaiting or accepted


def same_claim_identity(current, approved):
    """A heartbeat renews an existing lease; it does not transfer ownership.

    PM025 validates live timestamps and tracker synchronization separately.
    Preserve every other claim field, including the original acquisition time,
    session, scope, worktree and head. Unknown or missing fields fail closed.
    """
    from project_status import CLAIM_KEYS

    if not (type(current) is dict and type(approved) is dict
            and set(current) == set(approved) == CLAIM_KEYS):
        return False
    lease_fields = {"heartbeat_at", "expires_at"}
    return all(current[key] == approved[key] for key in CLAIM_KEYS - lease_fields)


def delivery_shape(item, contract):
    delivery = item["completion"]["delivery"]
    phases = mapping_phases(delivery, item["key"])
    normalized_path(delivery["contract_path"])
    points = closed(delivery["checkpoints"], phases, item["key"], "WDQ-TRANSITION")
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    nonnull = [s for s in points.values() if s is not None]
    require(all(type(s) is str for s in nonnull) and len(set(nonnull)) == len(nonnull),
            item["key"], "checkpoints must name distinct steps", "WDQ-TRANSITION")
    kinds = {"ready": {"planning", "governance"}, "activate": {"execution"},
             "verify": {"execution"}, "review": {"execution"}, "accept": {"owner_decision"}}
    previous = None
    # JSON object insertion order is not completion order.
    for phase in phases:
        sid = points[phase]
        optional = contract["category"] == "infrastructure" and phase in ("activate", "accept")
        require((sid is None) == optional, item["key"],
                f"{phase} checkpoint nullability disagrees with category", "WDQ-TRANSITION")
        if sid is None:
            continue
        require(sid in steps and steps[sid]["kind"] in kinds[phase], item["key"],
                f"wrong/missing {phase} checkpoint kind", "WDQ-TRANSITION")
        ancestors = set()
        pending = list(steps[sid]["depends_on"])
        while pending:
            dep = pending.pop()
            if dep not in ancestors:
                require(dep in steps, item["key"], "unknown dependency", "WDQ-TRANSITION")
                ancestors.add(dep)
                pending.extend(steps[dep]["depends_on"])
        require(sid not in ancestors and (previous is None or previous in ancestors),
                item["key"], "checkpoints must follow dependency order", "WDQ-TRANSITION")
        previous = sid
    return delivery


def required_phase(item):
    points = item["completion"]["delivery"]["checkpoints"]
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    phase = "structure"
    for name in mapping_phases(item["completion"]["delivery"], item["key"]):
        sid = points[name]
        if sid is not None and steps[sid]["status"] == "complete":
            phase = name
    if phase == "structure" and item.get("authorization") == "execution":
        phase = "ready"
    return phase


def validate_transition(item, base):
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    for step in steps.values():
        if step["status"] in ("complete", "in_progress"):
            require(all(dep in steps and steps[dep]["status"] == "complete"
                        for dep in step["depends_on"]), item["key"],
                    "incomplete transition dependency", "WDQ-TRANSITION")
    selected = item["completion"]["canonical_next_step_id"]
    if selected is not None:
        require(selected in steps, item["key"], "unknown selected step", "WDQ-TRANSITION")
        kind = steps[selected]["kind"]
        require(kind != "owner_decision" or (item["authorization"] == "owner_decision"
                and not item.get("claim")), item["key"],
                "owner boundary cannot carry execution authorization/claim", "WDQ-TRANSITION")
    if base is not None:
        old = {s["id"]: s for s in base["completion"]["steps"]}
        for phase, sid in item["completion"]["delivery"]["checkpoints"].items():
            if sid is not None and old.get(sid, {}).get("status") == "complete":
                require(steps[sid]["status"] == "complete", item["key"],
                        f"cannot discard completed {phase} obligation; owner rollback required",
                        "WDQ-TRANSITION")
        accept = item["completion"]["delivery"]["checkpoints"]["accept"]
        verify = item["completion"]["delivery"]["checkpoints"]["verify"]
        if accept and steps[accept]["status"] == "complete" and old.get(accept, {}).get("status") != "complete":
            require(old.get(verify, {}).get("status") == "complete", item["key"],
                    "cannot accept directly from pending/unverified baseline", "WDQ-TRANSITION")
            review = item["completion"]["delivery"]["checkpoints"].get("review")
            if review is not None:
                require(old.get(review, {}).get("status") == "complete", item["key"],
                        "cannot accept directly from pending/unreviewed baseline", "WDQ-TRANSITION")


def validate_delivery(tree, item, *, phase=None, trust=None, environment=None):
    try:
        return _validate_delivery(tree, item, phase=phase, trust=trust, environment=environment)
    except DeliveryInputError as error:
        error.key = item.get("key")
        error.step = None
        try:
            selected = phase or required_phase(item)
            error.step = (item["completion"].get("canonical_next_step_id") or
                          item["completion"]["delivery"].get("checkpoints", {}).get(selected))
        except (KeyError, TypeError, AttributeError):
            pass  # Malformed checkpoints cannot supply a trustworthy step label.
        error.scenario = None
        try:
            contract = tree.json(item["completion"]["delivery"]["contract_path"])
            scenarios = contract["scenarios"]
            affected = [s["id"] for s in scenarios if s["id"] in error.path]
            try:
                proof = tree.json(contract["proof_path"])
                affected.extend(r["scenario_id"] for r in proof["results"]
                                if any(a["path"] == error.path for a in r["artifacts"]))
            except (DeliveryInputError, KeyError, TypeError):
                pass
            error.scenario = ",".join(sorted(set(affected or [s["id"] for s in scenarios]))) or None
        except (DeliveryInputError, KeyError, TypeError, AttributeError):
            pass  # Do not invent a scenario identity from a malformed contract.
        raise


def _validate_delivery(tree, item, *, phase=None, trust=None, environment=None):
    value = item["completion"]["delivery"]
    phases = mapping_phases(value, item["key"])
    path = value["contract_path"]
    contract = load_contract(tree, path, frontier_key=item["key"], issue_id=item["issue_id"])
    require(item["issue_id"] in _issues(tree), item["key"],
            "contract issue absent from beads", "WDQ-IDENTITY")
    validate_handler_files(tree, contract)
    delivery_shape(item, contract)
    phase = phase or required_phase(item)
    bootstrap_structure = bool(
        trust and preparation_bootstrap_structure(tree, item, trust, contract)
    )
    if bootstrap_structure:
        phase = "structure"
    require(phase in ("structure", *phases), item["key"], "unknown checkpoint", "WDQ-TRANSITION")
    base_item = None
    if trust:
        trust.contract(tree, item, contract)
        promoted = trust.authority.json("specs/active_frontier.json")["frontier"]
        promoted_item = next(i for i in promoted if i["key"] == item["key"])
        validate_transition(item, promoted_item)
        candidates = trust.base.json("specs/active_frontier.json")["frontier"]
        base_item = next((i for i in candidates if i["key"] == item["key"]), None)
    validate_transition(item, base_item)
    if trust and phase in ("structure", "ready") and not bootstrap_structure:
        # Pending labels cannot let an enabling source edit land without proof.
        # A static checker cannot distinguish harmless implementation text from
        # activation, so changed reviewed build inputs require the bounded proof.
        current_inputs = tree.manifest(contract["input_roots"])
        try:
            previous_inputs = trust.base.manifest(contract["input_roots"])
        except DeliveryInputError as error:
            if error.code not in ("WDQ-STALE", "WDQ-ARTIFACT"):
                raise
            previous_inputs = None
        if current_inputs != previous_inputs:
            phase = "activate" if contract["category"] == "product" else "verify"
    authority = authority_sha256(tree, contract, ready=phase != "structure")
    if "review" in phases and phase != "structure":
        required_normal_consumers(contract)
    if phase in ("activate", "verify", "review", "accept"):
        proof = validate_proof(tree, contract)
        validate_environment(tree, proof, environment, contract=contract)
        validate_correlations(tree, contract, proof)
        if "review" in phases:
            validate_required_activation(tree, contract, proof)
        if phase == "review":
            require(trust is not None, item["key"], "review needs promoted enrollment", "WDQ-TRUST")
            validate_independent_review(tree, contract, proof, authority, trust, item, environment)
        elif phase == "accept":
            require(trust is not None, item["key"], "acceptance needs promoted authority", "WDQ-TRUST")
            validate_review(tree, contract, proof, authority, trust, item, environment)
    return phase
