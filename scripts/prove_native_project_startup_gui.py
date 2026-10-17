#!/usr/bin/env python3
"""Bounded native-Wayland startup QA through an owned Weston input envelope.

Run with /usr/bin/python3 on the existing desktop. No Cargo, genesis helpers,
private UI actions, owner Projects or HOME substitution participate.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time
import traceback

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib  # noqa: E402


REPO = Path(__file__).resolve().parents[1]
OWNED = REPO / ".datum/native-project-startup-repair"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class StartupQA:
    def __init__(self, root: Path):
        self.root = root
        self.process = None
        self.app_log = None
        self.actions = []
        self.observations = {}
        self.previous_focus = self.command("xdotool", "getwindowfocus").strip()
        self.runtime = tempfile.TemporaryDirectory(prefix="npd-wl-", dir="/tmp")
        self.compositor = None
        self.host_window = None
        self.env = dict(os.environ)
        self.env.update({
            "XDG_RUNTIME_DIR": self.runtime.name,
            "WAYLAND_DISPLAY": "datum-startup-qa",
            "XDG_CONFIG_HOME": str(root / "config"),
            "XDG_DATA_HOME": str(root / "data"),
            "XDG_CACHE_HOME": str(root / "cache"),
            "DATUM_GUI_PREFERENCES_PATH": str(root / "config/gui-preferences.json"),
            "HISTFILE": str(root / "terminal-history"),
            "DATUM_GUI_VERBOSE_LOG": "1",
        })
        for key in ["DISPLAY", "WAYLAND_SOCKET", "WINIT_UNIX_BACKEND", "DATUM_GPU_DIAGNOSTIC_BACKEND"]:
            self.env.pop(key, None)
        self.binary_hashes = {
            name: digest(REPO / "target/debug" / name)
            for name in ["datum-gui", "datum-eda", "eda-engine-daemon"]
        }

    def command(self, *args: str, timeout: float = 15, check: bool = True) -> str:
        result = subprocess.run(args, text=True, capture_output=True, timeout=timeout)
        if hasattr(self, "actions"):
            self.actions.append({"argv": list(args), "exit": result.returncode,
                                 "stdout": result.stdout, "stderr": result.stderr})
        if check and result.returncode:
            raise RuntimeError(f"command failed {args!r}: {result.stderr}")
        return result.stdout

    def wait(self, label, predicate, timeout=20):
        until = time.monotonic() + timeout
        last = None
        while time.monotonic() < until:
            if self.compositor and self.compositor.poll() is not None:
                raise RuntimeError(f"owned compositor exited: {label}")
            if self.process and self.process.poll() is not None:
                raise RuntimeError(f"App exited {self.process.returncode}: {label}")
            try:
                value = predicate()
                if value:
                    return value
            except (RuntimeError, GLib.Error) as error:
                last = str(error)
            time.sleep(0.1)
        raise RuntimeError(f"timed out: {label}; last={last}")

    def window(self, title=None):
        if title:
            label = title.split(" — ")[0]
            return self.host_window if any(n.get_name() == label and n.get_role_name() == "dialog"
                                           for n in self.nodes()) else None
        return self.host_window

    def focus(self, window):
        assert self.compositor.poll() is None and window == self.host_window
        self.command("xdotool", "windowactivate", "--sync", window)

    def start_envelope(self):
        env = dict(os.environ, XDG_RUNTIME_DIR=self.runtime.name)
        self.compositor_log = (self.root / "weston-process.log").open("w")
        args = ["weston", "--backend=x11", "--renderer=gl", "--shell=kiosk-shell.so",
                "--no-config", "--debug", "--width=1344", "--height=806", "--scale=1",
                "--idle-time=0", "--socket=datum-startup-qa", "--log=" + str(self.root / "weston.log")]
        self.compositor = subprocess.Popen(args, env=env, stdout=self.compositor_log,
                                           stderr=subprocess.STDOUT)
        self.actions.append({"owned_compositor_pid": self.compositor.pid, "argv": args})
        self.wait("private Wayland socket", lambda: Path(self.runtime.name, "datum-startup-qa").exists())
        text = (self.root / "weston.log").read_text()
        match = re.search(r"x11 output .*window id (\d+)", text)
        assert match, "host XID must come from this owned live compositor's log"
        self.host_window = match.group(1)
        metadata = self.command("xprop", "-id", self.host_window, "WM_CLASS", "_NET_WM_PID")
        assert "Weston Compositor" in metadata
        self.observations["input_envelope"] = {"pid": self.compositor.pid,
            "xid_from_owned_log": self.host_window, "metadata": metadata,
            "limit": "X11 input to owned Weston; Datum itself native Wayland/Vulkan, not KWin workflow qualification"}

    def close_envelope(self):
        if self.compositor:
            if self.compositor.poll() is None:
                self.compositor.terminate()
                self.compositor.wait(timeout=10)
            self.compositor_log.close()
        self.runtime.cleanup()

    def key(self, window, keys, repeat=1):
        self.focus(window)
        self.command("xdotool", "key", "--clearmodifiers", "--repeat", str(repeat),
                     "--repeat-delay", "20", keys)
        time.sleep(0.12)

    def text(self, window, value):
        self.focus(window)
        self.command("xdotool", "type", "--clearmodifiers", "--delay", "4", "--", value)
        time.sleep(0.15)

    def nodes(self):
        desktop = Atspi.get_desktop(0)
        application = None
        for i in range(desktop.get_child_count()):
            item = desktop.get_child_at_index(i)
            if item and item.get_process_id() == self.process.pid:
                application = item
                break
        if application is None:
            return []
        result, pending = [], [application]
        while pending:
            node = pending.pop()
            if node is None:
                continue
            result.append(node)
            assert len(result) < 3000, "bounded own-App accessibility tree"
            pending.extend(node.get_child_at_index(i) for i in range(node.get_child_count()))
        return result

    @staticmethod
    def attributes(node):
        return dict(node.get_attributes())

    def find(self, name):
        return next((node for node in self.nodes() if node.get_name() == name), None)

    def summary(self):
        node = self.find("Working units this Project will be created with")
        return self.attributes(node) if node else {}

    def capture(self, label, window):
        self.focus(window)
        # Capture the private Wayland compositor's presented output, never its
        # potentially stale X11 host backing pixmap. Each output directory is
        # unique because weston-screenshooter filenames have second precision.
        directory = self.root / f"{label}-native-capture"
        directory.mkdir()
        result = subprocess.run(["weston-screenshooter"], cwd=directory,
                                env=dict(self.env, XDG_PICTURES_DIR=str(directory)),
                                text=True, capture_output=True, timeout=10)
        self.actions.append({"argv": ["weston-screenshooter"], "cwd": str(directory),
                             "private_wayland_display": self.env["WAYLAND_DISPLAY"],
                             "exit": result.returncode, "stdout": result.stdout,
                             "stderr": result.stderr})
        images = list(directory.glob("*.png"))
        assert result.returncode == 0 and not result.stderr and len(images) == 1, (
            f"native capture failed: {result.stderr}; images={images}")
        images[0].rename(self.root / f"{label}.png")
        self.observations[label] = [
            {"name": node.get_name(), "attributes": self.attributes(node)}
            for node in self.nodes() if node.get_role_name() != "terminal"
        ]

    def launch(self, label):
        self.app_log = (self.root / f"{label}.log").open("w")
        self.phase = label
        self.env["DATUM_GUI_LOG"] = str(self.root / f"{label}-diagnostic.log")
        args = [str(REPO / "target/debug/datum-gui")]
        self.actions.append({"launch_argv": args, "isolated_xdg": str(self.root)})
        self.process = subprocess.Popen(args, cwd=REPO, env=self.env,
                                        stdout=self.app_log, stderr=subprocess.STDOUT,
                                        start_new_session=True)
        log = Path(self.env["DATUM_GUI_LOG"])
        self.wait("native Main visible", lambda: log.exists() and "window visible" in log.read_text())
        identities = [line for line in log.read_text().splitlines() if "surface identity" in line]
        assert identities and all("window_backend=wayland" in line and "gpu_backend=Vulkan" in line for line in identities)
        self.observations[label + "_surface_identity"] = identities
        self.focus(self.host_window)
        self.command("xdotool", "mousemove", "--window", self.host_window, "600", "400", "click", "1")
        return self.host_window

    def stop(self):
        if not self.process:
            return
        if self.process.poll() is None:
            # Do not close the host envelope: it is reused for a fresh bare App.
            self.actions.append({"terminate_owned_app_pid": self.process.pid,
                                 "signal": "SIGTERM",
                                 "limit": "Process termination, not ordinary window Close"})
            self.process.terminate()
            try:
                self.process.wait(timeout=6)
            except subprocess.TimeoutExpired:
                # Only this runner's process group; no name-based process sweep.
                os.killpg(self.process.pid, signal.SIGTERM)
                self.process.wait(timeout=6)
        self.app_log.close()
        log = Path(self.env["DATUM_GUI_LOG"])
        if log.exists():
            self.observations[self.phase + "_all_surface_identities"] = [
                line for line in log.read_text().splitlines() if "surface identity" in line]
        self.process = None

    def terminal_text(self):
        for node in self.nodes():
            if node.get_role_name() == "terminal":
                return Atspi.Text.get_text(node.get_text_iface(), 0, -1)
        return ""

    def observe_unopened(self, label):
        pending, seen, children = [self.process.pid], set(), []
        while pending:
            pid = pending.pop()
            if pid in seen:
                continue
            seen.add(pid)
            for path in Path(f"/proc/{pid}/task").glob("*/children"):
                try:
                    for child in path.read_text().split():
                        child = int(child)
                        if child not in seen:
                            executable = Path(f"/proc/{child}/exe").readlink().name
                            children.append({"pid": child, "executable": executable})
                            pending.append(child)
                except (FileNotFoundError, ProcessLookupError):
                    pass
        ptys = []
        for fd in Path(f"/proc/{self.process.pid}/fd").iterdir():
            try:
                target = str(fd.readlink())
                if target in ["/dev/ptmx", "/dev/pts/ptmx"]:
                    ptys.append(target)
            except FileNotFoundError:
                pass
        assert not any(c["executable"] in {"bash", "sh", "dash", "zsh", "fish"} for c in children)
        assert not ptys, "unopened Main must not allocate terminal PTYs"
        assert not self.terminal_text().strip()
        self.observations[label] = {"owned_descendants": children, "terminal_ptys": ptys}

    def terminal_command(self, main, command):
        geometry = self.command("xdotool", "getwindowgeometry", "--shell", main)
        height = int(dict(line.split("=", 1) for line in geometry.splitlines())["HEIGHT"])
        self.focus(main)
        self.command("xdotool", "mousemove", "--window", main, "45", str(height - 42), "click", "1")
        self.wait("terminal text interface", lambda: any(n.get_role_name() == "terminal" for n in self.nodes()))
        self.text(main, command)
        self.key(main, "Return")

    def menu_item(self, main, title, label):
        # Stock 1x chrome coordinates, verified in the captured native Main.
        # Availability/focus is read from this PID's actual menu projection.
        self.focus(main)
        x = {"File": 118, "View": 195}[title]
        self.command("xdotool", "mousemove", "--window", main, str(x), "17", "click", "1")
        self.wait(f"{title} menu entry {label}", lambda: self.find(label))
        assert self.attributes(self.find(label))["available"] == "true"
        for _ in range(30):
            node = self.find(label)
            if node.get_state_set().contains(Atspi.StateType.FOCUSED):
                self.capture(f"{title.lower()}-{label.lower().replace(' ', '-')}-reachable", main)
                self.key(main, "Return")
                self.wait(f"{title} menu closed after {label}", lambda: self.find(label) is None)
                return
            self.key(main, "Down")
        raise RuntimeError(f"{label} was not keyboard reachable in {title}")

    @staticmethod
    def source_files(project):
        return {str(p.relative_to(project)): digest(p) for p in project.rglob("*")
                if p.is_file() and (".datum" not in p.relative_to(project).parts
                    or p.relative_to(project).parts[:2] in
                    {(".datum", "journal"), (".datum", "revision")} )}

    def package_reachability(self, main, target):
        before = self.source_files(target)
        self.menu_item(main, "View", "Library Browser")
        time.sleep(0.4)
        self.key(main, "Tab", repeat=5)
        self.key(main, "Return")
        self.capture("package-kind-events-await-render-review", main)
        self.key(main, "Tab", repeat=5)
        self.key(main, "Return")
        self.capture("package-create-events-await-render-review", main)
        self.observations["package_render_review_required"] = (
            "Library menu availability/focus was asserted. Inspect the two captured native frames "
            "to confirm actual Package kind and create draft; there is no Library draft AT-SPI node."
        )
        self.key(main, "Escape")
        assert self.source_files(target) == before, "Package draft/cancel is nonmutating"
        self.observations["package_source_hashes"] = before

    def execute_package_only(self, target):
        before = self.source_files(target)
        self.start_envelope()
        main = self.launch("package-main")
        self.key(main, "ctrl+o")
        self.wait("ordinary Open dialog", lambda: self.window("Open Project — Datum"))
        self.capture("open-owned-gui-created-project", main)
        self.text(main, str(target))
        self.key(main, "Tab", repeat=2)
        self.key(main, "Return")
        self.wait("ordinary Open completed", lambda: self.window("Open Project — Datum") is None)
        self.terminal_command(main, 'printf "NPD_PACKAGE_ROOT:%s\\n" "$DATUM_PROJECT_ROOT"')
        self.wait("actual Project-bound terminal", lambda: "NPD_PACKAGE_ROOT:" + str(target) in self.terminal_text())
        self.package_reachability(main, target)
        assert self.source_files(target) == before
        self.observations["existing_gui_created_project"] = str(target)
        self.terminal_command(main, "exit")

    def execute(self):
        self.start_envelope()
        main = self.launch("first-main")
        self.capture("unopened-main", main)
        self.observe_unopened("unopened-no-shell")
        self.key(main, "ctrl+n")
        dialog = self.wait("actual New dialog", lambda: self.window("New Project — Datum"))
        self.wait("eight-value Units preview", lambda: self.summary().get("available") == "true")
        initial = self.summary()
        assert len(initial["value"].split("; ")) == 8
        self.capture("new-before-cancel", dialog)
        self.key(dialog, "Escape")
        self.wait("New cancelled", lambda: self.window("New Project — Datum") is None)
        self.observe_unopened("cancel-no-shell")
        self.capture("cancelled-unopened-main", main)

        collision = self.root / "occupied"
        collision.mkdir()
        (collision / "sentinel").write_bytes(b"preserve existing destination\n")
        before_collision = self.source_files(collision)
        target = self.root / "created-project"
        home = Path.home()
        collision_input = "~/" + str(collision.relative_to(home))
        target_input = "~/" + str(target.relative_to(home))
        self.key(main, "ctrl+n")
        dialog = self.wait("New reopened", lambda: self.window("New Project — Datum"))
        self.wait("Units ready", lambda: self.summary().get("available") == "true")
        original = self.summary()
        self.text(dialog, "Startup QA display name")
        self.key(dialog, "Tab")
        self.text(dialog, collision_input)
        self.key(dialog, "Tab", repeat=4)
        self.key(dialog, "Return")
        self.wait("actual occupied-destination refusal", lambda: self.find("Project not created"))
        assert self.source_files(collision) == before_collision
        assert not (collision / "Startup QA display name").exists()
        assert self.summary() == original, "path refusal preserves values and source context"
        assert self.attributes(self.find("Create"))["available"] == "true"
        assert self.find("Re-read Global defaults") is None
        self.capture("occupied-refusal-eight-units", dialog)

        # Create focus -> name -> Location. No controller calls or preference reread.
        self.key(dialog, "Tab", repeat=2)
        self.key(dialog, "BackSpace", repeat=len(collision_input))
        self.text(dialog, target_input)
        self.wait("correctable refusal cleared", lambda: self.find("Project not created") is None)
        assert self.summary() == original
        self.capture("corrected-ready-without-global-reread", dialog)
        self.key(dialog, "Tab", repeat=4)
        self.key(dialog, "Return")
        self.wait("GUI-created canonical Project", lambda: target.joinpath("project.json").exists())
        self.wait("automatic Open completed", lambda: self.window("New Project — Datum") is None)
        project = json.loads(target.joinpath("project.json").read_text())
        receipt = project["project_units_seed_receipt"]
        assert len(receipt["items"]) == 8
        assert project["name"] == "Startup QA display name"
        main = self.wait("loaded Main", self.window)
        self.terminal_command(main, 'printf "NPD_QA_READY:%s\\n" "$DATUM_PROJECT_ROOT"')
        self.wait("live Project-bound terminal output", lambda: "NPD_QA_READY:" + str(target) in self.terminal_text())
        self.observations["live_terminal"] = self.terminal_text()[-4000:]
        self.capture("created-open-live-terminal", main)
        self.menu_item(main, "File", "Save")
        time.sleep(0.5)
        before = self.source_files(target)

        self.package_reachability(main, target)
        self.terminal_command(main, "exit")
        self.stop()

        # Fresh no-argument process, ordinary Open dialog; no --project-root bypass.
        main = self.launch("fresh-main")
        self.key(main, "ctrl+o")
        dialog = self.wait("actual Open dialog", lambda: self.window("Open Project — Datum"))
        self.text(dialog, str(target))
        self.key(dialog, "Tab", repeat=2)
        self.key(dialog, "Return")
        self.wait("fresh Open completed", lambda: self.window("Open Project — Datum") is None)
        main = self.wait("fresh loaded Main", self.window)
        self.terminal_command(main, 'printf "NPD_QA_REOPEN:%s\\n" "$DATUM_PROJECT_ROOT"')
        self.wait("reopened Project-bound terminal", lambda: "NPD_QA_REOPEN:" + str(target) in self.terminal_text())
        assert self.source_files(target) == before
        assert json.loads(target.joinpath("project.json").read_text())["project_units_seed_receipt"] == receipt
        self.capture("fresh-open-receipt-and-source-preserved", main)
        self.terminal_command(main, "exit")
        self.observations["project"] = str(target)
        self.observations["canonical_source_hashes"] = before
        self.observations["collision_preserved"] = before_collision
        self.observations["units_receipt"] = receipt


def native_launch_smoke(root):
    """Bare launch on the existing desktop; no workflow or Close claim."""
    env = dict(os.environ)
    assert env.get("WAYLAND_DISPLAY"), "an existing Wayland desktop is required"
    for key in ["DISPLAY", "WINIT_UNIX_BACKEND", "DATUM_GPU_DIAGNOSTIC_BACKEND"]:
        env.pop(key, None)
    env.update(XDG_CONFIG_HOME=str(root / "config"), XDG_DATA_HOME=str(root / "data"),
               XDG_CACHE_HOME=str(root / "cache"), HISTFILE=str(root / "history"),
               DATUM_GUI_LOG=str(root / "gui.log"), DATUM_GUI_VERBOSE_LOG="1")
    args = [str(REPO / "target/debug/datum-gui")]
    result = {"argv": args, "wayland_display": env["WAYLAND_DISPLAY"],
              "desktop": env.get("XDG_CURRENT_DESKTOP"), "display_removed": True,
              "binary_sha256": digest(Path(args[0])), "status": "FAIL",
              "scope": "Native desktop launch only; no input/workflow/Close qualification"}
    with (root / "process.log").open("w") as log:
        app = subprocess.Popen(args, cwd=REPO, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            until = time.monotonic() + 30
            while time.monotonic() < until:
                assert app.poll() is None, "owned GUI exited"
                path = root / "gui.log"
                text = path.read_text() if path.exists() else ""
                identities = [line for line in text.splitlines() if "surface identity" in line]
                if "window visible" in text and identities:
                    assert all("window_backend=wayland" in line and "gpu_backend=Vulkan" in line
                               for line in identities)
                    result.update(surface_identities=identities, status="PASS bounded normal launch")
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError("No actual native desktop Wayland launch")
        finally:
            if app.poll() is None:
                app.terminate()
                app.wait(timeout=10)
            result["owned_process_exit"] = app.returncode
            (root / "result.json").write_text(json.dumps(result, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--native-launch-smoke", action="store_true")
    parser.add_argument("--package-project", type=Path)
    args = parser.parse_args()
    root = args.run_dir.resolve()
    if not root.is_relative_to(OWNED.resolve()) or root == OWNED.resolve() or root.exists():
        parser.error("run-dir must be a new owned child of .datum/native-project-startup-repair")
    if args.package_project:
        args.package_project = args.package_project.resolve()
        if (args.native_launch_smoke or not args.package_project.is_relative_to(OWNED.resolve())
                or not (args.package_project / "project.json").is_file()):
            parser.error("package-project must be an existing owned GUI-created Project")
    root.mkdir(parents=True)
    if args.native_launch_smoke:
        native_launch_smoke(root)
        return
    qa = StartupQA(root)
    result = {"mode": "bare native Wayland/WGPU/Vulkan Datum; real X11 input through owned Weston; PID-qualified GI inspection",
              "binary_hashes": qa.binary_hashes, "status": "FAIL",
              "persistence_limit": "Fresh process after owned SIGTERM; ordinary window Close is not exercised by this runner"}
    try:
        if args.package_project:
            result["scenario"] = "Targeted Package reachability on existing GUI-created Project"
            qa.execute_package_only(args.package_project)
        else:
            result["scenario"] = "New Project startup correction and persistence"
            qa.execute()
        result["status"] = "PASS for executed startup repair observations"
    except Exception as error:
        result["error"] = str(error)
        (root / "failure.txt").write_text(traceback.format_exc())
        raise
    finally:
        qa.stop()
        qa.close_envelope()
        result["observations"] = qa.observations
        (root / "result.json").write_text(json.dumps(result, indent=2) + "\n")
        (root / "commands.json").write_text(json.dumps(qa.actions, indent=2) + "\n")
        if qa.previous_focus:
            qa.command("xdotool", "windowactivate", qa.previous_focus, check=False)


if __name__ == "__main__":
    main()
