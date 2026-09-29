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
    def test_final_copy_boundary_covers_every_regional_graph(self):
        for graph in ([], ["frame"], ["dialog"], ["suffix"], ["frame", "suffix"], ["restore", "suffix"]):
            receipt, sample = fixtures()
            names = ["upload-leading"] + graph + ["frame-trailing"]
            ticks = [value for i in range(len(names)) for value in (10 + i * 20, 11 + i * 20)]
            sample.update(raw_ticks=ticks, passes_ns=[[name, 1] for name in names],
                          frame_span_ns=ticks[-1] - ticks[0], own_pass_sum_ns=len(names))
            record = sample["submission_manifest"][0]
            record.update(first_tick=ticks[0], last_tick=ticks[-1], transfer_first_tick=ticks[1],
                          transfer_last_tick=ticks[2], span_ns=ticks[-1] - ticks[0],
                          transfer_interval_ns=ticks[2] - ticks[1])
            result = validate([sample], receipt, DECLARATION, final_copy_marker=True)[0]
            self.assertEqual(result["frame_span_ns"], ticks[-1] - ticks[0])
            self.assertGreater(sample["frame_span_ns"], sample["own_pass_sum_ns"])
            # A marker before the final draw cannot stand in for copy completion.
            bad = copy.deepcopy(sample)
            bad["passes_ns"][-1][0] = "suffix"
            with self.assertRaises(ValueError):
                validate([bad], receipt, DECLARATION, final_copy_marker=True)
            # Old source evidence remains readable only under its older method.
            with self.assertRaises(ValueError): validate([sample], receipt, DECLARATION)
        receipt, old = fixtures()
        with self.assertRaises(ValueError):
            validate([old], receipt, DECLARATION, final_copy_marker=True)

    def test_repreparation_keeps_earlier_active_uploads_in_the_complete_span(self):
        receipt, sample = fixtures()
        old = [1, 1, 1, 1, 1, 1, 0]
        new = [1, 2, 2, 2, 1, 1, 0]
        sample.update(submission=2, raw_ticks=[10, 11, 20, 21, 30, 31, 40, 50],
                      passes_ns=[["upload-leading", 1], ["upload-trailing", 1],
                                 ["upload-leading", 1], ["frame", 10]],
                      own_pass_sum_ns=13, frame_span_ns=40)
        sample["submission_manifest"] = [
            dict(submission=1, kind="world", attempt=old,
                 workload=[7, 4, 0, 0, 1, 0, 0, 0], first_tick=10, last_tick=21,
                 transfer_first_tick=11, transfer_last_tick=20, span_ns=11, transfer_interval_ns=9),
            dict(submission=2, kind="final", attempt=new,
                 workload=[7, 32, 0, 0, 0, 0, 0, 2], first_tick=30, last_tick=50,
                 transfer_first_tick=31, transfer_last_tick=40, span_ns=20, transfer_interval_ns=9)]
        result = validate([sample], receipt, DECLARATION)[0]
        self.assertEqual(result["phase"], "active")
        self.assertEqual(result["phase_union"], [3, 6])
        self.assertEqual(result["frame_span_ns"], 40)
        self.assertEqual([r["attempt"] for r in sample["submission_manifest"]], [old, new])
        for field in range(7):
            bad = copy.deepcopy(sample)
            bad["submission_manifest"][1]["attempt"][field] = (
                old[field] + 1 if field in (0, 4, 5, 6) else old[field] - 1)
            with self.assertRaises(ValueError): validate([bad], receipt, DECLARATION)

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
