#!/usr/bin/env python3
"""Capture actual native pilot inputs on a private headless X11/accessibility bus.

Creates its own private session bus. Requires existing system weston, Xwayland, xdotool,
gdbus and ImageMagick import tools. No installs, product mutation API, seeded GUI
state, acceptance verdict or synthetic dispatch/accessible objects. The caller
supplies explicit input steps; raw observations remain separate from evaluation.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import select
import subprocess
import tarfile
import tempfile
import time

from workflow_delivery_pilot_x11 import close_window

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "research/process-quality/evidence/wdq-c01-current/native-fixture.tar.gz"
ACCESSIBLE = "org.a11y.atspi.Accessible"
OBJECT = re.compile(r"\('(:[\d.]+)', (?:objectpath )?'([^']+)'\)")


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def files(root):
    return {str(path.relative_to(root)): digest(path)
            for path in sorted(root.rglob("*")) if path.is_file()}


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


class Capture:
    def __init__(self, args):
        self.args = args
        self.out = args.output.resolve()
        self.out.mkdir(parents=True, exist_ok=False)
        self.scratch = Path(tempfile.mkdtemp(prefix="datum-wdq-native-"))
        self.env = dict(os.environ)
        self.env.pop("DISPLAY", None)
        self.env.pop("WAYLAND_DISPLAY", None)
        for key, name in [("XDG_RUNTIME_DIR", "runtime"), ("XDG_CONFIG_HOME", "config"),
                          ("XDG_CACHE_HOME", "cache"), ("XDG_DATA_HOME", "data")]:
            directory = self.scratch / name
            directory.mkdir(mode=0o700)
            self.env[key] = str(directory)
        self.env.update(EDA_CLI_BIN=str(args.cli.resolve()), DATUM_ACTION_EVIDENCE="1",
                        WINIT_UNIX_BACKEND="x11", GDK_BACKEND="x11")
        self.processes = []
        self.logs = []
        self.actions = []
        self.gui = None
        self.launch_count = 0
        self.project = self.scratch / "project"
        self.project.mkdir()
        with tarfile.open(FIXTURE) as archive:
            archive.extractall(self.project, filter="data")
        self.before = files(self.project)
        save(self.out / "source-before.json", self.before)

    def run(self, *args, check=True):
        result = subprocess.run(list(map(str, args)), env=self.env,
                                capture_output=True, text=True, timeout=20)
        if check and result.returncode:
            raise RuntimeError(f"{args}: {result.stderr or result.stdout}")
        return result.stdout.strip()

    def start(self, command, log, **kwargs):
        stream = (self.out / log).open("w")
        self.logs.append(stream)
        process = subprocess.Popen(command, env=self.env, stdout=stream, stderr=stream,
                                   **kwargs)
        self.processes.append(process)
        return process

    def setup(self):
        # Never mutate an inherited bus's activation environment, even when the
        # caller accidentally launches this tool from their desktop terminal.
        read_bus, write_bus = os.pipe()
        try:
            self.start(["dbus-daemon", "--session", "--nofork", f"--print-address={write_bus}",
                        "--address=unix:path=" + str(self.scratch / "session-bus")],
                       "session-bus.log", pass_fds=(write_bus,))
            os.close(write_bus)
            write_bus = None
            if not select.select([read_bus], [], [], 20)[0]:
                raise RuntimeError("private session bus did not start")
            address = os.read(read_bus, 1024).decode().strip()
            if not address.startswith("unix:path=" + str(self.scratch / "session-bus")):
                raise RuntimeError("unexpected private session bus address")
            self.env["DBUS_SESSION_BUS_ADDRESS"] = address
        finally:
            os.close(read_bus)
            if write_bus is not None:
                os.close(write_bus)
        self.env["WAYLAND_DISPLAY"] = "wayland-wdq-pilot"
        self.start(["weston", "--backend=headless", "--renderer=pixman", "--no-config",
                    "--socket=wayland-wdq-pilot", "--width=1600", "--height=1000",
                    "--idle-time=0"], "weston.log")
        socket = Path(self.env["XDG_RUNTIME_DIR"]) / self.env["WAYLAND_DISPLAY"]
        self.until(lambda: socket.exists(), "private Weston socket")
        read_fd, write_fd = os.pipe()
        try:
            self.start(["Xwayland", "-displayfd", str(write_fd), "-geometry", "1600x1000",
                        "-nolisten", "tcp", "-ac", "-glamor", "off"], "xwayland.log",
                       pass_fds=(write_fd,))
            os.close(write_fd)
            write_fd = None
            if not select.select([read_fd], [], [], 20)[0]:
                raise RuntimeError("private Xwayland did not allocate a display")
            number = os.read(read_fd, 32).decode().strip()
            if not number.isdigit():
                raise RuntimeError("invalid allocated X display")
            self.env["DISPLAY"] = ":" + number
        finally:
            os.close(read_fd)
            if write_fd is not None:
                os.close(write_fd)
        # Keep WAYLAND_DISPLAY for the server, but force the GUI onto X11 below.
        # Explicitly update THIS session bus activation environment;
        # otherwise at-spi-bus-launcher inherits the desktop runtime socket path.
        activation = {key: self.env[key] for key in (
            "XDG_RUNTIME_DIR", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME",
            "DISPLAY", "WAYLAND_DISPLAY")}
        self.run("gdbus", "call", "--session", "--dest", "org.freedesktop.DBus",
                 "--object-path", "/org/freedesktop/DBus", "--method",
                 "org.freedesktop.DBus.UpdateActivationEnvironment", repr(activation))
        address = self.run("gdbus", "call", "--session", "--dest", "org.a11y.Bus",
                           "--object-path", "/org/a11y/bus", "--method", "org.a11y.Bus.GetAddress")
        self.bus = re.search(r"'([^']+)'", address).group(1)
        if not self.bus.startswith("unix:path=" + self.env["XDG_RUNTIME_DIR"] + "/"):
            raise RuntimeError("accessibility bus escaped this run's private runtime directory")
        # Monitor the owned bus, including reconnects on normal close/reopen.
        # gdbus monitor requires a destination; without one it exits immediately
        # and produces an error log instead of native accessibility events.
        self.monitor = self.start(["dbus-monitor", "--address", self.bus, "type='signal'"],
                                  "accessibility-events.log")
        time.sleep(0.1)
        if self.monitor.poll() is not None:
            raise RuntimeError("native accessibility event monitor exited during setup")
        save(self.out / "run-identity.json", {
            "gui_sha256": digest(self.args.gui.resolve()), "cli_sha256": digest(self.args.cli.resolve()),
            "fixture_sha256": digest(FIXTURE), "scratch": str(self.scratch),
            "display": self.env["DISPLAY"], "bus": self.bus,
            "source_commit": self.run("git", "-C", ROOT, "rev-parse", "HEAD"),
            "window_size": [int(part) for part in self.args.window_size.split("x")], "scale": 1,
        })
        self.launch()
        save(self.out / "preferences-before.json", files(Path(self.env["XDG_CONFIG_HOME"])))

    def until(self, predicate, description):
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            result = predicate()
            if result:
                return result
            time.sleep(0.1)
        raise RuntimeError("timeout waiting for " + description)

    def launch(self):
        self.launch_count += 1
        wayland = self.env.pop("WAYLAND_DISPLAY")
        try:
            self.gui = self.start([str(self.args.gui.resolve()), "--project-root", str(self.project),
                                   "--window-size", self.args.window_size], f"gui-{self.launch_count}.log")
        finally:
            self.env["WAYLAND_DISPLAY"] = wayland
        def window():
            if self.gui.poll() is not None:
                raise RuntimeError(f"GUI exited {self.gui.returncode} before mapping")
            output = self.run("xdotool", "search", "--onlyvisible", "--pid", self.gui.pid, check=False)
            return output.splitlines()[0] if output else None
        self.window = self.until(window, "native GUI window")
        self.run("xdotool", "windowfocus", "--sync", self.window)
        time.sleep(2)

    def dbus(self, bus, path, method, *args, check=True):
        return self.run("gdbus", "call", "--address", self.bus, "--dest", bus,
                        "--object-path", path, "--method", method, *args, check=check)

    def accessible_tree(self):
        roots = self.dbus("org.a11y.atspi.Registry", "/org/a11y/atspi/accessible/root",
                          ACCESSIBLE + ".GetChildren")
        nodes = []
        queue = []
        for bus, path in OBJECT.findall(roots):
            pid = self.dbus("org.freedesktop.DBus", "/org/freedesktop/DBus",
                            "org.freedesktop.DBus.GetConnectionUnixProcessID", bus)
            if str(self.gui.pid) in re.findall(r"\d+", pid):
                queue.append((bus, path))
        visited = set()
        while queue:
            bus, path = queue.pop(0)
            if (bus, path) in visited:
                continue
            visited.add((bus, path))
            if len(visited) > 256:
                raise RuntimeError("unexpected accessibility tree size")
            properties = self.dbus(bus, path, "org.freedesktop.DBus.Properties.GetAll", ACCESSIBLE)
            states = self.dbus(bus, path, ACCESSIBLE + ".GetState")
            children = self.dbus(bus, path, ACCESSIBLE + ".GetChildren")
            nodes.append({"bus": bus, "path": path, "properties": properties,
                          "states": states, "children": children})
            queue.extend(OBJECT.findall(children))
        return nodes

    def step(self, action):
        if self.monitor.poll() is not None:
            raise RuntimeError("native accessibility event monitor stopped during capture")
        kind = action[0]
        if kind == "click":
            self.run("xdotool", "mousemove", "--window", self.window, *action[1:3], "click", "1")
        elif kind == "move":
            self.run("xdotool", "mousemove", "--window", self.window, *action[1:3])
        elif kind == "key":
            self.run("xdotool", "key", "--clearmodifiers", *action[1:])
        elif kind == "focus-window":
            windows = self.run("xdotool", "search", "--onlyvisible", "--pid", self.gui.pid).splitlines()
            matches = [window for window in windows
                       if self.run("xdotool", "getwindowname", window) == action[1]]
            if len(matches) != 1:
                raise RuntimeError("exactly one owned native window must match the focus request")
            self.run("xdotool", "windowfocus", "--sync", matches[0])
        elif kind == "wheel":
            self.run("xdotool", "click", "--repeat", action[1], "--delay", "120", "4")
        elif kind == "capture":
            name = action[1]
            if not re.fullmatch(r"[a-zA-Z0-9_-]+", name):
                raise ValueError("safe capture name required")
            # Asynchronous dialogs may publish after the preceding input returns.
            # Query live semantics first, let its frame present, then capture it;
            # do not pair an old screenshot with a later accessibility tree.
            time.sleep(1)
            nodes = self.accessible_tree()
            time.sleep(0.25)
            self.run("import", "-window", self.window, self.out / (name + ".png"))
            windows = self.run("xdotool", "search", "--onlyvisible", "--pid", self.gui.pid).splitlines()
            window_records = []
            for index, window in enumerate(windows):
                title = self.run("xdotool", "getwindowname", window)
                filename = name + ".png" if window == self.window else f"{name}-window-{index}.png"
                if window != self.window:
                    self.run("import", "-window", window, self.out / filename)
                window_records.append({"window": window, "title": title, "capture": filename})
            save(self.out / (name + "-windows.json"), window_records)
            after_nodes = self.accessible_tree()
            if nodes != after_nodes:
                raise RuntimeError("accessibility state changed across native capture; rerun")
            save(self.out / (name + "-atspi.json"), nodes)
            records = []
            for line in (self.out / f"gui-{self.launch_count}.log").read_text().splitlines():
                if line.startswith("DATUM_ACTION_EVIDENCE "):
                    records.append(json.loads(line.removeprefix("DATUM_ACTION_EVIDENCE ")))
            if not records:
                raise RuntimeError("production evidence stream absent")
            # The append-only GUI log retains the complete production stream.
            # Each screenshot needs its latest state, not another cumulative copy.
            save(self.out / (name + "-state.json"),
                 [row for row in records if row["event"] == "state"][-1:])
        elif kind == "close":
            close_window(self.env["DISPLAY"], self.window)
            code = self.gui.wait(timeout=20)
            if code:
                raise RuntimeError(f"normal GUI close exited {code}")
        elif kind == "reopen":
            if self.gui.poll() is None:
                raise RuntimeError("close first")
            self.launch()
        else:
            raise ValueError("unknown native input: " + kind)
        self.actions.append({"action": action, "time_ns": time.time_ns(), "launch": self.launch_count})
        save(self.out / "inputs.json", self.actions)
        time.sleep(0.3)

    def finish(self):
        after = files(self.project)
        save(self.out / "source-after.json", after)
        save(self.out / "source-diff.json", {
            "changed_or_removed": [p for p, sha in self.before.items() if after.get(p) != sha],
            "new_paths": {p: sha for p, sha in after.items() if p not in self.before},
        })
        save(self.out / "preferences-after.json", files(Path(self.env["XDG_CONFIG_HOME"])))
        for process in reversed(self.processes):
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        for log in self.logs:
            log.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gui", type=Path, required=True)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--window-size", default="1280x768",
                        help="native size; named pilot scenarios require 1280x768")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--actions-json")
    group.add_argument("--scenario", choices=[f"PILOT-S0{i}" for i in range(1, 6)]
                       + ["PILOT-S03-pointer-regression"])
    args = parser.parse_args()
    if not re.fullmatch(r"[1-9][0-9]{2,3}x[1-9][0-9]{2,3}", args.window_size):
        parser.error("window size must be WIDTHxHEIGHT")
    if args.scenario and args.window_size != "1280x768":
        parser.error("named scenarios use reviewed 1280x768 coordinates")
    capture = Capture(args)
    try:
        capture.setup()
        if args.scenario:
            from workflow_delivery_pilot_scenarios import actions
            steps = actions(args.scenario)
        else:
            steps = json.loads(args.actions_json)
        save(capture.out / "requested-inputs.json", steps)
        for action in steps:
            capture.step(action)
        if capture.gui.poll() is None:
            capture.step(["close"])
    finally:
        capture.finish()


if __name__ == "__main__":
    main()
