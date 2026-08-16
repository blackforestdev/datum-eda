//! Screen-authority regression for T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) and
//! decision 027 FT-001: only PTY bytes interpreted by the terminal core may
//! mutate terminal cells. Session lifecycle and GUI events route their
//! narration to the console sink, never the grid.

use super::*;
use std::fs;
use std::time::{Duration, Instant};

/// Non-PTY phrases that must never appear in terminal cells (T0-C01 /
/// decision 027 FT-001) — shared by the lifecycle regression, the T0-C03
/// production-path canary, and the T0-C04 regression boundary
/// (`terminal_regression_boundary_tests.rs`). One entry per narration-producing
/// event class: session lifecycle, clipboard, PTY/transport failures, activity
/// telemetry, pan/diagnostic traces, and production-status refresh. Substring
/// matched against grid rows, so a phrase covers every message containing it
/// (e.g. "terminal session" covers open/rename/detach/close/ended notices).
pub(super) const DATUM_LIFECYCLE_PHRASES: [&str; 23] = [
    "opened terminal session",
    "terminal restarted",
    "renamed active terminal session",
    "detached active terminal session",
    "terminal session",
    "activity summary",
    "workspace scene/status refreshed",
    "terminal write failed",
    "terminal restart failed",
    "terminal interrupt failed",
    "terminal exited",
    "terminated by signal",
    "clipboard",
    "scrollback copied",
    "production status refresh failed",
    "pan key physical=",
    "pan primary pressed",
    "terminal resize",
    "queued authoring command",
    "terminal status response failed",
    "terminal handoff prepare failed",
    "terminal mouse report failed",
    "terminal focus report failed",
];

#[test]
fn terminal_grid_holds_only_pty_rows_across_session_lifecycle_events() {
    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001 regression:
    // the terminal grid may be written only by PTY bytes interpreted by the
    // terminal core. GUI/session lifecycle events that historically injected
    // notice rows (open, restart, detach, close, tab sync) must leave the
    // grid byte-identical.
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-screen-authority-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let mut state = TerminalLaneState::default();

    // The grid starts empty — no seeded "ready" rows.
    assert!(
        state.grid_lines().is_empty() && state.grid_styled_lines().is_empty(),
        "terminal grid must start empty; only PTY output may create rows"
    );

    // The one legal writer: PTY bytes interpreted by the terminal core.
    let mut screen = crate::terminal_screen::TerminalScreen::default();
    screen.apply_bytes(
        &mut state,
        b"datum$ printf t0-canary\r\nt0-canary\r\ndatum$ ",
    );
    let pty_rows = state.grid_lines().to_vec();
    assert!(
        pty_rows.iter().any(|line| line.contains("t0-canary")),
        "PTY-derived canary rows should be visible in the grid"
    );

    // T0-C02: the renderer draws the tail `geometry.rows` grid rows from the
    // SAME shared geometry that sizes the PTY, so at the default dock every
    // canary row above falls inside the drawn range.
    let shell = datum_gui_render::ShellLayout::for_surface(1280, 800, 1.0, Some(220));
    let geometry = datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into());
    assert!(
        pty_rows.len() <= geometry.rows as usize,
        "real PTY output rows ({}) must lie within the drawn row range ({})",
        pty_rows.len(),
        geometry.rows
    );

    // Session lifecycle and GUI-side refresh events that previously wrote the
    // grid.
    registry.sync_lane_tabs(&mut state);
    registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session");
    registry.sync_lane_tabs(&mut state);
    assert!(registry.resize_active(101, 29).is_ok());
    registry
        .restart_active(&mut state, &context)
        .expect("restart active terminal session");
    registry
        .close_active(&mut state)
        .expect("arm active terminal close without touching PTY cells");

    assert_eq!(
        state.grid_lines(),
        pty_rows,
        "session lifecycle events must not add, remove, or edit terminal grid rows"
    );
    for line in state.grid_lines() {
        for phrase in DATUM_LIFECYCLE_PHRASES {
            assert!(
                !line.contains(phrase),
                "terminal grid row {line:?} carries non-PTY lifecycle text {phrase:?}"
            );
        }
    }
    let _ = fs::remove_dir_all(&root);
}

/// Accumulated per-chunk costs of the production drain body, printed as raw
/// evidence for the terminal performance packet (owner hands-on 2026-08-14:
/// terminal is "extremely sluggish").
#[derive(Default)]
struct DrainCost {
    chunks: usize,
    bytes: usize,
    event_log_append: Duration,
    activity_summary: Duration,
    apply_bytes: Duration,
    max_chunk: Duration,
}

/// Pump PTY output through the exact per-batch sequence of
/// `Runtime::poll_terminal_output` (`production_status_refresh.rs`):
/// per chunk `record_terminal_output_event`, then per drain batch
/// `apply_bytes_with_responses` -> response write-back -> ONE incremental
/// activity-summary refresh (terminal performance slice: summary refresh and
/// frame invalidation are batch-level, and the summary read is O(new log
/// bytes) through the per-session cache). `Runtime` itself owns a live wgpu
/// surface and cannot be constructed in a unit test, so the canary drives the
/// same production functions in the same order against the same session
/// registry, screen, and lane state; each received chunk is treated as its
/// own drain batch (the worst case for batch-level costs). Returns true once
/// `stop` observes the goal state after a drained batch (grid state only
/// changes on batch application).
fn drain_production_path(
    registry: &mut TerminalSessionRegistry,
    state: &mut TerminalLaneState,
    cost: &mut DrainCost,
    response_bytes_written: &mut usize,
    deadline: Instant,
    stop: &mut dyn FnMut(&TerminalLaneState) -> bool,
) -> bool {
    while Instant::now() < deadline {
        let Ok(event) = registry
            .active()
            .recv_event_timeout(Duration::from_millis(25))
        else {
            continue;
        };
        let TerminalEvent::Output(bytes) = event else {
            return false;
        };
        let chunk_started = Instant::now();
        let step = Instant::now();
        let _ =
            crate::terminal_session_events::record_terminal_output_event(registry.active(), &bytes);
        cost.event_log_append += step.elapsed();
        let step = Instant::now();
        let responses = registry
            .active_screen_mut()
            .apply_bytes_with_responses(state, &bytes);
        cost.apply_bytes += step.elapsed();
        for response in responses {
            *response_bytes_written += response.len();
            let _ = registry.active().write_bytes(&response);
        }
        // poll_terminal_output refreshes the activity summary once per drain
        // batch through the incremental per-session cache.
        let step = Instant::now();
        let _ = registry.active_activity_summary_lines(4);
        cost.activity_summary += step.elapsed();
        cost.chunks += 1;
        cost.bytes += bytes.len();
        cost.max_chunk = cost.max_chunk.max(chunk_started.elapsed());
        if stop(state) {
            return true;
        }
    }
    false
}

#[test]
fn production_real_shell_canary_proves_ordered_visible_output_and_exact_once_input() {
    // T0-C03 (DATUM_NATIVE_TERMINAL_SPEC.md §7.1) / decision 027 T0 gate:
    // launch a deterministic REAL shell through the production session
    // machinery, send a unique command through the production input path,
    // pump output through the production drain body, and prove from
    // renderer-facing state that:
    //   (a) the echoed command and its output are visible IN ORDER as grid
    //       rows;
    //   (b) both rows fall inside the drawn tail window of the SAME shared
    //       geometry that sizes the PTY (T0-C02: visible, not just stored);
    //   (c) every typed byte reached the child exactly once, proven from the
    //       session's input accounting (no duplicates, no drops);
    //   (d) no Datum lifecycle/diagnostic phrase entered the grid (T0-C01).
    // The canary also measures input-to-grid latency and per-chunk drain
    // costs as raw numbers for the performance packet; it asserts nothing
    // about them.
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-t0c03-canary-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal canary test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn production terminal session");
    let mut state = TerminalLaneState::default();

    // T0-C02: rows/columns for BOTH the renderer and the PTY come from the one
    // shared geometry solver, exactly as resize_terminal_to_dock derives them.
    let shell = datum_gui_render::ShellLayout::for_surface(1280, 800, 1.0, Some(220));
    let geometry = datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into());
    registry
        .resize_active(geometry.columns, geometry.rows)
        .expect("size PTY from the shared terminal screen geometry");
    registry.sync_lane_tabs(&mut state);
    assert_eq!(
        (state.columns, state.rows),
        (geometry.columns, geometry.rows),
        "lane state must carry the shared-geometry PTY size"
    );

    let mut cost = DrainCost::default();
    let mut response_bytes_written = 0usize;

    // Phase 1: wait for the real shell prompt to become visible, then let
    // start-up output settle so the command echo lands on the live prompt row.
    let prompt_visible = drain_production_path(
        &mut registry,
        &mut state,
        &mut cost,
        &mut response_bytes_written,
        Instant::now() + Duration::from_secs(8),
        &mut |state| !state.grid_lines().is_empty(),
    );
    assert!(
        prompt_visible,
        "real shell prompt output must become visible in the terminal grid"
    );
    let _ = drain_production_path(
        &mut registry,
        &mut state,
        &mut cost,
        &mut response_bytes_written,
        Instant::now() + Duration::from_millis(400),
        &mut |_| false,
    );

    // Phase 2: the unique canary command. printf assembles the output line
    // from '%s', so the expected OUTPUT text never appears literally in the
    // echoed INPUT line — echo row and output row stay distinguishable.
    let nonce = format!(
        "t0c03-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    );
    let expected_output = format!("{nonce}-canary-out");
    let command = format!("printf '%s-canary-out\\n' {nonce}\r");
    let input_written = Instant::now();
    registry
        .active()
        .write_bytes(command.as_bytes())
        .expect("write canary command through the production input path");
    let mut first_chunk_after_input: Option<Duration> = None;
    let output_visible = drain_production_path(
        &mut registry,
        &mut state,
        &mut cost,
        &mut response_bytes_written,
        Instant::now() + Duration::from_secs(8),
        &mut |state| {
            if first_chunk_after_input.is_none() {
                first_chunk_after_input = Some(input_written.elapsed());
            }
            state
                .grid_lines()
                .iter()
                .any(|line| line.trim() == expected_output)
        },
    );
    let input_to_visible = input_written.elapsed();
    assert!(
        output_visible,
        "canary output {expected_output:?} must become visible in the terminal grid; rows: {:?}",
        state.grid_lines()
    );

    // (a) Ordering: the echoed input row precedes the output row.
    let lines = state.grid_lines();
    let echo_row = lines
        .iter()
        .position(|line| line.contains("printf") && line.contains(&nonce))
        .expect("echoed canary command must be visible as a grid row");
    let output_rows = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == expected_output)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        output_rows.len(),
        1,
        "canary output must appear exactly once in the grid (exactly-once execution)"
    );
    assert!(
        echo_row < output_rows[0],
        "echoed input row {echo_row} must precede output row {} (ordered prompt/output)",
        output_rows[0]
    );

    // (b) Visibility: the renderer draws the TAIL geometry.rows grid rows from
    // the same shared geometry, so both rows must lie inside that window.
    let first_drawn_row = lines.len().saturating_sub(geometry.rows as usize);
    assert!(
        echo_row >= first_drawn_row && output_rows[0] >= first_drawn_row,
        "canary rows (echo {echo_row}, output {}) must fall inside the drawn tail window \
         starting at row {first_drawn_row} ({} grid rows, {} drawn)",
        output_rows[0],
        lines.len(),
        geometry.rows
    );

    // (c) Exactly-once input delivery, from the session's input accounting:
    // the only PTY writers were this canary command and the tracked terminal
    // status responses, so the recorded input byte total must match exactly
    // (no drops), and exactly one input event may carry the canary command
    // (no duplicates).
    let event_log = fs::read_to_string(registry.active_event_log_path())
        .expect("read terminal session event log");
    let mut input_bytes_total = 0usize;
    let mut nonce_input_events = 0usize;
    for line in event_log.lines().filter(|line| !line.trim().is_empty()) {
        let event: serde_json::Value =
            serde_json::from_str(line).expect("terminal event log line should parse");
        if event["event"] == "terminal_io" && event["direction"] == "input_accepted" {
            input_bytes_total += event["byte_count"].as_u64().unwrap_or(0) as usize;
            if event["text_preview"]
                .as_str()
                .is_some_and(|preview| preview.contains(&nonce))
            {
                nonce_input_events += 1;
            }
        }
    }
    assert_eq!(
        nonce_input_events, 1,
        "exactly one recorded input event may carry the canary command"
    );
    assert_eq!(
        input_bytes_total,
        command.len() + response_bytes_written,
        "recorded input bytes must equal written bytes exactly (no duplicated or dropped input)"
    );

    // (d) T0-C01 re-assertion: no Datum lifecycle/diagnostic phrase in cells.
    for line in state.grid_lines() {
        for phrase in DATUM_LIFECYCLE_PHRASES {
            assert!(
                !line.contains(phrase),
                "terminal grid row {line:?} carries non-PTY lifecycle text {phrase:?}"
            );
        }
    }

    // Phase 3 (measurement only): a sustained-output probe so per-chunk drain
    // costs are measured under real PTY throughput, then print raw numbers.
    let interactive_chunks = cost.chunks;
    let interactive_bytes = cost.bytes;
    registry
        .active()
        .write_bytes(b"seq 1 2000\r")
        .expect("write throughput probe command");
    let probe_started = Instant::now();
    let probe_done = drain_production_path(
        &mut registry,
        &mut state,
        &mut cost,
        &mut response_bytes_written,
        Instant::now() + Duration::from_secs(15),
        &mut |state| state.grid_lines().iter().any(|line| line.trim() == "2000"),
    );
    let probe_elapsed = probe_started.elapsed();
    let probe_chunks = cost.chunks - interactive_chunks;
    let probe_bytes = cost.bytes - interactive_bytes;

    // Phase 4 (measurement only): the sustained 20k probe that could not
    // finish before the terminal performance slice (quadratic per-chunk
    // activity-summary reload + per-chunk full-scene invalidation).
    let pre_20k_chunks = cost.chunks;
    let pre_20k_bytes = cost.bytes;
    registry
        .active()
        .write_bytes(b"seq 1 20000\r")
        .expect("write sustained 20k throughput probe command");
    let probe_20k_started = Instant::now();
    let probe_20k_done = drain_production_path(
        &mut registry,
        &mut state,
        &mut cost,
        &mut response_bytes_written,
        Instant::now() + Duration::from_secs(60),
        &mut |state| state.grid_lines().iter().any(|line| line.trim() == "20000"),
    );
    let probe_20k_elapsed = probe_20k_started.elapsed();
    let probe_20k_chunks = cost.chunks - pre_20k_chunks;
    let probe_20k_bytes = cost.bytes - pre_20k_bytes;

    let event_log_len = fs::metadata(registry.active_event_log_path())
        .map(|meta| meta.len())
        .unwrap_or(0);
    eprintln!("T0-C03 latency/perf raw data (performance packet input, not asserted):");
    eprintln!(
        "  input->first PTY chunk drained: {:?}; input->output visible in grid: {:?}",
        first_chunk_after_input.unwrap_or_default(),
        input_to_visible
    );
    eprintln!(
        "  interactive phase: {interactive_chunks} chunks; throughput probe (seq 1 2000): \
         done={probe_done} in {probe_elapsed:?} ({probe_chunks} chunks, {probe_bytes} bytes)"
    );
    eprintln!(
        "  sustained probe (seq 1 20000): done={probe_20k_done} in {probe_20k_elapsed:?} \
         ({probe_20k_chunks} chunks, {probe_20k_bytes} bytes)"
    );
    eprintln!(
        "  drain totals: {} chunks, {} bytes; per-step totals: event-log append {:?}, \
         activity-summary refresh {:?}, apply_bytes {:?}; worst single chunk {:?}",
        cost.chunks,
        cost.bytes,
        cost.event_log_append,
        cost.activity_summary,
        cost.apply_bytes,
        cost.max_chunk
    );
    eprintln!(
        "  event log at end: {event_log_len} bytes (activity-summary refresh is incremental — \
         O(new bytes) via the per-session cache; poll_terminal_output applies one coalesced \
         byte batch and invalidates the frame once per drain batch)"
    );
    let _ = fs::remove_dir_all(&root);
}
