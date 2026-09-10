"""Synthetic authored-specification INPUT; never actual authorship or ratification."""

from copy import deepcopy

from project_status import render_block
from workflow_delivery_clause_test_support import govern, inventory, matrix, refresh
from workflow_delivery_readiness_input import READY_VALID, prepare_readiness
from workflow_delivery_tree import Tree
from workflow_delivery_io import sha256

SPEC_VALID = "INFRA-S03-07.complete-matrix"
SPEC_PENDING = "INFRA-S03-10.pending-authorship"
PROPOSAL = "docs/decisions/PRODUCT_MECHANICS_999_FIXTURE.md"
SPECIFICATION_INPUT_CASES = {
    "INFRA-S02-03.execution-authorized": ("NEXT", "WDQ-COVERAGE",
                                          "specification classification cannot authorize implementation"),
    "INFRA-S02-03.execution-step-without-authorization": ("NEXT", "WDQ-COVERAGE",
                                                         "specification classification cannot authorize implementation"),
    SPEC_VALID: (None, None, None),
    SPEC_PENDING: (None, None, None),
    "INFRA-S03-08.unratified-proposal": (PROPOSAL, "WDQ-AUTHORITY",
                                        "ratification requires owner-promoted controlling doctrine"),
    "INFRA-S03-07.omitted-clause": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                    "matrix must dispose every clause for the completed step"),
    "INFRA-S03-07.duplicate-disposition": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                           "duplicate identity"),
    "INFRA-S03-06.non-inventory-boundary": ("docs/authority.md", "WDQ-CONTRACT", "Expecting value"),
    "INFRA-S03-09.pending-owner-valid": (None, None, None),
    "INFRA-S03-09.absent-owner": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                  "open clause requires a pending explicit owner-decision step"),
    "INFRA-S03-09.planning-owner": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                    "open clause requires a pending explicit owner-decision step"),
    "INFRA-S03-09.completed-owner": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                     "open clause requires a pending explicit owner-decision step"),
    "INFRA-S03-09.non-dependent-owner": ("docs/specification-matrix.json", "WDQ-COVERAGE",
                                         "owner decision must depend on the authored step"),
}


def prepare_specification_input(f, manifest, policy, variant):
    if variant not in SPECIFICATION_INPUT_CASES:
        raise ValueError("explicit implemented specification-input variant required")
    if variant == SPEC_PENDING:
        # Keep the original pending inventory; create no authored output,
        # completed step, future proof or owner decision.
        return
    if variant.startswith("INFRA-S02-03."):
        item = manifest["frontier"][1]
        # PM025 requires execution authorization to select an execution step.
        # Keep that prerequisite coherent; separately demonstrate that even a
        # pending execution step with no authorization violates this category.
        item.update(state="ready", authorization=("execution" if variant.endswith(".execution-authorized") else "none"))
        item["completion"]["steps"][0]["kind"] = "execution"
        f.save("specs/active_frontier.json", manifest)
        f.write("specs/PROGRESS.md", render_block(manifest).encode())
        return
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
    elif variant.startswith("INFRA-S03-09."):
        clauses["clauses"][0]["requires_owner_decision"] = True
        f.save("docs/specification-inventory.json", clauses)
        target = {"INFRA-S03-09.pending-owner-valid": "NEXT-C02",
                  "INFRA-S03-09.absent-owner": "ABSENT",
                  "INFRA-S03-09.planning-owner": "NEXT-C03",
                  "INFRA-S03-09.completed-owner": "NEXT-C02",
                  "INFRA-S03-09.non-dependent-owner": "NEXT-C02"}[variant]
        dispositions["dispositions"][0].update(disposition="pending_owner", owner_step_id=target)
        owner = item["completion"]["steps"][1]
        if variant.endswith(".completed-owner"):
            owner.update(status="complete", completion_evidence=[{"kind": "review", **f.ref}])
            item["completion"]["canonical_next_step_id"] = "NEXT-C03"
            item["authorization"] = "planning"
        elif variant.endswith(".non-dependent-owner"):
            owner["depends_on"] = []
    elif variant.endswith(".unratified-proposal"):
        clauses["clauses"][0]["requires_owner_decision"] = True
        f.save("docs/specification-inventory.json", clauses)
        raw = b"<!-- PROPOSED -->\nSynthetic proposal only; no controlling ratification.\n"
        f.write(PROPOSAL, raw)
        govern(f, PROPOSAL)
        routes = Tree(f.root).json("specs/evidence_traceability_manifest.json")
        routes["routes"].append({"id": "unratified-proposal", "sources": [PROPOSAL], "consumers": [],
            "reviewed_digest": sha256(PROPOSAL.encode() + b"\0" + raw + b"\0")})
        f.save("specs/evidence_traceability_manifest.json", routes)
        dispositions["dispositions"][0].update(disposition="ratified", owner_step_id=None,
            evidence_refs=[{"path": PROPOSAL, "marker": "<!-- PROPOSED -->"}])
    elif variant.endswith(".non-inventory-boundary"):
        # A general authority document is not a declared clause inventory.
        # Leave unused fixture artifacts intact; this tests the boundary reference.
        next(row for row in policy["coverage"]["rows"] if row["frontier_key"] == "NEXT")["boundary_ref"] = f.ref
    f.save(reference["path"], dispositions)
    # This refresh constructs synthetic INPUT before its fixture authority pin;
    # it neither reconciles real routes nor approves a real authored output.
    refresh(f)
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
