"""Synthetic authored-specification INPUT; never actual authorship or ratification."""

from copy import deepcopy

from project_status import render_block
from workflow_delivery_clause_test_support import inventory, matrix, refresh
from workflow_delivery_readiness_input import READY_VALID, prepare_readiness

SPEC_VALID = "INFRA-S03-07.complete-matrix"
SPECIFICATION_INPUT_CASES = {
    SPEC_VALID: (None, None, None),
    "INFRA-S03-07.omitted-clause": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                    "matrix must dispose every clause for the completed step"),
    "INFRA-S03-07.duplicate-disposition": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                           "duplicate identity"),
}


def prepare_specification_input(f, manifest, policy, variant):
    if variant not in SPECIFICATION_INPUT_CASES:
        raise ValueError("explicit implemented specification-input variant required")
    prepare_readiness(f, manifest, policy, READY_VALID)
    item = manifest["frontier"][1]
    del item["completion"]["delivery"]
    item["completion"]["steps"][2]["kind"] = "planning"
    next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "NEXT")["category"] = "specification"
    clauses = inventory(f, item, policy)
    second = dict(clauses["clauses"][0], id="NEXT-C01.SECOND")
    clauses["clauses"].append(second)
    f.save("docs/specification-inventory.json", clauses)
    step = item["completion"]["steps"][0]
    dispositions, reference = matrix(f, item, step, clauses["clauses"])
    step["completion_evidence"] = [reference]
    if variant.endswith(".omitted-clause"):
        dispositions["dispositions"].pop()
    elif variant.endswith(".duplicate-disposition"):
        dispositions["dispositions"].append(deepcopy(dispositions["dispositions"][0]))
    f.save(reference["path"], dispositions)
    # This refresh constructs synthetic INPUT before its fixture authority pin;
    # it neither reconciles real routes nor approves a real authored output.
    refresh(f)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
