#!/usr/bin/env python3
"""Bounded native-Wayland startup QA through an owned Weston input envelope.

Run with /usr/bin/python3 on the existing desktop. No Cargo, genesis helpers,
private UI actions, owner Projects or HOME substitution participate.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import signal
import shlex
import shutil
import subprocess
import tempfile
import time
import traceback

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib  # noqa: E402
from native_project_source_inventory import digest, source_files


REPO = Path(__file__).resolve().parents[1]
OWNED = REPO / ".datum/native-project-startup-repair"


class StartupQA:
    source_files = staticmethod(source_files)

    def __init__(self, root: Path, *, runtime_parent: Path | None = None):
        self.root = root
        self.process = None
        self.app_log = None
        self.actions = []
        self.observations = {}
        self.capture_counts = {}
        self.terminal_dock_open = True
        self.previous_focus = self.command("xdotool", "getwindowfocus").strip()
        self.runtime = tempfile.TemporaryDirectory(prefix="npd-wl-", dir=runtime_parent or "/tmp")
        self.compositor = None
        self.file_actions = False
        self.clipboard_writer = None
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
            "DATUM_PROJECT_ROOT": "/c08-stale-parent-project",
            "DATUM_DISCOVERY": "/c08-stale-parent-discovery",
            "DATUM_AGENT_CREDENTIAL_FILE": "/c08-stale-parent-credential",
            "EDA_ENGINE_SOCKET": "/c08-stale-parent-socket",
            "DATUM_ENGINE_SOCKET": "/c08-stale-parent-engine",
            "DATUM_MCP_ENDPOINT": "/c08-stale-parent-mcp",
            "DATUM_AGENT_PROVENANCE": "c08-stale-parent-provenance",
        })
        for key in ["DISPLAY", "WAYLAND_SOCKET", "WINIT_UNIX_BACKEND", "DATUM_GPU_DIAGNOSTIC_BACKEND"]:
            self.env.pop(key, None)
        self.binary_dir = Path(os.environ.get("DATUM_QA_BINARY_DIR", REPO / "target/debug")).resolve()
        self.binary_hashes = {
            name: digest(self.binary_dir / name)
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

    def start_envelope(self, desktop=False):
        env = dict(os.environ, XDG_RUNTIME_DIR=self.runtime.name)
        self.compositor_log = (self.root / "weston-process.log").open("w")
        config = "--no-config"
        if desktop:
            settings = self.root / "weston.ini"
            settings.write_text("[shell]\nbinding-modifier=ctrl\npanel-position=none\n")
            config = "--config=" + str(settings)
        args = ["weston", "--backend=x11", "--renderer=gl",
                "--shell=" + ("desktop-shell.so" if desktop else "kiosk-shell.so"), config, "--debug", "--width=1344", "--height=806", "--scale=1",
                "--idle-time=0", "--socket=datum-startup-qa", "--log=" + str(self.root / "weston.log")]
        if self.file_actions:
            args.append("--xwayland")
        self.compositor = subprocess.Popen(args, env=env, stdout=self.compositor_log,
                                           stderr=subprocess.STDOUT)
        self.actions.append({"owned_compositor_pid": self.compositor.pid, "argv": args})
        self.wait("private Wayland socket", lambda: Path(self.runtime.name, "datum-startup-qa").exists())
        text = (self.root / "weston.log").read_text()
        match = re.search(r"x11 output .*window id (\d+)", text)
        assert match, "host XID must come from this owned live compositor's log"
        self.host_window = match.group(1)
        if self.file_actions:
            display = self.wait("same owned compositor XWayland display", lambda:
                re.search(r"xserver listening on display (:\d+)", (self.root / "weston.log").read_text()))
            self.env["DISPLAY"] = display.group(1)
            assert self.env["DISPLAY"] != os.environ.get("DISPLAY"), "never use outer clipboard"
            self.observations["clipboard_provider"] = {
                "kind": "same-compositor XWayland clipboard bridge; not native Wayland selection",
                "owned_display": self.env["DISPLAY"], "wayland_display": self.env["WAYLAND_DISPLAY"],
                "outer_input_display": os.environ.get("DISPLAY")}
        metadata = self.command("xprop", "-id", self.host_window, "WM_CLASS", "_NET_WM_PID")
        assert "Weston Compositor" in metadata
        self.observations["input_envelope"] = {"pid": self.compositor.pid,
            "xid_from_owned_log": self.host_window, "metadata": metadata,
            "limit": "X11 input to owned Weston; Datum itself native Wayland/Vulkan, not KWin workflow qualification"}

    def close_envelope(self):
        if self.clipboard_writer and self.clipboard_writer.poll() is None:
            self.clipboard_writer.terminate()
            self.clipboard_writer.wait(timeout=5)
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

    def text(self, window, value, delay="4"):
        self.focus(window)
        self.command("xdotool", "type", "--clearmodifiers", "--delay", delay, "--", value)
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

    def assert_new_has_no_units(self):
        retired = {"Working units for this new Project", "Working units this Project will be created with",
                   "Use my Global Units defaults", "Use Datum factory Units"}
        assert not any(n.get_name() in retired or n.get_role_name() in {"radio button", "radio group"}
                       for n in self.nodes()), "New exposes no Units configuration/review controls"

    def wait_presented(self, after_damage=None):
        # Read-only presentation evidence: no private App action or bootstrap.
        log = Path(self.env["DATUM_GUI_LOG"])
        settled = None
        def current_frame():
            nonlocal settled
            text = log.read_text()
            identities = re.findall(r"surface identity window=(WindowId\([0-9]+\))", text)
            owned = any(n.get_role_name() == "dialog" for n in self.nodes())
            host = identities[-1] if owned else identities[0]
            native_focus = re.findall(r"window event (WindowId\([0-9]+\)) focused (true|false)", text)
            if not native_focus or native_focus[-1] != (host, "true"):
                settled = None
                return False
            records = [json.loads(line.split("native redraw ledger ", 1)[1])
                       for line in text.splitlines() if "native redraw ledger " in line]
            latest = next((r for r in reversed(records) if r["window"] == host), None)
            if not latest or latest["damage"] != latest["presented_damage"] or latest["pending"] or latest["rendering"] is not None:
                settled = None
                return False
            if after_damage is not None and latest["presented_damage"] <= after_damage:
                settled = None
                return False
            identity = (host, latest["presented_damage"])
            ready = settled == identity
            settled = identity
            return latest if ready else False
        receipt = self.wait("current dialog/Main frame presented before native capture", current_frame)
        self.actions.append({"capture_presentation_barrier": receipt})
        return receipt

    def capture(self, label, window):
        receipt = self.wait_presented()
        count = self.capture_counts.get(label, 0) + 1
        self.capture_counts[label] = count
        if count > 1:
            label = f"{label}-{count}"
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
        return receipt

    def launch(self, label, *, project_root: Path | None = None):
        self.app_log = (self.root / f"{label}.log").open("w")
        self.phase = label
        self.env["DATUM_GUI_LOG"] = str(self.root / f"{label}-diagnostic.log")
        args = [str(self.binary_dir / "datum-gui")]
        if project_root is not None:
            args.extend(["--project-root", str(project_root)])
        self.actions.append({"launch_argv": args, "isolated_xdg": str(self.root)})
        self.process = subprocess.Popen(args, cwd=Path.home() if os.environ.get("DATUM_QA_TERMINAL_HANDOFF") == "1" else self.root, env=self.env,
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

    def terminal_ptys_closed(self):
        for fd in Path(f"/proc/{self.process.pid}/fd").iterdir():
            try:
                if str(fd.readlink()) in {"/dev/ptmx", "/dev/pts/ptmx"}:
                    return False
            except FileNotFoundError:
                pass
        return True

    def select_terminal_tab(self, main, tab=0):
        geometry = self.command("xdotool", "getwindowgeometry", "--shell", main)
        height = int(dict(line.split("=", 1) for line in geometry.splitlines())["HEIGHT"])
        node = self.wait("terminal text interface", lambda: next(
            (n for n in self.nodes() if n.get_role_name() == "terminal"), None))
        bounds = Atspi.Component.get_extents(node.get_component_iface(), Atspi.CoordType.WINDOW)
        header_y = bounds.y - 22 if self.terminal_dock_open and bounds.height > 0 else height - 42
        self.actions.append({"terminal_bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
                             "selected_tab_index": tab})
        self.focus(main)
        self.command("xdotool", "mousemove", "--window", main, str(45 + tab * 156), str(header_y), "click", "1")
        self.terminal_dock_open = True
        self.wait("terminal explicitly owns input", lambda: any(
            n.get_role_name() == "terminal" and n.get_state_set().contains(Atspi.StateType.FOCUSED)
            for n in self.nodes()))

    def terminal_command(self, main, command, tab=0):
        self.select_terminal_tab(main, tab)
        self.text(main, command, delay="15")
        self.key(main, "Return")

    def menu_item(self, main, title, label, child=None):
        self.wait_presented()
        self.focus(main)
        x = {"File": 118, "Edit": 147, "View": 195}[title]
        self.command("xdotool", "mousemove", "--window", main, str(x), "17", "click", "1")
        for wanted, key in [(label, "Right" if child else "Return")] + ([(child, "Return")] if child else []):
            def entry():
                return next((n for n in self.nodes() if n.get_name() == wanted and n.get_role_name() == "menu item"), None)
            self.wait(f"{title} menu entry {wanted}", entry)
            assert self.attributes(entry())["available"] == "true"
            for _ in range(30):
                if entry().get_state_set().contains(Atspi.StateType.FOCUSED):
                    self.capture(f"{title.lower()}-{wanted.lower().replace(' ', '-')}-reachable", main)
                    self.key(main, key)
                    break
                self.key(main, "Down")
            else:
                raise RuntimeError(f"{wanted} was not keyboard reachable in {title}")
            if key == "Return":
                self.wait(f"{title} menu closed after {wanted}", lambda: entry() is None)

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
        self.menu_item(main, "File", "Open Project")
        self.wait("ordinary Open dialog", lambda: self.window("Open Project — Datum"))
        self.capture("open-owned-gui-created-project", main)
        self.text(main, str(target))
        self.key(main, "Tab", repeat=3)
        self.key(main, "Return")
        self.wait("ordinary Open completed", lambda: self.window("Open Project — Datum") is None)
        self.terminal_dock_open = False
        self.terminal_command(main, 'printf "NPD_PACKAGE_ROOT:%s\\n" "$DATUM_PROJECT_ROOT"', tab=1)
        self.wait("actual Project-bound terminal", lambda: "NPD_PACKAGE_ROOT:" + str(target) in self.terminal_text())
        self.package_reachability(main, target)
        assert self.source_files(target) == before
        self.observations["existing_gui_created_project"] = str(target)
        self.terminal_command(main, "exit", tab=1)
        self.terminal_command(main, "exit")
        self.wait("all owned terminal PTYs closed", self.terminal_ptys_closed)

    def focus_control(self, main, name):
        for _ in range(24):
            node = self.find(name)
            if node and node.get_state_set().contains(Atspi.StateType.FOCUSED):
                self.wait_presented()
                return node
            self.key(main, "Tab")
            self.wait_presented()
        raise RuntimeError(f"Control not keyboard reachable: {name}")

    def fill_control(self, main, name, value):
        self.focus_control(main, name)
        self.key(main, "ctrl+a")
        if value:
            self.text(main, value)
        else:
            self.key(main, "BackSpace")
        self.wait_presented()
        self.wait("exact field replacement", lambda:
                  self.attributes(self.find(name)).get("value", "") == value)

    def execute(self):
        self.start_envelope()
        main = self.launch("unbound-main")
        self.terminal_command(main, 'C08_SENTINEL=preserved; sleep 300 & C08_CHILD=$!; '
            'printf "C08_KEEP:%s:%s:%s\\n" "$$" "$C08_CHILD" "$C08_SENTINEL"; '
            'printf "C08_UNBOUND:%s:%s\\n" "${DATUM_PROJECT_ROOT-unset}" "$PWD"; '
            'printf "C08_CLEAR:%s:%s:%s:%s:%s:%s\\n" "${DATUM_DISCOVERY-unset}" "${DATUM_AGENT_CREDENTIAL_FILE-unset}" '
            '"${EDA_ENGINE_SOCKET-unset}" "${DATUM_ENGINE_SOCKET-unset}" "${DATUM_MCP_ENDPOINT-unset}" "${DATUM_AGENT_PROVENANCE-unset}"; '
            'printf "C08_RUNTIME:%s\\n" "$TERMINFO"')
        self.wait("real rootless shell with no stale authority", lambda:
            "C08_UNBOUND:unset:" + str(self.root) in self.terminal_text()
            and "C08_CLEAR:unset:unset:unset:unset:unset:unset" in self.terminal_text())
        initial = self.terminal_text()
        match = re.search(r"C08_KEEP:(\d+):(\d+):preserved", initial)
        assert match, "actual shell/child identities must come from expanded output"
        shell_pid, child_pid = map(int, match.groups())
        runtime_match = re.search(r"C08_RUNTIME:(/[^\r\n]+)", initial)
        assert runtime_match and "/datum-terminal-" in runtime_match.group(1)
        runtime = Path(runtime_match.group(1)).parents[1]
        self.observations["initial_unbound_terminal"] = initial
        self.observations["preserved_processes"] = {"shell": shell_pid, "child": child_pid}
        assert not self.root.joinpath(".datum").exists()
        self.capture("real-unbound-shell-before-project", main)
        self.terminal_command(main, 'printf "C08_CLI_PATH:%s\\n" "$(command -v datum-eda)"')
        self.wait("matching CLI is callable in ordinary shell", lambda:
            "C08_CLI_PATH:" + str(self.binary_dir / "datum-eda") in self.terminal_text())
        self.observations["ordinary_cli_resolution"] = self.terminal_text()
        self.capture("matching-cli-in-unbound-shell", main)
        self.terminal_command(main,
            'if command -v codex >/dev/null; then printf "C08_AGENT_KIND:%s\\n" codex; codex --version; '
            'elif command -v claude >/dev/null; then printf "C08_AGENT_KIND:%s\\n" claude; claude --version; '
            'else printf "C08_%s_AGENT\\n" NO_INSTALLED; fi; printf "C08_PROBE_DONE:%s\\n" "$?"')
        self.wait("actual CLI and installed-agent probe completed", lambda:
            "C08_PROBE_DONE:0" in self.terminal_text())
        probe = self.terminal_text()
        assert "C08_NO_INSTALLED_AGENT" not in probe, "direct-agent proof needs an installed executable"
        self.observations["direct_cli_agent_probe"] = probe
        self.capture("real-unbound-shell-cli-agent", main)

        self.menu_item(main, "File", "New Project")
        self.wait("ordinary New", lambda: self.window("New Project — Datum"))
        self.assert_new_has_no_units()
        self.capture("new-with-existing-unbound-session", main)
        self.key(main, "Escape")
        self.wait("New cancelled", lambda: self.window("New Project — Datum") is None)
        self.capture("cancel-preserves-unbound-session", main)
        self.menu_item(main, "File", "Open Project")
        self.wait("ordinary Open", lambda: self.window("Open Project — Datum"))
        self.capture("open-before-invalid-path", main)
        self.text(main, str(self.root / "missing-project"))
        self.key(main, "Tab", repeat=3)
        self.key(main, "Return")
        self.wait("invalid Open refused", lambda: self.find("Project not opened"))
        assert not self.root.joinpath("missing-project").exists()
        self.capture("invalid-open-preserves-session", main)
        self.key(main, "Escape")
        self.wait("Open cancelled", lambda: self.window("Open Project — Datum") is None)
        target = self.root / "cli-created-project"
        command = "datum-eda project new " + shlex.quote(str(target)) + " --name 'Terminal bootstrap QA' --units-source factory --json"
        self.terminal_command(main, command + '; printf "C08_PROJECT_READY:%s\\n" "$?"')
        self.wait("canonical CLI completion", lambda: "C08_PROJECT_READY:0" in self.terminal_text())
        self.wait("canonical CLI-created Project", lambda: target.joinpath("project.json").exists())
        project = json.loads(target.joinpath("project.json").read_text())
        assert len(project["project_units_seed_receipt"]["items"]) == 8
        self.capture("canonical-cli-project-created", main)
        self.terminal_command(main, 'printf "C08_HISTORY:%s\\n" before-open; '
            'for i in 1 2 3 4 5 6 7 8 9 10 11 12; do printf "C08_FOLLOW:%s\\n" "$i"; done')
        self.wait("history tail completed", lambda: "C08_FOLLOW:12" in self.terminal_text())
        self.key(main, "shift+Prior")
        self.observations["scrollback_before_open_atspi"] = self.terminal_text()
        self.observations["scrollback_render_review_required"] = (
            "Inspect actual native before/after frames for C08_HISTORY:before-open. "
            "AT-SPI did not refresh on scroll-only input in the retained diagnostic; its viewport text is not acceptance evidence.")
        self.capture("unbound-scrollback-before-open", main)
        self.menu_item(main, "File", "Open Project")
        self.wait("Open CLI-created Project", lambda: self.window("Open Project — Datum"))
        self.capture("open-cli-created-project", main)
        self.text(main, str(target))
        self.key(main, "Tab", repeat=3)
        self.key(main, "Return")
        self.wait("guarded Open published", lambda: self.window("Open Project — Datum") is None)
        self.terminal_dock_open = False  # ordinary loaded Project starts with closed dock
        self.terminal_command(main, 'printf "C08_BOUND:%s\\n" "$DATUM_PROJECT_ROOT"; '
            'printf "C08_BOUND_CLEAR:%s:%s:%s:%s\\n" "${DATUM_ENGINE_SOCKET-unset}" "${EDA_ENGINE_SOCKET-unset}" "${DATUM_MCP_ENDPOINT-unset}" "${DATUM_AGENT_PROVENANCE-unset}"; '
            'printf "C08_BOUND_SESSION:%s\\n" "$DATUM_SESSION_ID"; '
            'test "$DATUM_DISCOVERY" = "$DATUM_PROJECT_ROOT/.datum/terminal-contexts/$DATUM_SESSION_ID.discovery.json"; '
            'printf "C08_FRESH_DISCOVERY_PATH:%s\\n" "$?"', tab=1)
        self.wait("new bound terminal", lambda: "C08_BOUND:" + str(target) in self.terminal_text()
            and "C08_BOUND_CLEAR:unset:unset:unset:unset" in self.terminal_text()
            and "C08_FRESH_DISCOVERY_PATH:0" in self.terminal_text())
        bound_text = self.terminal_text()
        bound_session = re.search(r"C08_BOUND_SESSION:(terminal-[A-Za-z0-9-]+)", bound_text).group(1)
        discovery_path = target / ".datum/terminal-contexts" / f"{bound_session}.discovery.json"
        discovery = json.loads(discovery_path.read_text())
        assert discovery["project_root"] == str(target) and discovery["terminal_session_id"] == bound_session
        self.observations["new_bound_terminal"] = bound_text
        self.observations["fresh_project_discovery"] = {"path": str(discovery_path),
            "sha256": digest(discovery_path), "project_root": discovery["project_root"],
            "terminal_session_id": bound_session}
        self.capture("new-project-bound-session", main)
        before = self.source_files(target)
        self.select_terminal_tab(main)
        self.observations["scrollback_after_open_atspi"] = self.terminal_text()
        self.capture("original-scrollback-survives-open", main)
        self.terminal_command(main, 'printf "C08_SURVIVED:%s:%s:%s:%s\\n" "$$" "$C08_CHILD" "$C08_SENTINEL" "${DATUM_PROJECT_ROOT-unset}"')
        expected = f"C08_SURVIVED:{shell_pid}:{child_pid}:preserved:unset"
        self.wait("original shell and environment preserved", lambda: expected in self.terminal_text())
        assert Path(f"/proc/{child_pid}").exists(), "original child must survive Project Open"
        self.observations["preserved_unbound_terminal"] = self.terminal_text()
        self.capture("original-unbound-shell-survives-open", main)
        assert self.source_files(target) == before
        assert not self.root.joinpath(".datum").exists()
        self.terminal_command(main, "exit")
        self.wait("original shell reaped", lambda: not Path(f"/proc/{shell_pid}").exists())
        self.wait("original child reaped", lambda: not Path(f"/proc/{child_pid}").exists())
        self.wait("unbound runtime cleaned after closure", lambda: not runtime.exists())
        self.terminal_command(main, "exit")
        self.wait("all owned terminal PTYs closed", self.terminal_ptys_closed)
        self.observations["canonical_source_hashes"] = before
        self.observations["cli_created_project"] = str(target)
        self.observations["unbound_runtime_cleaned"] = str(runtime)
        self.observations["limits"] = "Native nested-Wayland input/capture; shell lifecycle shutdown, GUI process termination; installed-agent version is execution evidence, not MCP or model round-trip qualification"


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
    runtimes, shell_pids = [], []
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
                    result["surface_identities"] = identities
                    # End only this GUI's initial shell through OS job control,
                    # keeping Datum alive to supervise/reap and clean runtime.
                    runtimes = list(Path(env["XDG_RUNTIME_DIR"]).glob(f"datum-terminal-{app.pid}-*"))
                    assert runtimes, "bare launch must own an unbound terminal runtime"
                    for runtime in runtimes:
                        session = json.loads(runtime.joinpath("session.json").read_text())
                        child = session["process_group_id"]
                        assert isinstance(child, int) and child > 1
                        stat = Path(f"/proc/{child}/stat").read_text().rsplit(")", 1)[1].split()
                        assert int(stat[1]) == app.pid and os.getpgid(child) == child
                        shell_pids.append(child)
                        os.killpg(child, signal.SIGHUP)
                    def owned_transport_closed():
                        if any(Path(f"/proc/{pid}").exists() for pid in shell_pids):
                            return False
                        for fd in Path(f"/proc/{app.pid}/fd").iterdir():
                            try:
                                if str(fd.readlink()) in {"/dev/ptmx", "/dev/pts/ptmx"}:
                                    return False
                            except FileNotFoundError:
                                pass
                        return True
                    until_cleanup = time.monotonic() + 7
                    while not owned_transport_closed() and time.monotonic() < until_cleanup:
                        time.sleep(0.05)
                    assert owned_transport_closed(), "owned shells, descendants and PTY teardown must complete before ending GUI"
                    result["terminal_cleanup"] = "SIGHUP to verified owned shell group; Datum supervised closure; no ordinary GUI Close claim"
                    result["status"] = "PASS bounded normal launch"
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError("No actual native desktop Wayland launch")
        finally:
            if app.poll() is None:
                app.terminate()
                app.wait(timeout=10)
            result["owned_process_exit"] = app.returncode
            # Forced process termination skips Rust destructors. Remove only the
            # recorded private runtime after its shell and GUI have both gone;
            # this is harness housekeeping, not application cleanup evidence.
            if not any(Path(f"/proc/{pid}").exists() for pid in shell_pids):
                for runtime in runtimes:
                    if runtime.exists():
                        assert runtime.name.startswith(f"datum-terminal-{app.pid}-") and runtime.stat().st_uid == os.getuid()
                        shutil.rmtree(runtime)
                result["runtime_housekeeping"] = "Recorded owned runtime removed after forced GUI end; not product cleanup evidence"
            (root / "result.json").write_text(json.dumps(result, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", required=True, type=Path)
    parser.add_argument("--native-launch-smoke", action="store_true")
    parser.add_argument("--package-project", type=Path)
    parser.add_argument("--browse", action="store_true")
    parser.add_argument("--file-actions", action="store_true")
    parser.add_argument("--minimum-dialogs", action="store_true")
    parser.add_argument("--new-global", action="store_true")
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
    qa.file_actions = args.file_actions
    result = {"mode": "bare native Wayland/WGPU/Vulkan Datum; real X11 input through owned Weston; PID-qualified GI inspection",
              "binary_hashes": qa.binary_hashes, "status": "FAIL",
              "persistence_limit": ("Minimum forms cancelled then GUI SIGTERM; no terminal-exit claim" if args.minimum_dialogs else "Single-process terminal preservation and shell exit; final GUI SIGTERM is not ordinary window Close")}
    try:
        if args.minimum_dialogs:
            result["scenario"] = ("C09 minimum native New form, fixed refusal and explicit Global reader retry" if args.new_global else "C09 minimum native New/Open forms, fixed refusal and explicit Global reader retry")
            from native_project_browse_workflow import execute_minimum_forms
            execute_minimum_forms(qa, new_only=args.new_global)
        elif args.new_global:
            result["scenario"] = "C09 automatic Global inheritance in minimal native New; existing Project Preferences doorway"
            from native_project_browse_workflow import execute_browse
            execute_browse(qa, new_only=True)
        elif args.browse or args.file_actions:
            result["scenario"] = ("C09 compact close prompt on existing GUI Project" if os.environ.get("DATUM_QA_HANDOFF_REFIT_ONLY")=="1" else "C09 ordinary terminal handoff New/Open") if os.environ.get("DATUM_QA_TERMINAL_HANDOFF")=="1" else ("C09 shared single-line Location Home/End/selection in New/Open" if os.environ.get("DATUM_QA_EDITOR_REVEAL_ONLY")=="1" else "C09 single native New/Open dialogs, Create and Open with original terminal")
            from native_project_browse_workflow import execute_browse
            execute_browse(qa)
        elif args.package_project:
            result["scenario"] = "Targeted Package reachability on existing GUI-created Project"
            qa.execute_package_only(args.package_project)
        else:
            result["scenario"] = "C08 real unbound shell, canonical CLI creation and ordinary GUI adoption"
            qa.execute()
        result["status"] = "PASS for executed startup repair observations"
    except Exception as error:
        result["error"] = str(error)
        (root / "failure.txt").write_text(traceback.format_exc())
        if qa.process and qa.process.poll() is None:
            try:
                qa.observations["failure_terminal_text"] = qa.terminal_text()
                qa.capture("failure-visible-state", qa.host_window)
            except Exception as capture_error:
                result["failure_capture_error"] = str(capture_error)
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
