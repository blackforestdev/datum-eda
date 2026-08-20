use super::*;

fn adapter(session: &str) -> TerminalCoreSessionAdapter {
    TerminalCoreSessionAdapter::new(session, format!("context-{session}"), 12, 4).unwrap()
}

#[test]
fn production_profile_matches_the_owner_approved_p22_l1_values() {
    assert_eq!(
        PRODUCTION_CORE_LIMIT_VALUES,
        CoreLimitValues {
            parameter_count: 64,
            parameter_digits: 16,
            parameter_value: 1_000_000,
            subparameter_count: 64,
            intermediate_bytes: 16,
            control_string_bytes: 16 * 1024 * 1024,
            cluster_bytes: 4_096,
            title_bytes: 32_768,
            working_directory_bytes: 65_536,
            clipboard_bytes: 4 * 1024 * 1024,
            hyperlink_bytes: 1024 * 1024,
            input_bytes: 4 * 1024 * 1024,
            keyboard_stack: 32,
            notification_bytes: 65_536,
            reply_bytes: 65_536,
            pending_events: 4_096,
            pending_damage: 4_096,
            history_lines: 100_000,
            history_bytes: 64 * 1024 * 1024,
            graphic_objects: 256,
            graphic_pixels: 16_777_216,
            graphic_decoded_bytes: 64 * 1024 * 1024,
            graphic_frames: 1_024,
            compression_ratio: 1_024,
            parser_work: 67_108_864,
            search_work: 67_108_864,
            reflow_work: 67_108_864,
            screen_cells: 1_048_576,
            snapshot_cells: 33_554_432,
        }
    );
}

#[test]
fn adapter_keeps_session_context_identity_and_projects_core_state() {
    let mut adapter = adapter("session-a");
    let mut lane = TerminalLaneState::default();
    assert_eq!(adapter.session_id(), "session-a");
    assert_eq!(adapter.context_id(), "context-session-a");

    let first = adapter
        .apply_output(&mut lane, b"\x1b[31;44;1mA\x1b[0m\xe7\x95")
        .unwrap();
    assert!(first.semantic_errors.is_empty());
    adapter
        .apply_output(
            &mut lane,
            b"\x8c\x1b]2;agent session\x07\x1b]7;file:///tmp/project\x07\x1b[?2004h\x1b[?1003h\x1b[?1006h",
        )
        .unwrap();

    assert_eq!(lane.grid_lines()[0], "A\u{754c}");
    assert_eq!(lane.grid_styled_lines()[0].text, "A\u{754c}");
    assert_eq!(lane.grid_styled_lines()[0].spans.len(), 1);
    let style = &lane.grid_styled_lines()[0].spans[0];
    assert_eq!(style.fg.as_deref(), Some("ansi256:1"));
    assert_eq!(style.bg.as_deref(), Some("ansi256:4"));
    assert!(style.bold);
    assert_eq!(lane.title.as_deref(), Some("agent session"));
    assert_eq!(
        lane.current_working_directory.as_deref(),
        Some("file:///tmp/project")
    );
    assert!(adapter.bracketed_paste_enabled());
    assert_eq!(lane.mouse_reporting_mode.as_deref(), Some("any_event"));
    assert_eq!(lane.mouse_coordinate_encoding.as_deref(), Some("sgr"));
    assert_eq!(lane.screen_cursor_col, 3);
}

#[test]
fn terminal_replies_are_emitted_once_and_never_enter_the_grid() {
    let mut adapter = adapter("session-reply");
    let mut lane = TerminalLaneState::default();
    let update = adapter.apply_output(&mut lane, b"ok\x1b[6n").unwrap();
    assert_eq!(lane.grid_lines()[0], "ok");
    assert_eq!(update.replies.len(), 1);
    assert!(update.replies[0].starts_with(b"\x1b["));
    assert!(!lane.grid_lines()[0].contains('['));
}

#[test]
fn adapters_isolate_output_modes_resize_and_context() {
    let mut left = adapter("left");
    let mut right = adapter("right");
    let mut left_lane = TerminalLaneState::default();
    let mut right_lane = TerminalLaneState::default();

    left.apply_output(&mut left_lane, b"LEFT\x1b[?2004h")
        .unwrap();
    right
        .apply_output(&mut right_lane, b"RIGHT\x1b[?1002h")
        .unwrap();
    left.resize(20, 5, 200, 100).unwrap();
    left.project(&mut left_lane).unwrap();

    assert_eq!(left_lane.grid_lines()[0], "LEFT");
    assert_eq!(right_lane.grid_lines()[0], "RIGHT");
    assert!(left.bracketed_paste_enabled());
    assert!(!right.bracketed_paste_enabled());
    assert_eq!(
        right_lane.mouse_reporting_mode.as_deref(),
        Some("button_event")
    );
    assert_eq!((left_lane.columns, left_lane.rows), (20, 5));
    assert_eq!((right_lane.columns, right_lane.rows), (12, 4));
    assert_eq!(left.context_id(), "context-left");
    assert_eq!(right.context_id(), "context-right");
}

#[test]
fn render_state_preserves_surface_pixels_and_consumes_damage_once() {
    let mut adapter = adapter("render-state");
    let mut lane = TerminalLaneState::default();
    adapter.apply_output(&mut lane, b"pixel-aware").unwrap();
    adapter.resize(20, 5, 200, 100).unwrap();

    let (snapshot, first_damage) = adapter.take_render_state().unwrap();
    assert_eq!(snapshot.size().pixels.width, 200);
    assert_eq!(snapshot.size().pixels.height, 100);
    assert_eq!(first_damage, vec![Damage::Full]);

    let (_, second_damage) = adapter.take_render_state().unwrap();
    assert!(second_damage.is_empty());
    adapter.apply_output(&mut lane, b"!").unwrap();
    let (_, incremental_damage) = adapter.take_render_state().unwrap();
    assert!(!incremental_damage.is_empty());
    assert!(!incremental_damage.contains(&Damage::Full));
}

#[test]
fn stream_finish_repairs_incomplete_utf8_before_lifecycle_completion() {
    let mut adapter = adapter("finish");
    let mut lane = TerminalLaneState::default();
    adapter.apply_output(&mut lane, b"tail \xe2\x82").unwrap();
    assert_eq!(lane.grid_lines()[0], "tail ");
    let update = adapter.finish(&mut lane).unwrap();
    assert!(update.semantic_errors.is_empty());
    assert_eq!(lane.grid_lines()[0], "tail \u{fffd}");
}

#[test]
fn repeated_bells_remain_bounded_but_preserve_visible_count() {
    let mut adapter = adapter("bells");
    let mut lane = TerminalLaneState::default();
    let bells = vec![0x07; PRODUCTION_CORE_LIMIT_VALUES.pending_events + 17];
    let update = adapter.apply_output(&mut lane, &bells).unwrap();
    assert_eq!(lane.bell_count, bells.len());
    assert_eq!(
        update.events.len(),
        PRODUCTION_CORE_LIMIT_VALUES.pending_events
    );
    assert!(
        update
            .events
            .iter()
            .any(|event| matches!(event, CoreEvent::LimitReached(LimitKind::PendingEvents)))
    );
}
