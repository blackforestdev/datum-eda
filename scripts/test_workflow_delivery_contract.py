#!/usr/bin/env python3
"""Closed schema, authority and hash invariants; not the complete N/P oracle."""

from copy import deepcopy
import unittest

from workflow_delivery_authority import authority_sha256
from workflow_delivery_contract import canonical_contract, validate_contract
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class ContractTests(unittest.TestCase):
    def setUp(self):
        self.fixture = Fixture()
        self.addCleanup(self.fixture.close)
        self.contract = deepcopy(self.fixture.contract)

    def validate(self):
        return validate_contract(self.contract, "contract.json", frontier_key="TASK", issue_id="dat-test")

    def refused(self, code):
        before = self.fixture.snapshot()
        with self.assertRaises(DeliveryInputError) as caught:
            self.validate()
        self.assertEqual(caught.exception.code, code)
        self.assertIn("contract.json", caught.exception.path)
        self.assertEqual(before, self.fixture.snapshot())

    def test_valid_structure_is_not_proof(self):
        self.assertEqual(self.validate(), self.contract)
        self.assertFalse((self.fixture.root / self.contract["proof_path"]).exists())

    def test_unknown_and_missing_closed_fields(self):
        for mutate in (lambda c: c.update(extra=True), lambda c: c.pop("scope"),
                       lambda c: c["consumers"][0].update(extra=True),
                       lambda c: c["scenarios"][0].update(extra=True)):
            self.contract = deepcopy(self.fixture.contract)
            mutate(self.contract)
            self.refused("WDQ-CONTRACT")

    def test_boolean_version_is_not_integer(self):
        self.contract["schema_version"] = True
        self.refused("WDQ-CONTRACT")

    def test_wrong_identity_and_duplicates(self):
        for mutate in (lambda c: c.update(issue_id="dat-other"),
                       lambda c: c.update(frontier_key="OTHER"),
                       lambda c: c["scenarios"].append(deepcopy(c["scenarios"][0])),
                       lambda c: c["consumers"].append(deepcopy(c["consumers"][0])),
                       lambda c: c["consumers"][0].update(scenario_ids=["UNKNOWN"])):
            self.contract = deepcopy(self.fixture.contract)
            mutate(self.contract)
            self.refused("WDQ-IDENTITY")

    def test_missing_coverage(self):
        for mutate in (lambda c: c.update(consumers=[]), lambda c: c.update(scenarios=[]),
                       lambda c: c["scenarios"][0]["dimensions"].pop("cancel"),
                       lambda c: c["foundation_answers"].pop("units_precision")):
            self.contract = deepcopy(self.fixture.contract)
            mutate(self.contract)
            self.refused("WDQ-COVERAGE")

    def test_empty_na_reason_or_authority(self):
        for field, bad in (("reason", " "), ("authority_refs", [])):
            self.contract = deepcopy(self.fixture.contract)
            self.contract["foundation_answers"]["units_precision"][field] = bad
            self.refused("WDQ-CONTRACT")

    def test_generic_or_outside_handler(self):
        for handler in ({"path": "src/read.py", "symbol": "gui_local"},
                        {"path": "tests/fake.py", "symbol": "fake.read"}):
            self.contract["consumers"][0]["handler_ref"] = handler
            self.refused("WDQ-CONSUMER")

    def test_product_cannot_use_engine_method(self):
        self.contract["category"] = "product"
        self.refused("WDQ-RESULT")

    def test_hash_normalizes_sets_not_input_sequence(self):
        self.contract["input_roots"].append("other")
        self.contract["scenarios"][0]["inputs"].append("Read again")
        before = canonical_contract(self.contract)
        self.contract["input_roots"].reverse()
        self.assertEqual(before, canonical_contract(self.contract))
        self.contract["scenarios"][0]["inputs"].reverse()
        self.assertNotEqual(before, canonical_contract(self.contract))

    def test_authority_anchor_missing_duplicate_and_stale(self):
        for raw in (b"missing", b"<!-- RULE --> <!-- RULE -->", b"<!-- RULE --> changed"):
            self.fixture.write("docs/authority.md", raw)
            with self.assertRaises(DeliveryInputError) as caught:
                authority_sha256(Tree(self.fixture.root), self.contract)
            self.assertEqual(caught.exception.code, "WDQ-AUTHORITY")

    def test_ready_requires_resolved_mandatory_question(self):
        self.contract["open_decisions"] = [{"id": "D1", "question": "Which scope?",
            "required_for_scenarios": ["S01"], "disposition_ref": None}]
        self.validate()
        tree = Tree(self.fixture.root)
        authority_sha256(tree, self.contract, ready=False)
        with self.assertRaises(DeliveryInputError) as caught:
            authority_sha256(tree, self.contract)
        self.assertEqual(caught.exception.code, "WDQ-AUTHORITY")

    def test_real_pilot_structure_and_current_authority(self):
        from pathlib import Path
        tree = Tree(Path(__file__).resolve().parents[1])
        path = "specs/workflow_delivery/pilot.contract.json"
        value = validate_contract(tree.json(path), path,
                                  frontier_key="WORKFLOW-DELIVERY-GATE-PILOT",
                                  issue_id="dat-workflow-gate-pilot-b3s")
        self.assertEqual(len(value["scenarios"]), 5)
        self.assertEqual(len(authority_sha256(tree, value)), 64)


if __name__ == "__main__":
    unittest.main()
