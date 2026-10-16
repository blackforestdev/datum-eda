"""Behavioral regressions for holds, explicit idle, work start and delegated input."""
import copy
import io
import json
import unittest
from contextlib import redirect_stdout, redirect_stderr
from datetime import datetime, timedelta, timezone

import project_status as status
from project_work_authority import validate_item_authority, validate_appointments
import test_project_status as fixtures


class WorkAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.ProjectStatusTest()
        self.fixture.setUp()
        self.root = self.fixture.root
        self.manifest = self.fixture.manifest
        self.manifest.update(schema_version=7, selection={'mode': 'selected'})
        self.item = self.manifest['frontier'][0]
        self.issue = self.fixture.issues[0]
        self.doc = self.fixture.doc
        self.ref = {'path': self.doc, 'marker': '<!-- AUTHORITY -->'}
        doc = self.root / self.doc
        doc.write_text(doc.read_text() + '\n<!-- AUTHORITY -->\nRecorded owner disposition.\n')

    def tearDown(self):
        self.fixture.tearDown()

    def failures(self):
        self.fixture.write_fixture()
        return status.validate(self.root)[0]

    def run_cli(self, *args):
        self.fixture.write_fixture()
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(status.main(['--root', str(self.root), *args]), 0)
        return out.getvalue()

    def hold(self):
        self.item['hold'] = {'reason': 'Owner paused original sessions.',
                             'source_ref': self.ref,
                             'resume_requires': 'Actual owner resumption and synchronized claim.',
                             'completion_checkpoint': {'canonical_next_step_id': 'TEST-C01',
                                                       'step_statuses': {'TEST-C01': 'pending'}}}

    def claim(self):
        now = datetime.now(timezone.utc)
        self.item.update(state='in_progress', claim={
            'agent': 'codex', 'harness': 'codex-cli', 'session': 'test-worker',
            'worktree': str(self.root), 'head': self.fixture.head(),
            'scope': ['scripts'], 'claimed_at': now.isoformat(),
            'heartbeat_at': now.isoformat(), 'expires_at': (now + timedelta(hours=1)).isoformat(),
        })
        self.issue.update(status='in_progress', assignee='codex')
        self.item['completion']['steps'][0]['status'] = 'in_progress'

    def idle(self):
        self.item['canonical_next'] = False
        self.manifest['selection'] = {'mode': 'idle', 'reason': 'Accepted closure selects no successor.',
                                      'source_ref': self.ref}

    def appointment(self, classification='objective_acceptance'):
        self.fixture.configure_owner_boundary()
        step = self.item['completion']['steps'][0]
        source = self.root / self.doc
        marker = '<!-- APPOINTMENT -->'
        witness = {'owner_appointed': True, 'session': 'appointed-manager',
                   'task_keys': ['alignment'], 'decision_kinds': ['objective_acceptance', 'review_receipt']}
        source.write_text(source.read_text() + '\n' + marker + '\n' + json.dumps(witness) + '\n')
        self.manifest['management_appointments'] = {'sequence': {
            **{k: v for k, v in witness.items() if k != 'owner_appointed'},
            'source_ref': {'path': self.doc, 'marker': marker}}}
        self.item['decision_routing'] = {'TEST-C01': {'appointment': 'sequence',
                                                   'decision_kind': 'objective_acceptance'}}
        classification_marker = '<!-- CLASSIFICATION -->'
        source.write_text(source.read_text() + '\n' + classification_marker + '\n' + json.dumps({
            'task_key': 'alignment', 'step_id': 'TEST-C01', 'decision_kind': classification}) + '\n')
        step['decision_authority'] = {'kind': classification,
                                     'source_ref': {'path': self.doc, 'marker': classification_marker}}

    def test_held_next_and_named_details_preserve_grant_and_stop(self):
        self.hold()
        self.assertEqual(self.failures(), [])
        for command in [('next',), ('details', 'alignment')]:
            human = self.run_cli(*command)
            self.assertIn('Hold: Owner paused', human)
            self.assertIn('STOP: held work', human)
            self.assertNotIn('Before editing, atomically claim', human)
            payload = json.loads(self.run_cli(*command, '--json'))
            view = payload['next' if command[0] == 'next' else 'details']
            self.assertEqual(view['authorization'], 'execution')
            self.assertEqual(view['hold']['source_ref'], self.ref)
            self.assertIn('STOP:', view['work_start'])

    def test_held_task_rejects_claim_active_step_and_missing_provenance(self):
        self.hold()
        self.claim()
        self.assertIn('held work cannot carry a claim', '\n'.join(self.failures()))
        self.item.pop('claim')
        self.item['state'] = 'ready'
        self.issue.update(status='open', assignee=None)
        self.assertIn('held work cannot carry a claim', '\n'.join(self.failures()))
        self.item['completion']['steps'][0]['status'] = 'pending'
        self.item['hold']['source_ref']['marker'] = 'missing'
        self.assertIn('hold needs', '\n'.join(self.failures()))

    def test_held_completion_cannot_advance_even_without_a_claim(self):
        self.hold()
        self.item['completion']['steps'][0].update(status='complete', completion_evidence=[
            {'kind': 'commit', 'revision': self.fixture.head()}])
        self.item['completion']['canonical_next_step_id'] = None
        self.item['canonical_next'] = False
        self.manifest['selection'] = {'mode': 'idle', 'reason': 'No successor.', 'source_ref': self.ref}
        self.assertIn('held completion cannot advance', '\n'.join(self.failures()))

    def test_idle_next_and_untargeted_details_text_json_are_explicit(self):
        self.idle()
        self.assertEqual(self.failures(), [])
        for command in ['next', 'details']:
            self.assertIn('Next task: none; explicit idle', self.run_cli(command))
            payload = json.loads(self.run_cli(command, '--json'))
            self.assertIsNone(payload[command])
            self.assertEqual(payload['selection']['mode'], 'idle')
        self.assertIn('STOP: unselected', self.run_cli('details', 'alignment'))

    def test_idle_requires_durable_disposition_and_no_conflicting_selection(self):
        self.item['canonical_next'] = False
        self.assertTrue(self.failures())
        self.manifest.pop('selection')
        self.assertIn('absent selection does not mean idle', '\n'.join(self.failures()))
        self.idle()
        self.manifest['selection']['source_ref']['marker'] = 'missing'
        self.assertTrue(self.failures())
        self.manifest['selection']['source_ref'] = {'path': self.doc, 'marker': '<!-- AUTHORITY -->'}
        self.item['canonical_next'] = True
        self.assertIn('idle selection cannot also select', '\n'.join(self.failures()))

    def test_idle_cannot_hide_active_or_expired_ownership(self):
        self.claim()
        self.idle()
        self.assertIn('idle selection cannot hide', '\n'.join(self.failures()))
        self.item['claim']['expires_at'] = '2020-01-01T00:00:00Z'
        self.assertIn('claim expired', '\n'.join(self.failures()))

    def test_multiple_canonical_tasks_still_fail(self):
        second = copy.deepcopy(self.item)
        second.update(key='second', order=1, issue_id='dat-intake')
        self.manifest['frontier'].append(second)
        self.fixture.issues[1]['labels'] = ['roadmap:frontier']
        self.assertIn('exactly one canonical_next', '\n'.join(self.failures()))

    def test_completed_task_does_not_need_successor(self):
        self.idle()
        self.item.update(state='landed', authorization='none', landing_commit=self.fixture.head())
        self.item['completion']['canonical_next_step_id'] = None
        step = self.item['completion']['steps'][0]
        step.update(status='complete', completion_evidence=[{'kind': 'commit', 'revision': self.fixture.head()}])
        self.issue['status'] = 'closed'
        self.assertEqual(self.failures(), [])
        self.assertIn('STOP: completed work', self.run_cli('details', 'alignment'))

    def test_named_blocked_deferred_and_unselected_never_invite_claim(self):
        self.idle()
        for state in ['blocked', 'deferred', 'ready']:
            self.item.update(state=state, authorization='none' if state == 'deferred' else 'execution')
            self.issue['status'] = 'deferred' if state == 'deferred' else 'open'
            self.issue['labels'] = ['roadmap:deferred' if state == 'deferred' else 'roadmap:frontier']
            self.item['dependencies'] = ['dat-intake'] if state == 'blocked' else []
            self.issue['dependencies'] = [{'type': 'blocks', 'depends_on_id': 'dat-intake'}] if state == 'blocked' else []
            human = self.run_cli('details', 'alignment')
            self.assertIn('STOP:', human)
            self.assertNotIn('Before editing, atomically claim', human)

    def test_named_ungranted_placeholder_without_completion_is_honest(self):
        self.idle()
        self.item.update(state='planned', authorization='none')
        self.item.pop('completion')
        self.assertIn('STOP: unselected', self.run_cli('details', 'alignment'))
        view = json.loads(self.run_cli('details', 'alignment', '--json'))['details']
        self.assertIsNone(view['completion'])
        self.assertIn('STOP:', view['work_start'])

    def test_actively_owned_guidance_and_ready_none_authorization(self):
        self.item['authorization'] = 'none'
        self.assertIn('Before editing, atomically claim', self.run_cli('details'))
        self.claim()
        self.assertEqual(self.failures(), [])
        self.assertIn('Active claim: codex', self.run_cli('details'))
        self.assertNotIn('Before editing, atomically claim', self.run_cli('next'))

    def test_selector_itself_rejects_parent_child_claims(self):
        self.claim()
        self.item['claim']['scope'] = ['crates/gui-app/src']
        second = copy.deepcopy(self.item)
        second.update(key='second', order=1, issue_id='dat-intake', canonical_next=False)
        second['claim']['scope'] = ['crates/gui-app/src/main.rs']
        self.manifest['frontier'].append(second)
        self.fixture.issues[1].update(status='in_progress', assignee='codex', labels=['roadmap:frontier'])
        source = self.root / self.doc
        source.write_text(source.read_text() + '\n<!-- REQ:second:TEST-C01 -->\nSecond scope.\n')
        failures = self.failures()
        self.assertTrue(failures)
        self.assertTrue(all('overlapping active claims' in e for e in failures), failures)
        second['claim']['scope'] = ['crates/gui-protocol/src']
        self.assertEqual(self.failures(), [])

    def test_purpose_is_required_even_for_older_schema_and_ignores_tracker_labels(self):
        del self.item['work_purpose']
        for version in [5, 6, 7]:
            self.manifest['schema_version'] = version
            self.assertIn('requires explicit work_purpose', '\n'.join(self.failures()))

    def test_valid_manager_routing_same_in_next_details_and_reuses_appointment(self):
        self.appointment()
        self.assertEqual(self.failures(), [])
        for command in ['next', 'details']:
            human = self.run_cli(command)
            self.assertIn('Delegated boundary: INPUT REQUIRED', human)
            self.assertNotIn("request the project owner's", human)
            self.assertIn('appointed-manager', human)
            payload = json.loads(self.run_cli(command, '--json'))[command]
            self.assertFalse(payload['owner_input_required'])
            self.assertEqual(payload['decision_maker'], 'appointed-manager')
        self.item['decision_routing']['TEST-C01']['decision_kind'] = 'review_receipt'
        # A route cannot change the independently witnessed decision kind.
        self.assertTrue(json.loads(self.run_cli('details', '--json'))['details']['owner_input_required'])

    def test_reserved_decision_and_default_cannot_be_retagged_by_route(self):
        self.appointment('owner_reserved')
        self.assertEqual(self.failures(), [])
        for explicit in [True, False]:
            if not explicit:
                self.item['completion']['steps'][0].pop('decision_authority')
            view = json.loads(self.run_cli('next', '--json'))['next']
            self.assertTrue(view['owner_input_required'])
            self.assertEqual(view['decision_maker'], 'project_owner')

    def test_absent_out_of_scope_and_unsupported_routes_default_to_owner(self):
        self.appointment()
        route = self.item['decision_routing']['TEST-C01']
        for appointment, kind in [('absent', 'objective_acceptance'), ('sequence', 'new_product_rule')]:
            route.update(appointment=appointment, decision_kind=kind)
            self.assertIn('Owner boundary: INPUT REQUIRED', self.run_cli('details'))
        # A valid appointment restricted to a different task cannot cover this one.
        route.update(appointment='sequence', decision_kind='continuation')
        self.assertTrue(json.loads(self.run_cli('details', '--json'))['details']['owner_input_required'])

    def test_task_scope_is_checked_against_actual_receipt(self):
        self.appointment()
        other = self.fixture.item(key='other', order=1, issue_id='dat-intake', canonical_next=False,
                                  state='planned', authorization='planning')
        other.pop('completion')
        self.manifest['frontier'].append(other)
        self.fixture.issues[1]['labels'] = ['roadmap:frontier']
        appointment = self.manifest['management_appointments']['sequence']
        appointment['task_keys'] = ['other']
        source = self.root / self.doc
        source.write_text(source.read_text().replace('"task_keys": ["alignment"]', '"task_keys": ["other"]'))
        self.assertEqual(self.failures(), [])
        view = json.loads(self.run_cli('details', '--json'))['details']
        self.assertTrue(view['owner_input_required'])
        self.assertIsNone(view['decision_authority'])

    def test_sequence_appointment_reused_without_new_appointment(self):
        self.appointment()
        first = self.item['completion']['steps'][0]
        second = copy.deepcopy(first)
        second.update(id='TEST-C02', depends_on=['TEST-C01'],
                      requirement_refs=[{'path': self.doc, 'marker': 'TEST-C02'}])
        second['owner_input']['requests'][0]['source_ref']['marker'] = 'OPEN-2'
        marker = '<!-- CLASSIFICATION-2 -->'
        second['decision_authority']['source_ref']['marker'] = marker
        source = self.root / self.doc
        source.write_text(source.read_text() + '\n<!-- REQ:alignment:TEST-C02 -->\n'
                          '<!-- OWNER:alignment:TEST-C02:OPEN-2 -->\n' + marker + '\n' + json.dumps({
                              'task_key': 'alignment', 'step_id': 'TEST-C02',
                              'decision_kind': 'objective_acceptance'}) + '\n')
        first.update(status='complete', completion_evidence=[{'kind': 'review', 'path': self.doc,
                                                             'marker': '<!-- AUTHORITY -->'}])
        self.item['completion']['steps'].append(second)
        self.item['completion']['canonical_next_step_id'] = 'TEST-C02'
        self.item['decision_routing']['TEST-C02'] = copy.deepcopy(self.item['decision_routing']['TEST-C01'])
        self.issue['acceptance_criteria'] += ' TEST-C02: Second bounded acceptance.'
        self.assertEqual(self.failures(), [])
        self.assertEqual(json.loads(self.run_cli('details', '--json'))['details']['decision_maker'],
                         'appointed-manager')

    def assert_cli_refuses(self, commands, expected):
        # Invalid input must reach CLI validation, not fixture projection code.
        self.fixture.write_fixture(render=False)
        for command in commands:
            for json_mode in [False, True]:
                with self.subTest(command=command, json=json_mode):
                    out, err = io.StringIO(), io.StringIO()
                    args = ['--root', str(self.root), command] + (['--json'] if json_mode else [])
                    with redirect_stdout(out), redirect_stderr(err):
                        self.assertEqual(status.main(args), 1)
                    if json_mode:
                        payload = json.loads(out.getvalue())
                        self.assertFalse(payload['ok'])
                        self.assertIn(expected, '\n'.join(payload['failures']))
                    else:
                        self.assertIn(expected, err.getvalue())
                    self.assertNotIn('Traceback', out.getvalue() + err.getvalue())

    def test_explicit_null_decision_classification_refuses_all_query_modes(self):
        self.appointment()
        self.item['completion']['steps'][0]['decision_authority'] = None
        self.assert_cli_refuses(['next', 'details'], 'invalid governed decision classification')

    def test_explicit_null_selection_refuses_queries_and_render_checks(self):
        self.manifest['selection'] = None
        self.assert_cli_refuses(['next', 'details', 'check-render'], 'explicit selection is required')

    def test_fabricated_or_mismatched_appointment_witness_is_rejected(self):
        self.appointment()
        self.manifest['management_appointments']['sequence']['session'] = 'self-appointed'
        self.assertIn('exact owner appointment/scope receipt', '\n'.join(self.failures()))
        self.manifest['management_appointments']['sequence']['session'] = 'appointed-manager'
        self.item['completion']['steps'][0]['decision_authority']['kind'] = 'continuation'
        self.assertIn('invalid governed decision classification', '\n'.join(self.failures()))

    def test_helpers_refuse_malformed_shapes_without_tracebacks(self):
        for malformed in [None, [], 'bad']:
            self.item['completion'] = malformed
            self.item['hold'] = {'reason': 'hold', 'resume_requires': 'owner', 'source_ref': self.ref,
                                 'completion_checkpoint': {'canonical_next_step_id': None, 'step_statuses': {}}}
            self.assertEqual(validate_item_authority(self.root, self.item, self.fixture.governance['entries'], 7), [])
            self.assertEqual(validate_appointments(self.root, self.manifest, self.fixture.governance['entries']), [])
        self.manifest['management_appointments'] = []
        self.assertTrue(validate_appointments(self.root, self.manifest, {}))


if __name__ == '__main__':
    unittest.main()
