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

SOURCE = Path(__file__).with_name('prove_native_project_startup_gui.py')

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
            self.calls.append(('screenshooter', dict(self.frames[-1])))
            Image.new('RGB', (2, 2), (1, 2, 3)).save(Path(cwd) / 'original.png')
            return NS(returncode=0, stdout='', stderr='')
        nodes = next(n for n in ast.parse(SOURCE.read_text()).body
                     if isinstance(n, ast.ClassDef) and n.name == 'StartupQA')
        methods = [n for n in nodes.body if isinstance(n, ast.FunctionDef)
                   and n.name in {'focus', 'wait_presented', 'capture'}]
        namespace = {'Path': Path, 'json': json, 're': re, 'os': os,
                     'subprocess': NS(run=run),
                     'focus_arguments': lambda window, env: ('focus', window),
                     'require_native_capture_pixels': require_native_capture_pixels}
        exec(compile(ast.Module(body=methods, type_ignores=[]), str(SOURCE), 'exec'), namespace)
        self.qa = NS(root=self.root, env={'DATUM_GUI_LOG': str(self.log), 'WAYLAND_DISPLAY': 'owned'},
                     compositor=NS(poll=lambda: None), host_window='owned-host', command=command,
                     wait=wait, nodes=lambda: [], actions=[], capture_counts={}, observations={})
        for name in ['focus', 'wait_presented', 'capture']:
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
        self.assertEqual(len(shots), 1)
        self.assertEqual((shots[0]['damage'], shots[0]['presented_damage'], shots[0]['pending']),
                         (2, 2, False))
        self.assertEqual(self.qa.actions[0]['capture_presentation_barrier']['damage'], 2)
        self.assertTrue((self.root / 'focus-frame.png').exists())
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
        self.assertEqual([name for name, _ in self.calls], ['focus', 'screenshooter'])
        self.assertEqual(len(self.frames), 1)

if __name__ == '__main__':
    unittest.main(verbosity=2)
