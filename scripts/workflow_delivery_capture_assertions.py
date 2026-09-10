"""Evaluate exact raw case observations, never product or owner acceptance."""

from pathlib import Path

from workflow_delivery_io import parse_json, sha256
from workflow_delivery_shapes import require


def capture_envelope(output, case_id, surface, *, expected_kind="exited"):
    """Validate retained process/state artifacts independently of handler tracing."""
    output = Path(output)
    def read(name):
        return parse_json((output / name).read_bytes(), name)
    result, invocation = read("result.json"), read("invocation.json")
    require(expected_kind in ("exited", "interrupted") and result["kind"] == expected_kind,
            "capture", "timeout/launch/harness failure is not a refusal or observed interruption")
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


def evaluate_invocation_error(output, *, case_id, surface, script, expected_error):
    output, script = Path(output), Path(script).resolve(strict=True)
    result, invocation = capture_envelope(output, case_id, surface)
    command = invocation["argv"]
    require(surface in ("C", "R") and command[command.index("--script") + 1] == str(script),
            "calls", "explicit CLI invocation-error surface required")
    observer_calls(output, result, invocation, script, command[command.index("--") + 1:])
    require(result["returncode"] == 2, "status", "invocation error requires exit 2")
    require((output / "stdout.bin").read_bytes() == b"", "report", "invocation error cannot claim successful checks")
    stderr = (output / "stderr.bin").read_text()
    require(stderr.startswith("usage: ") and stderr.splitlines()[-1] ==
            script.name + ": error: " + expected_error, "report", "wrong invocation-error cause")
    return {"case_id": case_id, "surface": surface, "kind": "invocation_error",
            "observation_matches_expected": True, "expected_error": expected_error,
            "invocation_id": result["invocation_id"], "acceptance_asserted": False,
            "result_sha256": sha256((output / "result.json").read_bytes())}


def evaluate_capture(output, *, case_id, surface, expected_code, script,
                     expected_path=None, expected_detail=None, expected_diagnostics=None, expected_mode="enforce"):
    output, script = Path(output), Path(script).resolve(strict=True)
    result, invocation = capture_envelope(output, case_id, surface)
    command = invocation["argv"]
    require(command[command.index("--script") + 1] == str(script), "calls", "observer script differs")
    observer_calls(output, result, invocation, script, command[command.index("--") + 1:])
    require(expected_mode in ("enforce", "report-only") and
            (expected_mode == "enforce" or surface in ("C", "R")), "mode", "explicit CLI diagnostic mode required")
    expected_status = 0 if expected_code is None else 1
    if expected_mode == "report-only":
        expected_status = 0
    if expected_code == "WDQ-TRUST" and surface in ("C", "R"):
        expected_status = 2
    require(result["returncode"] == expected_status, "status", "unexpected exit status")
    report = parse_json((output / "stdout.bin").read_bytes(), "stdout.bin")
    if surface in ("C", "R"):
        require(report["mode"] == expected_mode, "report", "report-only is not enforcement")
        require(report["acceptance_asserted"] is False and report["readiness_asserted"] is False,
                "report", "diagnostic cannot assert readiness/acceptance")
        codes = [finding["code"] for finding in report["findings"]]
        diagnostics = [(finding["path"], finding["detail"]) for finding in report["findings"]]
    else:
        require(report["ok"] is (expected_code is None), "report", "unexpected selector result")
        codes = [failure.split(":", 1)[0] for failure in report.get("failures", [])]
        diagnostics = [tuple(failure.split(": ", 2)[1:]) for failure in report.get("failures", [])]
    expected = ([] if expected_code is None else
                [{"code": expected_code, "path": expected_path, "detail": expected_detail}])
    if expected_diagnostics is not None:
        require(surface == "S" and type(expected_diagnostics) is list and bool(expected_diagnostics)
                and all(type(row) is dict and set(row) == {"code", "path", "detail"}
                        for row in expected_diagnostics), "report", "explicit ordered selector diagnostics required")
        expected = expected_diagnostics
    require(codes == [row["code"] for row in expected],
            "report", "expected precise diagnostic, not an unrelated failure")
    for actual, row in zip(diagnostics, expected):
        if row["path"] is not None:
            require(len(actual) == 2 and actual[0] == row["path"], "report", "wrong diagnostic input path")
        if row["detail"] is not None:
            require(bool(actual) and row["detail"] in actual[-1], "report", "wrong diagnostic cause")
    return {"case_id": case_id, "surface": surface, "observation_matches_expected": True,
            "expected_code": expected_code, "invocation_id": result["invocation_id"],
            "expected_path": expected_path, "expected_detail": expected_detail,
            "expected_diagnostics": expected,
            "report_only": expected_mode == "report-only",
            "result_sha256": sha256((output / "result.json").read_bytes()),
            "acceptance_asserted": False}


def evaluate_text_output(output, *, surface):
    """Inspect actual text and observed descriptors, not a GUI accessibility claim."""
    output = Path(output)
    invocation = parse_json((output / "invocation.json").read_bytes(), "invocation")
    environment = invocation["environment"]
    require(all(environment.get(key) == value for key, value in
                {"TERM": "dumb", "NO_COLOR": "1", "CLICOLOR": "0", "FORCE_COLOR": "0"}.items())
            and not {"DISPLAY", "WAYLAND_DISPLAY"} & set(environment), "text",
            "explicit no-color and GUI-free environment required")
    start = parse_json((output / "calls.jsonl").read_bytes().splitlines()[0], "observer start")
    require(start["stdio_tty"] == {"stdin": False, "stdout": False, "stderr": False}
            and start["stdin_target"] == "/dev/null", "text", "observed noninteractive descriptors required")
    stdout, stderr = ((output / name).read_bytes() for name in ("stdout.bin", "stderr.bin"))
    require(b"\x1b" not in stdout + stderr and b"\x9b" not in stdout + stderr,
            "text", "color/control escape sequence present")
    report = parse_json(stdout.splitlines()[-1] if surface == "H" else stdout, "text report")
    if surface == "S":
        diagnostics = report.get("failures", [])
        require(all("WDQ-COVERAGE: src/unscoped.py: production change has no promoted scope" in item
                    for item in diagnostics), "text", "selector refusal lacks path and authority context")
    else:
        diagnostics = report["findings"]
        require(all(item.get("path") and item.get("detail") and item.get("owning_lane")
                    for item in diagnostics), "text", "refusal lacks path, explanation or responsible lane")
    return {"noninteractive_descriptors_observed": True, "no_color_escapes": True,
            "diagnostics": diagnostics, "gui_accessibility_asserted": False}


def recovery_link(prior, current, *, case_id, surface):
    """Bind a real ordinary refusal to a fresh invocation without authority repair."""
    prior, current = Path(prior), Path(current)
    previous, previous_invocation = capture_envelope(prior / "capture", "INFRA-S02-09.outside-scope", surface)
    result, invocation = capture_envelope(current, case_id, surface)
    require(previous["returncode"] == 1 and result["returncode"] == 0
            and previous["invocation_id"] != result["invocation_id"]
            and previous_invocation["fixture_nonce"] == invocation["fixture_nonce"],
            "recovery", "distinct refused and successful invocations of one retained recipe required")
    raw = (prior / "capture/stdout.bin").read_bytes()
    report = parse_json(raw.splitlines()[-1] if surface == "H" else raw, "prior refusal")
    if surface == "S":
        require(len(report["failures"]) == 1 and report["failures"][0].startswith(
            "WDQ-COVERAGE: src/unscoped.py: production change has no promoted scope"),
            "recovery", "wrong prior refusal")
    else:
        require(len(report["findings"]) == 1 and report["findings"][0]["code"] == "WDQ-COVERAGE"
                and report["findings"][0]["path"] == "src/unscoped.py", "recovery", "wrong prior refusal")
    before = parse_json((prior / "capture/before.json").read_bytes(), "prior state")
    restored = parse_json((current / "before.json").read_bytes(), "recovery state")
    for key in ("datum.workflowDeliveryAuthorityRef", "datum.workflowDeliveryBaseRef",
                "datum.workflowDeliveryEnvironmentPath"):
        require(before["local_trust"][key] == restored["local_trust"][key], "recovery", "authority was refreshed")
    for path in ("specs/workflow_delivery_policy.json", "requested-environment.json"):
        require(before["files"][path] == restored["files"][path], "recovery", "policy or environment changed")
    return {"prior_invocation_id": previous["invocation_id"],
            "prior_result_sha256": sha256((prior / "capture/result.json").read_bytes()),
            "prior_result_path": "prior-refusal/capture/result.json", "authority_refreshed": False}
