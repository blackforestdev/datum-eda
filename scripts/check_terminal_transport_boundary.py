#!/usr/bin/env python3
"""Enforce the Datum-owned PTY transport ownership boundary."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP_SRC = Path("crates/gui-app/src")
TRANSPORT = APP_SRC / "terminal_transport"

OWNERS = {
    'c"/dev/ptmx"': TRANSPORT / "linux/pty.rs",
    "libc::grantpt": TRANSPORT / "linux/pty.rs",
    "libc::unlockpt": TRANSPORT / "linux/pty.rs",
    "libc::ptsname_r": TRANSPORT / "linux/pty.rs",
    "libc::setsid": TRANSPORT / "linux/spawn.rs",
    "libc::TIOCSCTTY": TRANSPORT / "linux/spawn.rs",
    "libc::dup2": TRANSPORT / "linux/spawn.rs",
    "libc::TIOCSWINSZ": TRANSPORT / "linux/job_control.rs",
    "libc::kill": TRANSPORT / "linux/job_control.rs",
    "libc::F_DUPFD_CLOEXEC": TRANSPORT / "linux/io.rs",
    "libc::tcgetattr": TRANSPORT / "linux/termios.rs",
    "libc::tcsetattr": TRANSPORT / "linux/termios.rs",
}

IDENTIFIER_OWNERS = {
    "grantpt": {TRANSPORT / "linux/pty.rs"},
    "unlockpt": {TRANSPORT / "linux/pty.rs"},
    "ptsname_r": {TRANSPORT / "linux/pty.rs"},
    "setsid": {TRANSPORT / "linux/spawn.rs"},
    "pre_exec": {TRANSPORT / "linux/spawn.rs"},
    "TIOCSCTTY": {TRANSPORT / "linux/spawn.rs"},
    "dup2": {TRANSPORT / "linux/spawn.rs"},
    "TIOCSWINSZ": {TRANSPORT / "linux/job_control.rs"},
    "open_pty_pair": {
        TRANSPORT / "linux/pty.rs", TRANSPORT / "linux/termios.rs", TRANSPORT / "mod.rs",
    },
    "attach_child_pty": {TRANSPORT / "linux/spawn.rs", TRANSPORT / "mod.rs"},
    "signal_owned_process_group": {
        TRANSPORT / "linux/job_control.rs", TRANSPORT / "process_supervisor.rs",
    },
}

REQUIRED_FILES = {
    "mod.rs": "prepare_terminal_transport",
    "request.rs": "struct TerminalTransportRequest",
    "event.rs": "enum TerminalTransportEvent",
    "reader.rs": "spawn_reader",
    "input.rs": "struct InputQueue",
    "output.rs": "struct OutputBacklog",
    "control.rs": "struct ControlBacklog",
    "launch_error.rs": "enum TerminalLaunchStage",
    "limits.rs": "MAX_OUTPUT_BYTES",
    "session_handle.rs": "struct TerminalTransportSession",
    "wake.rs": "struct TerminalWakeGate",
    "linux/pty.rs": "open_pty_pair",
    "linux/spawn.rs": "attach_child_pty",
    "process_status.rs": "enum TerminalExitStatus",
    "process_supervisor.rs": "struct ProcessSupervisor",
    "shutdown.rs": "enum ShutdownPhase",
    "linux/process_session.rs": "discover_owned_session",
    "linux/job_control.rs": "signal_owned_process_group",
    "linux/io.rs": "wait_readable",
    "linux/termios.rs": "configure_interactive",
}

SEMANTIC_TOKENS = (
    "TerminalScreen", "TerminalLaneState", "pty_grid_mut", "apply_bytes",
    "datum_terminal_core", "DesignModel", "Operation", "commit(", "journal",
    "Clipboard", "gui_protocol", "datum_gui_protocol", "wgpu", "Renderer",
)

THIRD_PARTY_TOKENS = (
    "libghostty", "ghostty_vt", "alacritty_terminal", "portable_pty",
    "portable-pty", "libloading", "dlopen", "include!",
)


def rust_sources(root: Path) -> list[Path]:
    return sorted(
        path for path in (root / APP_SRC).rglob("*.rs")
        if not path.name.endswith("_tests.rs")
    )


def check(root: Path) -> list[str]:
    failures: list[str] = []
    transport_root = root / TRANSPORT
    sources = rust_sources(root)
    source_text = {
        path.relative_to(root): path.read_text(encoding="utf-8") for path in sources
    }

    for relative, marker in REQUIRED_FILES.items():
        path = transport_root / relative
        if not path.is_file():
            failures.append(f"terminal transport module is missing: {TRANSPORT / relative}")
            continue
        if marker not in path.read_text(encoding="utf-8"):
            failures.append(f"terminal transport module lacks owned marker {marker}: {path.relative_to(root)}")

    for marker, owner in OWNERS.items():
        locations = [path for path, text in source_text.items() if marker in text]
        if locations != [owner]:
            failures.append(
                f"{marker} must occur only in {owner} (found: "
                f"{', '.join(map(str, locations)) or 'none'})"
            )

    for marker, owners in IDENTIFIER_OWNERS.items():
        pattern = re.compile(rf"\b{re.escape(marker)}\b")
        escaped = [
            path for path, text in source_text.items()
            if pattern.search(text) and path not in owners
        ]
        if escaped:
            failures.append(
                f"terminal transport ownership escaped for {marker}: "
                + ", ".join(map(str, escaped))
            )

    transport_sources = {
        path: text for path, text in source_text.items() if TRANSPORT in path.parents
    }
    joined_transport = "\n".join(transport_sources.values())
    for marker in SEMANTIC_TOKENS + THIRD_PARTY_TOKENS:
        if marker in joined_transport:
            failures.append(f"terminal transport contains forbidden authority marker: {marker}")

    outside_transport = {
        path: text for path, text in source_text.items() if TRANSPORT not in path.parents
    }
    pty_raw_ownership = (
        "openpty", "forkpty", "login_tty", "master_fd", "slave_fd",
    )
    for marker in pty_raw_ownership:
        escaped = [path for path, text in outside_transport.items() if marker in text]
        if escaped:
            failures.append(
                f"terminal transport ownership escaped for {marker}: "
                + ", ".join(map(str, escaped))
            )

    # Raw descriptors and readiness polling are ordinary OS mechanisms also
    # used by non-PTY subsystems (for example the owned AT-SPI D-Bus bridge).
    # Reject their escape only when the same source actually reaches into the
    # terminal transport/PTY boundary; otherwise this guard would claim
    # ownership over unrelated sockets.
    terminal_raw_context = (
        "terminal_transport", "open_pty_pair", "attach_child_pty",
        "/dev/ptmx", "TIOCSCTTY", "TIOCSWINSZ", "master_fd", "slave_fd",
    )
    for marker in ("RawFd", "OwnedFd", "AsRawFd", "FromRawFd"):
        escaped = [
            path
            for path, text in outside_transport.items()
            if marker in text and any(context in text for context in terminal_raw_context)
        ]
        if escaped:
            failures.append(
                f"terminal transport ownership escaped for {marker}: "
                + ", ".join(map(str, escaped))
            )

    handle = transport_sources.get(TRANSPORT / "session_handle.rs", "")
    handle_production = re.sub(
        r"#\[cfg\(test\)\]\s*impl\s+TerminalTransportSession\s*\{.*\}\s*$",
        "",
        handle,
        flags=re.DOTALL,
    )
    if re.search(r"#\[derive\([^]]*Clone", handle_production):
        failures.append("terminal transport owning handle must not be Clone")
    for struct_name in ("PreparedTerminalTransport", "TerminalTransportSession"):
        body_match = re.search(
            rf"struct\s+{struct_name}\s*\{{(?P<body>.*?)\n\}}", handle_production, re.DOTALL
        )
        if not body_match:
            failures.append(f"terminal transport handle is missing: {struct_name}")
            continue
        if re.search(r"(?m)^\s*pub(?:\([^)]*\))?\s+\w+\s*:", body_match.group("body")):
            failures.append(f"terminal transport handle fields must all be private: {struct_name}")

    allowed_methods = {
        "process_group_id", "start", "try_recv_event", "recv_event_timeout",
        "try_recv_control_event", "try_recv_output", "has_pending_event",
        "write_bytes", "terminate", "force_kill", "shutdown_snapshot", "resize",
        "terminate_by", "presentation_complete",
        "retry_termination_by",
    }
    public_functions = list(re.finditer(
        r"(?P<visibility>pub(?:\([^)]*\))?)\s+fn\s+(?P<name>\w+)\s*\(", handle_production
    ))
    constructors = [match for match in public_functions if match.group("name") == "new"]
    if len(constructors) != 2 or any(
        match.group("visibility") != "pub(super)" for match in constructors
    ):
        failures.append("terminal transport must have exactly two parent-private constructors")
    for match in public_functions:
        name = match.group("name")
        visibility = match.group("visibility")
        if name == "new":
            continue
        if name not in allowed_methods or visibility != "pub(crate)":
            failures.append(f"terminal transport exposes unapproved handle API: {visibility} fn {name}")
        signature_end = handle_production.find("{", match.end())
        signature = handle_production[match.start():signature_end if signature_end >= 0 else match.end()]
        if any(token in signature for token in ("File", "RawFd", "OwnedFd", "Receiver", "Arc<Mutex<File>>")):
            failures.append(f"terminal transport handle API exposes raw ownership: {name}")

    root_module = transport_sources.get(TRANSPORT / "mod.rs", "")
    if not re.search(r"(?m)^mod linux;$", root_module):
        failures.append("terminal transport Linux module must be private to its root")
    linux_sources = {
        path: text for path, text in transport_sources.items()
        if TRANSPORT / "linux" in path.parents
    }
    for path, text in linux_sources.items():
        if "pub(crate)" in text:
            failures.append(f"raw Linux transport API is broader than its root: {path}")
    for marker in ("prepare_terminal_transport", "open_pty_pair", "into_command", ".spawn()"):
        if marker not in root_module:
            failures.append(f"terminal transport root lacks real orchestration marker: {marker}")

    production_transport = "\n".join(
        text.split("#[cfg(test)]", 1)[0] for text in transport_sources.values()
    )
    for marker in ("mpsc::channel()", ".write_all(", ".flush(", "from_utf8"):
        if marker in production_transport:
            failures.append(f"terminal transport contains unbounded or semantic I/O marker: {marker}")
    reader = transport_sources.get(TRANSPORT / "reader.rs", "")
    if not (0 <= reader.find("output.reserve()") < reader.find("reader.read_bytes(")):
        failures.append("terminal reader must reserve owner-budgeted capacity before PTY read")
    for marker in (
        "reader_retries_eintr_and_would_block_without_losing_bytes_or_spinning",
        "reader_drains_hup_tail_then_accepts_correlated_eio_as_eof",
        "reader_reports_uncorrelated_eio_and_invalid_descriptor_once",
    ):
        if marker not in reader:
            failures.append(f"terminal reader lacks deterministic transition proof: {marker}")
    input_transport = transport_sources.get(TRANSPORT / "input.rs", "")
    control_transport = transport_sources.get(TRANSPORT / "control.rs", "")
    supervisor = transport_sources.get(TRANSPORT / "process_supervisor.rs", "")
    io_transport = transport_sources.get(TRANSPORT / "linux/io.rs", "")
    lifecycle_markers = {
        "input cancellation before PTY writes": (input_transport, "queue.is_closed()"),
        "writer completion presentation barrier": (control_transport, "writer_finished"),
        "atomic Kill transition": (supervisor, "fn begin_kill_phase"),
        "bounded writable cancellation poll": (io_transport, "wait_with_timeout(fd, libc::POLLOUT, 100)"),
    }
    for description, (text, marker) in lifecycle_markers.items():
        if marker not in text:
            failures.append(f"terminal transport lacks {description}: {marker}")
    proof_markers = {
        APP_SRC / "terminal_job_control_tests.rs":
            "termination_cancels_backpressured_input_and_closes_every_master",
        TRANSPORT / "process_supervisor_tests.rs":
            "redundant_force_during_kill_cannot_queue_an_unauthorized_retry",
    }
    for relative, marker in proof_markers.items():
        path = root / relative
        if not path.is_file() or marker not in path.read_text(encoding="utf-8"):
            failures.append(f"terminal transport lacks governed lifecycle proof: {marker}")
    session_handle = transport_sources.get(TRANSPORT / "session_handle.rs", "")
    if "try_enqueue(bytes.to_vec())" in session_handle:
        failures.append("terminal input must reject oversized slices before allocation")
    limits = transport_sources.get(TRANSPORT / "limits.rs", "")
    expected_limits = {
        "MAX_OUTPUT_CHUNKS": "256",
        "MAX_OUTPUT_CHUNK_BYTES": "16 * 1024",
        "MAX_OUTPUT_BYTES": "4 * 1024 * 1024",
        "MAX_INPUT_REQUESTS": "64",
        "MAX_INPUT_BYTES": "4 * 1024 * 1024",
        "MAX_LIVE_SESSIONS": "16",
        "GUI_DRAIN_EVENT_LIMIT": "128",
        "GUI_DRAIN_BYTE_LIMIT": "64 * 1024",
        "MAX_SESSION_MEMBERS": "4_096",
        "MAX_SESSION_GROUPS": "4_096",
    }
    for name, value in expected_limits.items():
        if not re.search(rf"const\s+{name}:\s*usize\s*=\s*{re.escape(value)}\s*;", limits):
            failures.append(f"owner-ratified terminal limit changed or missing: {name}")
    expected_deadlines = {
        "HUP_GRACE_MS": "2_000",
        "TERM_GRACE_MS": "2_000",
        "KILL_VERIFY_MS": "2_000",
        "GLOBAL_SHUTDOWN_MS": "6_000",
    }
    for name, value in expected_deadlines.items():
        if not re.search(rf"const\s+{name}:\s*u64\s*=\s*{re.escape(value)}\s*;", limits):
            failures.append(f"owner-ratified terminal deadline changed or missing: {name}")
    drain = source_text.get(APP_SRC / "terminal_session_drain.rs", "")
    drain_tests_path = root / APP_SRC / "terminal_session_drain_tests.rs"
    drain_tests = (
        drain_tests_path.read_text(encoding="utf-8")
        if drain_tests_path.is_file()
        else ""
    )
    for marker in ("fn drain_all", "next_drain_index", "try_recv_control_event", "try_recv_output"):
        if marker not in drain:
            failures.append(f"all-session terminal drain lacks fairness marker: {marker}")
    if "control_priority_round_robin_cursor_and_exact_global_caps_are_literal" not in drain_tests:
        failures.append("all-session terminal drain lacks literal L3 proof")
    isolation_path = root / APP_SRC / "terminal_session_p06_isolation_tests.rs"
    isolation = (
        isolation_path.read_text(encoding="utf-8") if isolation_path.is_file() else ""
    )
    isolation_markers = (
        "const SESSION_COUNT: usize = 8;",
        "eight_real_sessions_isolate_io_resize_exit_termination_and_restart",
        "DTC06B-peer-survived",
        "presentation_complete()",
        "all_sessions_closed()",
        "P06 must not recreate a detached PTY state",
    )
    for marker in isolation_markers:
        if marker not in isolation:
            failures.append(f"DTC-P06B eight-session isolation proof missing: {marker}")
    p06c_proofs = {
        TRANSPORT / "output.rs": (
            "full_output_backlog_blocks_reservation_until_consumer_pop",
        ),
        TRANSPORT / "input.rs": (
            "input_admission_is_atomic_at_request_and_byte_limits",
        ),
        APP_SRC / "terminal_session_drain_tests.rs": (
            "control_priority_round_robin_cursor_and_exact_global_caps_are_literal",
            "seventeenth_session_is_refused_by_preallocation_guard",
        ),
        APP_SRC / "terminal_session_p06_stress_tests.rs": (
            "const P06_LIFECYCLE_CYCLES: usize = 100;",
            "fn p06_resource_helper",
            'proc_entry_count("/proc/self/fd")',
            'proc_entry_count("/proc/self/task")',
            "session.presentation_complete()",
            "snapshot.leader_reaped",
            "snapshot.surviving_processes.is_empty()",
        ),
    }
    for relative, markers in p06c_proofs.items():
        path = root / relative
        proof = path.read_text(encoding="utf-8") if path.is_file() else ""
        for marker in markers:
            if marker not in proof:
                failures.append(f"DTC-P06C bounded resource proof missing: {marker}")
    io_log_path = root / APP_SRC / "terminal_io_event_log.rs"
    io_log = io_log_path.read_text(encoding="utf-8") if io_log_path.is_file() else ""
    p06d_storage_markers = (
        "pub(crate) const IO_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;",
        "pub(crate) const IO_SEGMENT_COUNT: usize = 4;",
        "fn rotate_segments",
        'event: "terminal_io_rotation"',
        "four_segments_rotate_oldest_first_and_metadata_records_the_fact",
    )
    for marker in p06d_storage_markers:
        if marker not in io_log:
            failures.append(f"DTC-P06D bounded terminal I/O log proof missing: {marker}")
    rotation_test_path = root / APP_SRC / "terminal_activity_snapshot_rotation_tests.rs"
    rotation_test = (
        rotation_test_path.read_text(encoding="utf-8")
        if rotation_test_path.is_file()
        else ""
    )
    if "rotated_io_family_is_folded_chronologically_without_replay" not in rotation_test:
        failures.append("DTC-P06D rotation-safe activity cache proof missing")
    measurement_path = root / APP_SRC / "terminal_session_p06_measurement_tests.rs"
    measurement = (
        measurement_path.read_text(encoding="utf-8")
        if measurement_path.is_file()
        else ""
    )
    for marker in (
        "const SESSION_COUNT: usize = 8;",
        "p06_release_measurement_emits_reproducible_json",
        'contract: "datum_terminal_p06_measurement_v1"',
        "measure_aggregate_output",
        "measure_latency_and_input",
        "measure_resize",
        'proc_count("/proc/self/fd")',
        'proc_count("/proc/self/task")',
    ):
        if marker not in measurement:
            failures.append(f"DTC-P06D release measurement proof missing: {marker}")
    gui_measurement_path = root / APP_SRC / "terminal_session_p06_gui_measurement_tests.rs"
    gui_measurement = (
        gui_measurement_path.read_text(encoding="utf-8")
        if gui_measurement_path.is_file()
        else ""
    )
    for marker in (
        "p06_provisional_gui_path_emits_reproducible_json",
        'contract: "datum_terminal_p06_gui_measurement_v1"',
        "OUTPUT_BYTES_PER_SESSION: usize = 1024 * 1024",
        "registry.drain_all",
        "drain_work: distribution",
        "presentation_complete",
    ):
        if marker not in gui_measurement:
            failures.append(f"DTC-P06D provisional GUI measurement proof missing: {marker}")
    lifecycle_path = root / APP_SRC / "terminal_session_p06_lifecycle_measurement_tests.rs"
    lifecycle = lifecycle_path.read_text(encoding="utf-8") if lifecycle_path.is_file() else ""
    for marker in (
        "const P06_LIFECYCLE_CYCLES: usize = 1_000;",
        "p06_one_thousand_spawn_exit_restart_cycles_emit_reproducible_json",
        'contract: "datum_terminal_p06_lifecycle_v1"',
        "restart_active",
        "presentation_complete",
        "snapshot.surviving_processes.is_empty()",
        'proc_count("/proc/self/fd")',
        'proc_count("/proc/self/task")',
    ):
        if marker not in lifecycle:
            failures.append(f"DTC-P06D lifecycle measurement proof missing: {marker}")
    soak_path = root / APP_SRC / "terminal_session_p06_soak_tests.rs"
    soak = soak_path.read_text(encoding="utf-8") if soak_path.is_file() else ""
    for marker in (
        "p06_bounded_ci_budgets_are_literal",
        "p06_bounded_ci_emits_reproducible_json",
        'contract: "datum_terminal_p06_soak_v1"',
        '"ci" => Self',
        "Duration::from_secs(10 * 60)",
        "minimum_bytes_per_session: 8 * 1024 * 1024",
        "resize_requests: 1_000",
        "input_bytes_per_session",
        "output_bytes_per_session",
        "aggregate_input_bytes",
        "aggregate_output_bytes",
        "WorkloadRole::Saturation",
        "restart_echo",
        "presentation_complete",
        'proc_count("/proc/self/fd")',
        'proc_count("/proc/self/task")',
    ):
        if marker not in soak:
            failures.append(f"DTC-P06D bounded CI proof missing: {marker}")
    for removed_tier in ('"single-24h"', '"max-4h"'):
        if removed_tier in soak:
            failures.append(
                f"DTC-P06D owner-removed long-duration tier returned: {removed_tier}"
            )
    sustained_path = root / APP_SRC / "terminal_session_p06_throughput_tests.rs"
    sustained = sustained_path.read_text(encoding="utf-8") if sustained_path.is_file() else ""
    for marker in (
        "p06_sustained_throughput_budgets_are_literal",
        "p06_sustained_throughput_and_backlog_emit_reproducible_json",
        'contract: "datum_terminal_p06_sustained_v1"',
        "Duration::from_secs(60)",
        "output_queued_bytes_for_test",
        "output_queued_chunks_for_test",
        "max_fairness_gap_us",
    ):
        if marker not in sustained:
            failures.append(f"DTC-P06D sustained throughput proof missing: {marker}")
    runner_path = root / "scripts/run_terminal_transport_proof_gates.sh"
    runner = runner_path.read_text(encoding="utf-8") if runner_path.is_file() else ""
    for marker in (
        "--release --locked --offline",
        "DTC-P06D evidence requires a clean revision-addressed worktree",
        "wayland-primary",
        "x11-fallback",
        "single throughput >=20MiB/s",
        "aggregate throughput >=40MiB/s",
        "backend-canary|smoke|gui|throughput-60s|lifecycle-1000|ci",
        "WINIT_UNIX_BACKEND=wayland",
        "WINIT_UNIX_BACKEND=x11",
        '"datum_terminal_p06_backend_canary_v1"',
        'grep -q "window created"',
        'grep -q "renderer init end"',
        'grep -q "window visible"',
        "DATUM_P06_RUN_ORDINAL",
        "p06_bounded_ci_emits_reproducible_json",
        '"datum_terminal_p06_soak_v1"',
        '"ci-10-minute": (8, 600, 8 * 1024 * 1024, 1_000)',
        '"datum_terminal_p06_gui_measurement_v1"',
        "single provisional GUI throughput >=1MiB/s",
        "aggregate provisional GUI throughput >=4MiB/s",
        '"datum_terminal_p06_lifecycle_v1"',
        "exactly 1000 completed lifecycle cycles",
        '"datum_terminal_p06_sustained_v1"',
        "single duration >=60s",
        "aggregate duration >=60s",
        "backlog fixture emits exact 4MiB burst",
        "backlog saturates a governed queue limit",
        "backlog high-water remains within 4MiB",
    ):
        if marker not in runner:
            failures.append(f"DTC-P06D release proof runner missing: {marker}")
    for removed_tier in ("single-24h", "max-4h"):
        if removed_tier in runner:
            failures.append(
                f"DTC-P06D proof runner reintroduced owner-removed tier: {removed_tier}"
            )
    if "active().try_recv_event" in "\n".join(outside_transport.values()):
        failures.append("active-only terminal event draining must not return")

    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        for failure in failures:
            print(f"terminal transport boundary: FAIL: {failure}", file=sys.stderr)
        return 1
    print("terminal transport boundary: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
