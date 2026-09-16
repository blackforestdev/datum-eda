"""Raw trials, per-trial limits, storage accounting and resource ownership."""
from copy import deepcopy
import tempfile
import unittest
import os
from pathlib import Path
import sys

from global_preferences_acceptance.inputs import EvidenceError
from global_preferences_acceptance.inventory import BUDGETS
from global_preferences_acceptance.measurements import percentile, validate_trials
from global_preferences_acceptance.test_support import SyntheticBundle
from test_global_preferences_production_matrix import MATRIX


class MeasurementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.fixture = SyntheticBundle(cls.temporary.name, MATRIX)

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def row(self, budget, variant=None):
        return deepcopy(next(r for r in self.fixture.report['measurements'] if r['budget_id'] == budget
                             and (variant is None or r['variant_id'] == variant)))

    def check(self, row):
        self.fixture.rebind_trials(row, capture_samples=True)
        validate_trials(row, self.fixture.refresh())

    def test_nearest_rank_and_all_complete_variants(self):
        self.assertEqual(percentile(list(range(1, 22))), 20)
        self.assertEqual(percentile(list(range(1, 101))), 95)
        bundle = self.fixture.refresh()
        for row in self.fixture.report['measurements']:
            validate_trials(row, bundle)

    def test_sample_cannot_contradict_its_retained_capture(self):
        for field, value in [('exit_code', 7), ('timed_out', True),
                             ('outcome', 'failure'), ('elapsed_ns', 10**12), ('rss_kib', 10**9)]:
            row = self.row('release-query')
            sample = row['trials'][0]['samples'][0]
            raw = {'schema': 'datum.preferences.measurement-capture.v2',
                   'measurement': {k: v for k, v in row.items() if k != 'trials'},
                   'trial_index': 0, 'phase': 'samples', 'exit_code': 0, 'timed_out': False,
                   'observation': {k: v for k, v in sample.items() if k != 'evidence'}}
            (raw if field in ('exit_code', 'timed_out') else raw['observation'])[field] = value
            sample['evidence'] = self.fixture.put(raw)
            self.fixture.rebind_trials(row)
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                validate_trials(row, self.fixture.refresh())

    def test_all_sample_kinds_and_phases_bind_capture_values_and_identity(self):
        bundle = self.fixture.refresh()
        for budget, phase, field, value in [
            ('release-query', 'warmups', 'rss_kib', 10**9),
            ('native-window-open', 'cold_open', 'elapsed_ns', 10**12),
            ('storage-growth', 'samples', 'project_bytes', 10**9),
            ('window-lifecycle', 'samples', 'post_close_rss_kib', 10**9),
            ('durable-mutation', 'samples', 'launch_to_response_ns', 10**12),
            ('gui-feedback', 'samples', 'presented_elapsed_ns', 10**12),
        ]:
            row = self.row(budget)
            sample = row['trials'][0][phase] if phase == 'cold_open' else row['trials'][0][phase][0]
            capture = bundle.json(sample['evidence'])
            capture['observation'][field] = value
            sample['evidence'] = self.fixture.put(capture)
            self.fixture.rebind_trials(row)
            with self.subTest(budget=budget, phase=phase), self.assertRaisesRegex(EvidenceError, 'underlying measurement capture'):
                validate_trials(row, self.fixture.refresh())
        for field, value in [('trial_index', 2), ('phase', 'warmups'), ('measurement', {})]:
            row = self.row('release-query')
            sample = row['trials'][0]['samples'][0]
            capture = bundle.json(sample['evidence'])
            capture[field] = value
            sample['evidence'] = self.fixture.put(capture)
            self.fixture.rebind_trials(row)
            with self.subTest(field=field), self.assertRaisesRegex(EvidenceError, 'execution identity'):
                validate_trials(row, self.fixture.refresh())

    def test_consistently_reported_failed_capture_cannot_pass(self):
        row = self.row('release-query')
        sample = row['trials'][0]['samples'][0]
        sample['outcome'] = 'failure'
        self.fixture.rebind_trials(row, capture_samples=True)
        with self.assertRaises(EvidenceError):
            validate_trials(row, self.fixture.refresh())

    def test_each_timed_max_p95_and_rss(self):
        for budget, limits in BUDGETS.items():
            if 'p95_ms' not in limits:
                continue
            for failure in ('max', 'p95', 'rss'):
                if failure == 'rss' and 'max_rss_mib' not in limits:
                    continue
                row = self.row(budget)
                samples = row['trials'][1]['samples']
                for sample in samples[:6 if failure == 'p95' else 1]:
                    if failure == 'rss':
                        sample['rss_kib'] = limits['max_rss_mib'] * 1024 + 1
                    else:
                        sample['elapsed_ns'] = limits[failure + '_ms'] * 1_000_000 + 1
                        if budget == 'durable-mutation':
                            sample['launch_to_response_ns'] = sample['elapsed_ns']
                        if sample['presented_elapsed_ns'] is not None:
                            sample['presented_elapsed_ns'] = sample['elapsed_ns']
                with self.subTest(budget=budget, failure=failure), self.assertRaises(EvidenceError):
                    self.check(row)

    def test_cold_open_daemon_rss_sample_count_failure_and_raw_relabel(self):
        for budget, field, value in [('native-window-open', 'elapsed_ns', 1_000_000_001),
                                    ('durable-mutation', 'daemon_rss_kib', 32769),
                                    ('release-query', 'outcome', 'skipped'),
                                    ('release-query', 'rss_kib', True)]:
            row = self.row(budget, 'set/daemon' if budget == 'durable-mutation' else None)
            sample = row['trials'][0]['cold_open'] if budget == 'native-window-open' else row['trials'][0]['samples'][0]
            sample[field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(row)
        row = self.row('release-query')
        row['trials'][0]['samples'].pop()
        with self.assertRaises(EvidenceError):
            self.check(row)
        row = self.row('gui-feedback')
        row['environment_id'] = 'different-display'
        with self.assertRaisesRegex(EvidenceError, 'raw observations'):
            validate_trials(row, self.fixture.refresh())

    def test_storage_must_be_derived_and_nonempty(self):
        for variant in ('factory', 'generation'):
            row = self.row('storage-growth', variant)
            sample = row['trials'][0]['samples'][0]
            sample['project_bytes' if variant == 'factory' else 'generation_bytes'] = 0
            with self.assertRaisesRegex(EvidenceError, 'per-file manifests'):
                self.check(row)
        row = self.row('storage-growth', 'generation')
        row['trials'][0]['samples'][0]['before'] = self.fixture.put(b'not a manifest')
        with self.assertRaises(EvidenceError):
            self.check(row)

    def test_lifecycle_growth_orphans_and_invented_baseline(self):
        for field, value in [('post_close_rss_kib', 1024 + 4097), ('orphan_windows', 1),
                             ('orphan_writer_leases', 1), ('owned_windows', [{'identity': 'leak', 'owner': 'gui'}]),
                             ('writer_leases', [{'identity': 'leak', 'owner': 'gui', 'baseline': True}])]:
            row = self.row('window-lifecycle')
            row['trials'][0]['samples'][0][field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(row)

    def test_warmup_types_cross_trial_order_and_observed_baseline(self):
        for field, value in [('kind', 'wrong'), ('elapsed_ns', 'not-a-number'), ('rss_kib', -99)]:
            row = self.row('release-query')
            row['trials'][0]['warmups'][0][field] = value
            with self.subTest(field=field), self.assertRaises(EvidenceError):
                self.check(row)
        row = self.row('release-query')
        row['trials'][1] = deepcopy(row['trials'][0])
        row['trials'][1]['index'] = 1
        with self.assertRaisesRegex(EvidenceError, 'repeat or overlap'):
            self.check(row)
        row = self.row('window-lifecycle')
        row['trials'][0]['baseline_rss_kib'] += 1
        with self.assertRaisesRegex(EvidenceError, 'final warmed observation'):
            self.check(row)

    def test_storage_cannot_repeat_one_mutation_as_a_hundred(self):
        row = self.row('storage-growth', 'generation')
        sample = row['trials'][0]['samples'][0]
        row['trials'][0]['samples'][1]['before'] = sample['before']
        row['trials'][0]['samples'][1]['after'] = sample['after']
        with self.assertRaisesRegex(EvidenceError, 'continuous retained history'):
            self.check(row)


class StorageCaptureTests(unittest.TestCase):
    def test_actual_retained_bytes_and_continuous_growth(self):
        from global_preferences_acceptance.capture import Capture
        from global_preferences_acceptance.inputs import Bundle
        from global_preferences_acceptance.observations import state_manifest
        from global_preferences_acceptance.storage_capture import growth, snapshot
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            evidence = base / 'evidence'
            evidence.mkdir()
            capture = Capture(evidence)
            repository = base / 'repository'
            before, before_ref = snapshot(repository, capture, 'trial-0')
            generation = repository / 'generations/g000/manifest.json'
            generation.parent.mkdir(parents=True)
            generation.write_bytes(b'{"receipt":"retained inside generation"}')
            (repository / 'head.json').write_bytes(b'head')
            (repository / 'writer.lock').touch()
            after, after_ref = snapshot(repository, capture, 'trial-0')
            self.assertEqual(growth(before, after), {
                'generation_bytes': generation.stat().st_size, 'receipt_bytes': 0,
                'request_index_bytes': 0, 'fixed_overhead_bytes': 4})
            bundle = Bundle(evidence, list(capture.inventory.values()))
            self.assertEqual(state_manifest(before_ref, bundle), before)
            self.assertEqual(state_manifest(after_ref, bundle), after)
            generation.write_bytes(b'changed immutable history')
            changed, _ = snapshot(repository, capture, 'trial-0')
            with self.assertRaisesRegex(EvidenceError, 'immutable storage'):
                growth(after, changed)
            generation.unlink()
            removed, _ = snapshot(repository, capture, 'trial-0')
            with self.assertRaisesRegex(EvidenceError, 'immutable storage'):
                growth(after, removed)

    def test_symlink_and_special_file_cannot_be_measured_as_storage(self):
        from global_preferences_acceptance.capture import Capture
        from global_preferences_acceptance.storage_capture import snapshot
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            evidence, repository = base / 'evidence', base / 'repository'
            evidence.mkdir()
            repository.mkdir()
            capture = Capture(evidence)
            redirected = repository / 'linked'
            redirected.symlink_to(evidence, target_is_directory=True)
            with self.assertRaisesRegex(EvidenceError, 'symlink'):
                snapshot(repository, capture, 'trial-0')
            redirected.unlink()
            os.mkfifo(repository / 'pipe')
            with self.assertRaisesRegex(EvidenceError, 'regular file'):
                snapshot(repository, capture, 'trial-0')


class ProcessMeasurementTests(unittest.TestCase):
    def test_collector_cannot_inherit_owner_store_or_daemon_override(self):
        from unittest.mock import patch
        from measure_global_preferences_release import isolated_environment
        inherited = {'XDG_CONFIG_HOME': '/owner/config', 'DATUM_ENGINE_SOCKET': '/owner/daemon',
                     'EDA_ENGINE_SOCKET': '/owner/legacy-daemon', 'DATUM_GUI_PREFERENCES_PATH': '/owner/legacy.json'}
        with patch.dict(os.environ, inherited):
            environment = isolated_environment(Path('/private/config'))
        self.assertEqual(environment['XDG_CONFIG_HOME'], '/private/config')
        for key in inherited.keys() - {'XDG_CONFIG_HOME'}:
            self.assertNotIn(key, environment)

    def test_actual_output_exit_and_wait4_rss_are_retained(self):
        from global_preferences_acceptance.process import execute
        result = execute(Path(sys.executable), ['-c', 'import sys; print("observed"); sys.exit(3)'], os.environ.copy())
        self.assertEqual(result['stdout'], b'observed\n')
        self.assertEqual(result['exit_code'], 3)
        self.assertGreater(result['rss_kib'], 0)
        self.assertGreater(result['elapsed_ns'], 0)

    def test_timeout_has_bounded_failure_and_no_success_substitution(self):
        from global_preferences_acceptance.process import execute
        result = execute(Path(sys.executable), ['-c', 'import time; time.sleep(10)'], os.environ.copy(), timeout=0.02)
        self.assertEqual(result['exit_code'], 124)
        self.assertTrue(result['timed_out'])
        self.assertLess(result['elapsed_ns'], 2_000_000_000)

    def test_mutation_clock_starts_at_actual_tty_confirmation(self):
        from global_preferences_acceptance.process import execute
        code = 'import time; time.sleep(.03); print("Type APPLY to confirm:", flush=True); print(input(), flush=True)'
        result = execute(Path(sys.executable), ['-c', code], os.environ.copy(), confirmation=True)
        self.assertEqual(result['exit_code'], 0, result['stdout'])
        self.assertTrue(result['confirmed'])
        self.assertIn(b'APPLY', result['stdout'])
        self.assertGreater(result['launch_to_response_ns'] - result['elapsed_ns'], 20_000_000)


if __name__ == '__main__':
    unittest.main()
