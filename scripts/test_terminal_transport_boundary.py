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
            "input.rs": "struct InputQueue {} fn cancel(){queue.is_closed();}",
            "output.rs": "struct OutputBacklog {}",
            "control.rs": "struct ControlBacklog { writer_finished: bool }",
            "launch_error.rs": "enum TerminalLaunchStage {}",
            "limits.rs": (
                "const MAX_OUTPUT_CHUNKS: usize = 256;\n"
                "const MAX_OUTPUT_CHUNK_BYTES: usize = 16 * 1024;\n"
                "const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;\n"
                "const MAX_INPUT_REQUESTS: usize = 64;\n"
                "const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;\n"
                "const MAX_LIVE_SESSIONS: usize = 16;\n"
                "const GUI_DRAIN_EVENT_LIMIT: usize = 128;\n"
                "const GUI_DRAIN_BYTE_LIMIT: usize = 64 * 1024;\n"
                "const HUP_GRACE_MS: u64 = 2_000;\n"
                "const TERM_GRACE_MS: u64 = 2_000;\n"
                "const KILL_VERIFY_MS: u64 = 2_000;\n"
                "const GLOBAL_SHUTDOWN_MS: u64 = 6_000;\n"
                "const MAX_SESSION_MEMBERS: usize = 4_096;\n"
                "const MAX_SESSION_GROUPS: usize = 4_096;"
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
                "pub(crate) fn terminate() {} "
                "pub(crate) fn force_kill() {} pub(crate) fn shutdown_snapshot() {} "
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
                "fn signal_owned_process_group(){libc::kill();libc::TIOCSWINSZ;}"
            ),
            "process_status.rs": "enum TerminalExitStatus {}",
            "process_supervisor.rs": (
                "struct ProcessSupervisor { signal_owned_process_group: () } "
                "impl ProcessSupervisor { fn begin_kill_phase(){} }"
            ),
            "shutdown.rs": "enum ShutdownPhase {}",
            "linux/process_session.rs": "fn discover_owned_session(){}",
            "linux/io.rs": (
                "fn wait_readable(){libc::F_DUPFD_CLOEXEC;libc::poll();} "
                "fn wait_writable(){wait_with_timeout(fd, libc::POLLOUT, 100);}"
            ),
            "linux/termios.rs": (
                "fn configure_interactive(){libc::tcgetattr();libc::tcsetattr();}"
            ),
        }
        for relative, text in files.items():
            (transport / relative).write_text(text, encoding="utf-8")
        (root / guard.APP_SRC / "terminal_session_drain.rs").write_text(
            "fn drain_all(){next_drain_index;try_recv_control_event();try_recv_output();}\n",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_drain_tests.rs").write_text(
            "fn control_priority_round_robin_cursor_and_exact_global_caps_are_literal(){}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_isolation_tests.rs").write_text(
            "const SESSION_COUNT: usize = 8;\n"
            "fn eight_real_sessions_isolate_io_resize_exit_termination_and_restart(){\n"
            "let _ = \"DTC06B-peer-survived\"; presentation_complete();\n"
            "all_sessions_closed(); let _ = \"P06 must not recreate a detached PTY state\";\n}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_stress_tests.rs").write_text(
            "const P06_LIFECYCLE_CYCLES: usize = 100;\n"
            "fn p06_resource_helper(){\n"
            "proc_entry_count(\"/proc/self/fd\"); proc_entry_count(\"/proc/self/task\");\n"
            "session.presentation_complete(); snapshot.leader_reaped;\n"
            "snapshot.surviving_processes.is_empty();\n}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_io_event_log.rs").write_text(
            "pub(crate) const IO_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;\n"
            "pub(crate) const IO_SEGMENT_COUNT: usize = 4;\n"
            "fn rotate_segments(){}\n"
            "fn fact(){let _ = Rotation { event: \"terminal_io_rotation\" };}\n"
            "fn four_segments_rotate_oldest_first_and_metadata_records_the_fact(){}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_activity_snapshot_rotation_tests.rs").write_text(
            "fn rotated_io_family_is_folded_chronologically_without_replay(){}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_measurement_tests.rs").write_text(
            "const SESSION_COUNT: usize = 8;\n"
            "fn p06_release_measurement_emits_reproducible_json(){\n"
            "let _ = Evidence { contract: \"datum_terminal_p06_measurement_v1\" };\n"
            "measure_aggregate_output(); measure_latency_and_input(); measure_resize();\n"
            "proc_count(\"/proc/self/fd\"); proc_count(\"/proc/self/task\");}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_gui_measurement_tests.rs").write_text(
            "const OUTPUT_BYTES_PER_SESSION: usize = 1024 * 1024;\n"
            "fn p06_provisional_gui_path_emits_reproducible_json(){\n"
            "let _ = Evidence { contract: \"datum_terminal_p06_gui_measurement_v1\" };\n"
            "registry.drain_all(); drain_work: distribution(); presentation_complete();}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_lifecycle_measurement_tests.rs").write_text(
            "const P06_LIFECYCLE_CYCLES: usize = 1_000;\n"
            "fn p06_one_thousand_spawn_exit_restart_cycles_emit_reproducible_json(){\n"
            "let _ = Evidence { contract: \"datum_terminal_p06_lifecycle_v1\" };\n"
            "restart_active(); presentation_complete(); snapshot.surviving_processes.is_empty();\n"
            "proc_count(\"/proc/self/fd\"); proc_count(\"/proc/self/task\");}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_session_p06_soak_tests.rs").write_text(
            "fn p06_soak_tier_budgets_are_literal(){}\n"
            "fn p06_scheduled_soak_emits_reproducible_json(){\n"
            "let _ = Evidence { contract: \"datum_terminal_p06_soak_v1\" };\n"
            "match tier { \"ci\" => Self { duration: Duration::from_secs(10 * 60) },\n"
            "\"single-24h\" => Self { duration: Duration::from_secs(24 * 60 * 60),\n"
            "minimum_bytes_per_session: 128 * 1024 * 1024 },\n"
            "\"max-4h\" => Self { duration: Duration::from_secs(4 * 60 * 60),\n"
            "minimum_bytes_per_session: 128 * 1024 * 1024, resize_requests: 10_000 } };\n"
            "presentation_complete(); proc_count(\"/proc/self/fd\");\n"
            "proc_count(\"/proc/self/task\");}",
            encoding="utf-8",
        )
        scripts = root / "scripts"
        scripts.mkdir(exist_ok=True)
        (scripts / "run_terminal_transport_proof_gates.sh").write_text(
            "cargo test --release --locked --offline\n"
            "wayland-primary x11-fallback\n"
            "single throughput >=20MiB/s\n"
            "aggregate throughput >=40MiB/s\n"
            "smoke|gui|lifecycle-1000|ci|single-24h|max-4h DATUM_P06_RUN_ORDINAL\n"
            'p06_scheduled_soak_emits_reproducible_json "datum_terminal_p06_soak_v1"\n'
            '"ci-10-minute": (8, 600, 8 * 1024 * 1024, 1_000)\n'
            '"single-24-hour": (1, 24 * 60 * 60, 128 * 1024 * 1024, 500)\n'
            '"maximum-16-session-4-hour": (16, 4 * 60 * 60, 128 * 1024 * 1024, 10_000)\n'
            '"datum_terminal_p06_gui_measurement_v1"\n'
            "single provisional GUI throughput >=1MiB/s\n"
            "aggregate provisional GUI throughput >=4MiB/s\n"
            '"datum_terminal_p06_lifecycle_v1"\n'
            "exactly 1000 completed lifecycle cycles\n",
            encoding="utf-8",
        )
        (transport / "output.rs").write_text(
            (transport / "output.rs").read_text(encoding="utf-8")
            + "\nfn full_output_backlog_blocks_reservation_until_consumer_pop(){}",
            encoding="utf-8",
        )
        (transport / "input.rs").write_text(
            (transport / "input.rs").read_text(encoding="utf-8")
            + "\nfn input_admission_is_atomic_at_request_and_byte_limits(){}",
            encoding="utf-8",
        )
        drain_tests = root / guard.APP_SRC / "terminal_session_drain_tests.rs"
        drain_tests.write_text(
            drain_tests.read_text(encoding="utf-8")
            + "\nfn seventeenth_session_is_refused_by_preallocation_guard(){}",
            encoding="utf-8",
        )
        (root / guard.APP_SRC / "terminal_job_control_tests.rs").write_text(
            "fn termination_cancels_backpressured_input_and_closes_every_master(){}",
            encoding="utf-8",
        )
        (transport / "process_supervisor_tests.rs").write_text(
            "fn redundant_force_during_kill_cannot_queue_an_unauthorized_retry(){}",
            encoding="utf-8",
        )
        return temporary, root

    def test_valid_multifile_boundary_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual([], guard.check(root))

    def test_missing_eight_session_isolation_proof_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        proof = root / guard.APP_SRC / "terminal_session_p06_isolation_tests.rs"
        proof.write_text(
            proof.read_text(encoding="utf-8").replace(
                "eight_real_sessions_isolate_io_resize_exit_termination_and_restart",
                "weakened_single_session_smoke",
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any("DTC-P06B eight-session isolation proof missing" in failure for failure in guard.check(root))
        )

    def test_missing_p06c_resource_and_saturation_proofs_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        stress = root / guard.APP_SRC / "terminal_session_p06_stress_tests.rs"
        stress.write_text(
            stress.read_text(encoding="utf-8").replace(
                "const P06_LIFECYCLE_CYCLES: usize = 100;",
                "const P06_LIFECYCLE_CYCLES: usize = 1;",
            ).replace("snapshot.leader_reaped", "true"),
            encoding="utf-8",
        )
        output = root / guard.TRANSPORT / "output.rs"
        output.write_text("struct OutputBacklog {}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("P06_LIFECYCLE_CYCLES" in failure for failure in failures))
        self.assertTrue(any("snapshot.leader_reaped" in failure for failure in failures))
        self.assertTrue(any("full_output_backlog" in failure for failure in failures))

    def test_terminal_io_log_budget_or_rotation_proof_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        io_log = root / guard.APP_SRC / "terminal_io_event_log.rs"
        io_log.write_text(
            io_log.read_text(encoding="utf-8")
            .replace("16 * 1024 * 1024", "32 * 1024 * 1024")
            .replace("fn rotate_segments", "fn discard_without_rotation"),
            encoding="utf-8",
        )
        rotation_test = root / guard.APP_SRC / "terminal_activity_snapshot_rotation_tests.rs"
        rotation_test.write_text("fn weakened_rotation_smoke(){}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("IO_SEGMENT_BYTES" in failure for failure in failures))
        self.assertTrue(any("fn rotate_segments" in failure for failure in failures))
        self.assertIn("DTC-P06D rotation-safe activity cache proof missing", failures)

    def test_release_measurement_or_wayland_runner_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        measurement = root / guard.APP_SRC / "terminal_session_p06_measurement_tests.rs"
        measurement.write_text("fn weakened_debug_benchmark(){}", encoding="utf-8")
        runner = root / "scripts/run_terminal_transport_proof_gates.sh"
        runner.write_text("cargo test --release\nx11-only\n", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("release measurement proof missing" in failure for failure in failures))
        self.assertTrue(any("--release --locked --offline" in failure for failure in failures))
        self.assertTrue(any("wayland-primary" in failure for failure in failures))

    def test_scheduled_soak_or_tier_runner_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        soak = root / guard.APP_SRC / "terminal_session_p06_soak_tests.rs"
        soak.write_text("fn weakened_short_soak(){}", encoding="utf-8")
        runner = root / "scripts/run_terminal_transport_proof_gates.sh"
        runner.write_text(
            runner.read_text(encoding="utf-8")
            .replace("smoke|gui|lifecycle-1000|ci|single-24h|max-4h", "smoke")
            .replace("DATUM_P06_RUN_ORDINAL", "UNTRACKED_RUN"),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("scheduled soak proof missing" in failure for failure in failures))
        self.assertTrue(any("smoke|gui|lifecycle-1000|ci|single-24h|max-4h" in failure for failure in failures))
        self.assertTrue(any("DATUM_P06_RUN_ORDINAL" in failure for failure in failures))

    def test_provisional_gui_measurement_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        measurement = root / guard.APP_SRC / "terminal_session_p06_gui_measurement_tests.rs"
        measurement.write_text("fn bypassed_registry_smoke(){}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("provisional GUI measurement proof missing" in failure for failure in failures))

    def test_lifecycle_measurement_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        measurement = root / guard.APP_SRC / "terminal_session_p06_lifecycle_measurement_tests.rs"
        measurement.write_text("const P06_LIFECYCLE_CYCLES: usize = 10;", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("lifecycle measurement proof missing" in failure for failure in failures))

    def test_owner_ratified_shutdown_deadline_drift_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        limits = root / guard.TRANSPORT / "limits.rs"
        limits.write_text(
            limits.read_text(encoding="utf-8").replace(
                "const TERM_GRACE_MS: u64 = 2_000;",
                "const TERM_GRACE_MS: u64 = 3_000;",
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("TERM_GRACE_MS" in failure for failure in guard.check(root)))

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
        drain_tests = root / guard.APP_SRC / "terminal_session_drain_tests.rs"
        drain_tests.write_text("fn weakened_fairness_smoke(){}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("reserve" in failure for failure in failures))
        self.assertTrue(any("transition proof" in failure for failure in failures))
        self.assertIn("all-session terminal drain lacks literal L3 proof", failures)

    def test_missing_kill_and_writer_barrier_proofs_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        supervisor = root / guard.TRANSPORT / "process_supervisor.rs"
        supervisor.write_text(
            supervisor.read_text(encoding="utf-8").replace("fn begin_kill_phase", "fn removed"),
            encoding="utf-8",
        )
        control = root / guard.TRANSPORT / "control.rs"
        control.write_text(
            control.read_text(encoding="utf-8").replace("writer_finished", "writer_missing"),
            encoding="utf-8",
        )
        proof = root / guard.APP_SRC / "terminal_job_control_tests.rs"
        proof.write_text("", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("atomic Kill transition" in failure for failure in failures))
        self.assertTrue(any("writer completion" in failure for failure in failures))
        self.assertTrue(any("closes_every_master" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
