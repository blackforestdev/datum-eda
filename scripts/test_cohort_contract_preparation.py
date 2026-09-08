"""Draft contract shape and readiness refusals, never native delivery proof."""

import copy
import json
from pathlib import Path
import unittest

from workflow_delivery_authority import authority_sha256
from workflow_delivery_contract import validate_contract
from workflow_delivery_io import DeliveryInputError, parse_json


ROOT = Path(__file__).resolve().parents[1]
COHORTS = {
    "s5a": ("UVT-S5A-BUILD", "dat-uvt-s5a-build-1wv", 6),
    "gui-write-path": ("GUI-WRITE-PATH", "dat-gui-write-path-qiu", 6),
    "native-authoring": ("NATIVE-AUTHORING", "dat-native-authoring-depth-sf9", 8),
}


class RepositoryView:
    def read(self, path):
        return (ROOT / path).read_bytes()

    def json(self, path):
        return parse_json(self.read(path), path)


class CohortContractPreparationTest(unittest.TestCase):
    def setUp(self):
        self.tree = RepositoryView()

    def contract(self, slug):
        path = "specs/workflow_delivery/" + slug + ".contract.json"
        return path, self.tree.json(path)

    def test_real_contracts_match_existing_frontier_and_tracker_identities(self):
        items = {i["key"]: i for i in self.tree.json("specs/active_frontier.json")["frontier"]}
        issues = {i["id"] for i in map(json.loads,
                  (ROOT / ".beads/issues.jsonl").read_text().splitlines())}
        for slug, (key, issue, _) in COHORTS.items():
            with self.subTest(slug=slug):
                path, contract = self.contract(slug)
                validate_contract(contract, path, frontier_key=key, issue_id=issue)
                self.assertEqual(issue, items[key]["issue_id"])
                self.assertIn(issue, issues)

    def test_reserved_groups_are_not_dropped_by_a_passing_single_slice(self):
        for slug, (_, _, minimum) in COHORTS.items():
            _, contract = self.contract(slug)
            self.assertGreaterEqual(len(contract["scenarios"]), minimum)
            self.assertTrue(all(s["method"] == "native_input" for s in contract["scenarios"]))

    def test_proposed_mappings_preserve_distinct_phases_and_owner_authorization(self):
        packet = self.tree.json("docs/reviews/workflow-delivery-rollout/cohort-mappings.json")
        items = {i["key"]: i for i in self.tree.json("specs/active_frontier.json")["frontier"]}
        self.assertEqual({c[0] for c in COHORTS.values()},
                         {m["frontier_key"] for m in packet["mappings"]})
        for flag in ("readiness_asserted", "enrollment_asserted", "acceptance_asserted"):
            self.assertIs(False, packet[flag])
        for mapping in packet["mappings"]:
            steps = {s["id"]: s for s in items[mapping["frontier_key"]]["completion"]["steps"]}
            points = mapping["delivery"]["checkpoints"]
            self.assertEqual(5, len(set(points.values())))
            owner = mapping["intervening_owner_authorization"]
            self.assertEqual("owner_decision", steps[owner]["kind"])
            self.assertIn(points["ready"], steps[owner]["depends_on"])
            self.assertIn(owner, steps[points["activate"]]["depends_on"])
            self.assertEqual("execution", steps[points["review"]]["kind"])
            self.assertEqual("owner_decision", steps[points["accept"]]["kind"])

    def test_current_preparation_authority_is_resolvable_without_claiming_ready(self):
        for slug in COHORTS:
            _, contract = self.contract(slug)
            self.assertEqual(64, len(authority_sha256(self.tree, contract, ready=False)))

    def test_unresolved_mandatory_question_refuses_readiness(self):
        for slug in COHORTS:
            _, original = self.contract(slug)
            contract = copy.deepcopy(original)
            contract["open_decisions"] = [{
                "id": "test-unresolved-c01", "question": "Unresolved test input",
                "required_for_scenarios": [contract["scenarios"][0]["id"]],
                "disposition_ref": None,
            }]
            with self.assertRaisesRegex(DeliveryInputError, "mandatory owner question unresolved"):
                authority_sha256(self.tree, contract, ready=True)

    def test_future_proof_and_review_are_outside_their_authority_closure(self):
        routes = self.tree.json("specs/evidence_traceability_manifest.json")["routes"]
        for slug in COHORTS:
            _, contract = self.contract(slug)
            members = {p for r in routes if r["id"] in contract["route_ids"]
                       for p in r["sources"] + r["consumers"]}
            self.assertNotIn(contract["proof_path"], members)
            self.assertNotIn(contract["review_path"], members)
            self.assertNotEqual(contract["proof_path"], contract["review_path"])


if __name__ == "__main__":
    unittest.main()
