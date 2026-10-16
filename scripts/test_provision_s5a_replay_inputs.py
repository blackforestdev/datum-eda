"""Exact frozen input closure, independent maps, and destructive-path controls."""
import hashlib
import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import development_workspace as workspace
import provision_s5a_replay_inputs as replay


class ReplayInputs(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="datum-replay-inputs-")
        cls.base = Path(cls.temp.name)
        cls.output = cls.base / "private"
        cls.index = replay.provision(cls.output)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_all_frozen_family_files_match_independent_original_manifest(self):
        manifest = json.loads((replay.INPUTS.parent / "manifest.json").read_bytes())
        self.assertEqual(len(manifest['fixtures']), 11)
        for fixture in manifest['fixtures']:
            stem = fixture.get('frozen_root', fixture.get('persisted_root')).removeprefix('target/')
            for row in fixture['files']:
                path = self.output / stem / row.get('relative_path', row.get('path'))
                self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), row['sha256'], str(path))

    def test_recovered_setups_and_source_contracts_remain_exact(self):
        setup = self.output / 's5a-c04-inventory-keyboard-20261011'
        expected = [('n13-compound-move-setup.json', '508ed99ba365278b4ce26614df71e83aa8c590177f1049a23e8ff588c208bcbd'),
                    ('n14-two-project-setup.json', '784b724650b158c1de19426b209bcac0b5a653fbaf8217567f174e283f6c56f6')]
        for name, digest in expected:
            self.assertEqual(hashlib.sha256((setup / name).read_bytes()).hexdigest(), digest)
        n13 = json.loads((setup / expected[0][0]).read_bytes())
        root = self.output / 'user-edit-selection/journal-read-project'
        self.assertEqual(len(n13['canonical_maps']), 26)
        for rel, digest in n13['canonical_maps'].items():
            self.assertEqual(hashlib.sha256((root / rel).read_bytes()).hexdigest(), digest)
        for rel in n13['absent_source_roles']:
            self.assertFalse((root / rel).exists())
        n14 = json.loads((setup / expected[1][0]).read_bytes())
        self.assertEqual(sum(map(len, n14['canonical_maps'].values())), 69)
        self.assertNotEqual(n14['projects']['A']['project_id'], n14['projects']['B']['project_id'])

    def test_no_incidental_sessions_or_captures_are_packaged(self):
        for row in self.index['files']:
            for forbidden in ['tool-sessions', 'terminal', 'writer.lock', '.png', '.tar']:
                self.assertNotIn(forbidden, row['path'])

    def test_missing_generation_or_blob_refuses_complete_history_validation(self):
        rows = [r for r in self.index['files'] if r['path'].startswith('user-edit-selection/')]
        for role in ['/generations/', '/blobs/']:
            removed = next(r for r in rows if role in r['path'])
            broken = {**self.index, 'files': [r for r in rows if r is not removed]}
            with self.assertRaisesRegex(ValueError, 'incomplete revision closure'):
                replay.verify_history(broken, replay.INPUTS, {})

    def test_changed_project_binding_cannot_be_blessed_by_catalogue_hash(self):
        rows = [dict(r) for r in self.index['files'] if r['path'].startswith('user-edit-selection/')]
        head = next(r for r in rows if r['path'].endswith('/head.json'))
        value = json.loads((replay.INPUTS / head['source']).read_bytes())
        value['project_id'] = '00000000-0000-4000-8000-000000000000'
        data = replay.canonical(value)
        inputs = self.base / 'forged-inputs'
        for row in rows:
            destination = inputs / row['source']
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes((replay.INPUTS / row['source']).read_bytes())
        (inputs / head['source']).write_bytes(data)
        head.update(sha256=hashlib.sha256(data).hexdigest(), bytes=len(data))
        # checked_bytes is deliberately given a different source/hash here: the
        # independent generation binding must still refuse the forged identity.
        with self.assertRaisesRegex(ValueError, 'identity/ancestry mismatch'):
            replay.verify_history({**self.index, 'files': rows}, inputs, {})

    def test_inherited_prefix_is_exact_and_cannot_hide_missing_or_added_history(self):
        project = 's5a-c03-u5-semantic-v1/variants/semantic-scope'
        original = {**self.index, 'files': [r for r in self.index['files']
                                           if r['path'].startswith(project + '/')]}
        for damage in ['hash', 'identities', 'unregistered', 'extra-store-record']:
            broken = copy.deepcopy(original)
            prefix = broken['inherited_journal_prefixes'][project]
            if damage == 'hash':
                prefix['sha256'] = '0' * 64
            elif damage == 'identities':
                prefix['transaction_ids'].reverse()
            elif damage == 'unregistered':
                del broken['inherited_journal_prefixes'][project]
            else:
                head = next(r for r in broken['files'] if r['path'].endswith('/head.json'))
                broken['files'].append({**head, 'path': project + '/.datum/revision/v1/extra.json'})
            with self.assertRaises(ValueError):
                replay.verify_history(broken, replay.INPUTS, {})
        for name, pin in self.index['inherited_journal_prefixes'].items():
            root = self.output / name
            actual = sorted(p.relative_to(root).as_posix()
                            for p in (root / '.datum/revision/v1').rglob('*') if p.is_file())
            self.assertEqual(actual, pin['store_paths'])

    def test_existing_partial_and_checkout_outputs_refuse_before_mutation(self):
        before = (self.output / 'index.json').read_bytes()
        for path in [self.output, replay.ROOT / 'target/replay-inputs', Path('relative'),
                     self.base / 'other/../escape']:
            with self.assertRaises(ValueError):
                replay.provision(path)
        self.assertEqual(before, (self.output / 'index.json').read_bytes())
        partial = self.base / 'partial'
        partial.mkdir()
        (partial / 'owned').write_bytes(b'preserve')
        with self.assertRaises(ValueError):
            replay.provision(partial)
        self.assertEqual((partial / 'owned').read_bytes(), b'preserve')

    def test_symlink_output_and_ancestor_refuse_before_mutation(self):
        alias = self.base / 'output-alias'
        alias.symlink_to(self.output, target_is_directory=True)
        for path in [alias, alias / 'child', self.base / 'dangling']:
            if path.name == 'dangling':
                path.symlink_to(self.base / 'missing')
            with self.assertRaises(ValueError):
                replay.provision(path)
        self.assertFalse((self.output / 'child').exists())

    def test_selected_external_cargo_output_refuses_native_data(self):
        target = self.base / 'external-cargo'
        with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(target)}):
            for path in [target, target / 'native-project']:
                with self.assertRaises(ValueError):
                    replay.provision(path)
        self.assertFalse(target.exists())

    def test_damaged_or_symlink_seed_refuses_before_output_creation(self):
        inputs = self.base / 'damaged-inputs'
        inputs.mkdir()
        row = next(r for r in self.index['files'] if 'source' in r)
        index = {**self.index, 'files': [row]}
        (inputs / 'index.json').write_text(json.dumps(index))
        (inputs / 'dense-template.json').write_bytes((replay.INPUTS / 'dense-template.json').read_bytes())
        seed = inputs / row['source']
        seed.parent.mkdir(parents=True)
        seed.write_bytes(b'damaged')
        output = self.base / 'never-created'
        with self.assertRaises(ValueError):
            replay.provision(output, inputs)
        self.assertFalse(output.exists())
        seed.unlink()
        seed.symlink_to(replay.INPUTS / row['source'])
        with self.assertRaises(ValueError):
            replay.provision(output, inputs)
        self.assertFalse(output.exists())

    def test_resolution_uses_catalogue_and_private_copy_without_old_maps(self):
        with patch.dict(os.environ, {'DATUM_REPLAY_INPUTS': str(self.output),
                                    'DATUM_FIXTURE_RELOCATION_MAP': '/nonexistent'}):
            old = '/home/bfadmin/Documents/datum-eda/target/user-edit-selection/journal-read-project'
            self.assertEqual(workspace.relocated(old, checkout=self.base / 'clean'),
                             self.output / 'user-edit-selection/journal-read-project')
            with self.assertRaises(ValueError):
                workspace.relocated('target/user-edit-selection/../escape')
        marker = self.output / 'index.json'
        before = marker.read_bytes()
        try:
            marker.write_bytes(b'{}')
            with patch.dict(os.environ, {'DATUM_REPLAY_INPUTS': str(self.output)}):
                with self.assertRaises(ValueError):
                    workspace.relocated('target/user-edit-selection/journal-read-project')
        finally:
            marker.write_bytes(before)


if __name__ == '__main__':
    unittest.main()
