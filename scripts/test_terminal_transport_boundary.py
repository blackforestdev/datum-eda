#!/usr/bin/env python3
"""Hermetic regressions for the terminal transport ownership guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_transport_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_transport_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalTransportBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        transport = root / guard.TRANSPORT
        (transport / "linux").mkdir(parents=True)
        files = {
            "mod.rs": (
                "mod linux;\nmod request;\n"
                "fn prepare_terminal_transport(){open_pty_pair(); into_command(); command.spawn();}"
            ),
            "request.rs": "struct TerminalTransportRequest {}",
            "event.rs": "enum TerminalTransportEvent { Output(Vec<u8>), Exited(Option<i32>) }",
            "reader.rs": (
                "fn spawn_reader(){output.reserve();reader.read_bytes();}\n"
                "fn reader_retries_eintr_and_would_block_without_losing_bytes_or_spinning(){}\n"
                "fn reader_drains_hup_tail_then_accepts_correlated_eio_as_eof(){}\n"
                "fn reader_reports_uncorrelated_eio_and_invalid_descriptor_once(){}"
            ),
            "input.rs": "struct InputQueue {}",
            "output.rs": "struct OutputBacklog {}",
            "control.rs": "struct ControlBacklog {}",
            "launch_error.rs": "enum TerminalLaunchStage {}",
            "limits.rs": (
                "const MAX_OUTPUT_CHUNKS: usize = 256;\n"
                "const MAX_OUTPUT_CHUNK_BYTES: usize = 16 * 1024;\n"
                "const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;\n"
                "const MAX_INPUT_REQUESTS: usize = 64;\n"
                "const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;\n"
                "const MAX_LIVE_SESSIONS: usize = 16;\n"
                "const GUI_DRAIN_EVENT_LIMIT: usize = 128;\n"
                "const GUI_DRAIN_BYTE_LIMIT: usize = 64 * 1024;"
            ),
            "session_handle.rs": (
                "pub(crate) struct PreparedTerminalTransport {\n    child: (),\n}\n"
                "impl PreparedTerminalTransport { pub(super) fn new() {} "
                "pub(crate) fn process_group_id() {} pub(crate) fn start() {} }\n"
                "pub(crate) struct TerminalTransportSession {\n    writer: (),\n"
                "    events: (),\n    master_fd: i32,\n    process_group_id: i32,\n}\n"
                "impl TerminalTransportSession { pub(super) fn new() {} "
                "pub(crate) fn try_recv_event() {} pub(crate) fn recv_event_timeout() {} "
                "pub(crate) fn try_recv_control_event() {} pub(crate) fn try_recv_output() {} "
                "pub(crate) fn has_pending_event() {} "
                "pub(crate) fn process_group_id() {} pub(crate) fn write_bytes() {} "
                "pub(crate) fn interrupt() {} pub(crate) fn terminate() {} "
                "pub(crate) fn resize() {} }"
            ),
            "wake.rs": "struct TerminalWakeGate {}",
            "linux/pty.rs": (
                "fn open_pty_pair(){let _=c\"/dev/ptmx\";libc::grantpt();"
                "libc::unlockpt();libc::ptsname_r();}"
            ),
            "linux/spawn.rs": (
                "fn attach_child_pty(){libc::setsid();libc::TIOCSCTTY;libc::dup2();}"
            ),
            "linux/job_control.rs": (
                "fn signal_process_group(){libc::kill();libc::TIOCSWINSZ;}"
            ),
            "linux/io.rs": (
                "fn wait_readable(){libc::F_DUPFD_CLOEXEC;libc::poll();}"
            ),
            "linux/termios.rs": (
                "fn configure_interactive(){libc::tcgetattr();libc::tcsetattr();}"
            ),
        }
        for relative, text in files.items():
            (transport / relative).write_text(text, encoding="utf-8")
        (root / guard.APP_SRC / "terminal_session_drain.rs").write_text(
            "fn drain_all(){next_drain_index;try_recv_control_event();try_recv_output();}\n"
            "fn control_priority_round_robin_cursor_and_exact_global_caps_are_literal(){}",
            encoding="utf-8",
        )
        return temporary, root

    def test_valid_multifile_boundary_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual([], guard.check(root))

    def test_syscall_and_raw_handle_escape_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        outside = root / guard.APP_SRC / "rogue.rs"
        outside.write_text("fn rogue(){libc::TIOCSWINSZ; let master_fd=1;}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("TIOCSWINSZ" in failure for failure in failures))
        self.assertTrue(any("master_fd" in failure for failure in failures))

    def test_raw_api_and_fd_type_escape_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        outside = root / guard.APP_SRC / "rogue.rs"
        outside.write_text(
            "use std::os::fd::{RawFd, OwnedFd};\n"
            "fn rogue(fd: RawFd, owned: OwnedFd){"
            "crate::terminal_transport::linux::pty::open_pty_pair();"
            "let _ = (fd, owned);}",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("open_pty_pair" in failure for failure in failures))
        self.assertTrue(any("RawFd" in failure for failure in failures))
        self.assertTrue(any("OwnedFd" in failure for failure in failures))

    def test_unqualified_syscall_and_pre_exec_escape_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        outside = root / guard.APP_SRC / "rogue.rs"
        outside.write_text(
            "use libc::setsid; use std::os::unix::process::CommandExt;\n"
            "fn rogue(command: &mut std::process::Command){unsafe{setsid();"
            "command.pre_exec(|| Ok(()));}}",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("setsid" in failure for failure in failures))
        self.assertTrue(any("pre_exec" in failure for failure in failures))

    def test_semantics_dependency_and_clone_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        handle = root / guard.TRANSPORT / "session_handle.rs"
        handle.write_text(
            handle.read_text(encoding="utf-8").replace(
                "pub(crate) struct TerminalTransportSession {\n    writer:",
                "#[derive(Clone)] pub(crate) struct TerminalTransportSession {\n    pub writer:",
            )
            + "\nfn parse(){TerminalScreen::default();portable_pty();}",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("TerminalScreen" in failure for failure in failures))
        self.assertTrue(any("portable_pty" in failure for failure in failures))
        self.assertIn("terminal transport owning handle must not be Clone", failures)
        self.assertIn(
            "terminal transport handle fields must all be private: TerminalTransportSession",
            failures,
        )

    def test_missing_owner_and_forwarding_root_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        (root / guard.TRANSPORT / "linux/pty.rs").write_text(
            "fn open_pty_pair(){}", encoding="utf-8"
        )
        (root / guard.TRANSPORT / "mod.rs").write_text(
            "pub use request::*;", encoding="utf-8"
        )
        failures = guard.check(root)
        self.assertTrue(any('/dev/ptmx' in failure for failure in failures))
        self.assertTrue(any("real orchestration" in failure for failure in failures))

    def test_broad_linux_visibility_and_raw_constructor_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        root_module = root / guard.TRANSPORT / "mod.rs"
        root_module.write_text(
            root_module.read_text(encoding="utf-8").replace(
                "mod linux;", "pub(super) mod linux;"
            ),
            encoding="utf-8",
        )
        handle = root / guard.TRANSPORT / "session_handle.rs"
        handle.write_text(
            handle.read_text(encoding="utf-8").replace(
                "pub(super) fn new()", "pub fn new()", 1
            ),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertIn(
            "terminal transport Linux module must be private to its root", failures
        )
        self.assertIn(
            "terminal transport must have exactly two parent-private constructors", failures
        )

    def test_renamed_raw_fields_and_getters_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        handle = root / guard.TRANSPORT / "session_handle.rs"
        handle.write_text(
            handle.read_text(encoding="utf-8").replace(
                "writer: (),", "pub(crate) master: std::fs::File,"
            )
            + "\nimpl TerminalTransportSession {"
            "pub(crate) fn take_master() -> std::fs::File { todo!() }"
            "pub(crate) fn event_channel() -> std::sync::mpsc::Receiver<()> { todo!() }"
            "pub(crate) fn into_parts() -> (std::fs::File, i32) { todo!() }}\n",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("fields must all be private" in failure for failure in failures))
        self.assertTrue(any("take_master" in failure for failure in failures))
        self.assertTrue(any("event_channel" in failure for failure in failures))
        self.assertTrue(any("into_parts" in failure for failure in failures))

    def test_budget_increase_and_unbounded_channel_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        limits = root / guard.TRANSPORT / "limits.rs"
        limits.write_text(
            limits.read_text(encoding="utf-8").replace(
                "MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024",
                "MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024",
            ),
            encoding="utf-8",
        )
        reader = root / guard.TRANSPORT / "reader.rs"
        reader.write_text(reader.read_text(encoding="utf-8") + "\nfn bad(){mpsc::channel();}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("MAX_OUTPUT_BYTES" in failure for failure in failures))
        self.assertTrue(any("mpsc::channel" in failure for failure in failures))

    def test_active_only_drain_regression_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        rogue = root / guard.APP_SRC / "runtime.rs"
        rogue.write_text("fn poll(){sessions.active().try_recv_event();}", encoding="utf-8")
        self.assertIn("active-only terminal event draining must not return", guard.check(root))

    def test_missing_reader_and_fairness_proofs_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        reader = root / guard.TRANSPORT / "reader.rs"
        reader.write_text("fn spawn_reader(){reader.read_bytes();}", encoding="utf-8")
        drain = root / guard.APP_SRC / "terminal_session_drain.rs"
        drain.write_text(
            "fn drain_all(){next_drain_index;try_recv_control_event();try_recv_output();}",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("reserve" in failure for failure in failures))
        self.assertTrue(any("transition proof" in failure for failure in failures))
        self.assertIn("all-session terminal drain lacks literal L3 proof", failures)


if __name__ == "__main__":
    unittest.main()
