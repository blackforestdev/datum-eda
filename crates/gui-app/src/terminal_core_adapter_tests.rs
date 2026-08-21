use super::*;
use datum_terminal_core::{CellAttribute, Color, InputDisposition, PaletteIndex, SearchQuery};

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

    assert_eq!(adapter.test_plain_lines()[0], "A\u{754c}");
    let snapshot = adapter.test_render_snapshot();
    let style = snapshot.rows().next().unwrap().cells()[0].style;
    assert_eq!(style.foreground, Color::Indexed(PaletteIndex::new(1)));
    assert_eq!(style.background, Color::Indexed(PaletteIndex::new(4)));
    assert!(style.attributes.contains(CellAttribute::Bold));
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
fn terminal_replies_are_emitted_once_and_never_enter_terminal_cells() {
    let mut adapter = adapter("session-reply");
    let mut lane = TerminalLaneState::default();
    let update = adapter.apply_output(&mut lane, b"ok\x1b[6n").unwrap();
    assert_eq!(adapter.test_plain_lines()[0], "ok");
    assert_eq!(update.replies.len(), 1);
    assert!(update.replies[0].starts_with(b"\x1b["));
    assert!(!adapter.test_plain_lines()[0].contains('['));
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

    assert_eq!(left.test_plain_lines()[0], "LEFT");
    assert_eq!(right.test_plain_lines()[0], "RIGHT");
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
    assert_eq!(adapter.test_plain_lines()[0], "tail ");
    let update = adapter.finish(&mut lane).unwrap();
    assert!(update.semantic_errors.is_empty());
    assert_eq!(adapter.test_plain_lines()[0], "tail \u{fffd}");
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

#[test]
fn incoming_output_preserves_the_users_scrollback_anchor() {
    let mut adapter = adapter("stable-scrollback");
    let mut lane = TerminalLaneState {
        scroll_offset: 17,
        ..TerminalLaneState::default()
    };

    adapter
        .apply_output(&mut lane, b"background output\r\n")
        .unwrap();

    assert_eq!(lane.scroll_offset, 17);
}

#[test]
fn search_match_resolves_to_a_stable_scrollback_window() {
    let mut adapter = adapter("search-reveal");
    let mut lane = TerminalLaneState::default();
    for index in 0..12 {
        adapter
            .apply_output(&mut lane, format!("line-{index}\r\n").as_bytes())
            .unwrap();
    }
    let matched = adapter
        .search_all(&SearchQuery::literal(
            "line-2",
            datum_terminal_core::SearchCase::Sensitive,
        ))
        .unwrap()
        .matches()[0];
    let scroll = adapter
        .scroll_offset_for_logical_point(4, matched.start())
        .unwrap()
        .unwrap();

    assert!(
        scroll > 0,
        "an old match must move the viewport into history"
    );
}

#[test]
fn native_input_selection_search_and_links_use_the_same_core_state() {
    use datum_terminal_core::{
        FocusInput, ImeInput, KeyCode, KeyEventKind, KeyInput, KeyModifiers, MouseAction,
        MouseButton, MouseInput, MousePosition, SearchCase, SelectionScope,
    };

    let mut adapter = adapter("native-interaction");
    let mut lane = TerminalLaneState::default();
    adapter
        .apply_output(
            &mut lane,
            b"\x1b[?1004h\x1b[?2004h\x1b[?1000h\x1b[?1006h\x1b]8;;https://example.com\x07linked\x1b]8;;\x07 text",
        )
        .unwrap();

    let key = KeyInput {
        code: KeyCode::Tab,
        shifted_key: None,
        base_layout_key: None,
        modifiers: KeyModifiers::default(),
        kind: KeyEventKind::Press,
    };
    assert_eq!(adapter.encode_key(&key).unwrap().bytes(), Some(&b"\t"[..]));
    assert_eq!(
        adapter.encode_focus(FocusInput::Gained).unwrap().bytes(),
        Some(&b"\x1b[I"[..])
    );
    assert_eq!(
        adapter
            .encode_ime(&ImeInput::Preedit("draft".into()))
            .unwrap(),
        InputDisposition::LocalOnly
    );
    assert_eq!(
        adapter.encode_paste("a\nb").unwrap().bytes(),
        Some(&b"\x1b[200~a\nb\x1b[201~"[..])
    );
    assert_eq!(
        adapter
            .encode_mouse(MouseInput {
                action: MouseAction::Press(MouseButton::Left),
                position: MousePosition {
                    column: 2,
                    row: 1,
                    pixel_x: 20,
                    pixel_y: 16,
                },
                modifiers: KeyModifiers::default(),
                local_override: false,
            })
            .unwrap()
            .bytes(),
        Some(&b"\x1b[<0;3;2M"[..])
    );

    let start = adapter
        .logical_point_at_visible_cell(4, 0, 0, 0)
        .unwrap()
        .unwrap();
    let end = adapter
        .logical_point_at_visible_cell(4, 0, 0, 5)
        .unwrap()
        .unwrap();
    adapter
        .set_selection(start, end, SelectionScope::Grapheme)
        .unwrap();
    assert_eq!(adapter.copy_selection().unwrap(), "linked");

    let result = adapter
        .search_all(&SearchQuery::literal("text", SearchCase::Sensitive))
        .unwrap();
    assert!(!result.matches().is_empty());
    assert_eq!(
        adapter
            .hyperlink_at_visible_cell(4, 0, 0, 2)
            .unwrap()
            .map(|(_, uri)| uri),
        Some("https://example.com".into())
    );
    let accessibility = adapter.accessibility_snapshot(4, 0, true).unwrap();
    assert_eq!(accessibility.session_id, "native-interaction");
    assert!(accessibility.text.starts_with("linked text"));
    assert_eq!(accessibility.selection, Some((0, 6)));
    assert_eq!(accessibility.links.len(), 1);
    assert_eq!(accessibility.links[0].start, 0);
    assert_eq!(accessibility.links[0].end, 6);
    assert_eq!(accessibility.links[0].uri, "https://example.com");
    assert!(accessibility.focused);
    assert!(accessibility.caret <= accessibility.text.chars().count());
}
