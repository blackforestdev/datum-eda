use super::*;
use crate::TerminalRenderCache;
use datum_gui_protocol::{ApplicationFocus, DockTab};
use datum_gui_viewport::ScreenRectPx;
use datum_terminal_core::{
    CoreLimitValues, CoreLimits, Damage, Row, Rows, StreamingParser, TerminalCore, TerminalSize,
};

fn limits() -> CoreLimits {
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: 64,
        parameter_digits: 16,
        parameter_value: 1_000_000,
        subparameter_count: 64,
        intermediate_bytes: 16,
        control_string_bytes: 1 << 20,
        cluster_bytes: 4_096,
        title_bytes: 4_096,
        working_directory_bytes: 4_096,
        clipboard_bytes: 1 << 20,
        hyperlink_bytes: 1 << 20,
        input_bytes: 1 << 20,
        keyboard_stack: 32,
        notification_bytes: 4_096,
        reply_bytes: 4_096,
        pending_events: 1_024,
        pending_damage: 1_024,
        history_lines: 64,
        history_bytes: 1 << 20,
        graphic_objects: 16,
        graphic_pixels: 1 << 16,
        graphic_decoded_bytes: 1 << 18,
        graphic_frames: 16,
        compression_ratio: 1_024,
        parser_work: 1 << 20,
        search_work: 1 << 20,
        reflow_work: 1 << 20,
        screen_cells: 1 << 20,
        snapshot_cells: 1 << 20,
    })
    .unwrap()
}

fn snapshot(bytes: &[u8]) -> RenderSnapshot {
    let limits = limits();
    let mut core = TerminalCore::new(limits, TerminalSize::new(12, 3, 120, 48).unwrap()).unwrap();
    let mut parser = StreamingParser::new(limits);
    parser.feed(bytes, |action| {
        core.apply(action).unwrap();
    });
    core.render_snapshot().unwrap()
}

fn geometry() -> TerminalScreenGeometry {
    datum_gui_viewport::terminal_screen_geometry(ScreenRectPx {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 120.0,
    })
}

#[test]
fn immutable_core_snapshot_drives_complete_style_and_fixed_cluster_geometry() {
    let snapshot = snapshot(b"\x1b[1;3;4;9;53;38;2;1;2;3;48;2;4;5;6;58;2;7;8;9mA\xe7\x95\x8c");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(DockTab::Terminal);
    *state.ui.terminal.pty_grid_mut().lines = vec!["PROVISIONAL-MUST-NOT-RENDER".to_string()];
    let mut quads = Vec::new();
    let mut text = Vec::new();
    let mut hits = Vec::new();
    render_terminal_core_snapshot(
        &state,
        &snapshot,
        &geometry(),
        &mut quads,
        &mut text,
        &mut hits,
    );

    assert!(text.iter().all(|run| !run.text.contains("PROVISIONAL")));
    let ascii = text.iter().find(|run| run.text == "A").unwrap();
    assert_eq!(
        ascii.rich_spans[0].color,
        [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0]
    );
    assert!(ascii.rich_spans[0].bold);
    assert!(ascii.rich_spans[0].italic);
    let wide = text.iter().find(|run| run.text == "界").unwrap();
    assert_eq!(wide.x, ascii.x + TERMINAL_CELL_WIDTH_PX);
    assert!((wide.clip_bounds.unwrap().width - 2.0 * TERMINAL_CELL_WIDTH_PX).abs() < 0.001);
    assert!(
        quads.len() >= 8,
        "background and every decoration must be geometric"
    );
    assert!(
        quads
            .iter()
            .any(|quad| { quad.color == [7.0 / 255.0, 8.0 / 255.0, 9.0 / 255.0,] })
    );
    assert!(
        hits.iter()
            .any(|hit| hit.target == HitTarget::TerminalScreen)
    );
}

#[test]
fn cursor_uses_core_row_column_shape_and_palette_without_lane_projection() {
    let snapshot = snapshot(b"abc\x1b[6 q");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.focus = ApplicationFocus::Terminal;
    state.ui.terminal.screen_cursor_col = 0;
    let geometry = geometry();
    let mut quads = Vec::new();
    render_terminal_core_snapshot(
        &state,
        &snapshot,
        &geometry,
        &mut quads,
        &mut Vec::new(),
        &mut Vec::new(),
    );
    let cursor_x = geometry.screen.x + 3.0 * TERMINAL_CELL_WIDTH_PX + CURSOR_HORIZONTAL_INSET_PX;
    assert!(quads.iter().any(|quad| quad.points[0].0 == cursor_x));
}

#[test]
fn ime_preedit_is_rendered_at_the_core_cursor_without_entering_cells() {
    let snapshot = snapshot(b"abc");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.focus = ApplicationFocus::Terminal;
    state.ui.terminal.ime_preedit = Some("文".into());
    let geometry = geometry();
    let mut quads = Vec::new();
    let mut text = Vec::new();
    render_terminal_core_snapshot(
        &state,
        &snapshot,
        &geometry,
        &mut quads,
        &mut text,
        &mut Vec::new(),
    );
    let preedit = text.iter().find(|run| run.text == "文").unwrap();
    assert_eq!(preedit.x, geometry.screen.x + 3.0 * TERMINAL_CELL_WIDTH_PX);
    assert!(quads.iter().any(|quad| quad.color == TEXT_ACCENT));
    assert_eq!(
        snapshot.rows().next().unwrap().cells()[3].content,
        datum_terminal_core::CellContent::Empty
    );
}

#[test]
fn retained_rows_rebuild_only_for_declared_damage_or_geometry_change() {
    let snapshot = snapshot(b"first\r\nsecond");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(DockTab::Terminal);
    let geometry = geometry();
    let mut cache = TerminalRenderCache::new();
    cache.render(
        &state,
        &snapshot,
        &[Damage::Full],
        &geometry,
        (&mut Vec::new(), &mut Vec::new(), &mut Vec::new()),
    );
    let initial = cache.rebuilt_rows();
    assert!(initial > 0);

    cache.render(
        &state,
        &snapshot,
        &[Damage::Cursor],
        &geometry,
        (&mut Vec::new(), &mut Vec::new(), &mut Vec::new()),
    );
    assert_eq!(cache.rebuilt_rows(), initial);

    cache.render(
        &state,
        &snapshot,
        &[Damage::Row(Row::new(1, Rows::new(3).unwrap()).unwrap())],
        &geometry,
        (&mut Vec::new(), &mut Vec::new(), &mut Vec::new()),
    );
    assert_eq!(cache.rebuilt_rows(), initial + 1);
}

#[test]
fn sixel_snapshot_produces_clipped_gpu_image_placement() {
    let snapshot = snapshot(b"\x1bP0;1;0q\"1;1;2;2#2;2;100;0;0@@$AA\x1b\\");
    assert_eq!(snapshot.graphics().len(), 1);
    let geometry = geometry();
    let mut graphics = Vec::new();
    prepare_terminal_graphics(&snapshot, &geometry, 0, &mut graphics);
    assert_eq!(graphics.len(), 1);
    let graphic = &graphics[0];
    assert_eq!(graphic.graphic.placement().width(), 2);
    assert_eq!(graphic.graphic.placement().height(), 2);
    assert_eq!(graphic.rect.x, geometry.screen.x);
    assert_eq!(graphic.rect.y, geometry.screen.y);
    assert!(graphic.clip.width > 0.0 && graphic.clip.height > 0.0);
}
