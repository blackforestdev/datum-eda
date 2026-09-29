#!/usr/bin/env python3
"""Validate complete r3 GPU receipts against explicit causal workload demands.

No launch, retry, benchmark, log-arrival window, percentile filtering or acceptance
policy lives here. The bounded runner must separately enforce input and stop rules.
"""
from __future__ import annotations
import math

PHASES = ("startup", "warmup", "active", "still", "drain", "close")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def finite(value):
    return isinstance(value, (int, float)) and math.isfinite(value)


def validate_demands(receipt, declaration):
    require(receipt.get("complete") and not receipt.get("overflow"), "incomplete input receipt")
    manifest = receipt.get("workload_manifest") or {}
    for field in ("epoch", "warmup_ns", "active_ns", "still_ns", "drain_ns"):
        require(manifest.get(field) == declaration.get(field), f"workload declaration differs: {field}")
    epoch = declaration["epoch"]
    require(isinstance(epoch, int) and epoch > 0, "missing workload epoch")
    require(declaration["active_ns"] - declaration["warmup_ns"] == 5_000_000_000
            and declaration["still_ns"] - declaration["active_ns"] == 30_000_000_000
            and declaration["drain_ns"] - declaration["still_ns"] == 5_000_000_000,
            "wrong declared workload duration")
    demands = {}
    revisions = {}
    closed = False
    previous_time = 0
    for record in receipt["records"]:
        tag = record.get("workload", [])
        require(len(tag) == 3 and tag[0] == epoch and 1 <= tag[1] <= 6 and tag[2] > 0,
                "unknown native workload demand")
        require(tag[2] not in demands and record.get("completed_ns") is not None,
                "duplicate or unfinished native demand")
        now = record.get("workload_ns")
        require(isinstance(now, int) and now >= previous_time, "missing or reversed demand time")
        previous_time = now
        if record.get("demand_kind") == "close":
            require(now >= declaration["drain_ns"], "workload closed before fixed still interval")
            closed = True
        boundaries = [declaration[k] for k in ("warmup_ns", "active_ns", "still_ns", "drain_ns")]
        phase = 6 if closed else 1 + sum(now >= boundary for boundary in boundaries)
        require(tag[1] == phase, "demand phase differs from declared workload and explicit close")
        require(tag[2] == len(demands) + 1, "missing native demand sequence")
        demands[tag[2]] = tag[1]
        revisions[tag[2]] = record["before"]["render_revision"]
    return epoch, demands, revisions


def validate(samples, receipt, declaration, *, final_copy_marker=False):
    epoch, demands, revisions = validate_demands(receipt, declaration)
    seal = receipt.get("gpu_drained")
    require(isinstance(seal, list) and len(seal) == 3 and all(isinstance(v, int) and v > 0 for v in seal), "missing drained frame range")
    frames, submissions, output = set(), set(), []
    for sample in samples:
        require(sample.get("status") != "incomplete", "cancelled or incomplete GPU frame")
        identity = tuple(sample[k] for k in ("host", "device_epoch", "frame"))
        require(identity not in frames, "duplicate GPU frame")
        require(identity[:2] == tuple(seal[:2]) and 1 <= identity[2] <= seal[2], "frame outside drained host/epoch/range")
        frames.add(identity)
        period = sample["timestamp_period_ns"]
        require(finite(period) and period > 0, "invalid GPU timestamp period")
        ticks = sample["raw_ticks"]
        passes = sample["passes_ns"]
        require(passes and len(ticks) == len(passes) * 2, "missing GPU pass pairs")
        require(all(isinstance(t, int) and t >= 0 for t in ticks), "invalid raw timestamp")
        require(all(a <= b for a, b in zip(ticks, ticks[1:])), "overlapping or reversed GPU passes")
        for index, (_, duration) in enumerate(passes):
            first, last = ticks[index * 2:index * 2 + 2]
            require(last >= first and finite(duration)
                    and math.isclose(duration, (last - first) * period, rel_tol=1e-9, abs_tol=1e-6),
                    "invalid GPU pass interval")
        span = (ticks[-1] - ticks[0]) * period
        require(0 <= span < 2_000_000_000 and math.isclose(span, sample["frame_span_ns"], rel_tol=1e-9, abs_tol=1e-6),
                "invalid complete GPU frame span")
        require(math.isclose(sum(p[1] for p in passes), sample["own_pass_sum_ns"], rel_tol=1e-9, abs_tol=1e-6), "invalid pass sum")
        records = sample.get("submission_manifest", [])
        require(1 <= len(records) <= 6, "missing or excessive submission manifest")
        require(records[-1]["submission"] == sample["submission"], "final submission identity differs")
        cursor, previous_end, previous_attempt, phases = 0, None, None, set()
        for index, record in enumerate(records):
            submission = record["submission"]
            require(submission not in submissions, "duplicate GPU submission identity")
            submissions.add(submission)
            final = index == len(records) - 1
            require(record["kind"] == "final" if final else record["kind"] in ("world", "glyph", "terminal"), "invalid submission kind/order")
            names = [p[0] for p in passes]
            require(cursor < len(names) and names[cursor] == "upload-leading", "missing leading upload boundary")
            first, transfer_first = ticks[cursor * 2:cursor * 2 + 2]
            require(cursor + 1 < len(names), "missing transfer or final graph")
            transfer_last = ticks[(cursor + 1) * 2]
            if final:
                graph = names[cursor + 1:]
                if final_copy_marker:
                    require(graph[-1:] == ["frame-trailing"], "missing final copy boundary")
                    graph = graph[:-1]
                allowed = (["frame"], ["dialog"], ["suffix"], ["frame", "suffix"], ["restore", "suffix"])
                require(graph in allowed or (final_copy_marker and not graph), "incomplete final graph boundary")
                last = ticks[-1]
                cursor = len(names)
            else:
                require(names[cursor + 1] == "upload-trailing", "missing trailing upload boundary")
                last = ticks[(cursor + 1) * 2 + 1]
                cursor += 2
            require(first <= transfer_first <= transfer_last <= last and (previous_end is None or previous_end <= first), "reversed or overlapping submission bounds")
            previous_end = last
            for key, wanted in (("first_tick", first), ("last_tick", last), ("transfer_first_tick", transfer_first), ("transfer_last_tick", transfer_last)):
                require(record[key] == wanted, f"submission raw boundary differs: {key}")
            for key, wanted in (("span_ns", (last - first) * period), ("transfer_interval_ns", (transfer_last - transfer_first) * period)):
                require(finite(record[key]) and math.isclose(record[key], wanted, rel_tol=1e-9, abs_tol=1e-6), f"invalid submission duration: {key}")
            attempt = record["attempt"]
            require(len(attempt) == 7 and all(isinstance(v, int) and v >= 0 for v in attempt)
                    and all(attempt[i] > 0 for i in range(4)), "missing render attempt lineage")
            if previous_attempt:
                require(all(previous_attempt[i] == attempt[i] for i in (0, 4, 5, 6))
                        and all(previous_attempt[i] <= attempt[i] for i in (1, 2, 3)),
                        f"changed or reversed render lineage: previous={previous_attempt} current={attempt}")
            previous_attempt = attempt
            tag = record.get("workload", [])
            require(len(tag) == 8 and tag[0] == epoch and 0 < tag[1] < 64, "missing workload attribution")
            for phase in range(6):
                if tag[1] & (1 << phase):
                    require(demands.get(tag[phase + 2]) == phase + 1, "workload demand absent or attributed to wrong phase")
                    require(revisions[tag[phase + 2]] <= attempt[2], "GPU frame attributed to a future demand revision")
                    phases.add(phase + 1)
        require(cursor == len(passes), "unattributed GPU pass")
        # Explicit causal phase unions preserve active work through later demands.
        # Only an exclusively close-tagged frame is excluded as close-only.
        phase = "active" if 3 in phases else PHASES[min(phases) - 1]
        output.append({"identity": identity, "phase": phase, "phase_union": sorted(phases), "frame_span_ns": span})
    require(len(frames) == seal[2], "missing GPU frame from drained range")
    require(output, "no complete GPU frames")
    return output
