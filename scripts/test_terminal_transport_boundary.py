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
            "reader.rs": "fn spawn_event_threads() {}",
            "session_handle.rs": (
                "pub(crate) struct PreparedTerminalTransport {\n    child: (),\n}\n"
                "impl PreparedTerminalTransport { pub(super) fn new() {} "
                "pub(crate) fn process_group_id() {} pub(crate) fn start() {} }\n"
                "pub(crate) struct TerminalTransportSession {\n    writer: (),\n"
                "    events: (),\n    master_fd: i32,\n    process_group_id: i32,\n}\n"
                "impl TerminalTransportSession { pub(super) fn new() {} "
                "pub(crate) fn try_recv_event() {} pub(crate) fn recv_event_timeout() {} "
                "pub(crate) fn process_group_id() {} pub(crate) fn write_bytes() {} "
                "pub(crate) fn interrupt() {} pub(crate) fn terminate() {} "
                "pub(crate) fn resize() {} }"
            ),
            "wake.rs": "struct TerminalWakeGate {}",
            "linux/pty.rs": (
                "fn open_pty_pair(){libc::posix_openpt();libc::grantpt();"
                "libc::unlockpt();libc::ptsname_r();}"
            ),
            "linux/spawn.rs": (
                "fn attach_child_pty(){libc::setsid();libc::TIOCSCTTY;libc::dup2();}"
            ),
            "linux/job_control.rs": (
                "fn signal_process_group(){libc::kill();libc::TIOCSWINSZ;}"
            ),
        }
        for relative, text in files.items():
            (transport / relative).write_text(text, encoding="utf-8")
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
        self.assertTrue(any("libc::posix_openpt" in failure for failure in failures))
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


if __name__ == "__main__":
    unittest.main()
