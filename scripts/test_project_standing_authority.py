"""Proposed standing owner delegation; synthetic provenance, no live authority changes."""
import copy
import json
import unittest

import test_project_work_authority as legacy
from project_work_authority import decision_view, validate_appointments


class StandingAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.h = legacy.WorkAuthorityTests()
        self.h.setUp()
        self.addCleanup(self.h.tearDown)

    def project_appointment(self, kinds=None):
        self.h.appointment('owner_reserved')
        a = self.h.manifest['management_appointments']['sequence']
        old = {'owner_appointed': True, **{k: v for k, v in a.items() if k != 'source_ref'}}
        a['task_keys'] = ['*']
        a['decision_kinds'] = kinds or ['objective_acceptance', 'review_receipt', 'owner_reserved']
        new = {'owner_appointed': True, **{k: v for k, v in a.items() if k != 'source_ref'}}
        source = self.h.root / self.h.doc
        text = source.read_text()
        self.assertEqual(text.count(json.dumps(old)), 1)
        source.write_text(text.replace(json.dumps(old), json.dumps(new)))
        self.h.item['decision_routing'] = {}
        return a

    def view(self):
        return json.loads(self.h.run_cli('details', '--json'))['details']

    def test_one_owner_receipt_routes_reserved_next_and_details_to_pm(self):
        self.project_appointment()
        self.assertEqual(self.h.failures(), [])
        before = copy.deepcopy(self.h.item)
        for command in ['next', 'details']:
            v = json.loads(self.h.run_cli(command, '--json'))[command]
            self.assertFalse(v['owner_input_required'])
            self.assertTrue(v['decision_input_required'])
            self.assertEqual(v['decision_maker'], 'appointed-manager')
        self.assertEqual(self.h.item, before)

    def test_project_scope_covers_valid_new_task_without_another_owner_receipt(self):
        self.project_appointment()
        original_appointment = copy.deepcopy(self.h.manifest['management_appointments'])
        self.h.item['key'] = 'future-authorized-project-task'
        source = self.h.root / self.h.doc
        source.write_text(source.read_text().replace('alignment', self.h.item['key']))
        self.assertEqual(self.h.failures(), [])
        for command in ['next', 'details']:
            v = json.loads(self.h.run_cli(command, '--json'))[command]
            self.assertFalse(v['owner_input_required'])
            self.assertFalse(v['classification_required'])
            self.assertEqual(v['decision_maker'], 'appointed-manager')
        self.assertEqual(self.h.manifest['management_appointments'], original_appointment)

    def test_unclassified_decision_goes_to_pm_for_classification_not_acceptance(self):
        self.project_appointment(['owner_reserved', 'decision_classification'])
        self.h.item['completion']['steps'][0].pop('decision_authority')
        self.assertEqual(self.h.failures(), [])
        for command in ['next', 'details']:
            v = json.loads(self.h.run_cli(command, '--json'))[command]
            self.assertFalse(v['owner_input_required'])
            self.assertTrue(v['decision_input_required'])
            self.assertTrue(v['classification_required'])
            self.assertEqual(v['decision_routing_status'], 'classification_required')
            self.assertEqual(v['decision_maker'], 'appointed-manager')
            self.assertIn('Do not claim, edit, choose dispositions or advance', v['work_start'])
            self.assertIn('Classification required:', self.h.run_cli(command))
        self.assertEqual(self.h.item['completion']['steps'][0]['status'], 'pending')

    def test_project_reserved_grant_does_not_imply_classification_capability(self):
        self.project_appointment(['owner_reserved'])
        self.h.item['completion']['steps'][0].pop('decision_authority')
        self.assertEqual(self.h.failures(), [])
        self.assertTrue(self.view()['owner_input_required'])
        self.assertFalse(self.view()['classification_required'])

    def check_unclassified_explicit_route(self, kinds):
        self.project_appointment(kinds)
        self.h.item['completion']['steps'][0].pop('decision_authority')
        self.h.item['decision_routing']['TEST-C01'] = {
            'appointment': 'sequence', 'decision_kind': 'owner_reserved'}
        self.assertEqual(self.h.failures(), [])
        for command in ['next', 'details']:
            v = json.loads(self.h.run_cli(command, '--json'))[command]
            classify = 'decision_classification' in kinds
            self.assertEqual(v['classification_required'], classify)
            self.assertEqual(v['owner_input_required'], not classify)
            self.assertNotEqual(v['decision_routing_status'], 'delegated')
            self.assertIn('Do not claim, edit, choose dispositions or advance', v['work_start'])

    def test_explicit_route_cannot_accept_an_unclassified_decision(self):
        self.check_unclassified_explicit_route(['owner_reserved'])

    def test_explicit_route_with_classification_capability_requires_classification(self):
        self.check_unclassified_explicit_route(['owner_reserved', 'decision_classification'])

    def test_literal_owner_only_reservation_survives_project_scope(self):
        self.project_appointment()
        step = self.h.item['completion']['steps'][0]
        step['decision_authority']['kind'] = 'owner_only'
        source = self.h.root / self.h.doc
        source.write_text(source.read_text().replace(
            '"decision_kind": "owner_reserved"', '"decision_kind": "owner_only"'))
        self.assertEqual(self.h.failures(), [])
        self.assertTrue(self.view()['owner_input_required'])
        self.assertEqual(self.view()['decision_maker'], 'project_owner')

    def test_revocation_restores_owner_without_deleting_historical_receipt(self):
        self.project_appointment()
        self.h.manifest['management_appointments'] = {}
        self.assertEqual(self.h.failures(), [])
        self.assertTrue(self.view()['owner_input_required'])

    def test_unverified_session_cannot_claim_project_scope(self):
        self.project_appointment()['session'] = 'unappointed-session'
        self.assertIn('exact owner appointment/scope receipt', '\n'.join(self.h.failures()))

    def test_ambiguous_project_managers_refused(self):
        self.project_appointment()
        self.h.manifest['management_appointments']['second'] = copy.deepcopy(
            self.h.manifest['management_appointments']['sequence'])
        self.assertIn('only one standing project-wide manager', '\n'.join(self.h.failures()))

    def test_mixed_wildcard_scope_and_legacy_reserved_upgrade_refused(self):
        self.project_appointment()['task_keys'] = ['*', 'alignment']
        self.assertTrue(self.h.failures())
        self.h.manifest['management_appointments']['sequence']['task_keys'] = ['alignment']
        self.assertIn('only delegated decision kinds', '\n'.join(self.h.failures()))

    def test_explicit_invalid_route_does_not_silently_fall_back(self):
        self.project_appointment()
        self.h.item['decision_routing']['TEST-C01'] = {
            'appointment': 'absent', 'decision_kind': 'owner_reserved'}
        self.assertEqual(self.h.failures(), [])
        self.assertTrue(self.view()['owner_input_required'])

    def test_unsupported_decisions_and_work_holds_are_not_waived(self):
        self.project_appointment(['objective_acceptance'])
        self.assertEqual(self.h.failures(), [])
        self.assertTrue(self.view()['owner_input_required'])
        self.h.hold()
        self.assertIn('STOP: held work', self.view()['work_start'])


if __name__ == '__main__':
    unittest.main()
