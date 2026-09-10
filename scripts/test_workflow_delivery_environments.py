"""Synthetic selection tests over real Git snapshots; not observed native proof."""

from copy import deepcopy
import unittest

from workflow_delivery_environments import load_environments
from workflow_delivery_io import DeliveryInputError, canonical_json
from workflow_delivery_native_test_support import environment
from test_workflow_delivery_headless import environment as headless_environment
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class EnvironmentSelectionTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.policy = {"schema_version": 2, "enrolled": [
            {"frontier_key": "GUI"}, {"frontier_key": "INFRA"}]}
        self.expected = {"GUI": environment(), "INFRA": headless_environment()}
        self.selection = {"schema_version": 2, "kind": "datum.workflow-delivery.environments",
            "environments": [{"frontier_key": key, "environment": self.f.blob(
                key + ".json", canonical_json(value))} for key, value in self.expected.items()]}
        self.pin()

    def pin(self):
        self.f.save("selection.json", self.selection)
        self.f.stage()
        self.f.git("commit", "--allow-empty", "-qm", "fixture pin selected environments")
        self.authority = Tree(self.f.root, revision=self.f.git("rev-parse", "HEAD").decode().strip())

    def load(self, tree=None, **kwargs):
        return load_environments(tree or Tree(self.f.root), "selection.json", self.policy,
                                 authority=kwargs.get("authority", self.authority))

    def reject(self, **kwargs):
        with self.assertRaises(DeliveryInputError) as caught:
            self.load(**kwargs)
        self.assertEqual("WDQ-ENVIRONMENT", caught.exception.code)

    def test_mixed_environments_are_separate_in_worktree_index_and_commit(self):
        before = self.f.snapshot()
        for tree in (Tree(self.f.root), Tree(self.f.root, staged=True), self.authority):
            self.assertEqual(self.expected, self.load(tree))
        self.assertEqual(before, self.f.snapshot())

    def test_missing_duplicate_unknown_keys_refuse_even_when_pinned(self):
        original = deepcopy(self.selection)
        for keys in (("GUI",), ("GUI", "GUI"), ("GUI", "OTHER")):
            self.selection = deepcopy(original)
            self.selection["environments"] = [dict(original["environments"][0], frontier_key=k)
                                              for k in keys]
            self.pin()
            self.reject()

    def test_closed_version_and_entry_shapes_refuse(self):
        original = deepcopy(self.selection)
        for change in ({"schema_version": True}, {"schema_version": 2.0},
                       {"schema_version": 1}, {"kind": "other"}, {"extra": True},
                       {"environments": []}, {"environments": [{}]}):
            self.selection = dict(deepcopy(original), **change)
            self.pin()
            self.reject()

    def test_changed_selection_cannot_swap_environment_authority(self):
        rows = self.selection["environments"]
        rows[0]["environment"], rows[1]["environment"] = rows[1]["environment"], rows[0]["environment"]
        self.f.save("selection.json", self.selection)
        self.reject()
        self.assertEqual(self.expected, self.load(Tree(self.f.root, staged=True)))
        self.f.stage()
        self.reject(tree=Tree(self.f.root, staged=True))

    def test_blob_bytes_hashes_and_authority_membership_are_checked(self):
        self.f.save("GUI.json", {"invented": True})
        self.reject()
        self.selection["environments"][0]["environment"] = self.f.blob(
            "candidate-only.json", canonical_json(environment()))
        self.f.save("selection.json", self.selection)
        self.f.stage()
        self.reject(tree=Tree(self.f.root, staged=True))

    def test_bad_blob_digest_or_path_refuses_in_pinned_document(self):
        original = deepcopy(self.selection)
        for patch in ({"sha256": "0" * 64}, {"path": "../outside"},
                      {"path": "/absolute"}, {"path": "missing.json"}):
            self.selection = deepcopy(original)
            self.selection["environments"][0]["environment"].update(patch)
            self.pin()
            self.reject()

    def test_candidate_cannot_supply_blob_missing_from_authority(self):
        self.f.git("rm", "GUI.json")
        self.pin()
        self.f.save("GUI.json", self.expected["GUI"])
        self.f.stage()
        self.reject(tree=Tree(self.f.root, staged=True))

    def test_schema2_requires_explicit_path_and_authority(self):
        self.reject(authority=None)
        with self.assertRaises(DeliveryInputError):
            load_environments(Tree(self.f.root), None, self.policy, authority=self.authority)

    def test_legacy_single_environment_and_absent_request_remain_unchanged(self):
        policy = dict(self.policy, schema_version=1)
        result = load_environments(Tree(self.f.root), "GUI.json", policy)
        self.assertEqual({key: environment() for key in self.expected}, result)
        self.assertEqual({key: None for key in self.expected},
                         load_environments(Tree(self.f.root), None, policy))


if __name__ == "__main__":
    unittest.main()
