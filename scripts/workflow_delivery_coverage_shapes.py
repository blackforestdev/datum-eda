"""Closed schema-2 coverage permissions; no selection, approval or mutation."""

from workflow_delivery_io import normalized_path
from workflow_delivery_shapes import (
    array, closed, commit_id, enum, identifier, ids, ref, require, unique,
)


CATEGORIES = ("historical", "specification", "product", "infrastructure",
              "deferred", "external_lane")


def paths(value, label, *, nonempty=True):
    array(value, label, nonempty=nonempty, code="WDQ-COVERAGE")
    for path in value:
        normalized_path(path)
        require(".git" not in path.split("/"), label,
                "Git metadata is not a permitted production path", "WDQ-COVERAGE")
    unique(value, label, "WDQ-COVERAGE")


def coverage_shape(value):
    label = "coverage"
    closed(value, "baseline_ref rows source_scopes production_roots new_item_rule",
           label, "WDQ-COVERAGE")
    commit_id(value["baseline_ref"], label + ".baseline_ref")
    require(value["new_item_rule"] == "classification_required", label,
            "new Frontier work must require classification", "WDQ-COVERAGE")
    paths(value["production_roots"], label + ".production_roots")
    array(value["rows"], label + ".rows", code="WDQ-COVERAGE")
    for row in value["rows"]:
        closed(row, "frontier_key issue_id category boundary_ref external_handoff_ref",
               label + ".rows", "WDQ-COVERAGE")
        identifier(row["frontier_key"], label)
        identifier(row["issue_id"], label)
        enum(row["category"], CATEGORIES, label)
        ref(row["boundary_ref"], label)
        external = row["external_handoff_ref"]
        require((external is not None) == (row["category"] == "external_lane"),
                row["frontier_key"], "only external lanes require a handoff reference",
                "WDQ-COVERAGE")
        if external is not None:
            ref(external, label)
    keys = [r["frontier_key"] for r in value["rows"]]
    unique(keys, label + ".rows", "WDQ-COVERAGE")
    # Several historical Frontier records can legitimately share a tracker
    # epic. Uniqueness belongs to Frontier keys, not issue IDs.
    array(value["source_scopes"], label + ".source_scopes", nonempty=False,
          code="WDQ-COVERAGE")
    for scope in value["source_scopes"]:
        closed(scope, "frontier_key step_ids paths boundary_ref",
               label + ".source_scopes", "WDQ-COVERAGE")
        require(scope["frontier_key"] in keys, label,
                "source scope requires a classified Frontier key", "WDQ-COVERAGE")
        ids(scope["step_ids"], label)
        paths(scope["paths"], label)
        ref(scope["boundary_ref"], label)
        for path in scope["paths"]:
            require(any(path == root or path.startswith(root + "/")
                        for root in value["production_roots"]), path,
                    "source scope must stay inside production roots", "WDQ-COVERAGE")
    return value
