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


def _live_xwayland(pid, display, *, proc_root=Path("/proc")):
    try:
        proc = proc_root / str(pid)
        if (proc.stat().st_uid != os.getuid()
                or (proc / "exe").resolve() != Path("/usr/bin/Xwayland").resolve()):
            return False
        # -displayfd servers need not publish a .Xn-lock. Bind the allocated
        # display's listening socket inode to a descriptor held by this PID.
        names = {f"/tmp/.X11-unix/X{display[1:]}", f"@/tmp/.X11-unix/X{display[1:]}"}
        inodes = {fields[6] for line in (proc_root / "net/unix").read_text().splitlines()
                  if len(fields := line.split()) == 8 and fields[7] in names
                  and int(fields[3], 16) & 0x10000}
        return any(str(fd.readlink()) in {f"socket:[{inode}]" for inode in inodes}
                   for fd in (proc / "fd").iterdir())
    except (OSError, RuntimeError, ValueError):
        return False


def validate_input_display(env, owner_check=None):
    display = env.get(DISPLAY_KEY)
    if display is None:
        raise ValueError("owned input display required; use run_native_qa_isolated_input.py")
    owner = env.get(OWNER_KEY, "")
    if (not re.fullmatch(r":\d+", display) or env.get("DISPLAY") != display
            or not owner.isdigit() or not (owner_check or _live_xwayland)(int(owner), display)):
        raise ValueError("exclusive input display/owned Xwayland binding is invalid")
    return display


def focus_arguments(window, env, owner_check=None):
    validate_input_display(env, owner_check)
    return ["xdotool", "windowfocus", "--sync", window]


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


def private_bus_environment(original, runtime):
    """Activated services must not inherit access to the owner's GUI session."""
    env = dict(original)
    for key in ("WAYLAND_SOCKET", "XAUTHORITY", "DBUS_SESSION_BUS_ADDRESS",
                "DBUS_SESSION_BUS_PID", "DESKTOP_STARTUP_ID", "XDG_ACTIVATION_TOKEN"):
        env.pop(key, None)
    env.update(DATUM_QA_SCENARIO_PRIVATE_BUS=str(runtime), XDG_RUNTIME_DIR=str(runtime),
               DISPLAY="datum-qa-disabled", WAYLAND_DISPLAY="datum-qa-disabled",
               XDG_CURRENT_DESKTOP="", QT_QPA_PLATFORM="offscreen", GTK_USE_PORTAL="0")
    return env


def live_group_members(group, proc_root=Path("/proc")):
    members = []
    for entry in proc_root.iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            fields = (entry / "stat").read_text().rsplit(")", 1)[1].split()
            if int(fields[2]) == group and fields[0] not in ("Z", "X"):
                members.append(int(entry.name))
        except (FileNotFoundError, ProcessLookupError):
            continue
    return members


def stop_bus_group(process):
    for sig, seconds in ((signal.SIGTERM, 6), (signal.SIGKILL, 3)):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            pass
        until = time.monotonic() + seconds
        while live_group_members(process.pid):
            if time.monotonic() >= until:
                break
            time.sleep(.05)
        else:
            process.wait(timeout=1)
            return
    raise RuntimeError(f"Private bus processes still live; runtime retained: {live_group_members(process.pid)}")


def run_private_bus(argv, output):
    # A private activation catalogue exposes only accessibility. Desktop portals,
    # wallets and notifications are unnecessary for this scenario and cannot be
    # activated here, even if a browser or shell probes their service names.
    runtime = tempfile.TemporaryDirectory(prefix="nqb-", dir=os.environ["XDG_RUNTIME_DIR"])
    runtime._finalizer.detach()
    root = Path(runtime.name)
    services = root / "services"
    services.mkdir()
    (services / "org.a11y.Bus.service").symlink_to("/usr/share/dbus-1/services/org.a11y.Bus.service")
    config = root / "bus.conf"
    config.write_text(f'''<busconfig><type>session</type><auth>EXTERNAL</auth>
<listen>unix:tmpdir={root}</listen><servicedir>{services}</servicedir>
<policy context="default"><allow own="*"/><allow send_destination="*"/>
<allow receive_sender="*"/></policy></busconfig>''')
    process = None
    previous = {}
    report = {"runtime": str(root), "activation_services": ["org.a11y.Bus"],
              "desktop_environment_removed": True, "owned_group_stopped": False}
    def interrupted(signum, _frame):
        raise KeyboardInterrupt(f"Private GUI scenario interrupted by signal {signum}")
    try:
        for sig in (signal.SIGTERM, signal.SIGINT):
            previous[sig] = signal.signal(sig, interrupted)
        process = subprocess.Popen(["dbus-run-session", f"--config-file={config}", "--", *argv],
            env=private_bus_environment(os.environ, root), start_new_session=True)
        report["process_group"] = process.pid
        report["exit_code"] = process.wait()
        return report["exit_code"]
    finally:
        try:
            for sig in previous:
                signal.signal(sig, signal.SIG_IGN)
            if process is not None:
                stop_bus_group(process)
            report["owned_group_stopped"] = True
            runtime.cleanup()
        finally:
            report["runtime_removed"] = not root.exists()
            if output.is_dir():
                save(output / "private-bus.json", report)
            for sig, handler in previous.items():
                signal.signal(sig, handler)


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
    width, height = getattr(args, "input_width", 1600), getattr(args, "input_height", 1000)
    if any(type(value) is not int or value <= 0 for value in (width, height)):
        raise ValueError("input dimensions must be positive integers")
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
              "requested_input_dimensions": {"width": width, "height": height},
              "outer_capture": {
                  "debug_requested": bool(getattr(args, "outer_capture", False)),
                  "runtime": str(ipc), "socket": "datum-qa-input",
                  "limits": "Opt-in --debug exposes capture and debug globals to clients able "
                            "to connect. Mode 0700 does not exclude same-UID peers. Private "
                            "headless composed output, not a physical desktop witness."},
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
        outer_argv = ["/usr/bin/weston", "--backend=headless", "--renderer=gl",
                      "--shell=kiosk-shell.so", "--no-config", f"--width={width}", f"--height={height}",
                      "--scale=1", "--idle-time=0", "--socket=datum-qa-input",
                      "--log=" + str(output / "outer-weston-detail.log")]
        if result["outer_capture"]["debug_requested"]:
            outer_argv.append("--debug")
        result["outer_capture"]["argv"] = list(outer_argv)
        outer = start("outer-weston", outer_argv, env)
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
                           "-geometry", f"{width}x{height}", "-nolisten", "tcp", "-ac", "-noreset"],
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
        result["display"] = cenv["DISPLAY"]
        validate_input_display(cenv)
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
        child_started = any(o["label"] == "runner" and o["process"] is not None for o in owners)
        result["child_started"] = child_started
        if child_started:
            try:
                receipt = json.loads(args.child_result.read_text())
                result["child_owner_stops_verified"] = inner_owners_stopped(receipt)
            except BaseException as error:
                errors.append({"phase": "child owner receipt", "error": str(error)})
        removable = (may_remove_runtime(owners, receipt) if child_started
                     else all(o.get("stopped") is True for o in owners))
        if removable:
            try:
                runtime.cleanup()
            except BaseException as error:
                errors.append({"phase": "outer IPC removal", "error": str(error)})
        else:
            errors.append({"phase": "IPC retained", "error": "one or more owner stops unverified"})
        result["runtime_removed_after_owners_stop"] = not ipc.exists()
        if removable and ipc.exists():
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
    parser.add_argument("--child-result", type=Path)
    parser.add_argument("--scenario", choices=["preferences"],
                        help="ordinary menu, wide/narrow native and HTML captures, edit/restart/reset")
    parser.add_argument("--binary-dir", type=Path,
                        help="candidate GUI/CLI/daemon directory (scenario default: target/release)")
    parser.add_argument("--input-width", type=int, default=1600, help="owned input display width")
    parser.add_argument("--input-height", type=int, default=1000, help="owned input display height")
    parser.add_argument("--outer-capture", action="store_true",
                        help="Expose capture and debug globals on the owned private outer compositor")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if args.scenario:
        if args.command or args.child_result:
            parser.error("--scenario cannot be combined with a child command or --child-result")
        repo = Path(__file__).resolve().parents[1]
        binary_dir = (args.binary_dir or repo / "target/release").resolve()
        for binary in ("datum-gui", "datum-eda", "eda-engine-daemon"):
            if not (binary_dir / binary).is_file():
                parser.error(f"missing candidate: {binary_dir / binary}; build it with the guarded Cargo runner")
        args.output = args.output.resolve()
        if args.output.exists():
            parser.error("scenario output must be a fresh directory; existing evidence is never overwritten")
        args.child_result = args.output / "result.json"
        args.command = ["/usr/bin/python3", str(repo / "scripts/native_preferences_workflow.py"),
                        "--output", str(args.output), "--binary-dir", str(binary_dir)]
        os.environ.setdefault("LIBGL_ALWAYS_SOFTWARE", "1")
        lavapipe = Path("/usr/share/vulkan/icd.d/lvp_icd.json")
        if lavapipe.exists():
            os.environ.setdefault("VK_ICD_FILENAMES", str(lavapipe))
        if os.environ.get("DATUM_QA_SCENARIO_PRIVATE_BUS") != os.environ.get("XDG_RUNTIME_DIR"):
            raise SystemExit(run_private_bus([sys.executable,
                             str(Path(__file__).resolve()), *sys.argv[1:]], args.output))
        if (os.environ.get("DISPLAY") != "datum-qa-disabled"
                or os.environ.get("WAYLAND_DISPLAY") != "datum-qa-disabled"):
            parser.error("private bus activation environment is not isolated")
    elif args.child_result is None or not args.command or args.binary_dir:
        parser.error("provide --scenario, or --child-result and a child command")
    code = run(args)
    if args.scenario:
        print(f"Evidence: {args.output}\nBehavior/capture result: {args.child_result}\n"
              "Inspect native/{wide,narrow}/units.png and reference/prototype-units-{wide,narrow}.png.\n"
              "Visual comparison remains pending; capture success is not prototype conformance.")
    raise SystemExit(code)


if __name__ == "__main__":
    main()
