"""Read owner-pinned per-enrollment environments; never infer or execute them."""

from workflow_delivery_io import DeliveryInputError, parse_json
from workflow_delivery_proof import read_blob
from workflow_delivery_shapes import array, blob, closed, identifier, require, unique


def load_environments(tree, path, policy, *, authority=None):
    keys = {row["frontier_key"] for row in policy["enrolled"]}
    if policy["schema_version"] == 1:
        value = tree.json(path) if path else None
        return {key: value for key in keys}
    try:
        require(bool(path), "environment", "explicit environment selection path required")
        require(authority is not None, path, "owner-selected environment authority required")
        raw = tree.read(path, committed=True)
        require(raw == authority.read(path, committed=True), path,
                "environment selection change requires owner promotion")
        selection = parse_json(raw, path)
        closed(selection, "schema_version kind environments", path)
        require(type(selection["schema_version"]) is int and selection["schema_version"] == 2,
                path, "integer environment selection version 2 required")
        require(selection["kind"] == "datum.workflow-delivery.environments", path,
                "environment selection kind required")
        array(selection["environments"], path, nonempty=bool(keys))
        for row in selection["environments"]:
            closed(row, "frontier_key environment", path)
            identifier(row["frontier_key"], path)
            blob(row["environment"], path)
        selected = [row["frontier_key"] for row in selection["environments"]]
        unique(selected, path)
        require(set(selected) == keys, path,
                "environment selection must match every trusted enrolled key exactly")
        result = {}
        for row in selection["environments"]:
            artifact = row["environment"]
            data = read_blob(tree, artifact, code="WDQ-ENVIRONMENT")
            require(data == read_blob(authority, artifact, code="WDQ-ENVIRONMENT"),
                    artifact["path"], "environment Blob change requires owner promotion")
            result[row["frontier_key"]] = parse_json(data, artifact["path"])
        return result
    except DeliveryInputError as error:
        raise DeliveryInputError("WDQ-ENVIRONMENT", error.path, error.detail) from error
