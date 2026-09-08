"""Evaluate exact raw case observations, never product or owner acceptance."""

from pathlib import Path

from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require


def evaluate_capture(output, *, case_id, surface, expected_code, script):
    output, script = Path(output), Path(script).resolve(strict=True)
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
    events = [parse_json(line, "calls.jsonl") for line in (output / "calls.jsonl").read_bytes().splitlines()]
    require(bool(events) and events[0]["event"] == "start" and events[-1]["event"] == "finish",
            "calls", "complete observer invocation required")
    require(all(row["pid"] == result["pid"] and row["invocation_id"] == events[0]["invocation_id"]
                for row in events), "calls", "events do not belong to observed child")
    require(len({row["event_id"] for row in events}) == len(events), "calls", "reused event identity")
    require(events[-1]["profiler_still_installed"] is True, "calls", "observer was removed")
    command = invocation["argv"]
    require(command[command.index("--script") + 1] == str(script)
            and events[0]["argv"] == [str(script), *command[command.index("--") + 1:]]
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
    expected_status = 0 if expected_code is None else 1
    require(result["returncode"] == expected_status, "status", "unexpected exit status")
    report = read("stdout.bin")
    if surface in ("C", "R"):
        require(report["mode"] == "enforce", "report", "report-only is not enforcement")
        require(report["acceptance_asserted"] is False and report["readiness_asserted"] is False,
                "report", "diagnostic cannot assert readiness/acceptance")
        codes = [finding["code"] for finding in report["findings"]]
    else:
        require(report["ok"] is (expected_code is None), "report", "unexpected selector result")
        codes = [failure.split(":", 1)[0] for failure in report.get("failures", [])]
    require(codes == ([] if expected_code is None else [expected_code]),
            "report", "expected precise diagnostic, not an unrelated failure")
    return {"case_id": case_id, "surface": surface, "observation_matches_expected": True,
            "expected_code": expected_code, "invocation_id": result["invocation_id"],
            "result_sha256": sha256((output / "result.json").read_bytes()),
            "acceptance_asserted": False}
