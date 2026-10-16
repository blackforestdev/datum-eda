"""Ownership collision regressions for normalized files, directories and aliases."""
import tempfile
import unittest
from pathlib import Path

from project_claim_scope import claim_scope_failures


class ClaimScopeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / 'tree/src').mkdir(parents=True)
        (self.root / 'tree/src/main.rs').touch()
        (self.root / 'alias').symlink_to(self.root / 'tree', target_is_directory=True)
        (self.root / 'tree/alias-src').symlink_to('src', target_is_directory=True)

    def tearDown(self):
        self.temp.cleanup()

    def check(self, left, right, left_tree='tree', right_tree='tree'):
        return claim_scope_failures(self.root, [
            ('left', {'scope': [left], 'worktree': left_tree}),
            ('right', {'scope': [right], 'worktree': right_tree}),
        ])

    def test_parent_child_and_reverse_refuse(self):
        self.assertTrue(self.check('src', 'src/main.rs'))
        self.assertTrue(self.check('src/main.rs', 'src'))

    def test_identical_dot_and_parent_normalization_refuse(self):
        self.assertTrue(self.check('src/main.rs', './src/main.rs'))
        self.assertTrue(self.check('src/main.rs', 'src/../src/main.rs'))

    def test_equivalent_worktrees_refuse(self):
        self.assertTrue(self.check('src', 'src/main.rs', 'tree', 'alias'))
        self.assertTrue(self.check('src', 'src/main.rs', str(self.root / 'tree'), 'tree/.'))

    def test_file_directory_symlink_alias_refuses(self):
        self.assertTrue(self.check('src/main.rs', 'alias-src/main.rs'))

    def test_disjoint_and_prefix_lookalikes_pass(self):
        self.assertFalse(self.check('src/main.rs', 'src/other.rs'))
        self.assertFalse(self.check('src', 'src-other/main.rs'))
        self.assertFalse(self.check('src', 'src', 'tree', 'separate-tree'))

    def test_future_child_paths_still_overlap(self):
        self.assertTrue(self.check('future', 'future/new.py'))

    def test_subsystems_are_explicit_global_exact_reservations(self):
        self.assertTrue(self.check('subsystem:selection', 'subsystem:selection', 'tree', 'other'))
        self.assertFalse(self.check('subsystem:selection', 'subsystem:selection-editor'))
        self.assertFalse(self.check('subsystem:selection', 'selection'))

    def test_escape_absolute_ambiguous_subsystem_and_symlink_escape_refuse(self):
        (self.root / 'tree/out').symlink_to(self.root / 'external')
        for bad in ['../outside', '/absolute', 'subsystem:', 'subsystem:a/b', 'foo:bar', 'out/file']:
            with self.subTest(bad=bad):
                errors = self.check(bad, 'src/main.rs')
                self.assertTrue(any('invalid claim scope identity' in e for e in errors))


if __name__ == '__main__':
    unittest.main()
