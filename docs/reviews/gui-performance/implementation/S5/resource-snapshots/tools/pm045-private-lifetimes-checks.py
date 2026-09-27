"""Synthetic known-byte method controls; these are not native qualification."""
import copy
import importlib.util
from pathlib import Path
import unittest


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


method = load('lifetimes', 'pm045-private-lifetimes.py')
legacy = load('resource_gpu', 'pm045-resource-gpu-analyze.py')


def fixture():
    shapes = {1: (40, 136, 248, 8), 2: (20, 20, 28, 0)}
    def row(call, phase, live, allocation=None):
        initial, final, peak, excluded = shapes[call]
        report = dict(call_id=call, owner_id=call, allocator_installed=True,
                      initial_bytes=initial-excluded, final_bytes=final-excluded, peak_bytes=peak-excluded,
                      host_initial_bytes=initial, host_final_bytes=final, host_peak_bytes=peak,
                      process_initial_bytes=initial, process_final_bytes=final, process_peak_bytes=276,
                      exceeded=False)
        if phase == 'Allocation':
            # Deliberately stale/invalid accounting must not be used on transitions.
            report.update(peak_bytes=-999, exceeded=True)
        return dict(phase='call', observation_id=1, transition=phase, report=report,
                    renderer_origin=call, owner_label='synthetic', excluded_bytes=excluded,
                    credited_bytes=0, host_limit_bytes=4096, process_limit_bytes=8192,
                    scope_live_bytes=live, allocation=allocation)
    def block(address, size, allocated):
        return dict(address=address, bytes=size, allocated=allocated)
    rows = [row(1, 'Begin', 40), row(1, 'Allocation', 112, block(100,72,True)),
            row(2, 'Begin', 20), row(2, 'Allocation', 28, block(300,8,True)),
            row(1, 'Allocation', 248, block(200,136,True)),
            row(1, 'Allocation', 176, block(100,72,False)),
            row(1, 'Allocation', 136, block(50,40,False)),
            row(2, 'Allocation', 20, block(300,8,False)), row(2, 'Finished', 20),
            row(1, 'Allocation', 160, block(100,24,True)),
            row(1, 'Allocation', 136, block(100,24,False)), row(1, 'Finished', 136)]
    for seq, item in enumerate(rows, 1):
        item.update(sequence=seq, elapsed_ns=seq)
    return [dict(phase='start', observation_id=1, allocation_events=True)] + rows + [
        dict(phase='batch', observation_id=1, dropped_events=0, first_call_id=1, incomplete=False,
             total_events=12, active_calls=0, buffer_capacity_bytes=4096),
        dict(phase='end', observation_id=1, complete_delivery=True, event_loop_ok=True)]


class Controls(unittest.TestCase):
    def test_known_lifetimes_reuse_concurrency_and_summary_integration(self):
        rows = fixture()
        result = method.analyze(rows)
        first = result['calls'][0]
        self.assertEqual(first['final_bytes_by_lifetime'], dict(input=0, output=136, temporary=0))
        self.assertEqual(first['peak_bytes_by_lifetime'], dict(input=40, output=136, temporary=72))
        self.assertEqual(first['simultaneous_scope_peak_bytes'], 248)
        self.assertEqual(first['new_block_lifetimes'], 3)
        self.assertEqual(result['simultaneous_active_call_scope_peaks'], dict(input=60,output=136,temporary=80,total=276))
        self.assertFalse(result['qualification_pass'])
        integrated = legacy.validate_private(rows)
        self.assertEqual(integrated['allocation_lifetimes'], result)
        self.assertEqual(integrated['calls'], 2)
        for row in rows:
            if row.get('transition') == 'Allocation':
                for key in ('renderer_origin', 'owner_label', 'excluded_bytes', 'credited_bytes', 'host_limit_bytes', 'process_limit_bytes'):
                    del row[key]
                row['report'] = {key: row['report'][key] for key in ('call_id', 'owner_id', 'allocator_installed')}
        self.assertEqual(method.analyze(rows), result)
        self.assertEqual(legacy.validate_private(rows)['allocation_lifetimes'], result)

    def test_corruption_and_absent_coverage_rejected(self):
        def underreport_admission(rows):
            for row in rows:
                if row.get('report', {}).get('call_id') == 1:
                    row.update(host_limit_bytes=200, process_limit_bytes=200)
                    for kind in ('host', 'process'):
                        for endpoint in ('initial', 'final', 'peak'):
                            row['report'][kind + '_' + endpoint + '_bytes'] = 0
        forged = fixture()
        underreport_admission(forged)
        with self.assertRaises(ValueError):
            legacy.validate_private(forged)
        mutations = [underreport_admission, lambda r:r[0].update(allocation_events=False), lambda r:r.pop(2),
                     lambda r:r[-2].update(dropped_events=1), lambda r:r[2].update(scope_live_bytes=1),
                     lambda r:r[5]['allocation'].update(address=100),
                     lambda r:r[4]['allocation'].update(address=100),
                     lambda r:r[6]['allocation'].update(bytes=71),
                     lambda r:r[7]['allocation'].update(bytes=400),
                     lambda r:r[-3]['report'].update(peak_bytes=239),
                     lambda r:r[-3].update(transition='Abandoned'),
                     lambda r:r[3]['report'].update(owner_id=1),
                     lambda r:r[-1].update(event_loop_ok=False),
                     lambda r:r[-2].update(active_calls=1)]
        for change in mutations:
            rows = fixture()
            change(rows)
            with self.assertRaises((ValueError, KeyError)):
                method.analyze(rows)

    def test_reported_overrun_is_preserved_and_legacy_mode_is_not_upgraded(self):
        rows = fixture()
        rows[-3]['report'].update(exceeded=True, host_peak_bytes=5000)
        result = method.analyze(rows)
        self.assertEqual(result['overrun_calls'], [1])
        self.assertFalse(result['qualification_pass'])
        with self.assertRaises(AssertionError):
            legacy.validate_private(rows)
        rows = [copy.deepcopy(row) for row in fixture() if row.get('transition') != 'Allocation']
        rows[0]['allocation_events'] = False
        for seq, row in enumerate(rows[1:-2], 1):
            row['sequence'] = seq
        rows[-2]['total_events'] = 4
        self.assertIsNone(legacy.validate_private(rows)['allocation_lifetimes'])


if __name__ == '__main__':
    unittest.main()
