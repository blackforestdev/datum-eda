"""Refuse misleading native proof, including agreement without an exact oracle."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

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


if __name__ == '__main__':
    unittest.main()
