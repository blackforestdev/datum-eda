"""Real index/revision/worktree diff capture, never product proof."""

import os
import shutil
import unittest
from unittest.mock import patch

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

    def stale_stat_cache(self):
        source = self.f.root / "src/read.py"
        copy = source.with_suffix(".copy")
        shutil.copy2(source, copy)
        copy.replace(source)

    def test_stale_stat_cache_does_not_write_index_or_report_unchanged_file(self):
        self.stale_stat_cache()
        index = self.f.root / ".git/index"
        before = index.read_bytes()
        self.assertEqual([], self.changed())
        self.assertEqual(before, index.read_bytes())

    def test_stale_stat_cache_keeps_real_changes_and_index_bytes(self):
        self.stale_stat_cache()
        self.f.write("src/unscoped.py", b"real new input\n")
        index = self.f.root / ".git/index"
        before = index.read_bytes()
        self.assertEqual(["src/unscoped.py"], self.changed())
        self.assertEqual(before, index.read_bytes())

    def test_worktree_diff_preserves_selected_and_default_indexes(self):
        selected = self.f.root / ".git/selected-index"
        default = self.f.root / ".git/index"
        shutil.copyfile(default, selected)
        self.stale_stat_cache()
        before = (selected.read_bytes(), default.read_bytes())
        with patch.dict(os.environ, {"GIT_INDEX_FILE": str(selected)}):
            self.assertEqual([], self.changed())
        self.assertEqual(before, (selected.read_bytes(), default.read_bytes()))

    def test_untracked_enumeration_uses_the_same_selected_index(self):
        selected = self.f.root / ".git/selected-index"
        default = self.f.root / ".git/index"
        shutil.copyfile(default, selected)
        self.f.write("src/new.py", b"new source\n")
        self.f.git("add", "src/new.py")
        before = (selected.read_bytes(), default.read_bytes())
        with patch.dict(os.environ, {"GIT_INDEX_FILE": str(selected)}):
            self.assertEqual(["src/new.py"], self.changed())
        self.assertEqual(before, (selected.read_bytes(), default.read_bytes()))

    def test_split_index_and_shared_indexes_stay_unchanged(self):
        self.f.git("update-index", "--split-index")
        self.stale_stat_cache()
        gitdir = self.f.root / ".git"
        names = [gitdir / "index", *gitdir.glob("sharedindex.*")]
        before = {str(p): p.read_bytes() for p in names}
        self.assertEqual([], self.changed())
        self.assertEqual(before, {str(p): p.read_bytes() for p in names})
        self.assertEqual(set(before), {str(gitdir / "index"), *map(str, gitdir.glob("sharedindex.*"))})

    def test_diff_failure_cleans_private_index_without_changing_original(self):
        self.stale_stat_cache()
        tree = Tree(self.f.root)
        original = tree.git
        index = self.f.root / ".git/index"
        before = index.read_bytes()
        def refuse(*args, **kwargs):
            if "diff" in args:
                raise RuntimeError("diagnostic diff failure")
            return original(*args, **kwargs)
        with patch.object(tree, "git", side_effect=refuse):
            with self.assertRaisesRegex(RuntimeError, "diagnostic diff failure"):
                self.changed(tree)
        self.assertEqual(before, index.read_bytes())
        self.assertEqual([], list(index.parent.glob(".datum-wdq-index-*")))

    def test_missing_index_is_not_created_by_worktree_inspection(self):
        index = self.f.root / ".git/index"
        index.unlink()
        self.assertEqual(["src/read.py"], self.changed())
        self.assertFalse(index.exists())
        self.assertEqual([], list(index.parent.glob(".datum-wdq-index-*")))

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

    def test_revision_range_keeps_reverted_and_deleted_intermediate_inputs(self):
        original = self.f.root.joinpath("src/read.py").read_bytes()
        self.f.write("src/read.py", b"intermediate change\n")
        self.f.write("src/transient.py", b"temporary implementation\n")
        self.f.stage()
        self.f.git("commit", "-qm", "fixture intermediate inputs")
        self.f.write("src/read.py", original)
        self.f.git("rm", "src/transient.py")
        self.f.stage()
        self.f.git("commit", "-qm", "fixture net-zero source delta")
        revision = self.f.git("rev-parse", "HEAD").decode().strip()
        self.assertEqual(["src/read.py", "src/transient.py"],
                         self.changed(Tree(self.f.root, revision=revision)))
        # The ordinary next staged transaction is genuinely empty. It is not
        # a retroactive certificate for the preceding commit sequence.
        self.assertEqual([], changed_production_paths(
            Tree(self.f.root, staged=True), ["src"], base_ref=revision))

    def test_revision_range_refuses_nonancestor_base(self):
        tree_id = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        sibling = self.f.git("commit-tree", tree_id, "-p", self.f.head,
                             "-m", "fixture sibling").decode().strip()
        other = self.f.git("commit-tree", tree_id, "-p", self.f.head,
                           "-m", "fixture other sibling").decode().strip()
        with self.assertRaisesRegex(DeliveryInputError, "must descend"):
            changed_production_paths(Tree(self.f.root, revision=sibling), ["src"],
                                     base_ref=other)

    def test_merge_range_keeps_side_parent_source_edits(self):
        self.f.write("src/read.py", b"side-parent edit\n")
        self.f.stage()
        side_tree = self.f.git("write-tree").decode().strip()
        side = self.f.git("commit-tree", side_tree, "-p", self.f.head,
                          "-m", "fixture side parent").decode().strip()
        original_tree = self.f.git("rev-parse", self.f.head + "^{tree}").decode().strip()
        merge = self.f.git("commit-tree", original_tree, "-p", self.f.head, "-p", side,
                           "-m", "fixture merge retaining original tree").decode().strip()
        self.assertEqual(["src/read.py"], self.changed(Tree(self.f.root, revision=merge)))

    def test_root_names_are_literal_not_glob_patterns(self):
        self.f.write("src/literal[1]/input.py", b"owned\n")
        self.f.write("src/literal1/input.py", b"not owned\n")
        self.assertEqual(["src/literal[1]/input.py"],
                         self.changed(roots=("src/literal[1]",)))


if __name__ == "__main__":
    unittest.main()
