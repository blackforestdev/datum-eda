#!/usr/bin/env python3
"""Small ordinary-menu scenario for the existing isolated native QA runner."""
import argparse
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import traceback

from prove_native_project_startup_gui import StartupQA
from run_native_qa_isolated_input import save, validate_input_display


def mark(qa):
    return Path(qa.env["DATUM_GUI_LOG"]).stat().st_size


def preference(qa):
    result = subprocess.run([str(qa.binary_dir / "datum-eda"), "preferences", "get",
                             "datum.units.system"], env=qa.env, cwd=qa.root,
                            capture_output=True, text=True, timeout=15, check=True)
    payload = json.loads(result.stdout)
    if payload.get("ok") is not True:
        raise RuntimeError(f"Preference query failed: {payload}")
    return payload["result"]["result"]["result"]["value"]


def visible_value(qa):
    return next((qa.attributes(node).get("value") for node in qa.nodes()
                 if node.get_name() == "Measurement system"
                 and node.get_role_name() == "combo box"), None)


def open_units(qa, phase, width):
    host = qa.launch(phase)
    before_menu = mark(qa)
    qa.menu_item(host, "Edit", "Preferences", child="Global Preferences…")
    qa.wait("Global Preferences open", lambda: qa.find("Global Preferences"))
    # The ordinary menu creates exactly one new native surface. Bind it once;
    # do not infer a target from whichever window happens to be newest later.
    tail = Path(qa.env["DATUM_GUI_LOG"]).read_bytes()[before_menu:].decode()
    windows = re.findall(r"surface identity window=(WindowId\([0-9]+\))", tail)
    if len(windows) != 1:
        raise RuntimeError(f"Expected one Preferences surface from menu action, got {windows}")
    before_units = mark(qa)
    x, y = (50, 106) if width > 980 else (220, 70)
    qa.command("xdotool", "mousemove", "--window", host, str(x), str(y), "click", "1")
    qa.wait("Units section", lambda: qa.find("Search Units settings"))
    return {"window": windows[0], "opened_after_byte": before_menu}, before_units


def capture(qa, label, window, after, width, value, *, explanation=False):
    nodes = [{"name": "Global Preferences", "role": "dialog"},
             {"name": "Search Units settings"},
             {"name": "Measurement system", "role": "combo box",
              "attributes": {"value": value}}]
    if explanation:
        nodes.append({"name": "Measurement system resolver explanation"})
    return qa.capture(label, qa.host_window, expectation={
        "pid": qa.process.pid, **window, "dialog": "Global Preferences", "after_byte": after,
        "extent": [width, 720], "nodes": nodes})


def run_size(output, binary_dir, label, width, result):
    work = output / "native" / label
    work.mkdir(parents=True)
    qa = StartupQA(work, runtime_parent=Path(os.environ["XDG_RUNTIME_DIR"]),
                   binary_dir=binary_dir)
    observations = result["native"][label] = {"captures": {}, "checks": {},
        "candidate_binaries": qa.binary_hashes, "binary_dir": str(qa.binary_dir)}
    stopped = False
    try:
        qa.start_envelope(output_size=(width, 720))
        window, fence = open_units(qa, "initial", width)
        initial = preference(qa)
        if initial["effective_value"] != "metric" or initial["user_value"] is not None:
            raise RuntimeError("Fresh disposable configuration did not yield factory Metric")
        qa.wait("visible Metric", lambda: visible_value(qa) == "Metric")
        observations["checks"]["initial"] = initial
        observations["captures"]["units"] = capture(qa, "units", window, fence, width, "Metric")
        if label == "wide":
            qa.key(qa.host_window, "Tab", repeat=3)
            fence = mark(qa)
            qa.key(qa.host_window, "End")
            qa.wait("visible Imperial after keyboard edit", lambda: visible_value(qa) == "Imperial")
            changed = preference(qa)
            if changed["effective_value"] != "imperial" or changed["user_value"] != "imperial":
                raise RuntimeError("Visible edit was not committed by the preference resolver")
            observations["checks"]["changed"] = changed
            observations["captures"]["changed"] = capture(qa, "units-changed", window, fence, width, "Imperial")
            qa.key(qa.host_window, "Escape")
            qa.wait("Escape closes Preferences", lambda: qa.find("Global Preferences") is None)
            qa.stop()
            window, fence = open_units(qa, "reopened", width)
            qa.wait("Imperial after process restart", lambda: visible_value(qa) == "Imperial")
            persisted = preference(qa)
            if persisted["effective_value"] != "imperial":
                raise RuntimeError("Edit did not persist after process restart")
            observations["checks"]["persisted"] = persisted
            observations["captures"]["persisted"] = capture(qa, "units-persisted", window, fence, width, "Imperial")
            qa.key(qa.host_window, "Tab", repeat=4)
            fence = mark(qa)
            qa.key(qa.host_window, "Return")
            qa.wait("Reset restores Metric", lambda: visible_value(qa) == "Metric")
            reset = preference(qa)
            if reset["effective_value"] != "metric" or reset["user_value"] is not None:
                raise RuntimeError("Reset did not remove the user override")
            observations["checks"]["reset"] = reset
            observations["captures"]["reset"] = capture(qa, "units-reset", window, fence, width, "Metric")
        else:
            qa.key(qa.host_window, "Tab", repeat=2)
            fence = mark(qa)
            qa.key(qa.host_window, "Return")
            qa.wait("resolver explanation visible", lambda: qa.find("Measurement system resolver explanation"))
            observations["captures"]["explanation"] = capture(
                qa, "explanation", window, fence, width, "Metric", explanation=True)
    finally:
        errors = []
        for cleanup in (qa.stop, qa.close_envelope):
            try:
                cleanup()
            except BaseException as error:
                errors.append(str(error))
        stopped = not errors and qa.process is None and not Path(qa.runtime.name).exists()
        observations["cleanup"] = {"verified": stopped, "errors": errors}
        save(work / "commands.json", qa.actions)
        save(work / "observations.json", qa.observations)
        if not stopped:
            raise RuntimeError(f"Owned native cleanup failed: {errors}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--binary-dir", type=Path, required=True)
    args = parser.parse_args()
    validate_input_display(os.environ)
    result = {"status": "FAIL", "native": {}, "visual_review": "pending",
              "limits": "Focused native behavior and matched-state captures; images must be inspected. "
                        "No automatic prototype conformance, desktop-platform or full Preferences acceptance."}
    try:
        for label, width in (("wide", 1160), ("narrow", 720)):
            run_size(args.output, args.binary_dir, label, width, result)
        browser = subprocess.Popen(["node", str(Path(__file__).with_name("capture_gui_reference.mjs")),
                        "--output", str(args.output / "reference")], start_new_session=True)
        try:
            code = browser.wait(timeout=60)
            if code:
                raise RuntimeError(f"Browser reference capture failed with exit {code}")
        except BaseException:
            # Node may already have exited abnormally while Chromium descendants
            # remain. The session belongs solely to this reference capture.
            try:
                os.killpg(browser.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                try:
                    browser.wait(timeout=6)
                except subprocess.TimeoutExpired:
                    pass
            finally:
                try:
                    os.killpg(browser.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                browser.wait(timeout=5)
            raise
        result["reference"] = str(args.output / "reference/prototype-reference-capture.json")
        result["status"] = "BEHAVIOR_CHECKS_PASSED; visual comparison pending"
    except BaseException as error:
        result["error"] = f"{type(error).__name__}: {error}"
        (args.output / "failure.txt").write_text(traceback.format_exc())
        raise
    finally:
        # These flags are derived from successful owned GUI/companion/compositor
        # teardown, never asserted merely because the scenario reached finally.
        stopped = bool(result["native"]) and all(
            item.get("cleanup", {}).get("verified") is True for item in result["native"].values())
        result["verified_owner_stops"] = dict.fromkeys(["gui", "daemons", "clipboard", "Weston"], stopped)
        result["ipc_runtime"] = {"removed_after_owners_stop": stopped}
        save(args.output / "result.json", result)


if __name__ == "__main__":
    main()
