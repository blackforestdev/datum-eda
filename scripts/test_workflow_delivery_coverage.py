"""Real Git-view tests for candidate coverage; not installed/native proof."""

from copy import deepcopy
import unittest

from workflow_delivery_coverage import validate_classifications
from workflow_delivery_coverage_shapes import coverage_shape
from workflow_delivery_io import DeliveryInputError
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class CoverageTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.item = {"key": "TASK", "issue_id": "dat-test", "state": "landed",
                     "authorization": "none", "landing_commit": self.f.head,
                     "completion": {"outcome": "Read fixture", "steps": [],
                                    "post_completion": {"unblocks_issue_ids": ["dat-next"]}}}
        self.manifest = {"frontier": [self.item]}
        self.f.save("specs/active_frontier.json", self.manifest)
        self.f.stage()
        self.f.git("commit", "-qm", "test(coverage): establish historical fixture\n\nBind the test-only baseline to a real Git tree.")
        self.baseline = self.f.git("rev-parse", "HEAD").decode().strip()
        self.authority = Tree(self.f.root, revision=self.baseline)
        self.coverage = {"baseline_ref": self.baseline, "production_roots": ["src"],
                         "source_scopes": [], "new_item_rule": "classification_required",
                         "rows": [{"frontier_key": "TASK", "issue_id": "dat-test",
                                   "category": "historical", "boundary_ref": self.f.ref,
                                   "external_handoff_ref": None}]}

    def validate(self, *, staged=False):
        tree = Tree(self.f.root, staged=staged)
        return validate_classifications(tree, self.coverage,
                                        tree.json("specs/active_frontier.json"),
                                        authority=self.authority)

    def save(self):
        self.f.save("specs/active_frontier.json", self.manifest)

    def test_valid_baseline_is_read_only(self):
        before = self.f.git("status", "--porcelain")
        self.assertEqual({"TASK"}, set(self.validate()))
        self.assertEqual(before, self.f.git("status", "--porcelain"))

    def test_new_or_missing_classification_refuses(self):
        self.manifest["frontier"].append({"key": "NEW", "issue_id": "dat-new"})
        self.save()
        with self.assertRaisesRegex(DeliveryInputError, "classification mismatch"):
            self.validate()

    def test_duplicate_frontier_or_policy_key_refuses(self):
        self.coverage["rows"].append(deepcopy(self.coverage["rows"][0]))
        with self.assertRaises(DeliveryInputError):
            self.validate()
        self.coverage["rows"].pop()
        self.manifest["frontier"].append(deepcopy(self.item))
        self.save()
        with self.assertRaisesRegex(DeliveryInputError, "duplicate Frontier"):
            self.validate()

    def test_changed_issue_cannot_inherit_historical_permission(self):
        self.item["issue_id"] = "dat-other"
        self.save()
        with self.assertRaisesRegex(DeliveryInputError, "issue identity changed"):
            self.validate()

    def test_reopened_history_refuses_but_live_unblocks_may_shrink(self):
        self.item["completion"]["post_completion"]["unblocks_issue_ids"] = []
        self.save()
        self.validate()
        self.item["state"] = "ready"
        self.save()
        with self.assertRaisesRegex(DeliveryInputError, "historical reopening"):
            self.validate()

    def test_rewritten_historical_obligation_refuses(self):
        self.item["completion"]["steps"].append({"id": "NEW", "kind": "execution"})
        self.save()
        with self.assertRaisesRegex(DeliveryInputError, "historical completion"):
            self.validate()

    def test_index_does_not_use_repaired_worktree(self):
        self.item["state"] = "ready"
        self.save()
        self.f.stage()
        self.item["state"] = "landed"
        self.save()
        self.validate()
        with self.assertRaisesRegex(DeliveryInputError, "historical reopening"):
            self.validate(staged=True)

    def test_boundary_text_cannot_change_behind_same_marker(self):
        self.f.write(self.f.ref["path"], b"<!-- RULE -->\nCandidate grants itself permission.\n")
        with self.assertRaisesRegex(DeliveryInputError, "boundary changed"):
            self.validate()
        self.validate(staged=True)

    def test_shape_is_closed_and_permission_paths_are_bounded(self):
        self.coverage["unexpected"] = True
        with self.assertRaises(DeliveryInputError):
            coverage_shape(self.coverage)
        del self.coverage["unexpected"]
        for path in ("../src", ".git/config", "src/.git/config"):
            with self.subTest(path=path), self.assertRaises(DeliveryInputError):
                self.coverage["production_roots"] = [path]
                coverage_shape(self.coverage)

    def test_external_scope_requires_explicit_handoff(self):
        self.coverage["rows"][0]["category"] = "external_lane"
        with self.assertRaisesRegex(DeliveryInputError, "handoff reference"):
            coverage_shape(self.coverage)


if __name__ == "__main__":
    unittest.main()
