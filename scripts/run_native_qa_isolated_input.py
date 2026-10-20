#!/usr/bin/env python3
"""Run a native QA child on an owned input display, retaining its Wayland GUI.

Reuses the installed-tool bootstrap from workflow_delivery_pilot_capture only.
No fixture, pilot scenario, product backend override or acceptance verdict.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import sys
import tempfile
import time

DISPLAY_KEY = "DATUM_QA_EXCLUSIVE_INPUT_DISPLAY"
OWNER_KEY = "DATUM_QA_EXCLUSIVE_INPUT_OWNER_PID"


def _live_xwayland(pid):
    try:
        proc = Path("/proc") / str(pid)
        return (proc.stat().st_uid == os.getuid()
                and (proc / "exe").resolve() == Path("/usr/bin/Xwayland").resolve())
    except (OSError, RuntimeError):
        return False


def validate_input_display(env, owner_check=None):
    display = env.get(DISPLAY_KEY)
    if display is None:
        return None
    owner = env.get(OWNER_KEY, "")
    if (not re.fullmatch(r":\d+", display) or env.get("DISPLAY") != display
            or not owner.isdigit() or not (owner_check or _live_xwayland)(int(owner))):
        raise ValueError("exclusive input display/owned Xwayland binding is invalid")
    return display


def focus_arguments(window, env, owner_check=None):
    mode = "windowfocus" if validate_input_display(env, owner_check) else "windowactivate"
    return ["xdotool", mode, "--sync", window]


def child_environment(original, display, owner_pid, authority):
    if not re.fullmatch(r":\d+", display) or display == original.get("DISPLAY"):
        raise ValueError("input display must be freshly allocated and distinct")
    env = dict(original)
    env.pop("WAYLAND_DISPLAY", None)
    env.pop("WAYLAND_SOCKET", None)
    env.update(DISPLAY=display, XAUTHORITY=str(authority))
    env[DISPLAY_KEY], env[OWNER_KEY] = display, str(owner_pid)
    # Keep the original short runtime parent for the inner engine socket.
    return env


def stop_owners(owners, errors):
    for owner in reversed(owners):
        process = owner["process"]
        try:
            if process and process.poll() is None:
                if owner["label"] == "runner":
                    process.send_signal(signal.SIGINT)  # let its own finally run
                else:
                    process.terminate()
                try:
                    process.wait(timeout=40 if owner["label"] == "runner" else 10)
                except subprocess.TimeoutExpired:
                    errors.append({"phase": owner["label"], "error": "stop timeout; owned PID killed"})
                    process.kill()
                    process.wait(timeout=5)
        except BaseException as error:
            errors.append({"phase": owner["label"], "error": str(error)})
        finally:
            try:
                owner["stopped"] = process is None or process.poll() is not None
            except BaseException as error:
                owner["stopped"] = False
                errors.append({"phase": owner["label"] + " stop verification", "error": str(error)})
            try:
                owner["stream"].close()
            except BaseException as error:
                errors.append({"phase": owner["label"] + " log close", "error": str(error)})


def inner_owners_stopped(receipt):
    if not isinstance(receipt, dict):
        return False
    stops, ipc = receipt.get("verified_owner_stops"), receipt.get("ipc_runtime")
    return (isinstance(stops, dict) and isinstance(ipc, dict)
            and all(stops.get(key) is True for key in ("gui", "daemons", "clipboard", "Weston"))
            and ipc.get("removed_after_owners_stop") is True)


def may_remove_runtime(owners, receipt):
    return all(owner.get("stopped") is True for owner in owners) and inner_owners_stopped(receipt)


def save(path, value):
    pending = path.with_suffix(".tmp")
    pending.write_text(json.dumps(value, indent=2) + "\n")
    pending.replace(path)


def executable_binding(process):
    exe = Path("/proc") / str(process.pid) / "exe"
    with exe.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"pid": process.pid, "exe": str(exe.readlink()), "sha256": digest}


def run(args):
    output = args.output
    if not output.is_absolute() or output.resolve() != output or output.exists():
        raise ValueError("a fresh canonical absolute output directory is required")
    if not args.child_result.is_absolute() or args.child_result.exists() or not args.command:
        raise ValueError("fresh absolute child result and explicit child argv required")
    output.mkdir()
    runtime = tempfile.TemporaryDirectory(prefix="nqi-", dir=os.environ["XDG_RUNTIME_DIR"])
    runtime._finalizer.detach()  # IPC survives any unverified owner stop
    ipc = Path(runtime.name)
    owners, errors = [], []
    result = {"status": "FAIL", "argv": args.command, "runtime": str(ipc),
              "runtime_mode": oct(ipc.stat().st_mode & 0o777),
              "limits": "Input transport only; no native/D1 acceptance inferred."}
    primary = None
    child_code = None

    def start(label, argv, env, **kwargs):
        owner = {"label": label, "argv": argv, "process": None,
                 "stream": (output / (label + ".log")).open("w")}
        owners.append(owner)
        owner["process"] = subprocess.Popen(argv, env=env, stdout=owner["stream"],
                                             stderr=subprocess.STDOUT, **kwargs)
        return owner["process"]

    try:
        env = dict(os.environ, XDG_RUNTIME_DIR=runtime.name)
        for key in ("DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_SOCKET", DISPLAY_KEY, OWNER_KEY):
            env.pop(key, None)
        outer = start("outer-weston", ["/usr/bin/weston", "--backend=headless", "--renderer=gl",
                      "--shell=kiosk-shell.so", "--no-config", "--width=1600", "--height=1000",
                      "--scale=1", "--idle-time=0", "--socket=datum-qa-input",
                      "--log=" + str(output / "outer-weston-detail.log")], env)
        deadline = time.monotonic() + 20
        while not (ipc / "datum-qa-input").exists():
            if outer.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError("owned headless Weston did not become ready")
            time.sleep(0.05)
        result["outer_binding"] = executable_binding(outer)
        read_fd, write_fd = os.pipe()
        try:
            xenv = dict(env, WAYLAND_DISPLAY="datum-qa-input")
            server = start("xwayland", ["/usr/bin/Xwayland", "-displayfd", str(write_fd),
                           "-geometry", "1600x1000", "-nolisten", "tcp", "-ac", "-noreset"],
                           xenv, pass_fds=(write_fd,))
            os.close(write_fd)
            write_fd = None
            if not select.select([read_fd], [], [], 20)[0]:
                raise RuntimeError("owned Xwayland display readiness timeout")
            number = os.read(read_fd, 32).decode().strip()
            if not number.isdigit() or server.poll() is not None:
                raise RuntimeError("invalid owned Xwayland display receipt")
        finally:
            os.close(read_fd)
            if write_fd is not None:
                os.close(write_fd)
        result["xwayland_binding"] = executable_binding(server)
        authority = ipc / "empty-Xauthority"
        authority.touch(mode=0o600)
        # Operational authorization files are never read or hashed into receipts.
        cenv = child_environment(os.environ, ":" + number, server.pid, authority)
        validate_input_display(cenv)
        result["display"] = cenv["DISPLAY"]
        result["inherited_display"] = os.environ.get("DISPLAY")
        result["inner_runtime_parent"] = cenv["XDG_RUNTIME_DIR"]
        save(output / "transport.json", result)
        child = start("runner", args.command, cenv)
        child_code = child.wait()
        result["child_exit_code"] = child_code
        if child_code:
            result["error"] = "QA child failed; its original result is preserved"
    except BaseException as error:
        primary = error
        result["error"] = {"type": type(error).__name__, "message": str(error)}
    finally:
        stop_owners(owners, errors)
        receipt = {}
        try:
            receipt = json.loads(args.child_result.read_text())
            result["child_owner_stops_verified"] = inner_owners_stopped(receipt)
        except BaseException as error:
            errors.append({"phase": "child owner receipt", "error": str(error)})
        if may_remove_runtime(owners, receipt):
            try:
                runtime.cleanup()
            except BaseException as error:
                errors.append({"phase": "outer IPC removal", "error": str(error)})
        else:
            errors.append({"phase": "IPC retained", "error": "one or more owner stops unverified"})
        result["runtime_removed_after_owners_stop"] = not ipc.exists()
        if may_remove_runtime(owners, receipt) and ipc.exists():
            errors.append({"phase": "outer IPC removal", "error": "owned runtime still exists"})
        result["owners"] = [{"label": o["label"], "argv": o["argv"],
                             "pid": o["process"].pid if o["process"] else None,
                             "stopped": o["stopped"]} for o in owners]
        result["cleanup_errors"] = errors
        if child_code == 0 and not errors:
            result["status"] = "CAPTURE_ONLY; child and owned transport stopped, independent review pending"
        try:
            save(output / "transport.json", result)
        except BaseException as error:
            if primary is None:
                primary = error
            print("transport receipt persistence failed: " + str(error), file=sys.stderr)
    if primary is not None:
        raise primary.with_traceback(primary.__traceback__)
    return 0 if child_code == 0 and not errors else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--child-result", type=Path, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    raise SystemExit(run(args))


if __name__ == "__main__":
    main()
