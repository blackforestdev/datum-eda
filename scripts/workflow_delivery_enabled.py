"""Version-2 product outcomes cannot be satisfied by unavailable-only evidence."""

from workflow_delivery_native import _index, _json_artifact
from workflow_delivery_shapes import require


def required_normal_consumers(contract):
    if contract["category"] != "product":
        return {}
    required = {s["id"]: set(s["consumer_ids"]) for s in contract["scenarios"]
                if s["dimensions"]["normal"]["disposition"] == "required"}
    require(bool(required), contract["id"],
            "product delivery requires at least one normal enabled scenario", "WDQ-CONSUMER")
    consumers = {c["id"]: c for c in contract["consumers"]}
    for sid, ids in required.items():
        for cid in ids:
            require(consumers[cid]["handler_ref"] is not None, cid,
                    f"{sid}: required enabled consumer has no production handler",
                    "WDQ-CONSUMER")
    return required


def validate_required_activation(tree, contract, proof):
    """Call after full typed correlation; this adds outcome, not registry authority."""
    required = required_normal_consumers(contract)
    consumers = {c["id"]: c for c in contract["consumers"]}
    for result in proof["results"]:
        sid = result["scenario_id"]
        if sid not in required:
            continue
        invoked = {cid: set() for cid in required[sid]}
        roles = _index(tree, result)
        for blob in roles["events"]:
            event = _json_artifact(tree, blob)
            for dispatch in event["dispatches"]:
                if not (dispatch["eligible"] and dispatch["enabled"] and dispatch["invoked"]):
                    continue
                for cid in invoked:
                    if dispatch["dispatch_key"] == consumers[cid]["dispatch_key"]:
                        invoked[cid].add(dispatch["entry_surface"])
        for cid, surfaces in invoked.items():
            require(surfaces == set(consumers[cid]["entry_surfaces"]), cid,
                    f"{sid}: required enabled behavior lacks invocation on every reviewed surface",
                    "WDQ-CONSUMER")
