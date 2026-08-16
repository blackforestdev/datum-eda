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
