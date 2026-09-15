#!/usr/bin/env python3
"""Synthetic N05-N08/N20 and P02-P04 protocol cases; production proof is G04."""

from copy import deepcopy
import unittest

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_native import validate_correlations, validate_environment
from workflow_delivery_native_test_support import environment, replace_role, typed_proof
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class CorrelationTests(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.proof, self.roles, self.event, self.registry = typed_proof(self.f)

    def check(self):
        return validate_correlations(Tree(self.f.root), self.f.contract, self.proof)

    def refuse(self, code):
        before = self.f.snapshot()
        with self.assertRaises(DeliveryInputError) as caught:
            self.check()
        self.assertEqual(caught.exception.code, code, str(caught.exception))
        self.assertEqual(before, self.f.snapshot())

    def test_p04_infrastructure_correlations(self):
        self.assertEqual(self.check(), {self.roles["events"][0]["sha256"]})

    def test_p02_honest_unavailability(self):
        self.f.contract["consumers"][0]["handler_ref"] = None
        self.proof, self.roles, self.event, self.registry = typed_proof(self.f, disabled=True)
        self.assertTrue(self.check())

    def test_p03_native_protocol_not_a_native_run(self):
        self.f.contract["category"] = "product"
        self.f.contract["scenarios"][0]["method"] = "native_input"
        self.proof, self.roles, self.event, self.registry = typed_proof(self.f)
        self.assertTrue(self.check())

    def test_n05_enabled_without_production_handler(self):
        self.registry["entries"][0]["handler_ref"] = None
        replace_role(self.f, self.proof, self.roles, "registry", self.registry)
        self.refuse("WDQ-CONSUMER")

    def test_n05_test_only_handler_disagrees(self):
        self.registry["entries"][0]["handler_ref"]["symbol"] = "tests.fake"
        replace_role(self.f, self.proof, self.roles, "registry", self.registry)
        self.refuse("WDQ-CONSUMER")

    def test_n06_shortcut_invokes_ineligible_context(self):
        self.event["dispatches"][0]["eligible"] = False
        replace_role(self.f, self.proof, self.roles, "events", self.event)
        self.refuse("WDQ-CONSUMER")

    def test_n06_disabled_path_mutates(self):
        self.f.contract["consumers"][0]["handler_ref"] = None
        self.proof, self.roles, self.event, self.registry = typed_proof(self.f, disabled=True)
        self.event["dispatches"][0]["mutation_count"] = 1
        replace_role(self.f, self.proof, self.roles, "events", self.event)
        self.refuse("WDQ-CONSUMER")

    def test_n07_success_label_without_correlated_result(self):
        self.event["actual_visible"] = "no panel or fit change"
        replace_role(self.f, self.proof, self.roles, "events", self.event)
        self.refuse("WDQ-RESULT")

    def test_n08_cli_record_cannot_substitute_native_input(self):
        self.f.contract["category"] = "product"
        self.f.contract["scenarios"][0]["method"] = "native_input"
        self.refuse("WDQ-RESULT")

    def test_n08_screenshot_without_typed_events(self):
        self.proof["results"][0]["artifacts"] = self.roles["captures"]
        self.refuse("WDQ-RESULT")

    def test_n20_requested_environment_not_just_original_replay(self):
        tree = Tree(self.f.root)
        validate_environment(tree, self.proof, environment())
        for change in ({"scale": 2}, {"backend": "another-renderer"},
                       {"input_method": "another-platform"}):
            requested = deepcopy(environment())
            requested.update(change)
            with self.assertRaises(DeliveryInputError) as caught:
                validate_environment(tree, self.proof, requested)
            self.assertEqual(caught.exception.code, "WDQ-ENVIRONMENT")
        with self.assertRaises(DeliveryInputError):
            validate_environment(tree, self.proof, None)

    def test_unknown_key_fallback_can_be_observed_but_not_invoked(self):
        fallback = deepcopy(self.event["dispatches"][0])
        fallback.update(dispatch_key="unknown", eligible=False, enabled=False, invoked=False,
                        handler_ref=None, unavailable_reason="Unknown action")
        self.event["dispatches"].append(fallback)
        self.roles = replace_role(self.f, self.proof, self.roles, "events", self.event)
        self.assertTrue(self.check())
        fallback["invoked"] = True
        replace_role(self.f, self.proof, self.roles, "events", self.event)
        self.refuse("WDQ-CONSUMER")

    def test_n10_reviewed_authority_update_does_not_refresh_old_proof(self):
        import hashlib
        self.f.write("docs/authority.md", b"<!-- RULE -->\nChanged reviewed requirement.\n")
        manifest = Tree(self.f.root).json("specs/evidence_traceability_manifest.json")
        route = manifest["routes"][0]
        digest = hashlib.sha256()
        for path in sorted(route["sources"] + route["consumers"]):
            digest.update(path.encode() + b"\0" + (self.f.root / path).read_bytes() + b"\0")
        route["reviewed_digest"] = digest.hexdigest()
        self.f.save("specs/evidence_traceability_manifest.json", manifest)
        self.refuse("WDQ-STALE")


if __name__ == "__main__":
    unittest.main()
