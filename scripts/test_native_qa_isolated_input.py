"""Offline protection against the observed shared-display and cleanup failures."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

import run_native_qa_isolated_input as isolated


class PrivateInputTests(unittest.TestCase):
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
        self.assertEqual(isolated.focus_arguments("789", self.private_env(), lambda pid: pid == 123),
                         ["xdotool", "windowfocus", "--sync", "789"])
        self.assertEqual(isolated.focus_arguments("789", {}),
                         ["xdotool", "windowactivate", "--sync", "789"])

    def test_mismatched_or_dead_private_display_never_falls_back_to_desktop(self):
        for changes in [{"DISPLAY": ":1"}, {isolated.DISPLAY_KEY: "desktop"},
                        {isolated.OWNER_KEY: "not-a-pid"}]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                isolated.focus_arguments("789", {**self.private_env(), **changes}, lambda pid: True)
        with self.assertRaises(ValueError):
            isolated.focus_arguments("789", self.private_env(), lambda pid: False)

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
