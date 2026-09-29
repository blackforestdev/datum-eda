#!/usr/bin/env python3
"""Offline method controls; reuse archived input, never launch a native workload."""
import copy
import json
from pathlib import Path
import tarfile
import unittest
from gpu_r3_campaign import next_index
from gpu_r3_pointer_input import pointer_input

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT/'docs/reviews/gui-performance/gpu-redraw-proposal/stopped-experiment-r2'


class RunnerControls(unittest.TestCase):
    def test_single_use_campaign_stops_and_binds_declaration(self):
        declaration = dict(run_cap=12, runs=[None]*12)
        state = dict(status='ready', declaration_sha256='pinned', results=[])
        self.assertEqual(next_index(state,declaration,'pinned'),0)
        for status in ['running','stopped','pointer_complete']:
            with self.assertRaises(AssertionError):
                next_index(dict(state,status=status),declaration,'pinned')
        with self.assertRaises(AssertionError):
            next_index(state,declaration,'changed')
        for result in ['invalid','valid_budget_failure']:
            with self.assertRaises(AssertionError):
                next_index(dict(state,results=[dict(status=result)]),declaration,'pinned')
        with self.assertRaises(AssertionError):
            next_index(dict(state,results=[dict(status='valid_descriptive_run')]*12),declaration,'pinned')

    def test_mixed_native_demands_preserve_exact_archived_pointer_route(self):
        with tarfile.open(EVIDENCE/'candidate-0-raw.tar.gz') as archive:
            read = lambda name: json.load(archive.extractfile(name))
            receipt, schedule, result = read('input-receipt.json'),read('schedule.json'),read('result.json')
        declaration=json.loads((EVIDENCE/'declaration.json').read_text())
        records=[]
        for event in receipt['records']:
            records.append(event)
            # A completed native round/exposure is real causal evidence but is
            # not an additional pointer position or viewport acknowledgement.
            extra=copy.deepcopy(event)
            extra.update(position=None,route='unhandled')
            records.append(extra)
        for i,event in enumerate(records):event['sequence']=i
        receipt['records']=records
        receipt['final_state']['render_revision']=999
        for event in records:
            event['before']['render_revision']=1
            event['after']['render_revision']=1
        args=(schedule,receipt,result['tail_end']['monotonic_ns'],declaration['expected_context'],8_333_334)
        self.assertEqual(pointer_input(*args)['completed_viewport_routes'],1280)
        records[3]['button']=[1,True]
        with self.assertRaises(AssertionError):pointer_input(*args)
        records[3]['button']=None
        motion=next(event for event in records if event['position']==[301,150])
        motion['position']=[302,150]
        with self.assertRaises(AssertionError):pointer_input(*args)


if __name__=='__main__':unittest.main()
