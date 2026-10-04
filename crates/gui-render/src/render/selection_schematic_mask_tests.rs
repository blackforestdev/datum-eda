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

#[test]
#[ignore = "requires local hardware GPU: source-related baseline and unrelated dimming"]
fn native_related_context_preserves_parent_and_selection_while_dimming_only_foreign_material() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_related_state();
    let source = fixture.source_bytes();
    let selection = state.selection.clone();
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    state.ui.global_preferences.high_contrast_noncolor = true;
    state.ui.filters.dim_unrelated = false;
    let (board, schematic, ordinary_context) = prepare_schematic(&state);
    let mut host = hardware_renderer(1280, 800);
    let before = capture_schematic(&mut host, &ordinary_context, &board, &schematic);
    let world = related_world_uploads(&host);
    state.ui.filters.dim_unrelated = true;
    let (_, _, dimmed_context) = prepare_schematic(&state);
    let after = capture_schematic(&mut host, &dimmed_context, &board, &schematic);
    let request = &dimmed_context
        .selection_masks
        .iter()
        .find(|(_, s, _)| *s == SceneSurface::Schematic)
        .unwrap()
        .2;
    assert!(request.dim_unrelated());
    let scene = state.schematic_scene.as_ref().unwrap();
    let pass = dimmed_context
        .surface_passes()
        .iter()
        .find(|p| p.surface == SceneSurface::Schematic)
        .unwrap();
    let projection = crate::Projection::new(
        crate::inset_rect(pass.scene_viewport, 10., 10., 10., 10.),
        &scene.bounds,
        pass.camera,
    );
    let mut checked = [0_usize; 3];
    let mut witnesses = Vec::new();
    for span in &schematic.presentation.quads {
        let role = request.material_role(span);
        if role > 2 {
            continue;
        }
        for triangle in schematic.world_vertices()[span.range.clone()]
            .as_chunks::<6>()
            .0
        {
            let center = [0, 1, 2].map(|i| triangle[i].pos);
            let p = projection.project_point(datum_gui_protocol::PointNm {
                x: ((center[0][0] + center[1][0] + center[2][0]) / 3.).round() as i64,
                y: ((center[0][1] + center[1][1] + center[2][1]) / 3.).round() as i64,
            });
            witnesses.push((role, p, triangle[0].color));
        }
    }
    for span in &schematic.presentation.strokes {
        let role = request.material_role(span);
        if role > 2 {
            continue;
        }
        for stroke in &schematic.world_strokes()[span.range.clone()] {
            for fraction in [0.25, 0.5, 0.75] {
                let p = projection.project_point(datum_gui_protocol::PointNm {
                    x: (stroke.from[0] + (stroke.to[0] - stroke.from[0]) * fraction).round() as i64,
                    y: (stroke.from[1] + (stroke.to[1] - stroke.from[1]) * fraction).round() as i64,
                });
                witnesses.push((
                    role,
                    p,
                    [
                        stroke.color_min_px[0],
                        stroke.color_min_px[1],
                        stroke.color_min_px[2],
                    ],
                ));
            }
        }
    }
    for (role, p, color) in witnesses {
        if !pass.scene_viewport.contains(p.0, p.1) {
            continue;
        }
        let (x, y) = (p.0.floor() as u32, p.1.floor() as u32);
        let expected_before =
            color.map(|c| ((if role == 1 { (c * 1.08).min(1.) } else { c }) * 255.).round() as u8);
        if before.get_pixel(x, y).0[..3] != expected_before {
            continue;
        }
        if role == 0 {
            let expected = crate::dim_policy::dim_context_color(color, true)
                .map(|c| (c * 255.).round() as i16);
            for (actual, expected) in after.get_pixel(x, y).0[..3].iter().zip(expected) {
                assert!(
                    (*actual as i16 - expected).abs() <= 1,
                    "dim witness at {x},{y}"
                );
            }
            assert_ne!(before.get_pixel(x, y), after.get_pixel(x, y));
        } else {
            assert_eq!(
                before.get_pixel(x, y),
                after.get_pixel(x, y),
                "baseline role {role} at {x},{y}"
            );
        }
        checked[role as usize] += 1;
    }
    assert!(
        checked.iter().all(|n| *n > 0),
        "actual material pixel witnesses for all roles: {checked:?}"
    );
    assert!(world.iter().any(|v| v.2 > 0));
    assert_eq!(related_world_uploads(&host), world);
    assert_eq!(
        after,
        capture_schematic(&mut host, &dimmed_context, &board, &schematic)
    );
    assert_eq!(host.renderer.selection_mask.table_gpu.last_upload_bytes, 0);
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
    if let Ok(path) = std::env::var("S5A_NATIVE_RELATED_CAPTURE") {
        after.save(path).unwrap();
    }
}

fn related_world_uploads(host: &OffscreenRenderer) -> Vec<(u64, u64, u64)> {
    let ids: Vec<_> = [
        host.renderer.world_vertices_gpu.submission_ref(),
        host.renderer.world_strokes_gpu.submission_ref(),
        host.renderer.schematic_world_vertices_gpu.submission_ref(),
        host.renderer.schematic_world_strokes_gpu.submission_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|r| r.allocation_id)
    .collect();
    let records: Vec<_> = Renderer::gpu_process_allocations()
        .into_iter()
        .filter(|r| ids.contains(&r.id))
        .map(|r| (r.id, r.submitted_source_bytes, r.submitted_transfer_bytes))
        .collect();
    assert!(!ids.is_empty());
    assert_eq!(records.len(), ids.len());
    records
}
