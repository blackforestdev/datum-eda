"""Offline protection against the observed shared-display and cleanup failures."""
import os
import json
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

import run_native_qa_isolated_input as isolated


class PrivateInputTests(unittest.TestCase):
    def test_existing_scenario_evidence_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            receipt = output / "private-bus.json"
            receipt.write_text("original evidence\n")
            result = subprocess.run(["/usr/bin/python3", str(Path(isolated.__file__)),
                "--scenario", "preferences", "--output", str(output)],
                capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 2)
            self.assertIn("fresh directory", result.stderr)
            self.assertEqual(receipt.read_text(), "original evidence\n")

    def test_sigterm_stops_private_bus_group_before_runtime_removal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready = root / "ready"
            child = f"from pathlib import Path; import time; Path({str(ready)!r}).touch(); time.sleep(30)"
            code = ("from pathlib import Path; from run_native_qa_isolated_input import run_private_bus; "
                    f"run_private_bus(['/usr/bin/python3', '-c', {child!r}], Path({str(root)!r}))")
            process = subprocess.Popen(["/usr/bin/python3", "-c", code],
                cwd=Path(isolated.__file__).parent, env=dict(os.environ, XDG_RUNTIME_DIR=str(root)),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                until = time.monotonic() + 5
                while not ready.exists() and process.poll() is None and time.monotonic() < until:
                    time.sleep(.02)
                self.assertTrue(ready.exists(), "private bus child did not start")
                process.terminate()
                _, stderr = process.communicate(timeout=12)
                self.assertNotEqual(process.returncode, 0, stderr)
                cleanup = json.loads((root / "private-bus.json").read_text())
                self.assertTrue(cleanup["owned_group_stopped"])
                self.assertTrue(cleanup["runtime_removed"])
                self.assertEqual(isolated.live_group_members(cleanup["process_group"]), [])
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate(timeout=5)

    def test_private_bus_activates_accessibility_but_refuses_desktop_services(self):
        # Exercise the real D-Bus activation catalogue, not just environment dicts.
        code = '''import json, os, subprocess, sys
from pathlib import Path
results = {}
for name in ["org.a11y.Bus", "org.freedesktop.portal.Desktop", "org.kde.kwalletd6", "org.freedesktop.Notifications"]:
    run = subprocess.run(["dbus-send", "--session", "--print-reply", "--dest=org.freedesktop.DBus",
        "/org/freedesktop/DBus", "org.freedesktop.DBus.StartServiceByName", "string:" + name, "uint32:0"],
        capture_output=True, text=True, timeout=10)
    results[name] = {"exit": run.returncode, "error": run.stderr}
    assert (run.returncode == 0) == (name == "org.a11y.Bus"), results
assert os.environ["DISPLAY"] == "datum-qa-disabled"
Path(sys.argv[1]).write_text(json.dumps(results))
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.dict(os.environ, {"XDG_RUNTIME_DIR": str(root)}):
                self.assertEqual(isolated.run_private_bus(
                    ["/usr/bin/python3", "-c", code, str(root / "activation.json")], root), 0)
            proof = json.loads((root / "activation.json").read_text())
            self.assertEqual(proof["org.a11y.Bus"]["exit"], 0)
            self.assertIn("ServiceUnknown", proof["org.kde.kwalletd6"]["error"])
            cleanup = json.loads((root / "private-bus.json").read_text())
            self.assertTrue(cleanup["owned_group_stopped"])
            self.assertTrue(cleanup["runtime_removed"])

    def test_live_group_members_excludes_other_groups_and_dead_processes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for pid, state, group in ((10, "S", 10), (11, "S", 10), (12, "Z", 10), (13, "S", 99)):
                (root / str(pid)).mkdir()
                (root / str(pid) / "stat").write_text(f"{pid} (name with ) spaces) {state} 1 {group} 10")
            self.assertEqual(sorted(isolated.live_group_members(10, root)), [10, 11])

    def test_activated_services_cannot_inherit_owner_desktop_or_default_socket(self):
        owner = {"DISPLAY": ":1", "WAYLAND_DISPLAY": "wayland-0", "WAYLAND_SOCKET": "8",
                 "XAUTHORITY": "/owner/auth", "XDG_RUNTIME_DIR": "/run/user/1000",
                 "DBUS_SESSION_BUS_ADDRESS": "owner-bus", "XDG_CURRENT_DESKTOP": "KDE",
                 "XDG_ACTIVATION_TOKEN": "owner-token"}
        private = isolated.private_bus_environment(owner, "/owned/private-runtime")
        self.assertEqual(private["XDG_RUNTIME_DIR"], "/owned/private-runtime")
        self.assertEqual(private["DISPLAY"], "datum-qa-disabled")
        self.assertEqual(private["WAYLAND_DISPLAY"], "datum-qa-disabled")
        self.assertEqual(private["QT_QPA_PLATFORM"], "offscreen")
        for key in ("WAYLAND_SOCKET", "XAUTHORITY", "DBUS_SESSION_BUS_ADDRESS", "XDG_ACTIVATION_TOKEN"):
            self.assertNotIn(key, private)
        self.assertEqual(owner["DISPLAY"], ":1")
        child = isolated.child_environment(private, ":27", 123, "/owned/auth")
        self.assertEqual(child["DISPLAY"], ":27")
        self.assertNotIn("WAYLAND_DISPLAY", child)
        self.assertEqual(child["XDG_RUNTIME_DIR"], "/owned/private-runtime")

    def test_display_binding_requires_listening_socket_owned_by_exact_server(self):
        with tempfile.TemporaryDirectory() as temporary:
            proc = Path(temporary)
            (proc / "123/fd").mkdir(parents=True)
            (proc / "net").mkdir()
            (proc / "123/exe").symlink_to("/usr/bin/Xwayland")
            (proc / "123/fd/7").symlink_to("socket:[999]")
            table = proc / "net/unix"
            table.write_text("0000: 0002 0000 00010000 0001 01 999 @/tmp/.X11-unix/X27\n")
            self.assertTrue(isolated._live_xwayland(123, ":27", proc_root=proc))
            self.assertFalse(isolated._live_xwayland(123, ":1", proc_root=proc))
            table.write_text("0000: 0002 0000 00010000 0001 01 998 @/tmp/.X11-unix/X27\n")
            self.assertFalse(isolated._live_xwayland(123, ":27", proc_root=proc))
            table.write_text("0000: 0002 0000 00000000 0001 03 999 @/tmp/.X11-unix/X27\n")
            self.assertFalse(isolated._live_xwayland(123, ":27", proc_root=proc))

    def test_missing_isolation_refuses_before_any_desktop_command(self):
        from prove_native_project_startup_gui import StartupQA
        with patch.dict(os.environ, {}, clear=True), patch.object(StartupQA, "command") as command:
            with self.assertRaisesRegex(ValueError, "owned input display required"):
                StartupQA(Path("/unused"))
            command.assert_not_called()

    def test_named_scenario_and_generic_child_cannot_be_combined(self):
        result = subprocess.run(["/usr/bin/python3", str(Path(isolated.__file__)),
            "--scenario", "preferences", "--output", "/unused", "--", "false"],
            capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertIn("cannot be combined", result.stderr)

    def test_outer_capture_is_default_off_and_changes_only_owned_outer_debug_capability(self):
        for enabled, dimensions in ((False, None), (True, None), (False, (2304, 2048))):
            with self.subTest(enabled=enabled, dimensions=dimensions), tempfile.TemporaryDirectory() as temporary:
                parent = Path(temporary)
                args = SimpleNamespace(output=parent / "transport", child_result=parent / "child.json",
                                       command=["/owned/runner"])
                if enabled:
                    args.outer_capture = True
                if dimensions is not None:
                    args.input_width, args.input_height = dimensions
                width, height = dimensions or (1600, 1000)
                calls = []

                def spawn(argv, *, env, **kwargs):
                    process = Mock(pid=100 + len(calls), returncode=None)
                    calls.append((list(argv), dict(env)))
                    process.poll.side_effect = lambda: process.returncode
                    process.terminate.side_effect = lambda: setattr(process, "returncode", -15)

                    def wait(timeout=None):
                        if argv == args.command:
                            args.child_result.write_text(json.dumps(self.receipt()))
                        process.returncode = 0
                        return 0

                    process.wait.side_effect = wait
                    if argv[0] == "/usr/bin/weston":
                        (Path(env["XDG_RUNTIME_DIR"]) / "datum-qa-input").touch()
                    elif argv[0] == "/usr/bin/Xwayland":
                        os.write(int(argv[argv.index("-displayfd") + 1]), b"27\n")
                    return process

                with patch.dict(os.environ, {"XDG_RUNTIME_DIR": str(parent), "DISPLAY": ":1",
                                             "WAYLAND_DISPLAY": "foreign", "WAYLAND_SOCKET": "9"}), \
                        patch.object(isolated.subprocess, "Popen", side_effect=spawn), \
                        patch.object(isolated, "_live_xwayland", return_value=True), \
                        patch.object(isolated, "executable_binding", side_effect=lambda p: {"pid": p.pid}):
                    parent_environment = dict(os.environ)
                    self.assertEqual(isolated.run(args), 0)
                    self.assertEqual(dict(os.environ), parent_environment)
                outer, outer_env = calls[0]
                expected = ["/usr/bin/weston", "--backend=headless", "--renderer=gl",
                            "--shell=kiosk-shell.so", "--no-config", f"--width={width}", f"--height={height}",
                            "--scale=1", "--idle-time=0", "--socket=datum-qa-input",
                            "--log=" + str(args.output / "outer-weston-detail.log")]
                self.assertEqual(outer, expected + (["--debug"] if enabled else []))
                self.assertNotIn("WAYLAND_DISPLAY", outer_env)
                self.assertNotIn("WAYLAND_SOCKET", outer_env)
                self.assertNotIn("WAYLAND_DISPLAY", calls[-1][1])
                self.assertNotIn("WAYLAND_SOCKET", calls[-1][1])
                receipt = json.loads((args.output / "transport.json").read_text())
                self.assertEqual(calls[1][0][calls[1][0].index("-geometry") + 1], f"{width}x{height}")
                self.assertEqual(receipt["requested_input_dimensions"], {"width": width, "height": height})
                capability = receipt["outer_capture"]
                self.assertIs(capability["debug_requested"], enabled)
                self.assertEqual(capability["runtime"], receipt["runtime"])
                self.assertEqual(capability["socket"], "datum-qa-input")
                self.assertEqual(capability["argv"], outer)
                self.assertIn("same-UID", capability["limits"])
                self.assertTrue(receipt["runtime_removed_after_owners_stop"])

    def test_outer_capture_cli_requires_explicit_option(self):
        for option in ([], ["--outer-capture"], ["--input-width", "2304", "--input-height", "2048"]):
            with self.subTest(option=option), \
                    patch("sys.argv", ["runner", "--output", "/owned/out", "--child-result",
                                       "/owned/child.json", *option, "--", "/owned/child"]), \
                    patch.object(isolated, "run", return_value=0) as run:
                with self.assertRaises(SystemExit) as stopped:
                    isolated.main()
                self.assertEqual(stopped.exception.code, 0)
                self.assertIs(run.call_args.args[0].outer_capture, "--outer-capture" in option)
                expected = (2304, 2048) if "--input-width" in option else (1600, 1000)
                self.assertEqual((run.call_args.args[0].input_width, run.call_args.args[0].input_height), expected)
                self.assertEqual(run.call_args.args[0].command, ["/owned/child"])

    def test_invalid_dimensions_refuse_before_artifacts_or_processes(self):
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            for width, height in ((0, 1000), (1600, -1), (True, 1000), (1600, 1.5), ("2304", 2048)):
                with self.subTest(width=width, height=height):
                    args = SimpleNamespace(output=parent / "transport", child_result=parent / "child.json",
                                           command=["/owned/runner"], input_width=width, input_height=height)
                    with patch.object(isolated.subprocess, "Popen") as spawn, \
                            patch.object(isolated.tempfile, "TemporaryDirectory") as runtime:
                        with self.assertRaisesRegex(ValueError, "positive integers"):
                            isolated.run(args)
                    spawn.assert_not_called()
                    runtime.assert_not_called()
                    self.assertEqual(list(parent.iterdir()), [])

    def private_env(self):
        return {"DISPLAY": ":27", isolated.DISPLAY_KEY: ":27", isolated.OWNER_KEY: "123"}

    def test_child_targets_allocated_display_without_changing_outer_session(self):
        original = {"DISPLAY": ":1", "WAYLAND_DISPLAY": "desktop", "WAYLAND_SOCKET": "9",
                    "XDG_RUNTIME_DIR": "/run/user/1000", "DBUS_SESSION_BUS_ADDRESS": "original"}
        current = isolated.child_environment(original, ":27", 123, "/owned/empty-authority")
        self.assertEqual(current["DISPLAY"], ":27")
        self.assertEqual(current[isolated.DISPLAY_KEY], ":27")
        self.assertEqual(current[isolated.OWNER_KEY], "123")
        self.assertEqual(current["XDG_RUNTIME_DIR"], "/run/user/1000")
        self.assertEqual(current["DBUS_SESSION_BUS_ADDRESS"], "original")
        self.assertNotIn("WAYLAND_DISPLAY", current)
        self.assertNotIn("WAYLAND_SOCKET", current)
        self.assertEqual(original["DISPLAY"], ":1")
        self.assertEqual(original["WAYLAND_SOCKET"], "9")
        with self.assertRaises(ValueError):
            isolated.child_environment(original, ":1", 123, "/owned/empty-authority")

    def test_private_focus_uses_direct_owned_window_without_window_manager(self):
        self.assertEqual(isolated.focus_arguments("789", self.private_env(), lambda pid, display: pid == 123 and display == ":27"),
                         ["xdotool", "windowfocus", "--sync", "789"])
        with self.assertRaises(ValueError):
            isolated.focus_arguments("789", {})

    def test_mismatched_or_dead_private_display_never_falls_back_to_desktop(self):
        for changes in [{"DISPLAY": ":1"}, {isolated.DISPLAY_KEY: "desktop"},
                        {isolated.OWNER_KEY: "not-a-pid"}]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                isolated.focus_arguments("789", {**self.private_env(), **changes}, lambda pid, display: True)
        with self.assertRaises(ValueError):
            isolated.focus_arguments("789", self.private_env(), lambda pid, display: False)

    def receipt(self):
        return {"verified_owner_stops": dict.fromkeys(["gui", "daemons", "clipboard", "Weston"], True),
                "ipc_runtime": {"removed_after_owners_stop": True}}

    def test_ipc_retained_without_all_actual_inner_and_outer_stop_receipts(self):
        owners = [{"stopped": True}, {"stopped": True}]
        self.assertTrue(isolated.may_remove_runtime(owners, self.receipt()))
        for bad in [None, [], {}, {"verified_owner_stops": "true"},
                    {**self.receipt(), "ipc_runtime": {"removed_after_owners_stop": "true"}}]:
            with self.subTest(receipt=bad):
                self.assertFalse(isolated.may_remove_runtime(owners, bad))
        for key in ["gui", "daemons", "clipboard", "Weston"]:
            bad = self.receipt()
            bad["verified_owner_stops"][key] = False
            self.assertFalse(isolated.may_remove_runtime(owners, bad))
        self.assertFalse(isolated.may_remove_runtime([{"stopped": False}], self.receipt()))

    def test_failed_owner_does_not_skip_other_stops_or_log_closure(self):
        first, failed, last = Mock(), Mock(), Mock()
        for index, process in enumerate([first, failed, last]):
            process.pid = index + 10
            process.poll.return_value = None
            process.wait.side_effect = lambda timeout, process=process: setattr(process.poll, "return_value", 0)
        failed.terminate.side_effect = OSError("actual stop failure")
        logs = [Mock(), Mock(), Mock()]
        owners = [{"label": label, "process": process, "stream": log}
                  for label, process, log in zip(["outer-weston", "xwayland", "runner"],
                                                [first, failed, last], logs)]
        errors = []
        isolated.stop_owners(owners, errors)
        last.send_signal.assert_called_once_with(signal.SIGINT)
        failed.terminate.assert_called_once()
        first.terminate.assert_called_once()
        self.assertEqual([v["stopped"] for v in owners], [True, False, True])
        self.assertTrue(any("actual stop failure" in v["error"] for v in errors))
        self.assertTrue(all(log.close.call_count == 1 for log in logs))
        self.assertFalse(isolated.may_remove_runtime(owners, self.receipt()))

    def test_stop_timeout_is_retained_even_after_owned_pid_kill(self):
        process = Mock()
        process.poll.return_value = None
        def wait(timeout):
            if not process.kill.called:
                raise subprocess.TimeoutExpired("owned-server", timeout)
            process.poll.return_value = -9
        process.wait.side_effect = wait
        owner = {"label": "xwayland", "process": process, "stream": Mock()}
        errors = []
        isolated.stop_owners([owner], errors)
        process.kill.assert_called_once()
        self.assertTrue(owner["stopped"])
        self.assertTrue(errors)

    def test_startup_helper_validates_before_any_desktop_command(self):
        from prove_native_project_startup_gui import StartupQA
        with patch.dict(os.environ, {**self.private_env(), "DISPLAY": ":1"}), \
                patch.object(StartupQA, "command") as command:
            with self.assertRaises(ValueError):
                StartupQA(Path("/unused"))
            command.assert_not_called()

    def test_private_startup_never_queries_or_restores_shared_desktop_focus(self):
        import prove_native_project_startup_gui as startup
        with tempfile.TemporaryDirectory() as directory, \
                patch.dict(os.environ, self.private_env()), \
                patch.object(isolated, "_live_xwayland", return_value=True), \
                patch.object(startup, "digest", return_value="pinned"), \
                patch.object(startup.StartupQA, "command") as command:
            qa = startup.StartupQA(Path(directory), runtime_parent=Path(directory))
            try:
                self.assertEqual(qa.previous_focus, "")
                command.assert_not_called()
            finally:
                qa.runtime.cleanup()

    def test_startup_focus_integration_keeps_owned_xid_and_liveness_guards(self):
        from prove_native_project_startup_gui import StartupQA
        qa = object.__new__(StartupQA)
        qa.host_window, qa.compositor, qa.command = "789", Mock(), Mock()
        qa.compositor.poll.return_value = None
        with patch.dict(os.environ, self.private_env()), \
                patch.object(isolated, "_live_xwayland", return_value=True):
            qa.focus("789")
            qa.command.assert_called_once_with("xdotool", "windowfocus", "--sync", "789")
            with self.assertRaises(AssertionError):
                qa.focus("foreign")
            qa.compositor.poll.return_value = 1
            with self.assertRaises(AssertionError):
                qa.focus("789")


if __name__ == "__main__":
    unittest.main()
