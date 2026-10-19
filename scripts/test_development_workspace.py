"""Safety checks for separating native inputs from disposable Cargo output."""
import contextlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import development_workspace as workspace
import provision_s5a_replay_inputs as replay
import relocate_target_workspaces as relocate
import run_cargo_guarded as guard


class WorkspaceSafetyTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.checkout = self.base / 'checkout'
        self.checkout.mkdir()

    def test_external_paths_reject_checkout_aliases_and_traversal(self):
        (self.base / 'alias').symlink_to(self.checkout, target_is_directory=True)
        for path in [self.checkout / 'child', self.base / 'alias/child', self.base / 'external/../checkout', Path('relative')]:
            with self.assertRaises(ValueError):
                workspace.external(path, self.checkout)
        self.assertEqual(workspace.external(self.base / 'external/new', self.checkout), self.base / 'external/new')

    def test_scratch_child_symlink_cannot_redirect_to_checkout(self):
        external = self.base / 'external'; external.mkdir()
        (external / 'scratch').symlink_to(self.checkout, target_is_directory=True)
        with patch.object(workspace, 'root', lambda: external):
            with self.assertRaises(ValueError):
                workspace.scratch('family')
        (external / 'scratch').unlink(); (external / 'scratch').mkdir()
        (external / 'scratch/family').symlink_to(self.checkout, target_is_directory=True)
        with patch.object(workspace, 'root', lambda: external):
            with self.assertRaises(ValueError):
                workspace.scratch('family')
        self.assertFalse((self.checkout / 'family').exists())

    def test_frozen_replay_exception_does_not_admit_other_checkout_paths(self):
        valid = self.checkout / '.datum/scratch/c04-owner/s5a-replay-inputs'
        self.assertEqual(replay.replay_location(valid, self.checkout), valid)
        for relative in ['target/s5a-replay-inputs', '.datum/scratch/c04-owner',
                         '.datum/scratch/c04-owner/other', 'other/s5a-replay-inputs',
                         '.datum/scratch/c04.owner/s5a-replay-inputs',
                         '.datum/scratch/c04-owner/s5a-replay-inputs/child']:
            with self.subTest(relative=relative), self.assertRaises(ValueError):
                replay.replay_location(self.checkout / relative, self.checkout)
        self.assertFalse((self.checkout / '.datum').exists())

    def test_frozen_replay_exception_refuses_symlinks_and_cargo_overlap(self):
        scratch = self.checkout / '.datum/scratch'
        scratch.mkdir(parents=True)
        owner = scratch / 'c04-owner'
        owner.symlink_to(self.base / 'missing', target_is_directory=True)
        with self.assertRaises(ValueError):
            replay.replay_location(owner / 's5a-replay-inputs', self.checkout)
        owner.unlink()
        output = owner / 's5a-replay-inputs'
        for cargo in [scratch, output, output / 'build']:
            with patch.dict('os.environ', {'CARGO_TARGET_DIR': str(cargo)}):
                with self.assertRaises(ValueError):
                    replay.replay_location(output, self.checkout)
        self.assertFalse(owner.exists())

    def test_double_leading_slashes_cannot_bypass_replay_boundaries(self):
        def doubled(path):
            return Path('//' + str(path).lstrip('/'))

        valid = self.checkout / '.datum/scratch/c04-owner/s5a-replay-inputs'
        for checkout in [self.checkout, doubled(self.checkout)]:
            self.assertEqual(replay.replay_location(doubled(valid), checkout), valid)
            for relative in ['target/reviewer-probe', '.datum/unapproved-replay-root']:
                path = self.checkout / relative
                for candidate in [path, doubled(path)]:
                    with self.subTest(path=candidate, checkout=checkout):
                        with self.assertRaises(ValueError):
                            replay.replay_location(candidate, checkout)
        cargo = self.base / 'external-cargo'
        for target in [cargo, doubled(cargo)]:
            with patch.dict('os.environ', {'CARGO_TARGET_DIR': str(target)}):
                with self.assertRaises(ValueError):
                    replay.replay_location(doubled(cargo / 'native-copy'), self.checkout)
        self.assertFalse(cargo.exists())
        self.assertFalse((self.checkout / '.datum').exists())

    def test_legacy_suffix_resolution_and_incomplete_or_retired_refusal(self):
        row = {'source_relative': 'target/replay', 'destination': str(self.base / 'external'), 'state': 'preserved'}
        record = {'schema_version': 1, 'collections': [row]}
        self.assertEqual(workspace.relocated('target/replay/file', checkout=self.checkout, record=record), self.base / 'external/file')
        for path in ['target/replay-other/file', 'target/replay/../escape']:
            with self.assertRaises(ValueError):
                workspace.relocated(path, checkout=self.checkout, record=record)
        for state in ['prepared', 'retired']:
            row['state'] = state
            with self.assertRaises(ValueError):
                workspace.relocated('target/replay', checkout=self.checkout, record=record)
        row['state'] = 'preserved'; row['disposition'] = 'retired-released-test-output'
        with self.assertRaises(ValueError):
            workspace.relocated('target/replay', checkout=self.checkout, record=record)

    def test_delivery_recovery_is_scoped_and_uses_a_separate_map(self):
        control = self.checkout / '.datum/history-cleanup'
        control.mkdir(parents=True)
        record = {'schema_version': 1, 'collections': [
            {'source_relative': '.datum/library-gui-delivery',
             'destination': str(self.base / 'external/library'), 'state': 'preserved'}]}
        (control / 'development-relocation-map.json').write_text(json.dumps(record))
        self.assertEqual(workspace.relocated('.datum/library-gui-delivery/project/file', checkout=self.checkout),
                         self.base / 'external/library/project/file')
        self.assertFalse((control / 'target-relocation-map.json').exists())
        for source in ['.datum/history-cleanup', '.datum/footprint-reference-audit',
                       '.datum/library-gui-delivery/../escape']:
            record['collections'][0]['source_relative'] = source
            with self.assertRaises(ValueError):
                workspace.relocated('.datum/library-gui-delivery/file', checkout=self.checkout, record=record)
        record['collections'][0]['source_relative'] = '.datum/library-gui-delivery'
        alias = self.base / 'external/library/project'
        alias.parent.mkdir(parents=True)
        alias.symlink_to(self.checkout, target_is_directory=True)
        with self.assertRaises(ValueError):
            workspace.relocated('.datum/library-gui-delivery/project/file', checkout=self.checkout, record=record)

    def test_cargo_preflight_refuses_native_data_but_accepts_build_output(self):
        target = self.checkout / 'target'
        (target / 'debug').mkdir(parents=True)
        (target / 'debug/binary').write_bytes(b'ELF')
        guard.refuse_native_projects(target)
        (target / 'fixture').mkdir()
        (target / 'fixture/project.json').write_text('{}')
        with self.assertRaises(guard.ResourcePolicyError):
            guard.refuse_native_projects(target)

    def fixture(self, name='retained-replay'):
        source = self.checkout / 'target' / name
        source.mkdir(parents=True)
        (source / 'project.json').write_text('{"native":"preserve every byte"}')
        (source / 'empty-directory').mkdir()
        (source / 'project.json').chmod(0o640)
        return source

    @contextlib.contextmanager
    def relocation_context(self):
        control = self.checkout / '.datum/history-cleanup'
        external = workspace.external
        with contextlib.ExitStack() as stack:
            for owner, name, value in [
                (relocate, 'ROOT', self.checkout), (relocate, 'CONTROL', control),
                (relocate, 'MAP', control / 'target-relocation-map.json'),
                (workspace, 'root', lambda: self.base / 'external'),
                (workspace, 'external', lambda path, checkout=self.checkout: external(path, checkout)),
                (relocate, 'references', lambda name: []),
                (relocate.prune, 'refuse_open_output', lambda paths: None),
                (guard, 'acquire_lock', lambda *args: contextlib.nullcontext()),
            ]:
                stack.enter_context(patch.object(owner, name, value))
            stack.enter_context(patch.object(relocate.subprocess, 'check_output', return_value=b''))
            yield control

    def test_verified_move_preserves_all_bytes_modes_and_empty_directories(self):
        source = self.fixture(); before = relocate.manifest(source)
        with self.relocation_context() as control:
            record = relocate.plan(); relocate.apply(record)
            destination = Path(record['collections'][0]['destination'])
            self.assertEqual(before, relocate.manifest(destination))
            self.assertFalse(source.exists())
            receipt = json.loads((control / 'target-relocation-map.json').read_text())
            self.assertEqual(receipt['collections'][0]['state'], 'preserved')
            sealed = json.loads(Path(receipt['sealed_plan']['path']).read_text())
            self.assertEqual(sealed['collections'][0]['entries'], before)
            with self.assertRaises(ValueError):
                relocate.apply(record)

    def test_stale_plan_and_destination_collision_refuse_before_mutation(self):
        source = self.fixture()
        with self.relocation_context() as control:
            record = relocate.plan()
            (source / 'changed').write_text('user work after preview')
            with self.assertRaises(ValueError):
                relocate.apply(record)
            self.assertFalse(control.exists())
            self.assertTrue((source / 'changed').exists())
            record = relocate.plan()
            destination = Path(record['collections'][0]['destination']); destination.mkdir(parents=True)
            (destination / 'owned').write_text('never overwrite')
            with self.assertRaises(ValueError):
                relocate.apply(record)
            self.assertEqual((destination / 'owned').read_text(), 'never overwrite')
            self.assertTrue(source.exists())

    def test_only_explicit_released_test_collection_is_retired(self):
        source = self.fixture('open-project-app-tests')
        with self.relocation_context() as control:
            record = relocate.plan(); relocate.apply(record)
            self.assertFalse(source.exists())
            self.assertFalse(Path(record['collections'][0]['destination']).exists())
            receipt = json.loads((control / 'target-relocation-map.json').read_text())
            self.assertEqual(receipt['collections'][0]['state'], 'retired')
            self.assertTrue(Path(receipt['collections'][0]['manifest']).is_file())

    def test_unknown_symlink_entries_refuse_inventory(self):
        source = self.fixture(); (source / 'alias').symlink_to(self.base)
        with self.assertRaises(ValueError):
            relocate.manifest(source)


if __name__ == '__main__':
    unittest.main()
