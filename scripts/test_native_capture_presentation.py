#!/usr/bin/env python3
"""Exercise the actual ordinary capture/focus/frame barrier with external I/O fakes."""
import ast
import json
import os
from pathlib import Path
import re
import tempfile
from types import MethodType, SimpleNamespace as NS
import unittest
from PIL import Image
from native_capture_pixels import require_native_capture_pixels
from native_capture_presentation import PresentationReadiness, matched_presentation
from test_native_capture_readiness import REQUIREMENT, content, present, record, ledger

SOURCE = Path(__file__).with_name('native_readiness_capture.py')

class CapturePresentationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.log = self.root / 'gui.log'
        self.focus_damage = True
        self.complete_focus_frame = True
        self.focus_requested = False
        self.calls = []
        self.frames = []
        self.shot_sequence = None
        self.during_shot = None
        self.append_frame(1, 1, False, initial=True)
        def command(*argv):
            self.calls.append(('focus', argv))
            self.focus_requested = True
            if self.focus_damage:
                self.append_frame(2, 1, True)
        def wait(label, predicate):
            for _ in range(8):
                value = predicate()
                if value:
                    return value
                # Simulate external GUI completion; no manufactured production frame.
                if self.focus_requested and self.focus_damage and self.complete_focus_frame:
                    self.append_frame(2, 2, False)
            raise TimeoutError(label)
        def run(argv, *, cwd, env, text, capture_output, timeout):
            index = sum(1 for name, _ in self.calls if name == 'screenshooter')
            self.calls.append(('screenshooter', dict(self.frames[-1])))
            colour = (1, 2, 3)
            if self.shot_sequence is not None:
                colour = self.shot_sequence[min(index, len(self.shot_sequence) - 1)]
            Image.new('RGB', (2, 2), colour).save(Path(cwd) / 'original.png')
            if self.during_shot:
                self.during_shot(index)
            return NS(returncode=0, stdout='', stderr='')
        nodes = next(n for n in ast.parse(SOURCE.read_text()).body
                     if isinstance(n, ast.ClassDef) and n.name == 'NativeReadinessCapture')
        methods = [n for n in nodes.body if isinstance(n, ast.FunctionDef)
                   and n.name in {'focus', 'wait_presented', 'capture', '_screenshoot_read', '_matched_snapshot'}]
        namespace = {'Path': Path, 'json': json, 're': re, 'os': os, 'time': NS(sleep=lambda *_: None),
                     'PresentationReadiness': PresentationReadiness,
                     'matched_presentation': matched_presentation,
                     'subprocess': NS(run=run),
                     'focus_arguments': lambda window, env: ('focus', window),
                     'require_native_capture_pixels': require_native_capture_pixels}
        exec(compile(ast.Module(body=methods, type_ignores=[]), str(SOURCE), 'exec'), namespace)
        self.qa = NS(root=self.root, env={'DATUM_GUI_LOG': str(self.log), 'WAYLAND_DISPLAY': 'owned'},
                     compositor=NS(poll=lambda: None), host_window='owned-host', command=command,
                     wait=wait, nodes=lambda: [], actions=[], capture_counts={}, observations={})
        for name in ['focus', 'wait_presented', 'capture', '_screenshoot_read', '_matched_snapshot']:
            setattr(self.qa, name, MethodType(namespace[name], self.qa))
    def append_frame(self, damage, presented, pending, initial=False):
        frame = {'window': 'WindowId(1)', 'damage': damage, 'presented_damage': presented,
                 'pending': pending, 'rendering': None}
        self.frames.append(frame)
        prefix = ('surface identity window=WindowId(1)\n'
                  'window event WindowId(1) focused true\n') if initial else ''
        with self.log.open('a') as out:
            out.write(prefix + 'native redraw ledger ' + json.dumps(frame) + '\n')
    def test_focus_generated_damage_finishes_before_screenshot_and_receipt(self):
        receipt = self.qa.capture('focus-frame', 'owned-host')
        self.assertEqual((receipt['damage'], receipt['presented_damage']), (2, 2))
        shots = [value for name, value in self.calls if name == 'screenshooter']
        # The de-flake takes two byte-identical reads of the settled frame before admitting.
        self.assertEqual(len(shots), 2)
        self.assertEqual((shots[0]['damage'], shots[0]['presented_damage'], shots[0]['pending']),
                         (2, 2, False))
        self.assertEqual(self.qa.actions[0]['capture_presentation_barrier']['damage'], 2)
        self.assertTrue((self.root / 'focus-frame.png').exists())
        self.assertEqual(next(a['capture_stable_after_reads'] for a in self.qa.actions
                              if 'capture_stable_after_reads' in a), 2)
    def test_unfinished_focus_damage_refuses_before_any_capture(self):
        self.complete_focus_frame = False
        with self.assertRaises(TimeoutError):
            self.qa.capture('still-pending', 'owned-host')
        self.assertFalse(any(name == 'screenshooter' for name, _ in self.calls))
        self.assertFalse((self.root / 'still-pending.png').exists())
    def test_already_focused_unchanged_frame_needs_no_extra_damage_or_retry(self):
        self.focus_damage = False
        receipt = self.qa.capture('unchanged-frame', 'owned-host')
        self.assertEqual((receipt['damage'], receipt['presented_damage']), (1, 1))
        self.assertEqual([name for name, _ in self.calls], ['focus', 'screenshooter', 'screenshooter'])
        self.assertEqual(len(self.frames), 1)

    def test_stable_frame_admits_on_two_byte_identical_reads(self):
        # The common case: a settled frame reads byte-identically twice -> admitted after exactly two
        # reads, no retry.
        receipt = self.qa.capture('stable', 'owned-host')
        self.assertEqual((receipt['damage'], receipt['presented_damage']), (2, 2))
        self.assertEqual(sum(1 for name, _ in self.calls if name == 'screenshooter'), 2)
        self.assertTrue((self.root / 'stable.png').exists())

    def test_mismatched_read_retries_until_two_consecutive_match(self):
        # A one-off differing read (mid-recomposite) is not accepted; capture retries until two
        # consecutive reads match, then admits that stable frame.
        self.shot_sequence = [(1, 2, 3), (9, 9, 9), (9, 9, 9)]
        receipt = self.qa.capture('retry', 'owned-host')
        self.assertEqual((receipt['damage'], receipt['presented_damage']), (2, 2))
        self.assertEqual(sum(1 for name, _ in self.calls if name == 'screenshooter'), 3)
        self.assertTrue((self.root / 'retry.png').exists())
        self.assertEqual(next(a['capture_stable_after_reads'] for a in self.qa.actions
                              if 'capture_stable_after_reads' in a), 3)

    def test_never_stable_refuses_loudly_and_writes_no_frame(self):
        # Every read differs -> no two consecutive match within the bound -> LOUD AssertionError, and NO
        # frame is written (never ship unstable evidence).
        self.shot_sequence = [(1, 1, 1), (2, 2, 2), (3, 3, 3), (4, 4, 4)]
        with self.assertRaises(AssertionError) as caught:
            self.qa.capture('unstable', 'owned-host')
        self.assertIn('two consecutive byte-identical', str(caught.exception))
        self.assertEqual(sum(1 for name, _ in self.calls if name == 'screenshooter'), 4)
        self.assertFalse((self.root / 'unstable.png').exists())

    def test_selected_capture_waits_for_same_frame_present_without_extra_wait(self):
        self.focus_damage = False
        self.log.write_text('surface identity window=WindowId(1)\n'
                            'window event WindowId(1) focused true\n'
                            + record('native redraw ledger ', ledger(18))
                            + 'window event WindowId(1) mouse input Left Released\n'
                            + record('native submitted content ', content()))
        wait_calls = []
        def wait(label, predicate):
            wait_calls.append(label)
            self.assertFalse(predicate())  # Older settled damage18 is insufficient.
            self.assertFalse(any(name == 'screenshooter' for name, _ in self.calls))
            with self.log.open('a') as out:
                out.write(record('native_surface_lifecycle ', present()))
                out.write(record('native redraw ledger ', ledger(20)))
            self.assertFalse(predicate())  # Existing two-read stability remains.
            return predicate()
        self.qa.wait = wait
        receipt = self.qa.capture('selected', 'owned-host', frame_requirement=REQUIREMENT)
        self.assertEqual(receipt['content_presentation']['native_identity']['submission_receipt'], 18)
        self.assertEqual(len(wait_calls), 1)
        self.assertEqual([name for name, _ in self.calls],
                         ['focus', 'screenshooter', 'screenshooter'])

    def matched_case(self):
        self.focus_damage = False
        self.qa.process = NS(pid=42, poll=lambda: None)
        self.value = 'Metric'
        self.qa.attributes = lambda node: node.attrs
        def node(name, role, attrs):
            children = []
            return NS(get_name=lambda: name, get_role_name=lambda: role,
                      get_process_id=lambda: 42, attrs=attrs, children=children,
                      get_child_count=lambda: len(children),
                      get_child_at_index=lambda index: children[index])
        def nodes():
            dialog = node('Global Preferences', 'dialog', {})
            dialog.children.extend([node('Search Units settings', 'text', {}),
                                    node('Measurement system', 'combo box', {'value': self.value})])
            return [dialog, *dialog.children]
        self.qa.nodes = nodes
        self.lifecycle = dict(present(), configured_extent=[720, 720])
        self.log.write_text('surface identity window=WindowId(1)\n'
                            'window event WindowId(1) focused true\n')
        self.expectation = {'pid': 42, 'window': 'WindowId(1)', 'dialog': 'Global Preferences',
                            'opened_after_byte': 0, 'after_byte': self.log.stat().st_size,
                            'extent': [720, 720], 'nodes': [
                                {'name': 'Global Preferences', 'role': 'dialog'},
                                {'name': 'Search Units settings'},
                                {'name': 'Measurement system', 'role': 'combo box',
                                 'attributes': {'value': 'Metric'}}]}
        self.publish_matched_frame()

    def publish_matched_frame(self):
        with self.log.open('a') as out:
            out.write(record('native_surface_lifecycle ', self.lifecycle))
            out.write(record('native redraw ledger ', ledger(20)))

    def test_matched_capture_records_real_target_state_and_review_scope(self):
        self.matched_case()
        receipt = self.qa.capture('matched', 'owned-host', expectation=self.expectation)
        self.assertEqual(receipt['matched_state']['pid'], 42)
        self.assertEqual(receipt['matched_state']['presentation']['native_identity']['configured_extent'], [720, 720])
        self.assertEqual(receipt['capture_path'], str(self.root / 'matched.png'))
        self.assertTrue(receipt['review_pending'])
        measurement = next(node for node in receipt['matched_state']['nodes']
                           if node['name'] == 'Measurement system')
        self.assertEqual(measurement['attributes']['value'], 'Metric')

    def test_matched_capture_rejects_changes_during_pixels_and_retains_raw(self):
        for mutation in ['extent', 'generation', 'target', 'value', 'pid']:
            with self.subTest(mutation=mutation):
                self.matched_case()
                def mutate(index):
                    if index != 0:
                        return
                    if mutation == 'value':
                        self.value = 'Imperial'
                    elif mutation == 'pid':
                        self.qa.process.pid = 43
                    elif mutation == 'target':
                        with self.log.open('a') as out:
                            out.write('window event WindowId(2) focused true\n')
                    else:
                        self.lifecycle['configured_extent' if mutation == 'extent'
                                       else 'configuration_generation'] = [1160, 720] if mutation == 'extent' else 2
                        self.publish_matched_frame()
                self.calls.clear()
                self.during_shot = mutate
                with self.assertRaises(RuntimeError):
                    self.qa.capture(mutation, 'owned-host', expectation=self.expectation)
                self.assertFalse((self.root / f'{mutation}.png').exists())
                self.assertTrue(list((self.root / f'{mutation}-native-capture-0').glob('*.png')))

    def test_matched_capture_refuses_stale_action_and_wrong_state_before_pixels(self):
        for wrong in ['fence', 'value', 'ambiguous', 'foreign_pid']:
            with self.subTest(wrong=wrong):
                self.matched_case()
                self.calls.clear()
                if wrong == 'fence':
                    self.expectation['after_byte'] = self.log.stat().st_size
                elif wrong == 'value':
                    self.value = 'Imperial'
                else:
                    nodes = self.qa.nodes()
                    if wrong == 'ambiguous':
                        nodes[0].children.append(nodes[-1])
                    else:
                        nodes[-1].get_process_id = lambda: 99
                    self.qa.nodes = lambda: nodes
                with self.assertRaises((RuntimeError, TimeoutError)):
                    self.qa.capture(wrong, 'owned-host', expectation=self.expectation)
                self.assertFalse(any(name == 'screenshooter' for name, _ in self.calls))

    def test_matched_dialog_refuses_main_inactive_subtree_and_additional_windows(self):
        for wrong in ['main', 'wrong_subtree', 'second_dialog', 'second_surface']:
            with self.subTest(wrong=wrong):
                self.matched_case()
                self.calls.clear()
                nodes = self.qa.nodes()
                if wrong in ['main', 'second_surface']:
                    # The requested Main is settled/focused and the inactive
                    # Preferences subtree exists, but the new surface is different.
                    with self.log.open('a') as out:
                        out.write('surface identity window=WindowId(2)\n')
                    if wrong == 'main':
                        self.expectation['opened_after_byte'] = self.expectation['after_byte']
                elif wrong == 'wrong_subtree':
                    # Measurement remains discoverable in the PID, outside dialog.
                    nodes[0].children.remove(nodes[-1])
                else:
                    nodes.append(self.qa.nodes()[0])
                self.qa.nodes = lambda: nodes
                with self.assertRaises((RuntimeError, TimeoutError)):
                    self.qa.capture(wrong, 'owned-host', expectation=self.expectation)
                self.assertFalse(any(name == 'screenshooter' for name, _ in self.calls))

    def test_diagnostic_capture_does_not_claim_matched_state(self):
        receipt = self.qa.capture('diagnostic', 'owned-host')
        self.assertTrue(receipt['review_pending'])
        self.assertNotIn('matched_state', receipt)
        self.assertIn('diagnostic', receipt['capture_scope'])

if __name__ == '__main__':
    unittest.main(verbosity=2)
