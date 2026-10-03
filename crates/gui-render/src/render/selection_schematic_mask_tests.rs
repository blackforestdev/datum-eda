//! Actual canonical schematic glyph/point material through the shared mask owner.
use super::*;
use crate::SceneSurface;
use crate::native_selection_overlay::tests::{
    selected_points_state, selected_schematic_text_state,
};
fn prepare_schematic(
    state: &ReviewWorkspaceState,
) -> (RetainedScene, RetainedScene, PreparedScene) {
    let board = RetainedScene::from_workspace(state, 1280, 800);
    let schematic =
        RetainedScene::from_workspace_schematic_for_surface(state, 1280, 800, 1.0).unwrap();
    let mut prepared = PreparedScene::from_workspace(
        state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &board,
    )
    .unwrap();
    let source = state.schematic_scene.as_ref().unwrap();
    let mut camera = CameraState::fit_to_bounds(&source.bounds);
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for v in schematic.world_vertices() {
        for axis in 0..2 {
            lo[axis] = lo[axis].min(v.pos[axis]);
            hi[axis] = hi[axis].max(v.pos[axis]);
        }
    }
    let viewport = prepared
        .surface_passes()
        .iter()
        .find(|p| p.surface == SceneSurface::Schematic)
        .unwrap()
        .scene_viewport;
    let field = crate::inset_rect(viewport, 10.0, 10.0, 10.0, 10.0);
    let scale = crate::Projection::new(field, &source.bounds, camera).scale;
    let desired =
        (field.width / (hi[0] - lo[0]).max(1.0)).min(field.height / (hi[1] - lo[1]).max(1.0)) * 0.8;
    camera.zoom = (desired / scale).clamp(0.35, 8.0);
    camera.center_x_nm = (lo[0] + hi[0]) * 0.5;
    camera.center_y_nm = (lo[1] + hi[1]) * 0.5;
    prepared.set_schematic_camera(camera);
    prepared.append_native_selection(state).unwrap();
    (board, schematic, prepared)
}
fn capture_schematic(
    host: &mut OffscreenRenderer,
    prepared: &PreparedScene,
    board: &RetainedScene,
    schematic: &RetainedScene,
) -> RgbaImage {
    let target =
        crate::capture_resource::CaptureTarget::new(&host.device, host.extent(), OUTPUT_FORMAT)
            .unwrap();
    host.renderer
        .render(
            &host.device,
            &host.queue,
            target.create_view(&Default::default()),
            prepared,
            board,
            Some(schematic),
            host.width,
            host.height,
        )
        .unwrap();
    host.read_texture(&target).unwrap()
}
#[test]
#[ignore = "requires local hardware GPU: canonical schematic glyph and junction silhouettes"]
fn native_schematic_mask_treats_actual_glyph_and_point_without_layout_boxes_or_writes() {
    for text in [true, false] {
        let (fixture, mut state) = if text {
            selected_schematic_text_state()
        } else {
            selected_points_state()
        };
        let source = fixture.source_bytes();
        let selection = state.selection.clone();
        state.ui.layout.open_beside_root(
            datum_gui_protocol::PaneContent::Schematic,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            true,
        );
        state.ui.global_preferences.high_contrast_noncolor = true;
        let (board, schematic, prepared) = prepare_schematic(&state);
        let material = prepared
            .selected_paint_ranges(&board, Some(&schematic))
            .find(|r| r.surface == SceneSurface::Schematic && !r.stroke && !r.range.is_empty())
            .unwrap_or_else(|| panic!("positive actual selected silhouette: text={text}, masks={:?}, presentation={:?}, vertices={}, strokes={}", prepared.selection_masks, schematic.presentation, schematic.world_vertices().len(), schematic.world_strokes().len()));
        let color = schematic.world_vertices()[material.range.start]
            .color
            .map(|v| (v * 255.0).round() as u8);
        let viewport = prepared
            .surface_passes()
            .iter()
            .find(|p| p.surface == SceneSurface::Schematic)
            .unwrap()
            .scene_viewport;
        let mut baseline_state = state.clone();
        baseline_state.selection = datum_gui_protocol::SelectionTarget::None;
        let (_, _, baseline) = prepare_schematic(&baseline_state);
        let mut host = hardware_renderer(1280, 800);
        let ordinary = capture_schematic(&mut host, &baseline, &board, &schematic);
        let selected = capture_schematic(&mut host, &prepared, &board, &schematic);
        let mut material_changes = 0;
        let mut empty_checks = 0;
        for y in (viewport.y.ceil() as u32 + 4)..((viewport.y + viewport.height).floor() as u32 - 4)
        {
            for x in
                (viewport.x.ceil() as u32 + 4)..((viewport.x + viewport.width).floor() as u32 - 4)
            {
                let before = ordinary.get_pixel(x, y);
                let after = selected.get_pixel(x, y);
                if before.0[..3] == color && before != after {
                    material_changes += 1;
                }
                // The crisp two-pixel cue plus partial sample coverage cannot
                // change remote background, layout corners or large glyph holes.
                let near_material = (-3_i32..=3).any(|dy| {
                    (-3_i32..=3).any(|dx| {
                        ordinary
                            .get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)
                            .0[..3]
                            == color
                    })
                });
                if !near_material {
                    assert_eq!(before, after, "no substitute layout box at {x},{y}");
                    empty_checks += 1;
                }
            }
        }
        assert!(
            material_changes > 0,
            "actual selected schematic material must change"
        );
        assert!(empty_checks > 0);
        assert_eq!(
            selected,
            capture_schematic(&mut host, &prepared, &board, &schematic)
        );
        assert_eq!(host.renderer.selection_mask.table_gpu.last_upload_bytes, 0);
        assert_eq!(state.selection, selection);
        assert_eq!(fixture.source_bytes(), source);
        if text && let Ok(path) = std::env::var("S5A_NATIVE_GLYPH_CAPTURE") {
            selected.save(path).unwrap();
        }
    }
}
