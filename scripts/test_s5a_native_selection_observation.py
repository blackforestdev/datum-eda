"""Refuse misleading native proof, including agreement without an exact oracle."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from s5a_native_selection_observation import KIND, PREFIX, latest_state, qualify
from native_project_source_inventory import digest, validate_frozen_sources


class SelectionObservationTests(unittest.TestCase):
    def setUp(self):
        root = Path(__file__).resolve().parents[1]
        fixture = root / 'crates/test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-majority-v1/s5a-fixture.json'
        contract = json.loads(fixture.read_text())
        members = sorted(({'class': 'symbol', 'id': s['source_id']}
                          for s in contract['subjects']), key=lambda v: v['id'])
        self.expected = {'pid': 123, 'project': contract['project_id'],
                         'revision': contract['model_revision'], 'kind': 'compound',
                         'members': members, 'oracle': str(fixture.relative_to(root))}
        basis = {'project-id': contract['project_id'], 'model-revision': contract['model_revision'],
                 'subject-kind': 'compound'}
        self.snapshot = {'pid': 123, 'before': basis, 'after': dict(basis), 'count': 2,
                         'members': [{'attributes': {**basis, 'selected': 'true',
                                      'source-identity': json.dumps(m)}, 'standard_selected': True}
                                     for m in members]}
        self.record = {'kind': KIND, 'event': 'state', 'sequence': 7, 'value': {'selection': {
            'kind': 'compound', 'project': contract['project_id'], 'revision': contract['model_revision'],
            'scope': None, 'member_count': 2, 'complete_inspection': False, 'stale': False,
            # Actual production v2 digest from native-wayland-majority-04.
            'members_sha256': 'sha256:b0c36f54d53ae8a7868c9a8ea0fc79b1b0a14cd848e84ad4fd6af8a687388ffa'}}}

    def test_full_public_read_qualifies_against_frozen_fixture(self):
        result = qualify(self.record, self.snapshot, self.expected)
        self.assertEqual(result['members'], self.expected['members'])

    def test_parent_checkpoint_focus_is_independent_and_complete(self):
        expected, record, snapshot = map(copy.deepcopy, (self.expected, self.record, self.snapshot))
        focus = expected['members'][1]
        expected['focus'] = focus
        record['value']['selection']['focus'] = {
            **focus, 'instance_path_count': 0, 'instance_path_sample': [],
            'instance_path_sha256': 'sha256:' + hashlib.sha256(b'[]').hexdigest()}
        for i, member in enumerate(snapshot['members']):
            member['attributes']['selection-focus'] = str(i == 1).lower()
        qualify(record, snapshot, expected)
        for damage in ['missing', 'other-parent', 'truncated-path', 'boolean-count', 'path-hash', 'public-focus']:
            r, s = copy.deepcopy(record), copy.deepcopy(snapshot)
            if damage == 'missing':
                del r['value']['selection']['focus']
            elif damage == 'other-parent':
                r['value']['selection']['focus']['id'] = expected['members'][0]['id']
            elif damage == 'truncated-path':
                r['value']['selection']['focus']['instance_path_count'] = 1
            elif damage == 'boolean-count':
                r['value']['selection']['focus']['instance_path_count'] = False
            elif damage == 'path-hash':
                r['value']['selection']['focus']['instance_path_sha256'] = 'sha256:wrong'
            else:
                s['members'][0]['attributes']['selection-focus'] = 'true'
            with self.subTest(damage=damage), self.assertRaises(ValueError):
                qualify(r, s, expected)
        expected['focus'] = None
        with self.assertRaises(ValueError):
            qualify(record, snapshot, expected)
        record['value']['selection']['focus'] = None
        for member in snapshot['members']:
            member['attributes']['selection-focus'] = 'false'
        qualify(record, snapshot, expected)

    def test_single_parent_object_focus_qualifies_without_a_child_member(self):
        expected, record, snapshot = map(copy.deepcopy, (self.expected, self.record, self.snapshot))
        focus = expected['members'][0]
        expected.update(kind='object', members=[focus], focus=focus, scope='symbol')
        snapshot['members'] = snapshot['members'][:1]
        snapshot['count'] = 1
        for attrs in [snapshot['before'], snapshot['after'], snapshot['members'][0]['attributes']]:
            attrs['subject-kind'] = 'object'
        snapshot['members'][0]['attributes']['selection-focus'] = 'true'
        record['value']['selection'].update(kind='object', member_count=1, scope='symbol',
            members_sha256='sha256:' + hashlib.sha256(json.dumps([focus], separators=(',', ':')).encode()).hexdigest(),
            focus={**focus, 'instance_path_count': 0, 'instance_path_sample': [],
                   'instance_path_sha256': 'sha256:' + hashlib.sha256(b'[]').hexdigest()})
        qualify(record, snapshot, expected)
        wrong_scope = {**expected, 'scope': None}
        with self.assertRaisesRegex(ValueError, 'inspection scope'):
            qualify(record, snapshot, wrong_scope)
        expected['members'] = [{'class': 'pin', 'id': focus['id']}]
        with self.assertRaises(ValueError):
            qualify(record, snapshot, expected)

    def test_compact_counts_do_not_replace_missing_or_duplicate_members(self):
        for members in [[], self.snapshot['members'][:1], self.snapshot['members'][:1] * 2]:
            with self.subTest(members=members), self.assertRaises(ValueError):
                qualify(self.record, {**self.snapshot, 'members': members}, self.expected)

    def test_projection_agreement_does_not_replace_independent_oracle(self):
        expected = copy.deepcopy(self.expected)
        expected['members'][0]['instance_path'] = ['10000000-0000-4000-8000-000000000001']
        with self.assertRaises(ValueError):
            qualify(self.record, self.snapshot, expected)

    def test_actual_wire_digest_requires_exact_namespace(self):
        for prefix in ['', 'sha512:', 'sha256:sha256:']:
            record = copy.deepcopy(self.record)
            record['value']['selection']['members_sha256'] = prefix + 'b0c36f54d53ae8a7868c9a8ea0fc79b1b0a14cd848e84ad4fd6af8a687388ffa'
            with self.subTest(prefix=prefix), self.assertRaises(ValueError):
                qualify(record, self.snapshot, self.expected)

    def test_nonstale_requires_explicit_boolean_false(self):
        for value in [True, None, 0, 1, '', 'false', [], {}]:
            record = copy.deepcopy(self.record)
            record['value']['selection']['stale'] = value
            with self.subTest(value=value), self.assertRaises(ValueError):
                qualify(record, self.snapshot, self.expected)
        record = copy.deepcopy(self.record)
        del record['value']['selection']['stale']
        with self.assertRaises(ValueError):
            qualify(record, self.snapshot, self.expected)

    def test_foreign_stale_or_unselected_data_refuses(self):
        for damage in ['pid', 'project', 'revision', 'during-read', 'standard-state', 'digest']:
            snapshot, record = copy.deepcopy(self.snapshot), copy.deepcopy(self.record)
            if damage == 'pid':
                snapshot['pid'] = 456
            elif damage in ['project', 'revision']:
                snapshot['members'][0]['attributes'][{'project': 'project-id', 'revision': 'model-revision'}[damage]] = 'foreign'
            elif damage == 'during-read':
                snapshot['after']['subject-kind'] = 'object'
            elif damage == 'standard-state':
                snapshot['members'][0]['standard_selected'] = False
            else:
                record['value']['selection']['members_sha256'] = '0' * 64
            with self.subTest(damage=damage), self.assertRaises(ValueError):
                qualify(record, snapshot, self.expected)

    def test_incomplete_old_or_nonmonotonic_diagnostics_refuse(self):
        line = lambda r: PREFIX + json.dumps(r) + '\n'
        self.assertEqual(latest_state(line(self.record)), self.record)
        for record in [{**self.record, 'event': 'diagnostics_incomplete'},
                       {**self.record, 'kind': 'datum.action-evidence/v1'}, self.record]:
            with self.assertRaises(ValueError):
                latest_state(line(self.record) + line(record))
        with self.assertRaises(ValueError):
            latest_state(line(self.record).rstrip('\n'))

    def test_frozen_source_identity_is_more_than_an_unchanged_marker(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source.json'
            source.write_bytes(b'original frozen bytes')
            expected = {'source.json': digest(source)}
            self.assertEqual(validate_frozen_sources(root, expected)['canonical_files'], expected)
            lock = root / '.datum/revision/writer.lock'
            lock.parent.mkdir(parents=True)
            lock.touch()
            self.assertTrue(validate_frozen_sources(root, expected)['operational_writer_lock_present'])
            source.write_bytes(b'changed, even if a copied marker is unchanged')
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)
            source.write_bytes(b'original frozen bytes')
            (root / 'unexpected-source.json').write_bytes(b'extra')
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)

    def test_catalogue_keeps_electrical_ids_and_nested_project_history(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = ['project.json', '.datum/electrical_identities/net.json',
                     'variants/second/.datum/journal/cursor.json']
            for name in paths:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'fixed source bytes')
            expected = {name: digest(root / name) for name in paths}
            self.assertEqual(validate_frozen_sources(root, expected)['canonical_files'], expected)
            runtime = root / '.datum/tool-sessions/session/context.json'
            runtime.parent.mkdir(parents=True)
            runtime.write_bytes(b'operational bytes outside canonical inventory')
            def canonical_only(path):
                if path == runtime:
                    raise AssertionError('operational payload must not be read')
                return digest(path)
            with patch('native_project_source_inventory.digest', side_effect=canonical_only):
                self.assertEqual(validate_frozen_sources(root, expected)['canonical_files'], expected)
                with self.assertRaises(ValueError):
                    validate_frozen_sources(root, {**expected, runtime.relative_to(root).as_posix(): 'unread'})
            (root / paths[-1]).write_bytes(b'changed nested history')
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)
            (root / paths[-1]).write_bytes(b'fixed source bytes')
            extra = root / '.datum/electrical_identities/untracked.json'
            extra.write_bytes(b'extra canonical identity')
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)

    def test_matching_bytes_do_not_bless_canonical_symlinks_or_similar_operational_names(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / 'target.bin'
            target.write_bytes(b'frozen bytes')
            link = root / 'source.json'
            link.symlink_to(target)
            expected = {'target.bin': digest(target), 'source.json': digest(target)}
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)
            link.unlink()
            expected.pop('source.json')
            extra = root / '.datum/tool-sessions-unapproved/payload.json'
            extra.parent.mkdir(parents=True)
            extra.write_bytes(b'not an established operational role')
            with self.assertRaises(ValueError):
                validate_frozen_sources(root, expected)


if __name__ == '__main__':
    unittest.main()
