"""Finite record-order controls for the shared native capture admission owner.

Compact records preserve the distinct case05 counter values: old damage18,
held damage17, selected acquisition17/receipt18, then settled damage20.
Case04 instead already reached selected acquisition19/receipt20/damage21.
"""
import copy
import json
import unittest

from native_capture_presentation import PresentationReadiness


HOST = 'WindowId(1)'
SELECTION = {'kind': 'compound', 'scope': 'via', 'focus': None,
             'members': [{'class': 'via', 'id': 'fixture-via'}]}
REQUIREMENT = {'after_byte': 0, 'event': 'mouse input Left Released',
               'selection_fields': SELECTION,
               'text_by_region': {'inspector': ['Via: object 1', 'SELECTED', 'All 1', 'Vias 1'],
                                  'footer': ['compound 1']}}


def record(marker, value):
    return marker + json.dumps(value) + '\n'


def ledger(damage):
    return {'window': HOST, 'damage': damage, 'presented_damage': damage,
            'pending': False, 'rendering': None}


def content(acquisition=17, receipt=18):
    return {'native_identity': {'window': HOST, 'host': 1, 'queue_epoch': 1,
                               'configuration': 1, 'acquisition': acquisition,
                               'submission_receipt': receipt},
            'content_revision': 55, 'current_selection': copy.deepcopy(SELECTION),
            'cpu_host_text': [{'region': region, 'text': text}
                              for region, texts in REQUIREMENT['text_by_region'].items()
                              for text in texts]}


def present(acquisition=17, receipt=18):
    return {'window': HOST, 'host': 1, 'queue_epoch': 1, 'configuration_generation': 1,
            'reason': 'present', 'acquired': acquisition, 'active': 0,
            'last_frame_submission_receipt': receipt, 'last_present_receipt': receipt}


def prefix(frame=None):
    return ('surface identity window=' + HOST + '\n'
            'window event ' + HOST + ' focused true\n'
            + record('native redraw ledger ', ledger(18))
            + 'window event ' + HOST + ' mouse input Left Released\n'
            + record('native submitted content ', frame or content()))


def completed(frame=None, receipt=None, damage=20):
    return (prefix(frame) + record('native_surface_lifecycle ', receipt or present())
            + record('native redraw ledger ', ledger(damage)))


def admit(text, requirement=None, **kwargs):
    readiness = PresentationReadiness(frame_requirement=requirement or REQUIREMENT, **kwargs)
    readiness(text)
    return readiness(text)


class NativeCaptureReadinessTests(unittest.TestCase):
    def test_old_settled_frame_cannot_satisfy_selected_frame(self):
        self.assertFalse(admit(prefix(), after_damage=17))
        # The default historical barrier does accept that same old damage18.
        old = PresentationReadiness(after_damage=17)
        self.assertFalse(old(prefix()))
        self.assertEqual(old(prefix())['damage'], 18)

    def test_matching_present_then_ledger_admits_without_another_frame(self):
        result = admit(completed(), after_damage=17)
        self.assertEqual(result['damage'], 20)
        binding = result['content_presentation']
        self.assertEqual(binding['native_identity']['submission_receipt'], 18)
        positions = [binding[k]['line'] for k in
                     ['event', 'submitted_content', 'native_present', 'settled_ledger']]
        self.assertEqual(positions, sorted(set(positions)))

    def test_case04_already_qualified_native_receipt_remains_admitted(self):
        result = admit(completed(content(19, 20), present(19, 20), damage=21), after_damage=19)
        self.assertEqual(result['content_presentation']['native_identity']['acquisition'], 19)
        self.assertEqual(result['presented_damage'], 21)

    def test_selected_model_with_immutable_no_selection_text_refuses(self):
        frame = content()
        frame['cpu_host_text'] = [{'region': 'inspector', 'text': 'No selection'},
                                 {'region': 'footer', 'text': 'none'}]
        self.assertFalse(admit(completed(frame)))

    def test_wrong_native_identity_never_borrows_another_receipt(self):
        for key, value in [('window', 'WindowId(2)'), ('host', 2), ('queue_epoch', 2),
                           ('configuration_generation', 2), ('acquired', 18),
                           ('last_frame_submission_receipt', 20),
                           ('last_present_receipt', 20), ('reason', 'submit_upload')]:
            with self.subTest(key=key):
                receipt = present()
                receipt[key] = value
                self.assertFalse(admit(completed(receipt=receipt)))

    def test_current_selection_identity_is_separate_and_required(self):
        for key, value in [('kind', 'none'), ('scope', 'pad'), ('focus', 'other'), ('members', [])]:
            frame = content()
            frame['current_selection'][key] = value
            self.assertFalse(admit(completed(frame)))

    def test_receipt_before_content_or_ledger_before_receipt_refuses(self):
        self.assertFalse(admit(record('native_surface_lifecycle ', present()) + prefix()
                               + record('native redraw ledger ', ledger(20))))
        self.assertFalse(admit(prefix() + record('native redraw ledger ', ledger(20))
                               + record('native_surface_lifecycle ', present())))

    def test_frame_before_release_or_before_requested_offset_refuses(self):
        text = completed().replace('window event ' + HOST + ' mouse input Left Released\n', '')
        self.assertFalse(admit(text))
        requirement = dict(REQUIREMENT, after_byte=len(completed().encode('utf-8')))
        self.assertFalse(admit(completed(), requirement))

    def test_partial_or_malformed_record_refuses(self):
        self.assertFalse(admit(completed().rstrip('\n')))
        self.assertFalse(admit(completed() + 'native submitted content {'))
        self.assertFalse(admit(completed() + 'native submitted content {broken}\n'))

    def test_latest_conflicting_frame_invalidates_historical_selected_frame(self):
        frame = content(18, 20)
        frame['cpu_host_text'] = []
        self.assertFalse(admit(completed() + record('native submitted content ', frame)))

    def test_new_inflight_acquisition_blocks_old_ready_ledger(self):
        active = dict(present(18, 20), active=1, reason='submit')
        self.assertFalse(admit(completed() + record('native_surface_lifecycle ', active)))

    def test_upload_only_receipt_does_not_equal_damage_or_replace_present(self):
        upload = dict(present(), reason='submit_upload', last_frame_submission_receipt=19)
        self.assertTrue(admit(completed() + record('native_surface_lifecycle ', upload)))
        self.assertFalse(admit(prefix() + record('native_surface_lifecycle ', upload)
                               + record('native redraw ledger ', ledger(18))))

    def test_duplicate_release_or_receipt_refuses_ambiguous_binding(self):
        self.assertFalse(admit(completed() + 'window event ' + HOST
                               + ' mouse input Left Released\n'))
        self.assertFalse(admit(prefix() + record('native_surface_lifecycle ', present()) * 2
                               + record('native redraw ledger ', ledger(20))))

    def test_default_owned_dialog_and_focus_behavior(self):
        text = ('surface identity window=' + HOST + '\n'
                'surface identity window=WindowId(2)\n'
                'window event WindowId(2) focused true\n'
                + record('native redraw ledger ', dict(ledger(3), window='WindowId(2)')))
        ready = PresentationReadiness()
        self.assertFalse(ready(text))
        self.assertFalse(ready(text, owned_dialog=True))
        self.assertEqual(ready(text, owned_dialog=True)['window'], 'WindowId(2)')
        self.assertFalse(ready(text + 'window event WindowId(2) focused false\n', owned_dialog=True))


if __name__ == '__main__':
    unittest.main(verbosity=2)
