"""Assemble explicitly chosen environment references; never infer or promote."""

from copy import deepcopy

from workflow_delivery_headless import headless_shape
from workflow_delivery_io import parse_json
from workflow_delivery_proof import read_blob
from workflow_delivery_shapes import array, blob, closed, identifier, require, unique


def prepare_environment_selection(tree, policy, choices):
    """Return a proposed document without writing files, trust or proof.

    Choices name exact existing committed Blobs. Their selection is caller input,
    not something inferred from a proof or the observer's current host. Product
    compatibility beyond the headless exclusion remains the delivery validator's
    responsibility; this preparer does not certify native environment evidence.
    """
    require(type(policy.get("schema_version")) is int and policy["schema_version"] == 2,
            "selection preparation", "schema-2 proposed policy required", "WDQ-ENVIRONMENT")
    expected = [row["frontier_key"] for row in policy["enrolled"]]
    unique(expected, "enrolled", "WDQ-ENVIRONMENT")
    array(choices, "choices", nonempty=bool(expected), code="WDQ-ENVIRONMENT")
    selected = []
    categories = {row["frontier_key"]: row["category"] for row in policy["coverage"]["rows"]}
    for choice in choices:
        closed(choice, "frontier_key environment", "choices", "WDQ-ENVIRONMENT")
        identifier(choice["frontier_key"], "choices")
        blob(choice["environment"], "choices")
        selected.append(choice["frontier_key"])
    unique(selected, "choices", "WDQ-ENVIRONMENT")
    require(set(selected) == set(expected), "choices",
            "explicit choices must match every proposed enrollment exactly", "WDQ-ENVIRONMENT")
    for choice in choices:
        key, artifact = choice["frontier_key"], choice["environment"]
        value = parse_json(read_blob(tree, artifact, code="WDQ-ENVIRONMENT"), artifact["path"])
        require(type(value) is dict, artifact["path"], "environment object required", "WDQ-ENVIRONMENT")
        if "schema_version" in value:
            require(categories.get(key) == "infrastructure", key,
                    "headless environment cannot substitute for product native evidence", "WDQ-ENVIRONMENT")
            headless_shape(value)
        else:
            closed(value, "os backend toolchain scale input_method window_size reproduction_commands",
                   artifact["path"], "WDQ-ENVIRONMENT")
    return {"schema_version": 2, "kind": "datum.workflow-delivery.environments",
            "environments": sorted(deepcopy(choices), key=lambda row: row["frontier_key"])}
