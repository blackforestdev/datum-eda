"""Thin Frontier delivery extension: validation only, never a selector or writer."""

from workflow_delivery_authority import authority_sha256
from workflow_delivery_contract import load_contract, validate_handler_files
from workflow_delivery_io import DeliveryInputError, normalized_path
from workflow_delivery_native import validate_correlations, validate_environment
from workflow_delivery_proof import _issues, validate_proof
from workflow_delivery_review import validate_review
from workflow_delivery_shapes import closed, require


PHASES = ("ready", "activate", "verify", "accept")


def delivery_shape(item, contract):
    delivery = item["completion"]["delivery"]
    closed(delivery, "contract_path checkpoints", item["key"], "WDQ-TRANSITION")
    normalized_path(delivery["contract_path"])
    points = closed(delivery["checkpoints"], PHASES, item["key"], "WDQ-TRANSITION")
    steps = {s["id"]: s for s in item["completion"]["steps"]}
    nonnull = [s for s in points.values() if s is not None]
    require(all(type(s) is str for s in nonnull) and len(set(nonnull)) == len(nonnull),
            item["key"], "checkpoints must name distinct steps", "WDQ-TRANSITION")
    kinds = {"ready": {"planning", "governance"}, "activate": {"execution"},
             "verify": {"execution"}, "accept": {"owner_decision"}}
    previous = None
    # JSON object insertion order is not completion order.
    for phase in PHASES:
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
    for name in PHASES:
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
    closed(value, "contract_path checkpoints", item["key"], "WDQ-TRANSITION")
    path = value["contract_path"]
    contract = load_contract(tree, path, frontier_key=item["key"], issue_id=item["issue_id"])
    require(item["issue_id"] in _issues(tree), item["key"],
            "contract issue absent from beads", "WDQ-IDENTITY")
    validate_handler_files(tree, contract)
    delivery_shape(item, contract)
    phase = phase or required_phase(item)
    require(phase in ("structure", *PHASES), item["key"], "unknown checkpoint", "WDQ-TRANSITION")
    base_item = None
    if trust:
        trust.contract(tree, item, contract)
        promoted = trust.authority.json("specs/active_frontier.json")["frontier"]
        promoted_item = next(i for i in promoted if i["key"] == item["key"])
        validate_transition(item, promoted_item)
        candidates = trust.base.json("specs/active_frontier.json")["frontier"]
        base_item = next((i for i in candidates if i["key"] == item["key"]), None)
    validate_transition(item, base_item)
    if trust and phase in ("structure", "ready"):
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
    if phase in ("activate", "verify", "accept"):
        proof = validate_proof(tree, contract)
        validate_environment(tree, proof, environment)
        validate_correlations(tree, contract, proof)
        if phase == "accept":
            require(trust is not None, item["key"], "acceptance needs promoted authority", "WDQ-TRUST")
            validate_review(tree, contract, proof, authority, trust, item, environment)
    return phase
