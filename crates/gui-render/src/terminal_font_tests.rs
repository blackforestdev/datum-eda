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
fn terminal_cell_advance_combines_smaller_ink_with_explicit_spacing() {
    let font_size = std::hint::black_box(bottom_dock::TERMINAL_FONT_SIZE_PX);
    let letter_spacing = std::hint::black_box(bottom_dock::TERMINAL_LETTER_SPACING_EM);
    assert!(
        font_size < datum_gui_viewport::TERMINAL_CELL_WIDTH_PX / 0.6,
        "terminal glyph ink must not consume the entire logical cell"
    );
    assert!(letter_spacing > 0.0);
    assert!(
        (font_size * (0.6 + letter_spacing) - datum_gui_viewport::TERMINAL_CELL_WIDTH_PX).abs()
            < 0.001
    );
}

#[test]
fn styled_terminal_colors_share_one_shaping_origin() {
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
    let lines = prepared
        .text_runs
        .iter()
        .filter(|run| run.face == TextFace::Terminal)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "a styled row must shape as one text buffer");
    let line = lines[0];
    assert_eq!(line.text, fixture);
    assert_eq!(
        line.rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>(),
        vec!["cursor", " gaps ", "must", " not grow"]
    );
    let visible_end = line.x
        + measured_text_run_width_px(
            fixture,
            bottom_dock::TERMINAL_FONT_SIZE_PX,
            TextFace::Terminal,
        );
    let logical_cursor_x =
        line.x + fixture.chars().count() as f32 * datum_gui_viewport::TERMINAL_CELL_WIDTH_PX;
    assert!(
        (visible_end - logical_cursor_x).abs() < 0.01,
        "logical cursor x {logical_cursor_x} must equal visible text end {visible_end}"
    );
}

#[test]
fn colored_shell_prompt_preserves_dollar_space_command_and_cursor_cells() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    let host = "bfadmin@debian3520";
    let path = ":/tmp/datum-eda/gui-imports/DOA2526-5825f90fe7490128";
    let command = " cd ~/Documents/datum-eda/";
    let fixture = format!("{host}{path}${command}");
    let host_end = host.chars().count();
    let path_end = host_end + path.chars().count();
    let dollar_end = path_end + 1;
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
    *state.ui.terminal.pty_grid_mut().lines = vec![fixture.clone()];
    *state.ui.terminal.pty_grid_mut().styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
        text: fixture.clone(),
        spans: vec![
            span(0, host_end, "green"),
            span(host_end, path_end, "blue"),
            span(path_end, dollar_end, "yellow"),
        ],
    }];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = fixture.chars().count();
    state.ui.terminal.screen_cursor_visible = true;

    let retained = RetainedScene::from_workspace(&state, 1200, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1200,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let lines = prepared
        .text_runs
        .iter()
        .filter(|run| run.face == TextFace::Terminal)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "the colored prompt must shape only once");
    let line = lines[0];
    assert_eq!(line.text, fixture);
    assert_eq!(
        line.rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>(),
        vec![host, path, "$", command]
    );
    assert!(
        line.rich_spans[3].text.starts_with(" cd"),
        "the real shell-emitted space after '$' must not be lost"
    );
    let visible_end =
        line.x + fixture.chars().count() as f32 * datum_gui_viewport::TERMINAL_CELL_WIDTH_PX;
    let logical_cursor_x = line.x
        + state.ui.terminal.screen_cursor_col as f32 * datum_gui_viewport::TERMINAL_CELL_WIDTH_PX;
    assert_eq!(visible_end, logical_cursor_x);
}

#[test]
fn prompt_style_boundaries_do_not_restart_glyph_positioning() {
    let mut font_system = FontSystem::new();
    load_datum_fonts(&mut font_system);
    let fixture = "bfadmin@debian3520:~/Documents/datum-eda$ cd ~/Documents/datum-eda/";
    let metrics = Metrics::new(
        bottom_dock::TERMINAL_FONT_SIZE_PX,
        bottom_dock::TERMINAL_FONT_SIZE_PX * 1.22,
    );
    let attrs = text_attrs(TextFace::Terminal);

    let mut plain = Buffer::new(&mut font_system, metrics);
    plain.set_text(
        &mut font_system,
        fixture,
        &attrs,
        Shaping::Basic,
        None,
    );
    plain.shape_until_scroll(&mut font_system, false);

    let mut rich = Buffer::new(&mut font_system, metrics);
    rich.set_rich_text(
        &mut font_system,
        [
            (
                "bfadmin@debian3520",
                attrs.clone().color(Color::rgb(115, 209, 122)),
            ),
            (
                ":~/Documents/datum-eda",
                attrs.clone().color(Color::rgb(107, 158, 242)),
            ),
            ("$", attrs.clone().color(Color::rgb(245, 199, 82))),
            (" cd ~/Documents/datum-eda/", attrs.clone()),
        ],
        &attrs,
        Shaping::Basic,
        None,
    );
    rich.shape_until_scroll(&mut font_system, false);

    let positions = |buffer: &Buffer| {
        buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter())
            .map(|glyph| {
                (
                    glyph.start,
                    glyph.end,
                    glyph.x.to_bits(),
                    glyph.w.to_bits(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        positions(&plain),
        positions(&rich),
        "ANSI color changes must not restart shaping or move prompt glyphs"
    );
    assert!(
        rich.layout_runs()
            .flat_map(|run| run.glyphs.iter())
            .any(|glyph| glyph.color_opt.is_some()),
        "rich prompt must retain per-span colors"
    );
}

#[test]
fn terminal_rich_span_colors_participate_in_the_buffer_cache_key() {
    let mut run = TextRun {
        text: "$ command".to_string(),
        rich_spans: vec![TextRunSpan {
            text: "$ command".to_string(),
            color: TEXT_PRIMARY,
        }],
        x: 0.0,
        y: 0.0,
        size: bottom_dock::TERMINAL_FONT_SIZE_PX,
        color: TEXT_PANEL_VALUE,
        face: TextFace::Terminal,
        clip_bounds: None,
    };
    let original = text_buffer_key(&run, 1280, 768);
    run.rich_spans[0].color = TEXT_SECONDARY;
    assert_ne!(
        original,
        text_buffer_key(&run, 1280, 768),
        "rich terminal colors are shaped into the cached buffer"
    );
}
