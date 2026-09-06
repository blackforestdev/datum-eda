#!/usr/bin/env python3
"""Capture actual native pilot inputs on a private headless X11/accessibility bus.

Run under dbus-run-session. Requires existing system weston, Xwayland, xdotool,
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
        address = self.run("gdbus", "call", "--session", "--dest", "org.a11y.Bus",
                           "--object-path", "/org/a11y/bus", "--method", "org.a11y.Bus.GetAddress")
        self.bus = re.search(r"'([^']+)'", address).group(1)
        self.start(["gdbus", "monitor", "--address", self.bus], "accessibility-events.log")
        save(self.out / "run-identity.json", {
            "gui_sha256": digest(self.args.gui.resolve()), "cli_sha256": digest(self.args.cli.resolve()),
            "fixture_sha256": digest(FIXTURE), "scratch": str(self.scratch),
            "display": self.env["DISPLAY"], "bus": self.bus,
            "source_commit": self.run("git", "-C", ROOT, "rev-parse", "HEAD"),
            "window_size": [1280, 768], "scale": 1,
        })
        self.launch()

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
                                   "--window-size", "1280x768"], f"gui-{self.launch_count}.log")
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
        kind = action[0]
        if kind == "click":
            self.run("xdotool", "mousemove", "--window", self.window, *action[1:3], "click", "1")
        elif kind == "move":
            self.run("xdotool", "mousemove", "--window", self.window, *action[1:3])
        elif kind == "key":
            self.run("xdotool", "key", "--clearmodifiers", *action[1:])
        elif kind == "wheel":
            self.run("xdotool", "click", "--repeat", action[1], "--delay", "120", "4")
        elif kind == "capture":
            name = action[1]
            if not re.fullmatch(r"[a-zA-Z0-9_-]+", name):
                raise ValueError("safe capture name required")
            self.run("import", "-window", self.window, self.out / (name + ".png"))
            save(self.out / (name + "-atspi.json"), self.accessible_tree())
            records = []
            for line in (self.out / f"gui-{self.launch_count}.log").read_text().splitlines():
                if line.startswith("DATUM_ACTION_EVIDENCE "):
                    records.append(json.loads(line.removeprefix("DATUM_ACTION_EVIDENCE ")))
            if not records:
                raise RuntimeError("production evidence stream absent")
            save(self.out / (name + "-state.json"), records)
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
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--actions-json")
    group.add_argument("--scenario", choices=[f"PILOT-S0{i}" for i in range(1, 6)])
    args = parser.parse_args()
    if not os.environ.get("DBUS_SESSION_BUS_ADDRESS"):
        parser.error("run inside a private dbus-run-session")
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
