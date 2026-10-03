//! Focused mask mechanism proof, separate from complete native S5A acceptance.
use super::*;
use crate::native_selection_overlay::tests::{selected_occluded_track_state, selected_track_state};
fn native_frame(state: &ReviewWorkspaceState) -> (RetainedScene, PreparedScene, crate::Projection) {
    let retained = RetainedScene::from_workspace(state, 1280, 800);
    let mut camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
    let initial = PreparedScene::from_workspace(state, 1280, 800, camera, &retained).unwrap();
    let field = crate::inset_rect(initial.scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let scale = crate::Projection::new(field, &state.scene.bounds, camera).scale;
    camera.zoom *= 0.0001 / scale;
    camera.center_x_nm = 1_000_000.0;
    camera.center_y_nm = 0.0;
    let mut prepared = PreparedScene::from_workspace(state, 1280, 800, camera, &retained).unwrap();
    prepared.append_native_selection(state).unwrap();
    let projection = crate::Projection::new(field, &state.scene.bounds, camera);
    (retained, prepared, projection)
}
#[test]
#[ignore = "requires local hardware GPU: native source-bound mask and lifetime"]
fn native_mask_preserves_material_join_and_occlusion_without_authored_writes() {
    let (fixture, mut state) = selected_occluded_track_state();
    let source = fixture.source_bytes();
    let selection = state.selection.clone();
    state.ui.global_preferences.high_contrast_noncolor = true;
    let (retained, prepared, projection) = native_frame(&state);
    assert!(
        prepared.selection_strokes.is_empty(),
        "detailed paths are mask-owned"
    );
    assert_eq!(prepared.selection_masks.len(), 1);
    let mut baseline_state = state.clone();
    baseline_state.selection = datum_gui_protocol::SelectionTarget::None;
    let (_, baseline, _) = native_frame(&baseline_state);
    let mut host = hardware_renderer(1280, 800);
    let baseline_image = capture_retained(&mut host, &baseline, &retained);
    let image = capture_retained(&mut host, &prepared, &retained);
    let center = projection.project_point(crate::PointNm { x: 1_000_000, y: 0 });
    let center_pixel = (center.0.floor() as u32, center.1.floor() as u32);
    assert_eq!(
        image.get_pixel(center_pixel.0, center_pixel.1),
        baseline_image.get_pixel(center_pixel.0, center_pixel.1),
        "unselected canonical track must occlude the selected source"
    );
    let selected = projection.project_point(crate::PointNm { x: 500_000, y: 0 });
    let pixel = (selected.0.floor() as u32, selected.1.floor() as u32);
    let color = baseline_image.get_pixel(pixel.0, pixel.1).0;
    assert_eq!(
        image.get_pixel(pixel.0, pixel.1).0[..3],
        color[..3]
            .iter()
            .map(|v| (f32::from(*v) * 1.08).min(255.0).round() as u8)
            .collect::<Vec<_>>()
    );
    let radius = state.scene.tracks[0].width_nm as f32 * projection.scale * 0.5;
    assert_eq!(
        image
            .get_pixel(pixel.0, (selected.1 + radius + 0.5).floor() as u32)
            .0[..3],
        [206, 90, 146]
    );
    let snapshot = host.renderer.selection_mask_attachment_snapshot().unwrap();
    assert_eq!(snapshot.payload_bytes, Some(1280 * 100 * 4 * 4));
    let color_snapshot = host
        .renderer
        .selection_mask_color_attachment_snapshot()
        .unwrap();
    assert_eq!(color_snapshot.payload_bytes, snapshot.payload_bytes);
    let color_observer = host.renderer.selection_mask_color_attachment_observer();
    let observer = host.renderer.selection_mask_attachment_observer();
    let cpu_bytes = host.renderer.selection_mask_cpu_storage_bytes();
    assert_eq!(image, capture_retained(&mut host, &prepared, &retained));
    assert_eq!(
        host.renderer.selection_mask_attachment_snapshot(),
        Some(snapshot)
    );
    assert_eq!(host.renderer.selection_mask_cpu_storage_bytes(), cpu_bytes);
    assert_eq!(host.renderer.selection_mask.table_gpu.last_upload_bytes, 0);
    // A source-neutral subdivision must not introduce an internal cue seam.
    let (single_fixture, mut single_state) = selected_track_state(2_000_000);
    single_state.ui.global_preferences.high_contrast_noncolor = true;
    let (single_retained, single_prepared, _) = native_frame(&single_state);
    let single = capture_retained(&mut host, &single_prepared, &single_retained);
    single_state.scene.tracks[0]
        .path
        .insert(1, crate::PointNm { x: 1_000_000, y: 0 });
    let (joined_retained, joined_prepared, _) = native_frame(&single_state);
    assert_eq!(joined_retained.world_strokes().len(), 2);
    assert_eq!(
        single,
        capture_retained(&mut host, &joined_prepared, &joined_retained)
    );
    let single_source = single_fixture.source_bytes();
    capture_retained(&mut host, &baseline, &retained);
    assert!(host.renderer.selection_mask_attachment_snapshot().is_none());
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert!(observer.allocations().is_empty());
    assert!(color_observer.allocations().is_empty());
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
    assert_eq!(single_fixture.source_bytes(), single_source);
}

#[test]
#[ignore = "requires local hardware GPU: canonical via material and drill void"]
fn native_mask_treats_actual_via_silhouette_without_filling_drill_void() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_via_state();
    let before = fixture.source_bytes();
    state.ui.global_preferences.high_contrast_noncolor = true;
    let selection = state.selection.clone();
    let (retained, prepared, projection) = native_frame(&state);
    let mut baseline_state = state.clone();
    baseline_state.selection = datum_gui_protocol::SelectionTarget::None;
    let (_, baseline, _) = native_frame(&baseline_state);
    let mut host = hardware_renderer(1280, 800);
    let ordinary = capture_retained(&mut host, &baseline, &retained);
    let selected = capture_retained(&mut host, &prepared, &retained);
    let center = projection.project_point(crate::PointNm { x: 1_000_000, y: 0 });
    let (x, y) = (center.0.floor() as u32, center.1.floor() as u32);
    for dx in [-20_i32, -10, 0, 10, 20] {
        assert_eq!(
            selected.get_pixel((x as i32 + dx) as u32, y),
            ordinary.get_pixel((x as i32 + dx) as u32, y),
            "the complete authored drill void is unchanged"
        );
    }
    let rim = ordinary.get_pixel(x, y + 35).0;
    assert_eq!(
        selected.get_pixel(x, y + 35).0[..3],
        rim[..3]
            .iter()
            .map(|v| (f32::from(*v) * 1.08).min(255.0).round() as u8)
            .collect::<Vec<_>>()
    );
    assert!(
        prepared
            .selected_paint_ranges(&retained, None)
            .any(|range| !range.stroke)
    );
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
#[ignore = "requires local hardware GPU: complete compound groups and focus parity"]
fn native_compound_focus_keeps_equal_canvas_treatment_and_complete_groups() {
    let (fixture, mut state) = selected_occluded_track_state();
    let before = fixture.source_bytes();
    let members: Vec<_> = state
        .scene
        .tracks
        .iter()
        .map(|track| datum_gui_protocol::AuthoredSelectionIdentity {
            class: datum_gui_protocol::AuthoredSelectionClass::Track,
            id: uuid::Uuid::parse_str(&track.source_object_uuid).unwrap(),
            instance_path: vec![],
        })
        .collect();
    let subject = |focus| {
        datum_gui_protocol::SelectionSubject::Compound(
            datum_gui_protocol::CompoundSelection::new(members.clone(), Some(focus)).unwrap(),
        )
    };
    state.ui.global_preferences.high_contrast_noncolor = true;
    fixture.select(&mut state, subject(members[0].clone()));
    let (retained, prepared, _) = native_frame(&state);
    let groups: std::collections::BTreeSet<_> = prepared
        .selected_paint_ranges(&retained, None)
        .map(|range| range.group)
        .collect();
    assert_eq!(groups, std::collections::BTreeSet::from([1, 2]));
    let mut host = hardware_renderer(1280, 800);
    let first = capture_retained(&mut host, &prepared, &retained);
    fixture.select(&mut state, subject(members[1].clone()));
    let (second_retained, second_prepared, _) = native_frame(&state);
    let second = capture_retained(&mut host, &second_prepared, &second_retained);
    let viewport = prepared.scene_viewport;
    let crop = |image: &RgbaImage| {
        image::imageops::crop_imm(
            image,
            viewport.x.ceil() as u32,
            viewport.y.ceil() as u32,
            viewport.width.floor() as u32,
            viewport.height.floor() as u32,
        )
        .to_image()
    };
    assert_eq!(
        crop(&first),
        crop(&second),
        "optional compound focus is an Inspector state, not a second canvas intensity"
    );
    let datum_gui_protocol::SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    assert_eq!(view.members.len(), 2);
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
#[ignore = "requires local hardware GPU: bounded selection mask residency"]
fn native_mask_4k_residency_is_height_independent_and_refusal_is_retryable() {
    let (fixture, state) = selected_track_state(2_000_000);
    let source = fixture.source_bytes();
    let (retained, prepared, _) = native_frame(&state);
    let mut host = hardware_renderer(1280, 800);
    capture_retained(&mut host, &prepared, &retained);
    let renderer = &mut host.renderer;
    let prepare = |renderer: &mut Renderer, extent| {
        renderer.selection_mask.prepare(
            &host.device,
            &host.queue,
            &renderer.scene_bind_group_layout,
            &renderer.damage_masks.layout,
            renderer.msaa_format,
            renderer.msaa_samples,
            &prepared,
            &retained,
            None,
            extent,
            extent,
        )
    };
    prepare(renderer, [3840, 2160]).unwrap();
    let labels = renderer.selection_mask_attachment_snapshot().unwrap();
    let colors = renderer.selection_mask_color_attachment_snapshot().unwrap();
    assert_eq!(labels.payload_bytes, Some(3840 * 100 * 4 * 4));
    assert_eq!(colors.payload_bytes, labels.payload_bytes);
    prepare(renderer, [3840, 4320]).unwrap();
    assert_eq!(renderer.selection_mask_attachment_snapshot(), Some(labels));
    assert_eq!(
        renderer.selection_mask_color_attachment_snapshot(),
        Some(colors)
    );
    // A source-pin mismatch refuses before publishing any replacement binding.
    let mut wrong = state.clone();
    wrong.scene.source_revision.push_str("-incompatible");
    let incompatible = RetainedScene::from_workspace(&wrong, 1280, 800);
    assert!(
        renderer
            .selection_mask
            .prepare(
                &host.device,
                &host.queue,
                &renderer.scene_bind_group_layout,
                &renderer.damage_masks.layout,
                renderer.msaa_format,
                renderer.msaa_samples,
                &prepared,
                &incompatible,
                None,
                [3840, 2160],
                [3840, 2160]
            )
            .is_err()
    );
    assert!(!renderer.selection_mask.active());
    assert_eq!(renderer.selection_mask_attachment_snapshot(), Some(labels));
    prepare(renderer, [3840, 2160]).unwrap();
    assert!(renderer.selection_mask.active());
    assert_eq!(fixture.source_bytes(), source);
}
