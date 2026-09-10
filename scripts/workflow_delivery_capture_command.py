"""Run one explicitly owned fixture command and retain raw observations.

No expected-result evaluation, fixture repair, trust installation or acceptance.
The higher-level case recipe supplies its ownership nonce and sanitized environment.
"""

import math
import os
from pathlib import Path
import signal
import subprocess
import time
import uuid

from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_capture_interrupt import interrupt_after_call, validate_trigger

ENV_KEYS = {"PATH", "HOME", "XDG_CONFIG_HOME", "LANG", "LC_ALL", "TZ",
            "GIT_CONFIG_NOSYSTEM", "GIT_CONFIG_GLOBAL", "PYTHONDONTWRITEBYTECODE", "DATUM_WDQ_OBSERVE_PATH",
            "TERM", "NO_COLOR", "CLICOLOR", "FORCE_COLOR"}


def save(path, value):
    with path.open("xb") as stream:
        stream.write(canonical_json(value))


def stop_owned(process):
    if process.poll() is None:
        # start_new_session made this still-live child its own process-group leader.
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)


def capture_command(root, output, command, environment, *, fixture_nonce, case_id,
                    surface, untracked_roots, timeout, interrupt_on_call=None):
    root, output = Path(root).resolve(strict=True), Path(output).resolve()
    marker = root / ".git/datum-wdq-capture-owner"
    if (not fixture_nonce or not (root / ".git").is_dir() or marker.is_symlink()
            or not marker.is_file() or marker.read_text().strip() != fixture_nonce):
        raise ValueError("explicit owned fixture nonce required; no shared-worktree fallback")
    if output.is_relative_to(root) or root.is_relative_to(output):
        raise ValueError("capture output must be separate from protected fixture")
    if (type(command) is not list or not command
            or any(type(arg) is not str or not arg or "\0" in arg for arg in command)
            or not Path(command[0]).is_absolute()):
        raise ValueError("explicit absolute executable and string argv required")
    if (type(environment) is not dict or not set(environment) <= ENV_KEYS
            or any(type(value) is not str or "\0" in value for value in environment.values())
            or environment.get("GIT_CONFIG_NOSYSTEM") != "1"
            or environment.get("GIT_CONFIG_GLOBAL") != os.devnull
            or not {"PATH", "HOME", "XDG_CONFIG_HOME"} <= set(environment)):
        raise ValueError("explicit sanitized environment with isolated Git configuration required")
    if type(timeout) not in (int, float) or not math.isfinite(timeout) or timeout <= 0:
        raise ValueError("finite positive timeout required")
    if not case_id or surface not in ("C", "R", "S", "H"):
        raise ValueError("named case and actual entry surface required")
    if interrupt_on_call is not None:
        validate_trigger(interrupt_on_call)
    output.mkdir(exist_ok=False)
    invocation = str(uuid.uuid4())
    save(output / "invocation.json", {
        "invocation_id": invocation, "case_id": case_id, "surface": surface,
        "argv": command, "cwd": str(root), "environment": environment,
        "timeout_seconds": timeout, "started_ns": time.time_ns(),
        "fixture_nonce": fixture_nonce,
        "interrupt_on_call": interrupt_on_call,
    })
    before = protected_state(root, untracked_roots)
    save(output / "before.json", before)
    process = None
    result = {"invocation_id": invocation, "kind": "not_started", "returncode": None,
              "pid": None, "acceptance_asserted": False,
              "descendant_process_closure_verified": False}
    try:
        with (output / "stdout.bin").open("xb") as stdout, (output / "stderr.bin").open("xb") as stderr:
            try:
                process = subprocess.Popen(command, cwd=root, env=environment,
                    stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr, start_new_session=True)
                result.update(pid=process.pid, kind="exited")
                save(output / "started.json", {"pid": process.pid, "invocation_id": invocation,
                                              "observed_ns": time.time_ns()})
                try:
                    if interrupt_on_call is None:
                        process.wait(timeout=timeout)
                    else:
                        interruption = interrupt_after_call(process, output, interrupt_on_call, timeout)
                        if interruption is not None:
                            result["kind"] = ("interrupted" if process.returncode < 0
                                              else "interrupt_requested_but_exited")
                            save(output / "interruption.json", interruption)
                except subprocess.TimeoutExpired:
                    result["kind"] = "timeout"
                    stop_owned(process)
            except OSError as error:
                result.update(kind="launch_or_process_error", error_type=type(error).__name__,
                              errno=error.errno, detail=str(error))
    except BaseException as error:
        result.update(kind="harness_exception", error_type=type(error).__name__)
        raise
    finally:
        if process is not None:
            stop_owned(process)
            result["returncode"] = process.returncode
        result["finished_ns"] = time.time_ns()
        try:
            after = protected_state(root, untracked_roots)
            save(output / "after.json", after)
            result["changed_state_fields"] = sorted(key for key in before if before[key] != after[key])
            result["changed_files"] = sorted(path for path in set(before["files"]) | set(after["files"])
                if before["files"].get(path) != after["files"].get(path))
        except Exception as error:
            result["after_capture_error"] = {"type": type(error).__name__, "detail": str(error)}
        result["artifacts"] = [{"path": path.name, "sha256": sha256(path.read_bytes())}
                               for path in sorted(output.iterdir()) if path.is_file()]
        # No verdict: expected refusal, successful native proof and independent
        # review require the separately reviewed case/assertion machinery.
        save(output / "result.json", result)
    return result
