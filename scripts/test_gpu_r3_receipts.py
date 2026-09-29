#!/usr/bin/env python3
"""Offline r3 receipt consumer controls; no runtime experiment."""
import copy
import unittest
from gpu_r3_receipts import validate

DECLARATION = dict(epoch=7, warmup_ns=10, active_ns=5_000_000_010, still_ns=35_000_000_010, drain_ns=40_000_000_010)


def fixtures():
    receipt = dict(gpu_drained=[1, 1, 1], complete=True, overflow=False, workload_manifest=DECLARATION.copy(), records=[
        dict(workload=[7, 3, 1], workload_ns=DECLARATION["active_ns"] + 1, demand_kind="pointer", before={"render_revision":1}, completed_ns=20), dict(workload=[7, 6, 2], workload_ns=DECLARATION["drain_ns"] + 1, demand_kind="close", before={"render_revision":1}, completed_ns=40)])
    sample = dict(host=1, device_epoch=1, frame=1, submission=1, timestamp_period_ns=1.0,
                  raw_ticks=[10, 11, 20, 30, 40, 50], passes_ns=[["upload-leading", 1], ["restore", 10], ["suffix", 10]],
                  frame_span_ns=40, own_pass_sum_ns=21, submission_manifest=[dict(submission=1, kind="final", attempt=[1, 1, 1, 1, 1, 1, 0],
                    workload=[7, 4, 0, 0, 1, 0, 0, 0], first_tick=10, last_tick=50, transfer_first_tick=11,
                    transfer_last_tick=20, span_ns=40, transfer_interval_ns=9)])
    return receipt, sample


class Receipts(unittest.TestCase):
    def test_late_active_and_explicit_close_are_semantic(self):
        receipt, sample = fixtures()
        sample["log_arrival_ns"] = DECLARATION["drain_ns"] + 1000
        self.assertEqual(validate([sample], receipt, DECLARATION)[0]["phase"], "active")
        sample["submission_manifest"][0]["workload"] = [7, 32, 0, 0, 0, 0, 0, 2]
        self.assertEqual(validate([sample], receipt, DECLARATION)[0]["phase"], "close")
        sample["submission_manifest"][0]["workload"] = [7, 36, 0, 0, 1, 0, 0, 2]
        self.assertEqual(validate([sample], receipt, DECLARATION)[0]["phase"], "active")

    def test_missing_final_upload_unknown_and_duplicate_work_fail(self):
        receipt, sample = fixtures()
        variants = []
        for key, value in [("submission_manifest", []), ("status", "incomplete"), ("frame_span_ns", 20)]:
            bad = copy.deepcopy(sample); bad[key] = value; variants.append(bad)
        bad = copy.deepcopy(sample); bad["passes_ns"][0][0] = "unmeasured-upload"; variants.append(bad)
        bad = copy.deepcopy(sample); bad["submission_manifest"][0]["workload"][4] = 99; variants.append(bad)
        bad = copy.deepcopy(sample); bad["submission_manifest"][0]["workload"][0] = 8; variants.append(bad)
        bad = copy.deepcopy(sample); bad["submission_manifest"][0]["transfer_first_tick"] = 21; variants.append(bad)
        for bad in variants:
            with self.assertRaises(ValueError): validate([bad], receipt, DECLARATION)
        with self.assertRaises(ValueError): validate([sample, sample], receipt, DECLARATION)
        bad = copy.deepcopy(sample); bad["frame"] += 1
        with self.assertRaises(ValueError): validate([sample, bad], receipt, DECLARATION)

    def test_missing_frames_and_overlapping_passes_fail(self):
        receipt, sample = fixtures()
        receipt["gpu_drained"][2] = 2
        with self.assertRaises(ValueError): validate([sample], receipt, DECLARATION)
        receipt["gpu_drained"][2] = 1
        sample["raw_ticks"] = [10, 11, 20, 80, 40, 50]
        sample["passes_ns"][1][1] = 60
        sample["own_pass_sum_ns"] = 71
        with self.assertRaises(ValueError): validate([sample], receipt, DECLARATION)


if __name__ == "__main__":
    unittest.main()
