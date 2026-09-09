"""Assess actual handler-triggered termination separately from ordinary results."""

from pathlib import Path
import signal

from workflow_delivery_capture_assertions import capture_envelope
from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require

INTERRUPTION_CASE = "INFRA-S06-06.observed-call"


def evaluate_interruption(output, *, case_id, surface, trigger):
    output = Path(output)
    result, invocation = capture_envelope(output, case_id, surface, expected_kind="interrupted")
    require(case_id == INTERRUPTION_CASE and invocation["interrupt_on_call"] == trigger,
            "interruption", "exact interruption case and trigger required")
    record = parse_json((output / "interruption.json").read_bytes(), "interruption.json")
    require(record["pid"] == result["pid"] and record["signal"] == "SIGTERM"
            and type(record["escalated"]) is bool, "interruption", "wrong signal or child")
    status = -signal.SIGKILL if record["escalated"] else -signal.SIGTERM
    require(result["returncode"] == record["returncode"] == status,
            "interruption", "actual signal termination required")
    times = [record[key] for key in ("live_observed_ns", "signal_sent_ns", "finished_ns")]
    require(all(type(value) is int and value > 0 for value in times) and times == sorted(times),
            "interruption", "ordered observed liveness/signal/exit times required")
    require(record["acceptance_asserted"] is False, "interruption", "termination is not acceptance")
    line = bytes.fromhex(record["event_line_hex"])
    require(sha256(line) == record["event_line_sha256"] and line.endswith(b"\n"),
            "interruption", "exact retained call event required")
    raw = (output / trigger["trace"]).read_bytes()
    lines = [part for part in raw.splitlines(keepends=True) if part.endswith(b"\n")]
    require(line in lines and record["trace"] == trigger["trace"], "interruption", "event absent from trace")
    events = [parse_json(part, "trace") for part in lines]
    require(events and events[0]["event"] == "start", "interruption", "observed interpreter start required")
    event = parse_json(line, "call event")
    require(event == record["event"] and event["event"] == "call"
            and event["function"] == trigger["function"]
            and event["source"]["path"] == trigger["source"]
            and event["source"]["sha256"] == sha256(Path(trigger["source"]).read_bytes()),
            "interruption", "wrong handler source or function")
    require(all(row["pid"] == result["pid"] and row["invocation_id"] == events[0]["invocation_id"]
                for row in events), "interruption", "trace belongs to another child or invocation")
    require(len({row["event_id"] for row in events}) == len(events), "interruption", "reused event identity")
    require(not any(row["event"] == "finish" for row in events),
            "interruption", "completed observer cannot stand for interrupted execution")
    return {"case_id": case_id, "surface": surface, "invocation_id": result["invocation_id"],
            "observation_matches_expected": True, "termination_status": status,
            "result_sha256": sha256((output / "result.json").read_bytes()),
            "complete_matrix": False, "child_tool_closure_complete": False, "acceptance_asserted": False}
