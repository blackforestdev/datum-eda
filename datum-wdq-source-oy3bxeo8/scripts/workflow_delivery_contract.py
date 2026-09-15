"""Pure contract structure and cross-reference validation for PM041.

Does not infer that a handler works or that an authority-backed N/A is truthful.
Those claims require the separate consumer/native and independent-review checks.
"""

from copy import deepcopy

from workflow_delivery_io import DeliveryInputError, canonical_json, normalized_path, sha256
from workflow_delivery_shapes import (
    DIMENSIONS, FOUNDATIONS, answers, array, closed, enum, identifier,
    ids, ref, refs, require, text, unique, version,
)


CONTRACT_FIELDS = """schema_version id frontier_key issue_id category consuming_workflow
intent scope exclusions authority_refs route_ids foundation_answers open_decisions
input_roots consumers scenarios proof_path review_path"""
CONSUMER_FIELDS = """id entry_surfaces dispatch_key handler_ref scope timing persistence
unavailable_reason scenario_ids"""
SCENARIO_FIELDS = """id consumer_ids requirement_refs preconditions inputs
expected_visible expected_state dimensions method"""


def under_roots(path, roots):
    return any(path == root or path.startswith(root + "/") for root in roots)


def load_contract(tree, path, **identity):
    try:
        value = tree.json(path)
    except DeliveryInputError as error:
        if error.code == "WDQ-INDEX" or isinstance(error.__cause__, FileNotFoundError):
            raise DeliveryInputError("WDQ-CONTRACT", path, "required contract reference is missing") from error
        raise
    return validate_contract(value, path, **identity)


def validate_contract(value, path, *, frontier_key=None, issue_id=None):
    closed(value, CONTRACT_FIELDS, path)
    version(value["schema_version"], path + ".schema_version")
    for key in ("id", "frontier_key", "issue_id"):
        identifier(value[key], path + "." + key)
    for key, expected in (("frontier_key", frontier_key), ("issue_id", issue_id)):
        require(expected is None or value[key] == expected, path + "." + key,
                f"expected {expected}", "WDQ-IDENTITY")
    enum(value["category"], ("product", "infrastructure"), path + ".category")
    for key in ("consuming_workflow", "intent", "scope", "exclusions"):
        text(value[key], path + "." + key)
    refs(value["authority_refs"], path + ".authority_refs")
    ids(value["route_ids"], path + ".route_ids")
    answers(value["foundation_answers"], FOUNDATIONS, path + ".foundation_answers")
    array(value["input_roots"], path + ".input_roots")
    for root in value["input_roots"]:
        normalized_path(root)
    unique(value["input_roots"], path + ".input_roots")
    for key in ("proof_path", "review_path"):
        normalized_path(value[key])
    require(value["proof_path"] != value["review_path"], path,
            "proof and review must be separate records", "WDQ-IDENTITY")
    array(value["consumers"], path + ".consumers", code="WDQ-COVERAGE")
    array(value["scenarios"], path + ".scenarios", code="WDQ-COVERAGE")
    for consumer in value["consumers"]:
        _consumer(consumer, path + ".consumers", value["input_roots"])
    for scenario in value["scenarios"]:
        _scenario(scenario, path + ".scenarios", value["category"])
    consumers = {c["id"]: c for c in value["consumers"]}
    scenarios = {s["id"]: s for s in value["scenarios"]}
    unique([c["id"] for c in value["consumers"]], path + ".consumers")
    unique([s["id"] for s in value["scenarios"]], path + ".scenarios")
    for consumer in consumers.values():
        for sid in consumer["scenario_ids"]:
            require(sid in scenarios and consumer["id"] in scenarios[sid]["consumer_ids"],
                    path + ".consumers." + consumer["id"],
                    f"unknown/nonreciprocal scenario {sid}", "WDQ-IDENTITY")
    for scenario in scenarios.values():
        for cid in scenario["consumer_ids"]:
            require(cid in consumers and scenario["id"] in consumers[cid]["scenario_ids"],
                    path + ".scenarios." + scenario["id"],
                    f"unknown/nonreciprocal consumer {cid}", "WDQ-IDENTITY")
    array(value["open_decisions"], path + ".open_decisions", nonempty=False)
    for decision in value["open_decisions"]:
        label = path + ".open_decisions"
        closed(decision, "id question required_for_scenarios disposition_ref", label)
        identifier(decision["id"], label)
        text(decision["question"], label)
        ids(decision["required_for_scenarios"], label)
        require(set(decision["required_for_scenarios"]) <= set(scenarios), label,
                "unknown required scenario", "WDQ-IDENTITY")
        if decision["disposition_ref"] is not None:
            ref(decision["disposition_ref"], label)
    unique([d["id"] for d in value["open_decisions"]], path + ".open_decisions")
    return value


def _consumer(value, path, roots):
    closed(value, CONSUMER_FIELDS, path)
    identifier(value["id"], path + ".id")
    ids(value["entry_surfaces"], path + ".entry_surfaces")
    ids(value["scenario_ids"], path + ".scenario_ids")
    for key in ("dispatch_key", "scope", "timing", "persistence", "unavailable_reason"):
        text(value[key], path + "." + key)
    handler = value["handler_ref"]
    if handler is not None:
        closed(handler, "path symbol", path + ".handler_ref")
        normalized_path(handler["path"])
        text(handler["symbol"], path + ".handler_ref.symbol")
        require(under_roots(handler["path"], roots), path,
                "handler must be inside reviewed input roots", "WDQ-CONSUMER")
        require(handler["symbol"].lower() not in {"gui_local", "guilocal"}, path,
                "generic dispatch class is not a production handler", "WDQ-CONSUMER")


def _scenario(value, path, category):
    closed(value, SCENARIO_FIELDS, path)
    identifier(value["id"], path + ".id")
    ids(value["consumer_ids"], path + ".consumer_ids")
    refs(value["requirement_refs"], path + ".requirement_refs")
    for key in ("preconditions", "expected_visible", "expected_state"):
        text(value[key], path + "." + key)
    array(value["inputs"], path + ".inputs")
    for index, item in enumerate(value["inputs"]):
        text(item, f"{path}.inputs[{index}]")
    answers(value["dimensions"], DIMENSIONS, path + ".dimensions")
    enum(value["method"], ("native_input", "infrastructure"), path + ".method")
    require(value["method"] == ("native_input" if category == "product" else "infrastructure"),
            path + ".method", "method must match category", "WDQ-RESULT")


def canonical_contract(value):
    """Normalize only set-like collections; preserve actual ordered user inputs."""
    result = deepcopy(value)
    for key in ("input_roots", "route_ids"):
        result[key].sort()
    for key in ("consumers", "scenarios", "open_decisions"):
        result[key].sort(key=lambda x: x["id"])
    for consumer in result["consumers"]:
        consumer["entry_surfaces"].sort()
        consumer["scenario_ids"].sort()
    for scenario in result["scenarios"]:
        scenario["consumer_ids"].sort()
    for decision in result["open_decisions"]:
        decision["required_for_scenarios"].sort()
    _sort_refs(result)
    return canonical_json(result)


def _sort_refs(value):
    if type(value) is dict:
        for child in value.values():
            _sort_refs(child)
    elif type(value) is list:
        if value and all(type(x) is dict and set(x) == {"path", "marker"} for x in value):
            value.sort(key=lambda x: (x["path"], x["marker"]))
        for child in value:
            _sort_refs(child)


def contract_sha256(value):
    return sha256(canonical_contract(value))


def validate_handler_files(tree, contract):
    for consumer in contract["consumers"]:
        handler = consumer["handler_ref"]
        if handler is not None:
            require(bool(tree.read(handler["path"])), handler["path"],
                    "production handler source must exist and be nonempty", "WDQ-CONSUMER")
