"""Prepare and retain synthetic validator INPUTS, never real owner/native evidence."""

from pathlib import Path
import os
import subprocess
import uuid

from project_status import render_block
from workflow_delivery_capture_command import save
from workflow_delivery_capture_state import git, protected_state
from workflow_delivery_environment_cases import (
    FIXTURE_VARIANTS, MIXED_VARIANTS, PREPARED_VARIANTS, SWAPPED_VARIANT, PRODUCT_HEADLESS_VARIANT, malformed_selection,
)
from workflow_delivery_mixed_test_support import add_infrastructure, headless_environment
from workflow_delivery_clause_test_support import inventory
from workflow_delivery_io import canonical_json, normalized_path, parse_json, sha256
from workflow_delivery_shapes import closed, commit_id, require
from workflow_delivery_native_test_support import environment
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_trust_test_support import accepted_fixture
from workflow_delivery_readiness_input import READINESS_CASES, prepare_readiness
from workflow_delivery_review_input import REVIEW_INPUT_CASES, prepare_review_input
from workflow_delivery_coverage_input import BOUNDARY_CASES, COVERAGE_INPUT_CASES, prepare_coverage_input
from workflow_delivery_specification_input import SPECIFICATION_INPUT_CASES, prepare_specification_input
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES, prepare_review_phase


def prepare_fixture(root, packet, *, fixture_variant=None):
    """Caller supplies fresh sibling locations in its owned proposal store.

    Keep the complete bundle and exact local parameters for later reconstruction.
    This fixture is the accepted-history/specification baseline, not the full
    real-roadmap snapshot or the complete hostile-case matrix.
    """
    root, packet = Path(root).resolve(), Path(packet).resolve()
    if fixture_variant is not None and fixture_variant not in PREPARED_VARIANTS:
        raise ValueError("unknown fixture variant")
    if root.exists() or packet.exists() or root.is_relative_to(packet) or packet.is_relative_to(root):
        raise ValueError("fresh separate fixture and packet locations required")
    packet.mkdir(parents=False, exist_ok=False)
    f = Fixture(root)
    _, _, _, baseline = accepted_fixture(f)
    if fixture_variant in MIXED_VARIANTS:
        baseline = add_infrastructure(f, fixture_variant)
    manifest = Tree(f.root).json("specs/active_frontier.json")
    next_item = manifest["frontier"][1]
    next_item.update(state="specified", authorization="planning")
    next_item["completion"]["steps"][0]["kind"] = "planning"
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())
    policy = Tree(f.root).json("specs/workflow_delivery_policy.json")
    policy.update(schema_version=2, coverage={
        "baseline_ref": baseline, "new_item_rule": "classification_required",
        "production_roots": ["src"], "source_scopes": [],
        "rows": [{"frontier_key": item["key"], "issue_id": item["issue_id"],
                  "category": "historical" if item["state"] == "landed" else "specification",
                  "boundary_ref": f.ref, "external_handoff_ref": None}
                 for item in manifest["frontier"]]})
    inventory(f, next_item, policy)
    if fixture_variant in READINESS_CASES:
        prepare_readiness(f, manifest, policy, fixture_variant)
    if fixture_variant in REVIEW_INPUT_CASES:
        prepare_review_input(f, fixture_variant)
    if fixture_variant in COVERAGE_INPUT_CASES:
        prepare_coverage_input(policy, fixture_variant, fixture=f, manifest=manifest)
    if fixture_variant in SPECIFICATION_INPUT_CASES:
        prepare_specification_input(f, manifest, policy, fixture_variant)
    if fixture_variant in REVIEW_PHASE_CASES:
        prepare_review_phase(f, manifest, policy, fixture_variant)
    f.save("specs/workflow_delivery_policy.json", policy)
    # Synthetic INPUT for the actual earlier formatting gate. The real-roadmap
    # fixture must retain its actual exemption manifest and referenced files.
    f.save("specs/rustfmt_exemption_manifest.json", {"schema_version": 1, "exemptions": {}})
    selected = f.blob("selected-environment.json", canonical_json(environment()))
    infra_selected = (f.blob("selected-headless.json", canonical_json(headless_environment(fixture_variant)))
                      if fixture_variant in MIXED_VARIANTS else None)
    selection = {
        "schema_version": 2, "kind": "datum.workflow-delivery.environments",
        "environments": [{"frontier_key": row["frontier_key"],
                          "environment": infra_selected if row["frontier_key"] == "INFRA" else selected}
                         for row in policy["enrolled"]]}
    if fixture_variant in FIXTURE_VARIANTS:
        selection = malformed_selection(selection, fixture_variant)
    if fixture_variant == SWAPPED_VARIANT:
        rows = selection["environments"]
        rows[0]["environment"], rows[1]["environment"] = rows[1]["environment"], rows[0]["environment"]
    if fixture_variant == PRODUCT_HEADLESS_VARIANT:
        # Change only the product's requested environment; retain the valid
        # infrastructure selection rather than conflating this with a swap.
        next(row for row in selection["environments"] if row["frontier_key"] == "TASK")["environment"] = infra_selected
    f.save("requested-environment.json", selection)
    f.stage()
    f.git("commit", "-qm", "test(workflow): freeze synthetic capture input\n\nNo actual native proof or owner approval.")
    authority = f.git("rev-parse", "HEAD").decode().strip()
    configuration = {"datum.workflowDeliveryAuthorityRef": authority,
                     "datum.workflowDeliveryBaseRef": authority,
                     "datum.workflowDeliveryEnvironmentPath": "requested-environment.json"}
    for key, value in configuration.items():
        f.git("config", "--local", key, value)
    nonce = str(uuid.uuid4())
    f.write(".git/datum-wdq-capture-owner", (nonce + "\n").encode())
    bundle = packet / "fixture.bundle"
    f.git("bundle", "create", str(bundle), "--all")
    f.git("bundle", "verify", str(bundle))
    input_roots = (["src", "external", "crates", "mcp-server", "scripts"]
                   if fixture_variant in BOUNDARY_CASES else ["src"])
    state = protected_state(f.root, input_roots)
    save(packet / "prepared-state.json", state)
    recipe = {"schema_version": 1, "kind": "synthetic-validator-input",
              "fixture_family": ("external-boundary" if fixture_variant in BOUNDARY_CASES
                                 else "accepted-history-and-readiness" if fixture_variant in READINESS_CASES
                                 else "synthetic-review-only" if fixture_variant in REVIEW_PHASE_CASES
                                 else "synthetic-specification-output" if fixture_variant in SPECIFICATION_INPUT_CASES
                                 else "malformed-coverage-input" if fixture_variant in COVERAGE_INPUT_CASES
                                 else "accepted-history-and-review" if fixture_variant in REVIEW_INPUT_CASES
                                 else "accepted-history-and-pending-specification"),
              "head": authority, "symbolic_head": state["symbolic_head"],
              "fixture_nonce": nonce, "local_configuration": configuration,
              "bundle": {"path": bundle.name, "sha256": sha256(bundle.read_bytes())},
              "prepared_state": {"path": "prepared-state.json",
                                 "sha256": sha256((packet / "prepared-state.json").read_bytes())},
              "untracked_roots": input_roots, "acceptance_asserted": False,
              "warning": "All embedded native/owner records are synthetic hostile-test INPUT, not proof."}
    if fixture_variant is not None:
        recipe.update(schema_version=2, fixture_variant=fixture_variant)
    save(packet / "recipe.json", recipe)
    return recipe


def restore_fixture(packet, root):
    """Restore reviewed synthetic history/parameters without new fixture commits."""
    packet, root = Path(packet).resolve(strict=True), Path(root).resolve()
    if root.exists() or root.is_relative_to(packet) or packet.is_relative_to(root):
        raise ValueError("fresh separate restoration location required")
    recipe = parse_json((packet / "recipe.json").read_bytes(), "recipe.json")
    variant_fields = " fixture_variant" if recipe.get("schema_version") == 2 else ""
    closed(recipe, "schema_version kind fixture_family head symbolic_head fixture_nonce "
           "local_configuration bundle prepared_state untracked_roots acceptance_asserted warning" + variant_fields, "recipe")
    variant = recipe.get("fixture_variant")
    require(variant is None or type(variant) is str, "recipe", "string fixture variant required")
    family = ("external-boundary" if variant in BOUNDARY_CASES
              else "accepted-history-and-readiness" if variant in READINESS_CASES
              else "synthetic-review-only" if variant in REVIEW_PHASE_CASES
              else "synthetic-specification-output" if variant in SPECIFICATION_INPUT_CASES
              else "malformed-coverage-input" if variant in COVERAGE_INPUT_CASES
              else "accepted-history-and-review" if variant in REVIEW_INPUT_CASES
              else "accepted-history-and-pending-specification")
    require(type(recipe["schema_version"]) is int and recipe["schema_version"] in (1, 2)
            and recipe["kind"] == "synthetic-validator-input"
            and recipe["fixture_family"] == family
            and recipe["acceptance_asserted"] is False, "recipe", "synthetic baseline recipe required")
    if recipe["schema_version"] == 2:
        require(recipe["fixture_variant"] in PREPARED_VARIANTS, "recipe", "known prepared-input variant required")
    commit_id(recipe["head"], "recipe.head")
    uuid.UUID(recipe["fixture_nonce"])
    config = recipe["local_configuration"]
    closed(config, "datum.workflowDeliveryAuthorityRef datum.workflowDeliveryBaseRef "
           "datum.workflowDeliveryEnvironmentPath", "configuration")
    require(config["datum.workflowDeliveryAuthorityRef"] == recipe["head"]
            and config["datum.workflowDeliveryBaseRef"] == recipe["head"], "configuration",
            "baseline fixture pins must equal the retained head")
    normalized_path(config["datum.workflowDeliveryEnvironmentPath"])
    for field, filename in (("bundle", "fixture.bundle"), ("prepared_state", "prepared-state.json")):
        closed(recipe[field], "path sha256", field)
        require(recipe[field]["path"] == filename and not (packet / filename).is_symlink(),
                field, "fixed regular packet artifact required")
        require(sha256((packet / filename).read_bytes()) == recipe[field]["sha256"],
                field, "packet artifact hash mismatch")
    environment = {"PATH": os.environ.get("PATH", os.defpath), "LC_ALL": "C",
                   "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull}
    subprocess.run(["git", "clone", "--quiet", str(packet / "fixture.bundle"), str(root)],
                   env=environment, check=True, capture_output=True)
    # Remove only the remote created by this fresh clone. The retained bundle
    # remains intact; the fixture must not gain new tracking refs on replay.
    git(root, "remote", "remove", "origin")
    require(git(root, "rev-parse", "HEAD").decode().strip() == recipe["head"],
            "restore", "restored head differs")
    require(git(root, "symbolic-ref", "HEAD").decode().strip() == recipe["symbolic_head"],
            "restore", "restored symbolic head differs")
    for key, value in config.items():
        git(root, "config", "--local", key, value)
    with (root / ".git/datum-wdq-capture-owner").open("x") as stream:
        stream.write(recipe["fixture_nonce"] + "\n")
    restored = protected_state(root, recipe["untracked_roots"])
    prepared = parse_json((packet / "prepared-state.json").read_bytes(), "prepared-state.json")
    for field in ("files", "index_entries_hex", "git_info_exclude", "head", "symbolic_head", "refs", "local_trust"):
        require(restored[field] == prepared[field], "restore." + field, "restored state differs")
    # Raw index stat-cache bytes are observed afresh, not misrepresented as
    # byte-identical metadata from the original checkout's filesystem.
    return recipe
