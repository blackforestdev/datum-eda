"""Owner-pinned correction grants: planning and return to review, never acceptance.

The record must already be present in externally selected authority. Candidate
text, matching approval words and a completed decision flag confer no authority.
PM025 independently validates the actual claim, tracker, requirements and proof.
"""

from copy import deepcopy

from workflow_delivery_authority import resolve_ref
from workflow_delivery_shapes import (
    array, closed, commit_id, identifier, ref, require, text, unique,
)
from workflow_delivery_tree import Tree


CODE = "WDQ-REVISION"
FRONTIER = "specs/active_frontier.json"


def revision_scopes_shape(value, rows):
    array(value, "owner_revisions", nonempty=False, code=CODE)
    categories = {row["frontier_key"]: row["category"] for row in rows}
    for grant in value:
        closed(grant, "frontier_key record_ref", "owner_revisions", CODE)
        identifier(grant["frontier_key"], "owner_revisions.frontier_key")
        require(categories.get(grant["frontier_key"]) == "external_lane",
                grant["frontier_key"], "revision grant requires an external lane", CODE)
        ref(grant["record_ref"], "owner_revisions.record_ref")
    unique([g["frontier_key"] for g in value], "owner_revisions", CODE)


def _without(value, *fields):
    return {key: content for key, content in value.items() if key not in fields}


def _item(tree, key):
    found = [item for item in tree.json(FRONTIER)["frontier"] if item["key"] == key]
    require(len(found) == 1, key, "unique source Frontier item required", CODE)
    return found[0]


def load_revision(authority, grant):
    """Validate the exact promoted record and its immutable source transition."""
    key, reference = grant["frontier_key"], grant["record_ref"]
    resolve_ref(authority, reference)
    record = authority.json(reference["path"])
    closed(record, "schema_version kind marker frontier_key issue_id disposition decision_ref "
           "owner_instruction owner_instruction_ref provenance source_commit source_step_id "
           "correction_step_id review_step_id target", reference["path"], CODE)
    require(type(record["schema_version"]) is int and record["schema_version"] == 1,
            key, "integer revision schema 1 required", CODE)
    require(record["kind"] == "owner_directed_revision" and
            record["disposition"] == "revise" and record["marker"] == reference["marker"],
            key, "explicit revision disposition and exact marker required", CODE)
    require(record["frontier_key"] == key, key, "revision identity mismatch", CODE)
    ref(record["decision_ref"], key + ".decision_ref")
    decision = record["decision_ref"]["path"]
    resolve_ref(authority, record["decision_ref"])
    require(decision.startswith("docs/decisions/PRODUCT_MECHANICS_") and
            authority.json("specs/spec_governance_manifest.json")["entries"][decision].get("class") in
            ("doctrine", "governed"), key, "revision requires promoted numbered doctrine", CODE)
    for field in ("issue_id", "source_step_id", "correction_step_id", "review_step_id"):
        identifier(record[field], key + "." + field)
    text(record["owner_instruction"], key + ".owner_instruction")
    ref(record["owner_instruction_ref"], key + ".owner_instruction_ref")
    instruction = resolve_ref(authority, record["owner_instruction_ref"]).decode("utf-8")
    require(instruction.count(record["owner_instruction"]) == 1, key,
            "verbatim owner instruction must resolve exactly once", CODE)
    closed(record["provenance"], "source recorded_at", key + ".provenance", CODE)
    for field in ("source", "recorded_at"):
        text(record["provenance"][field], key + ".provenance." + field)
    commit_id(record["source_commit"], key + ".source_commit")
    source = Tree(authority.root, revision=record["source_commit"])
    require(authority.git("merge-base", "--is-ancestor", source.revision,
                          authority.revision) == b"", key,
            "revision source must precede promoted authority", CODE)
    before, target = _item(source, key), record["target"]
    require(type(target) is dict and target.get("issue_id") == record["issue_id"] ==
            before.get("issue_id"), key, "revision issue mismatch", CODE)
    require(before.get("authorization") == "owner_decision" and
            before.get("state") == "specified" and not before.get("claim"), key,
            "revision source must be an unclaimed owner boundary", CODE)
    require(before["completion"]["canonical_next_step_id"] == record["source_step_id"],
            key, "revision must name the source selected decision", CODE)
    require(_without(target, "state", "authorization", "claim", "governing_docs", "completion") ==
            _without(before, "state", "authorization", "claim", "governing_docs", "completion"),
            key, "revision changes identity, dependencies or product boundary", CODE)
    docs = target.get("governing_docs")
    require(type(docs) is list and len(docs) == len(set(docs)) and
            set(before["governing_docs"]) <= set(docs) and reference["path"] in docs and
            record["owner_instruction_ref"]["path"] in docs and decision in docs,
            key, "revision must retain authorities and include its owner record", CODE)
    require(target.get("state") == "specified" and target.get("authorization") == "planning"
            and not target.get("claim"), key, "revision target is unclaimed planning only", CODE)
    old, new = before["completion"], target.get("completion", {})
    require(_without(old, "steps", "canonical_next_step_id") ==
            _without(new, "steps", "canonical_next_step_id"), key,
            "revision must preserve outcome, delivery mapping and completion policies", CODE)
    steps = new.get("steps", [])
    require(type(steps) is list and len(steps) >= len(old["steps"]) + 2, key,
            "revision appends correction and review steps", CODE)
    matched = False
    for previous, revised in zip(old["steps"], steps):
        if previous["id"] != record["source_step_id"]:
            require(previous == revised, key, "revision rewrote another existing step", CODE)
            continue
        matched = True
        require(previous["kind"] == "owner_decision" and previous["status"] == "pending" and
                _without(previous, "status", "completion_evidence") ==
                _without(revised, "status", "completion_evidence"), key,
                "revision rewrote the selected owner's requirements", CODE)
        require(revised["status"] == "complete" and revised["completion_evidence"] ==
                [*previous["completion_evidence"], {"kind": "review", **record["owner_instruction_ref"]}], key,
                "source decision must record only its actual revision disposition", CODE)
    require(matched, key, "source owner step is absent", CODE)
    correction, review = steps[len(old["steps"]):len(old["steps"]) + 2]
    require(correction["id"] == record["correction_step_id"] and
            correction["kind"] == "planning" and correction["status"] == "pending" and
            correction["depends_on"] == [record["source_step_id"]] and
            not correction["completion_evidence"], key, "invalid correction step", CODE)
    require(review["id"] == record["review_step_id"] and review["kind"] == "owner_decision"
            and review["status"] == "pending" and review["depends_on"] == [correction["id"]]
            and not review["completion_evidence"] and bool(review.get("owner_input")), key,
            "correction must return to a pending owner review", CODE)
    review_descendants = {review["id"]}
    for future in steps[len(old["steps"]) + 2:]:
        require(future["status"] == "pending" and not future["completion_evidence"] and
                bool(set(future["depends_on"]) & review_descendants), key,
                "future work must remain pending behind owner review", CODE)
        review_descendants.add(future["id"])
    require(new["canonical_next_step_id"] == correction["id"], key,
            "correction must be the selected target", CODE)
    return record


def validate_revision_records(tree, coverage, authority):
    """Pin records and referenced governance in all candidate views."""
    for grant in coverage.get("owner_revisions", []):
        reference = grant["record_ref"]
        require(resolve_ref(tree, reference) == resolve_ref(authority, reference),
                reference["path"], "revision record changed after owner promotion", CODE)
        record = load_revision(authority, grant)
        instruction_ref = record["owner_instruction_ref"]
        require(resolve_ref(tree, instruction_ref) == resolve_ref(authority, instruction_ref),
                instruction_ref["path"], "owner instruction changed after promotion", CODE)
        decision_ref = record["decision_ref"]
        require(resolve_ref(tree, decision_ref) == resolve_ref(authority, decision_ref),
                decision_ref["path"], "revision doctrine changed after promotion", CODE)
        source = _item(Tree(authority.root, revision=record["source_commit"]),
                       grant["frontier_key"])
        current = _item(tree, grant["frontier_key"])
        committed = _item(Tree(tree.root, revision=tree.resolve("HEAD")),
                          grant["frontier_key"])
        if current == source:
            require(committed == source, grant["frontier_key"],
                    "cannot discard a committed owner-directed correction", CODE)
        previous = next((s for s in committed["completion"]["steps"]
                         if s["id"] == record["correction_step_id"]), None)
        if previous and previous["status"] == "complete":
            actual = next((s for s in current["completion"]["steps"]
                           if s["id"] == record["correction_step_id"]), None)
            require(actual == previous, grant["frontier_key"],
                    "completed correction evidence cannot be rewritten", CODE)
        for path in record["target"]["governing_docs"]:
            require(path in tree.json("specs/spec_governance_manifest.json")["entries"],
                    path, "revised authority must remain governed", CODE)


def validate_owner_revision(item, coverage, authority):
    """Return false for legacy lanes, or validate bounded correction/review state."""
    grants = [g for g in coverage.get("owner_revisions", []) if g["frontier_key"] == item["key"]]
    if not grants:
        return False
    require(len(grants) == 1, item["key"], "duplicate revision grant", CODE)
    record = load_revision(authority, grants[0])
    source = _item(Tree(authority.root, revision=record["source_commit"]), item["key"])
    if item == source:
        return True  # Promotion may install the grant before the owning lane applies it.
    target = deepcopy(record["target"])
    key = item["key"]
    require(_without(item, "state", "authorization", "claim", "completion") ==
            _without(target, "state", "authorization", "claim", "completion"), key,
            "candidate differs from the exact revised governance boundary", CODE)
    current, expected = item["completion"], target["completion"]
    require(_without(current, "steps", "canonical_next_step_id") ==
            _without(expected, "steps", "canonical_next_step_id"), key,
            "revision cannot change completion policies or product outcome", CODE)
    require(len(current["steps"]) == len(expected["steps"]), key,
            "revision cannot add or remove steps", CODE)
    for actual, fixed in zip(current["steps"], expected["steps"]):
        if fixed["id"] != record["correction_step_id"]:
            require(actual == fixed, key,
                    "revision changed a frozen step or completed owner review", CODE)
        else:
            require(_without(actual, "status", "completion_evidence") ==
                    _without(fixed, "status", "completion_evidence"), key,
                    "correction requirements changed after promotion", CODE)
    correction = next(s for s in current["steps"] if s["id"] == record["correction_step_id"])
    selected = current["canonical_next_step_id"]
    if selected == record["correction_step_id"]:
        require(item["authorization"] == "planning" and
                ((item["state"] == "specified" and correction["status"] == "pending" and
                  not item.get("claim") and not correction["completion_evidence"]) or
                 (item["state"] == "in_progress" and correction["status"] == "in_progress" and
                  bool(item.get("claim")))), key, "invalid planning lifecycle", CODE)
    else:
        require(selected == record["review_step_id"] and item["state"] == "specified" and
                item["authorization"] == "owner_decision" and not item.get("claim") and
                correction["status"] == "complete" and bool(correction["completion_evidence"]),
                key, "revision may advance only to unclaimed pending owner review", CODE)
    return True
