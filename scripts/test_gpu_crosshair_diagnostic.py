"""Offline coverage/capture controls using preserved native input evidence."""
import copy
import json
from pathlib import Path
import tarfile
import tempfile
import subprocess
import sys
import unittest
import ctypes
import runpy
import signal
from unittest.mock import patch
from gpu_crosshair_diagnostic import analyze, ensure_alive
from gpu_r3_trial_outcome import record_failure

ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'docs/reviews/gui-performance/gpu-redraw-proposal/stopped-experiment-r2'

class DiagnosticControls(unittest.TestCase):
    def fixture(self):
        with tarfile.open(BASE/'candidate-0-raw.tar.gz') as a:
            read=lambda name:json.load(a.extractfile(name))
            receipt,schedule,report=read('input-receipt.json'),read('schedule.json'),read('result.json')
        declaration=json.loads((BASE/'declaration.json').read_text())
        snapshot=copy.deepcopy(receipt)
        origin=snapshot['monotonic_origin_ns']
        last=origin+snapshot['records'][-1]['completed_ns']
        snapshot.update(schema='datum.input-receipt/v1',mode='output-diagnostic',complete=False,
                        coverage_complete=True,first_error=None,pending=None,gpu_drained=None,
                        record_count=len(snapshot['records']),snapshot_ns=last-origin+3_000_000)
        for r in snapshot['records']:
            r['workload']=[0,0,0]
            r['demand_kind']='pointer' if r['position'] is not None else 'native_event'
        report.update(final_capture_started_ns=last+1_000_000,final_capture_finished_ns=last+2_000_000)
        return snapshot,schedule,report,declaration,receipt

    def test_exact_archived_input_can_support_output_diagnosis_without_gpu_tags(self):
        result=analyze(*self.fixture())
        self.assertTrue(result['capture_state_matches'])
        self.assertEqual(result['input']['received'],1280)

    def test_cleared_cursor_is_reported_not_reclassified_as_correct_output(self):
        args=list(self.fixture());snapshot,_,report,_,_=args
        r=copy.deepcopy(snapshot['records'][-1]);r['sequence']=len(snapshot['records'])
        r.update(position=None,button=None,demand_kind='cursor_leave',route='unhandled')
        r['before']=copy.deepcopy(r['after']);r['after']['cursor']=None;r['after']['native_cursor']=None
        r['received_ns']=r['completed_ns']+100;r['completed_ns']=r['received_ns']+100
        snapshot['records'].append(r);snapshot['record_count']+=1
        result=analyze(*args)
        self.assertFalse(result['capture_state_matches'])
        self.assertEqual(result['transitions_after_last_pointer'][-1]['kind'],'cursor_leave')

    def test_missing_coverage_and_capture_transition_never_pass(self):
        for field,value in [('coverage_complete',False),('complete',True),('overflow',True),('first_error','gap')]:
            args=list(self.fixture());args[0][field]=value
            with self.assertRaises(AssertionError):analyze(*args)
        args=list(self.fixture());snapshot,_,report,_,_=args
        r=snapshot['records'][-1]
        report['final_capture_started_ns']=snapshot['monotonic_origin_ns']+r['received_ns']
        report['final_capture_finished_ns']=snapshot['monotonic_origin_ns']+r['completed_ns']
        with self.assertRaises(AssertionError):analyze(*args)

    def test_actual_producer_cancellation_exports_partial_schedule(self):
        class Function:
            def __init__(self, action): self.action=action
            def __call__(self,*args): return self.action()
        class Library:
            def __init__(self): self.flushes=0
            def __getattr__(self,name):
                if name == 'XFlush':
                    def flush():
                        self.flushes+=1
                        if self.flushes == 4: signal.raise_signal(signal.SIGTERM)
                        return 1
                    result=Function(flush)
                else: result=Function(lambda:1)
                setattr(self,name,result)
                return result
        clock=[0]
        def sleep(seconds): clock[0]+=round(seconds*1e9)
        old=signal.getsignal(signal.SIGTERM)
        try:
            with tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp)/'partial.json'
                with patch.object(sys,'argv',['producer','0','0',str(path),'1000000']), \
                     patch.object(ctypes,'CDLL',return_value=Library()), \
                     patch('time.monotonic_ns',side_effect=lambda:clock[0]), \
                     patch('time.sleep',side_effect=sleep):
                    with self.assertRaises(SystemExit):
                        runpy.run_path(str(ROOT/'scripts/gpu_r3_pointer_stream.py'))
                data=json.loads(path.read_text())
                self.assertEqual(len(data['rows']),3)
                self.assertEqual(data['scheduled_count'],3600)
        finally: signal.signal(signal.SIGTERM,old)

    def test_dead_child_reason_precedes_focus_or_cleanup_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'stderr.log'
            with path.open('w') as stderr:
                p=subprocess.Popen([sys.executable,'-c',"import sys; print('datum-gui error: original failure',file=sys.stderr); sys.exit(7)"],stderr=stderr)
            p.wait(timeout=2)
            report={}
            try:ensure_alive(p,report,path)
            except RuntimeError as error:record_failure(report,error)
            record_failure(report,RuntimeError('later cleanup'))
            self.assertEqual(report['exit_code'],7)
            self.assertIn('original failure',report['error'])
            self.assertEqual(report['status'],'invalid')

if __name__=='__main__':unittest.main()
