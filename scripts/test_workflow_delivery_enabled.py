"""Required normal delivery versus honest unavailable/refusal observations."""

from copy import deepcopy
import unittest

import test_workflow_delivery_review_checkpoint as support
from workflow_delivery_authority import authority_sha256
from workflow_delivery_checkpoints import validate_delivery
from workflow_delivery_contract import validate_contract
from workflow_delivery_enabled import required_normal_consumers
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_native_test_support import typed_proof
from workflow_delivery_proof import packet_sha256
from workflow_delivery_tree import Tree


class RequiredEnabledTest(unittest.TestCase):
    def setUp(self):
        self.h = support.ReviewCheckpointTest()
        self.h.setUp()
        self.addCleanup(self.h.doCleanups)
        self.f = self.h.f

    def disabled_runs(self):
        self.h.proof, _, _, _ = typed_proof(self.f, disabled=True)
        replay, _, _, _ = typed_proof(self.f, "reviewer", disabled=True)
        self.h.review["replay"] = self.f.blob("docs/reviews/replay.json", canonical_json(replay))
        self.f.save(self.f.contract["proof_path"], self.h.proof)
        self.h.review["packet_sha256"] = packet_sha256(self.f.contract, self.h.proof,
            authority_sha256(Tree(self.f.root), self.f.contract))
        self.h.save()

    def test_actual_review_cannot_use_unavailable_only_normal_observations(self):
        self.disabled_runs()
        code, report = self.h.cli()
        self.assertEqual(1, code, report)
        self.assertEqual("WDQ-CONSUMER", report["findings"][0]["code"])
        self.assertIn("required enabled behavior", report["findings"][0]["detail"])

    def test_required_handler_absence_refuses_readiness_before_future_proof(self):
        self.f.contract["consumers"][0]["handler_ref"] = None
        self.f.save("contract.json", self.f.contract)
        with self.assertRaisesRegex(DeliveryInputError, "no production handler"):
            validate_delivery(Tree(self.f.root), self.h.item, phase="ready")

    def test_all_normal_not_applicable_cannot_claim_enabled_product_delivery(self):
        self.f.contract["scenarios"][0]["dimensions"]["normal"]["disposition"] = "not_applicable"
        self.f.save("contract.json", self.f.contract)
        with self.assertRaisesRegex(DeliveryInputError, "at least one normal enabled scenario"):
            validate_delivery(Tree(self.f.root), self.h.item, phase="ready")

    def test_explicit_refusal_scenario_can_keep_an_unavailable_future_consumer(self):
        contract = deepcopy(self.f.contract)
        consumer = deepcopy(contract["consumers"][0])
        consumer.update(id="future", dispatch_key="future", handler_ref=None, scenario_ids=["S02"])
        scenario = deepcopy(contract["scenarios"][0])
        scenario.update(id="S02", consumer_ids=["future"])
        scenario["dimensions"]["normal"].update(disposition="not_applicable",
            reason="Governed unavailable-only future control, not delivered normal behavior")
        contract["consumers"].append(consumer)
        contract["scenarios"].append(scenario)
        validate_contract(contract, "fixture-contract")
        self.assertEqual({"S01": {"reader"}}, required_normal_consumers(contract))

    def test_infrastructure_does_not_claim_normal_native_product_delivery(self):
        contract = deepcopy(self.f.contract)
        contract["category"] = "infrastructure"
        self.assertEqual({}, required_normal_consumers(contract))


if __name__ == "__main__":
    unittest.main()
