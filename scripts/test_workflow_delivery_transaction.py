"""Real index/revision/worktree diff capture, never product proof."""

import unittest

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_test_support import Fixture
from workflow_delivery_transaction import changed_production_paths
from workflow_delivery_tree import Tree


class TransactionPathsTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)

    def changed(self, tree=None, roots=("src",)):
        return changed_production_paths(tree or Tree(self.f.root), list(roots),
                                        base_ref=self.f.head)

    def test_clean_and_unrelated_documents_do_not_change_production_paths(self):
        self.assertEqual([], self.changed())
        self.f.write("docs/unrelated.md", b"note\n")
        self.assertEqual([], self.changed())

    def test_worktree_includes_tracked_new_and_ignored_inputs(self):
        self.f.write("src/read.py", b"changed\n")
        self.f.write("src/new.py", b"new\n")
        self.f.write(".gitignore", b"src/generated.py\n")
        self.f.write("src/generated.py", b"generated\n")
        self.assertEqual(["src/generated.py", "src/new.py", "src/read.py"], self.changed())

    def test_index_ignores_unstaged_repair_and_untracked_inputs(self):
        self.f.write("src/read.py", b"staged change\n")
        self.f.stage()
        tree = Tree(self.f.root, staged=True)
        self.f.write("src/read.py", b"def read(path):\n    return path.read_bytes()\n")
        self.f.write("src/untracked.py", b"not staged\n")
        self.assertEqual(["src/read.py"], self.changed(tree))
        self.f.git("add", "src/read.py")
        self.assertEqual(["src/read.py"], self.changed(tree))
        self.assertEqual([], self.changed(Tree(self.f.root, staged=True)))

    def test_deleted_root_and_renamed_source_are_both_observed(self):
        self.f.git("mv", "src/read.py", "read.py")
        self.assertEqual(["src/read.py"], self.changed(Tree(self.f.root, staged=True)))
        self.assertEqual(["src/read.py"], self.changed())

    def test_file_mode_changes_are_not_hidden_by_identical_bytes(self):
        self.f.git("update-index", "--chmod=+x", "src/read.py")
        self.assertEqual(["src/read.py"], self.changed(Tree(self.f.root, staged=True)))

    def test_revision_view_never_reads_later_worktree(self):
        self.f.write("src/new.py", b"new\n")
        self.f.stage()
        self.f.git("commit", "-qm", "test(transaction): add fixture input\n\nExercise exact revision paths.")
        revision = self.f.git("rev-parse", "HEAD").decode().strip()
        tree = Tree(self.f.root, revision=revision)
        self.f.write("src/read.py", b"later dirty change\n")
        self.assertEqual(["src/new.py"], self.changed(tree))

    def test_missing_baseline_does_not_fall_back_to_head(self):
        with self.assertRaises(DeliveryInputError):
            changed_production_paths(Tree(self.f.root), ["src"], base_ref=None)

    def test_root_names_are_literal_not_glob_patterns(self):
        self.f.write("src/literal[1]/input.py", b"owned\n")
        self.f.write("src/literal1/input.py", b"not owned\n")
        self.assertEqual(["src/literal[1]/input.py"],
                         self.changed(roots=("src/literal[1]",)))


if __name__ == "__main__":
    unittest.main()
