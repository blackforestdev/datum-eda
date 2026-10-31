#!/usr/bin/env python3
"""Exercise the real StartupQA.wait extend-while-progress launch deadline (dat-xjjn #1) with a mocked
clock: a slow-but-progressing launch is admitted, a genuine hang fails fast-ish, a runaway is capped by
the hard ceiling, and the no-progress path is byte-for-byte the old behaviour."""
import ast
from pathlib import Path
from types import MethodType, SimpleNamespace as NS
import unittest

SOURCE = Path(__file__).with_name('native_readiness_capture.py')


def load_wait():
    """Extract the real wait() method over a mocked monotonic clock (sleep advances it deterministically)."""
    tree = ast.parse(SOURCE.read_text())
    cls = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == 'NativeReadinessCapture')
    fn = next(n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name == 'wait')
    clock = {'t': 0.0}
    time_mock = NS(monotonic=lambda: clock['t'],
                   sleep=lambda d: clock.__setitem__('t', round(clock['t'] + d, 6)))

    class GErr(Exception):
        pass
    namespace = {'time': time_mock, 'GLib': NS(Error=GErr)}
    exec(compile(ast.Module(body=[fn], type_ignores=[]), str(SOURCE), 'exec'), namespace)
    return namespace['wait'], clock


class LaunchProgressWaitTests(unittest.TestCase):
    def make(self, *, process=None):
        wait, clock = load_wait()
        qa = NS(compositor=None, process=process)
        qa.wait = MethodType(wait, qa)
        return qa, clock

    def test_slow_but_progressing_launch_is_admitted_past_the_stall_budget(self):
        # Predicate only true at t>=3s -- well past the 1s stall budget -- but progress advances every
        # poll, so the deadline keeps renewing and the live-but-slow launch is admitted (not starved).
        qa, clock = self.make()
        progress = lambda: int(round(clock['t'] * 10))  # advances each 0.1s sleep
        value = qa.wait('native Main visible', lambda: clock['t'] >= 3.0 or None,
                        timeout=1.0, progress=progress, ceiling=10.0)
        self.assertTrue(value)
        self.assertGreaterEqual(clock['t'], 3.0)
        self.assertLess(clock['t'], 10.0)

    def test_genuine_hang_fails_fast_ish_at_the_stall_budget_not_the_ceiling(self):
        # No progress signal ever advances -> the stall budget trips FAST (near 1s), long before the 10s
        # ceiling: a hang is refused loudly, not waited out to the ceiling.
        qa, clock = self.make()
        with self.assertRaises(RuntimeError) as caught:
            qa.wait('native Main visible', lambda: None, timeout=1.0, progress=lambda: 0, ceiling=10.0)
        self.assertIn('genuine hang', str(caught.exception))
        self.assertLess(clock['t'], 2.0)  # fast-ish: nowhere near the 10s ceiling

    def test_runaway_progress_is_capped_by_the_hard_ceiling(self):
        # Progress advances forever (never paints) -> stall keeps renewing, but the hard ceiling still
        # ends the wait loudly: never an unbounded wait.
        qa, clock = self.make()
        progress = lambda: int(round(clock['t'] * 10))
        with self.assertRaises(RuntimeError) as caught:
            qa.wait('native Main visible', lambda: None, timeout=1.0, progress=progress, ceiling=5.0)
        self.assertIn('hard ceiling', str(caught.exception))
        self.assertGreaterEqual(clock['t'], 5.0)
        self.assertLess(clock['t'], 5.5)

    def test_no_progress_path_is_the_unchanged_legacy_timeout(self):
        # Without progress/ceiling the message and timing are byte-for-byte the old behaviour.
        qa, clock = self.make()
        with self.assertRaises(RuntimeError) as caught:
            qa.wait('some label', lambda: None, timeout=2.0)
        self.assertEqual(str(caught.exception), 'timed out: some label; last=None')
        self.assertGreaterEqual(clock['t'], 2.0)

    def test_app_exit_still_fails_loudly_before_any_deadline(self):
        qa, clock = self.make(process=NS(poll=lambda: 1, returncode=1))
        with self.assertRaises(RuntimeError) as caught:
            qa.wait('native Main visible', lambda: None, timeout=1.0, progress=lambda: 0, ceiling=10.0)
        self.assertIn('App exited 1', str(caught.exception))


if __name__ == '__main__':
    unittest.main(verbosity=2)
