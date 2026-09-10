"""Explicit proposal preparation, not runtime environment observation or trust."""

from copy import deepcopy
import unittest

import test_workflow_delivery_environments as selection_support
from workflow_delivery_environment_selection_prepare import prepare_environment_selection
from workflow_delivery_environments import load_environments
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_tree import Tree


class SelectionPreparationTest(unittest.TestCase):
    def setUp(self):
        self.fixture = selection_support.EnvironmentSelectionTest()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.f = self.fixture.f
        self.policy = dict(self.fixture.policy, coverage={"rows": [
            {"frontier_key": "GUI", "category": "product"},
            {"frontier_key": "INFRA", "category": "infrastructure"}]})
        self.choices = deepcopy(self.fixture.selection["environments"])

    def prepare(self, choices=None):
        return prepare_environment_selection(self.fixture.authority, self.policy,
            self.choices if choices is None else choices)

    def test_explicit_mixed_selection_preserves_inputs_and_loads_when_separately_pinned(self):
        before = self.f.snapshot()
        original = deepcopy(self.choices)
        result = self.prepare()
        self.assertEqual(self.fixture.selection, result)
        self.assertEqual(original, self.choices)
        self.assertEqual(before, self.f.snapshot())
        # Existing synthetic authority already pins these exact bytes. Preparing
        # the document itself never creates that separate authority.
        self.assertEqual(self.fixture.expected, load_environments(Tree(self.f.root), "selection.json",
            self.policy, authority=self.fixture.authority))

    def test_missing_duplicate_unknown_and_swapped_choices_refuse(self):
        missing = self.choices[:1]
        duplicate = [self.choices[0], self.choices[0]]
        unknown = deepcopy(self.choices)
        unknown[0]["frontier_key"] = "UNKNOWN"
        swapped = deepcopy(self.choices)
        swapped[0]["environment"], swapped[1]["environment"] = swapped[1]["environment"], swapped[0]["environment"]
        for choices in (missing, duplicate, unknown, swapped):
            with self.subTest(choices=choices), self.assertRaises(DeliveryInputError):
                self.prepare(choices)

    def test_artifact_changes_and_legacy_policy_refuse(self):
        self.choices[0]["environment"]["sha256"] = "0" * 64
        with self.assertRaises(DeliveryInputError):
            self.prepare()
        self.policy["schema_version"] = 1
        with self.assertRaises(DeliveryInputError):
            self.prepare()


if __name__ == "__main__":
    unittest.main()
