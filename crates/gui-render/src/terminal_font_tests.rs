use super::*;

#[test]
fn terminal_face_shapes_box_drawing_and_powerline_without_notdef() {
    let mut font_system = FontSystem::new();
    load_datum_fonts(&mut font_system);
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(11.0, 13.42));
    let fixture = "┌─┬─┐│├┼┤└─┴─┘\u{e0a0}\u{e0b0}\u{e0b1}";
    buffer.set_text(
        &mut font_system,
        fixture,
        &text_attrs(TextFace::Terminal),
        Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let glyphs = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .collect::<Vec<_>>();
    assert_eq!(glyphs.len(), fixture.chars().count());
    assert!(glyphs.iter().all(|glyph| glyph.glyph_id != 0));
    assert!(glyphs.iter().all(|glyph| {
        font_system.db().face(glyph.font_id).is_some_and(|face| {
            face.families
                .iter()
                .any(|(family, _)| family == "JetBrains Mono")
        })
    }));
}

#[test]
fn terminal_cell_runs_use_terminal_face_without_changing_chrome_mono() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state
        .ui
        .terminal
        .pty_grid_mut()
        .lines
        .push("┌─ agent ─┐".to_string());
    let retained = RetainedScene::from_workspace(&state, 1200, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1200,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let terminal_runs = prepared
        .text_runs
        .iter()
        .filter(|run| run.text.contains("┌─ agent ─┐"))
        .collect::<Vec<_>>();
    assert!(!terminal_runs.is_empty());
    assert!(
        terminal_runs
            .iter()
            .all(|run| run.face == TextFace::Terminal)
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.face == TextFace::Mono)
    );
}

#[test]
fn terminal_font_advance_matches_shared_logical_cell_width() {
    let fixture = "the cursor remains aligned across this long agent prompt";
    let shaped_width = measured_text_run_width_px(
        fixture,
        bottom_dock::TERMINAL_FONT_SIZE_PX,
        TextFace::Terminal,
    );
    let logical_width = fixture.chars().count() as f32 * datum_gui_viewport::TERMINAL_CELL_WIDTH_PX;

    assert!(
        (shaped_width - logical_width).abs() < 0.01,
        "terminal shaping width {shaped_width} must equal logical cell width {logical_width}"
    );
}

#[test]
fn styled_terminal_fragments_share_contiguous_cell_origins() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    let fixture = "cursor gaps must not grow";
    let span = |start, end, fg: &str| datum_gui_protocol::TerminalStyleSpan {
        start,
        end,
        fg: Some(fg.to_string()),
        bg: None,
        bold: false,
        dim: false,
        italic: false,
        underline: false,
        overline: false,
        blink: false,
        strikethrough: false,
        conceal: false,
        inverse: false,
    };
    *state.ui.terminal.pty_grid_mut().lines = vec![fixture.to_string()];
    *state.ui.terminal.pty_grid_mut().styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
        text: fixture.to_string(),
        spans: vec![span(0, 6, "green"), span(12, 16, "yellow")],
    }];

    let retained = RetainedScene::from_workspace(&state, 1200, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1200,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let fragments = prepared
        .text_runs
        .iter()
        .filter(|run| run.face == TextFace::Terminal)
        .collect::<Vec<_>>();
    assert_eq!(
        fragments
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>(),
        vec!["cursor", " gaps ", "must", " not grow"]
    );

    for pair in fragments.windows(2) {
        let left = pair[0];
        let right = pair[1];
        let shaped_end = left.x + measured_text_run_width_px(&left.text, left.size, left.face);
        assert!(
            (right.x - shaped_end).abs() < 0.01,
            "styled split introduced a gap: {:?} ends at {shaped_end}, {:?} starts at {}",
            left.text,
            right.text,
            right.x
        );
    }

    let first = fragments[0];
    let visible_end = first.x
        + measured_text_run_width_px(
            fixture,
            bottom_dock::TERMINAL_FONT_SIZE_PX,
            TextFace::Terminal,
        );
    let logical_cursor_x =
        first.x + fixture.chars().count() as f32 * datum_gui_viewport::TERMINAL_CELL_WIDTH_PX;
    assert!(
        (visible_end - logical_cursor_x).abs() < 0.01,
        "logical cursor x {logical_cursor_x} must equal visible text end {visible_end}"
    );
}
