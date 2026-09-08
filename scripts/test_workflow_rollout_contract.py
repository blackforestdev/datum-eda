"""Prepared infrastructure contract consistency, never delivery or activation proof."""

import ast
from copy import deepcopy
from pathlib import Path
import unittest

from workflow_delivery_authority import authority_manifest
from workflow_delivery_contract import validate_contract, validate_handler_files
from workflow_delivery_io import DeliveryInputError, parse_json


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = "specs/workflow_delivery/rollout.contract.json"
SPEC = "specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md"
MECHANISM = "docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md"


class RepositoryView:
    """Read current preparation artifacts without pretending they are promoted."""

    def __init__(self, overrides=None):
        self.overrides = overrides or {}

    def read(self, path):
        return self.overrides[path] if path in self.overrides else (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class RolloutContractTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()
        self.contract = self.tree.json(CONTRACT)

    def test_contract_identity_and_existing_schema(self):
        result = validate_contract(self.contract, CONTRACT,
            frontier_key="WORKFLOW-DELIVERY-IMPLEMENTATION",
            issue_id="dat-wdq-rollout-implementation-ffy")
        self.assertEqual("infrastructure", result["category"])
        self.assertTrue(all(s["method"] == "infrastructure" for s in result["scenarios"]))

    def test_mapping_resolves_the_prepared_contract(self):
        proposal = self.tree.json("docs/reviews/workflow-delivery-rollout/rollout-mapping.json")
        self.assertEqual(CONTRACT, proposal["delivery"]["contract_path"])
        for key in ("frontier_key", "issue_id", "category"):
            self.assertEqual(self.contract[key], proposal[key])

    def test_handlers_name_real_entry_functions_not_synthetic_product_verbs(self):
        validate_handler_files(self.tree, self.contract)
        self.assertEqual({"validator", "selector"}, {c["id"] for c in self.contract["consumers"]})
        for consumer in self.contract["consumers"]:
            handler = consumer["handler_ref"]
            module = ast.parse(self.tree.read(handler["path"]))
            self.assertIn(handler["symbol"],
                          [n.name for n in module.body if isinstance(n, ast.FunctionDef)])
            self.assertEqual(Path(handler["path"]).stem + "." + handler["symbol"],
                             consumer["dispatch_key"])

    def test_both_cli_snapshots_and_real_selector_are_covered(self):
        consumers = {c["id"]: c for c in self.contract["consumers"]}
        self.assertEqual({"staged_cli", "candidate_cli"}, set(consumers["validator"]["entry_surfaces"]))
        self.assertEqual(["project_status"], consumers["selector"]["entry_surfaces"])
        self.assertEqual({f"INFRA-S0{i}" for i in range(1, 7)},
                         {s["id"] for s in self.contract["scenarios"]})
        for scenario in self.contract["scenarios"]:
            self.assertEqual(set(consumers), set(scenario["consumer_ids"]))
            self.assertEqual("required", scenario["dimensions"]["normal"]["disposition"])

    def test_authority_is_complete_but_not_its_own_progress_or_receipts(self):
        members = {p["path"] for p in authority_manifest(self.tree, self.contract, ready=False)}
        self.assertEqual({SPEC, CONTRACT, MECHANISM,
            "research/process-quality/WORKFLOW_INFRASTRUCTURE_CONTRACT_BASIS.md"}, members)
        self.assertNotIn(self.contract["proof_path"], members)
        self.assertNotIn(self.contract["review_path"], members)
        self.assertNotIn("specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md", members)

    def test_changed_mechanism_or_contract_requires_fresh_route_review(self):
        for path in (MECHANISM, CONTRACT, SPEC):
            with self.subTest(path=path):
                tree = RepositoryView({path: self.tree.read(path) + b"\n"})
                with self.assertRaises(DeliveryInputError) as raised:
                    authority_manifest(tree, self.contract, ready=False)
                self.assertEqual("WDQ-AUTHORITY", raised.exception.code)

    def test_operational_progress_does_not_rewrite_proof_authority(self):
        path = "specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md"
        tree = RepositoryView({path: self.tree.read(path) + b"\nOperational progress.\n"})
        self.assertEqual(authority_manifest(self.tree, self.contract, ready=False),
                         authority_manifest(tree, self.contract, ready=False))

    def test_future_result_records_do_not_supply_their_own_authority(self):
        tree = RepositoryView({self.contract["proof_path"]: b"fixture-proof-placeholder",
                               self.contract["review_path"]: b"fixture-review-placeholder"})
        self.assertEqual(authority_manifest(self.tree, self.contract, ready=False),
                         authority_manifest(tree, self.contract, ready=False))

    def test_adding_own_review_to_authority_is_refused(self):
        contract = deepcopy(self.contract)
        contract["review_path"] = SPEC
        with self.assertRaises(DeliveryInputError) as raised:
            authority_manifest(self.tree, contract, ready=False)
        self.assertEqual("WDQ-AUTHORITY", raised.exception.code)

    def test_input_inventory_is_explicit_and_foundations_do_not_fake_cad_proof(self):
        self.assertTrue(all(p.endswith((".py", ".sh", ".yml", "/pre-commit"))
                            for p in self.contract["input_roots"]))
        self.assertNotIn("scripts", self.contract["input_roots"])
        for key in ("units_precision", "numeric_entry", "selection_identity",
                    "grid_snap", "library_connectivity"):
            self.assertEqual("not_applicable", self.contract["foundation_answers"][key]["disposition"])
        for key in ("undo_cancel", "persistence_recovery", "settings_scope"):
            self.assertEqual("required", self.contract["foundation_answers"][key]["disposition"])


if __name__ == "__main__":
    unittest.main()
