"""Evaluate exact raw case observations, never product or owner acceptance."""

from pathlib import Path

from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require


def capture_envelope(output, case_id, surface):
    """Validate retained process/state artifacts independently of handler tracing."""
    output = Path(output)
    def read(name):
        return parse_json((output / name).read_bytes(), name)
    result, invocation = read("result.json"), read("invocation.json")
    require(result["kind"] == "exited", "capture", "timeout/launch/harness failure is not a refusal")
    require(invocation["case_id"] == case_id and invocation["surface"] == surface,
            "capture", "wrong case or surface")
    require(result["invocation_id"] == invocation["invocation_id"], "capture", "invocation mismatch")
    names = [row["path"] for row in result["artifacts"]]
    require(len(set(names)) == len(names) and set(names) ==
            {path.name for path in output.iterdir()} - {"result.json"}, "capture", "artifact set differs")
    for artifact in result["artifacts"]:
        name = artifact["path"]
        require(Path(name).name == name and not (output / name).is_symlink(), name,
                "literal regular capture filename required")
        require(sha256((output / name).read_bytes()) == artifact["sha256"], name, "capture hash mismatch")
    require(read("before.json") == read("after.json"), "state", "observed protected state changed")
    require(result.get("changed_state_fields") == [] and "after_capture_error" not in result,
            "state", "state capture incomplete or inconsistent")
    started = read("started.json")
    require(started["pid"] == result["pid"] and started["invocation_id"] == result["invocation_id"],
            "capture", "observed process start differs")
    return result, invocation


def observer_calls(output, result, invocation, script, arguments):
    output, script = Path(output), Path(script).resolve(strict=True)
    events = [parse_json(line, "calls.jsonl") for line in (output / "calls.jsonl").read_bytes().splitlines()]
    require(bool(events) and events[0]["event"] == "start" and events[-1]["event"] == "finish",
            "calls", "complete observer invocation required")
    require(all(row["pid"] == result["pid"] and row["invocation_id"] == events[0]["invocation_id"]
                for row in events), "calls", "events do not belong to observed child")
    require(len({row["event_id"] for row in events}) == len(events), "calls", "reused event identity")
    require(events[-1]["profiler_still_installed"] is True, "calls", "observer was removed")
    require(events[0]["flags"] == {key: 1 for key in
            ("isolated", "no_site", "ignore_environment", "dont_write_bytecode")},
            "calls", "isolated interpreter observation required")
    observer = script.parent / "workflow_delivery_observe_python.py"
    require(events[0]["observer"]["path"] == str(observer)
            and events[0]["observer"]["sha256"] == sha256(observer.read_bytes()),
            "calls", "observer source identity differs")
    require(events[0]["argv"] == [str(script), *arguments]
            and events[0]["cwd"] == invocation["cwd"], "calls", "observer invocation differs")
    require(events[-1]["script_after"] == events[0]["script"], "calls", "runtime script changed")
    require(events[-1]["acceptance_asserted"] is False, "calls", "observer cannot assert acceptance")
    require(events[-1]["outcome"].get("exit_code") == result["returncode"],
            "calls", "observer outcome differs from actual exit status")
    require(events[0]["script"]["path"] == str(script)
            and events[0]["script"]["sha256"] == sha256(script.read_bytes()),
            "calls", "observed script differs from requested runtime")
    require(any(row["event"] == "call" and row["function"] == "main"
                and row["source"] == events[0]["script"] for row in events),
            "calls", "actual entrypoint main invocation missing")
    return events


def evaluate_capture(output, *, case_id, surface, expected_code, script,
                     expected_path=None, expected_detail=None):
    output, script = Path(output), Path(script).resolve(strict=True)
    result, invocation = capture_envelope(output, case_id, surface)
    command = invocation["argv"]
    require(command[command.index("--script") + 1] == str(script), "calls", "observer script differs")
    observer_calls(output, result, invocation, script, command[command.index("--") + 1:])
    expected_status = 0 if expected_code is None else 1
    require(result["returncode"] == expected_status, "status", "unexpected exit status")
    report = parse_json((output / "stdout.bin").read_bytes(), "stdout.bin")
    if surface in ("C", "R"):
        require(report["mode"] == "enforce", "report", "report-only is not enforcement")
        require(report["acceptance_asserted"] is False and report["readiness_asserted"] is False,
                "report", "diagnostic cannot assert readiness/acceptance")
        codes = [finding["code"] for finding in report["findings"]]
        diagnostics = [(finding["path"], finding["detail"]) for finding in report["findings"]]
    else:
        require(report["ok"] is (expected_code is None), "report", "unexpected selector result")
        codes = [failure.split(":", 1)[0] for failure in report.get("failures", [])]
        diagnostics = [tuple(failure.split(": ", 2)[1:]) for failure in report.get("failures", [])]
    require(codes == ([] if expected_code is None else [expected_code]),
            "report", "expected precise diagnostic, not an unrelated failure")
    if expected_path is not None:
        require(len(diagnostics) == 1 and len(diagnostics[0]) == 2
                and diagnostics[0][0] == expected_path, "report", "wrong diagnostic input path")
    if expected_detail is not None:
        require(len(diagnostics) == 1 and expected_detail in diagnostics[0][-1],
                "report", "wrong diagnostic cause")
    return {"case_id": case_id, "surface": surface, "observation_matches_expected": True,
            "expected_code": expected_code, "invocation_id": result["invocation_id"],
            "expected_path": expected_path, "expected_detail": expected_detail,
            "result_sha256": sha256((output / "result.json").read_bytes()),
            "acceptance_asserted": False}
