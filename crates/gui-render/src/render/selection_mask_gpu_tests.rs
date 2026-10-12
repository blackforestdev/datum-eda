//! Focused mask mechanism proof, separate from complete native S5A acceptance.
use super::*;
use crate::native_selection_overlay::tests::{selected_occluded_track_state, selected_track_state};
use crate::{
    Output, PROPOSAL_BASE, PointNm, ProposalOverlayPrimitive, Quad, RectPx, TEXT_ACCENT,
    quads_to_vertices,
};
use datum_gui_protocol::SelectionTarget;
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
    if let Ok(path) = std::env::var("S5A_NATIVE_TRACK_CAPTURE") {
        image.save(path).unwrap();
    }
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
            &renderer.text_renderer,
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
                [3840, 2160],
                &renderer.text_renderer,
            )
            .is_err()
    );
    assert!(!renderer.selection_mask.active());
    assert_eq!(renderer.selection_mask_attachment_snapshot(), Some(labels));
    prepare(renderer, [3840, 2160]).unwrap();
    assert!(renderer.selection_mask.active());
    assert_eq!(fixture.source_bytes(), source);
}

#[test]
#[ignore = "requires local hardware GPU: shared screen cue parity and pane fallback"]
fn shared_screen_cues_preserve_original_selected_paint_and_fallback_base() {
    let (_fixture, state) = selected_track_state(2_000_000);
    let (retained, mut prepared, projection) = native_frame(&state);
    let pane = prepared.surface_passes[0].pane_id;
    let viewport = prepared.surface_passes[0].scene_viewport;
    let camera = prepared.surface_passes[0].camera;
    let overlay = ProposalOverlayPrimitive {
        overlay_id: "source-route".into(),
        primitive_kind: "route".into(),
        proposal_action_id: "source-action".into(),
        layer_id: None,
        render_role: "proposed_overlay".into(),
        width_nm: Some(400_000),
        drill_nm: None,
        diameter_nm: None,
        path: vec![
            PointNm { x: 0, y: 1_000_000 },
            PointNm {
                x: 2_000_000,
                y: 1_000_000,
            },
        ],
    };
    let mut old = Vec::new();
    crate::overlay_channels::push_overlay(
        &mut old,
        &overlay,
        &projection,
        PROPOSAL_BASE,
        true,
        false,
    );
    crate::hit_clipping::clip_content(
        &mut old,
        &mut Vec::new(),
        &mut Vec::new(),
        0,
        0,
        0,
        viewport,
    );
    let mut plan =
        crate::native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut channels = Vec::new();
    crate::overlay_channels::push_overlay_channels(
        &mut channels,
        &mut plan,
        &overlay,
        &projection,
        PROPOSAL_BASE,
        true,
    );
    assert!(plan.detail_primitives > 0);
    let offset = prepared.viewport_overlay_vertices.len() as u32;
    for span in plan.screen.iter_mut() {
        span.range.start += offset;
        span.range.end += offset;
    }
    for range in plan.detail.iter_mut() {
        range.start += offset;
        range.end += offset;
    }
    prepared
        .pane_cues
        .push((pane, std::sync::Arc::new(plan.freeze().unwrap())));
    prepared.selection_masks[0]
        .2
        .set_screen(
            prepared
                .pane_cues
                .iter()
                .filter(|(id, _)| *id == pane)
                .map(|(_, plan)| plan.clone()),
        )
        .unwrap();
    let mut reference = prepared.clone();
    reference
        .viewport_overlay_vertices
        .extend(quads_to_vertices(&old));
    prepared
        .viewport_overlay_vertices
        .extend(quads_to_vertices(&channels));
    let mut host = hardware_renderer(1280, 800);
    let expected = capture_retained(&mut host, &reference, &retained);
    let detailed = capture_retained(&mut host, &prepared, &retained);
    assert_eq!(
        detailed, expected,
        "below-cap paint matches the original selected emitter over the whole frame"
    );
    assert!(!host.renderer.selection_mask.detailed_overflow(pane));

    // A bounded actual emitted stream crosses the established limit. The
    // minimum-cue stream and canonical authored membership remain unchanged.
    let shape = RectPx {
        x: viewport.x + 40.0,
        y: viewport.y + 180.0,
        width: 24.0,
        height: 18.0,
    };
    let mut fallback = prepared.clone();
    let mut extra =
        crate::native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
    let mut vertices = Vec::new();
    {
        let mut base = extra.base_output(&mut vertices, true);
        base.push(Quad::from_rect(shape, PROPOSAL_BASE));
    }
    let start = vertices.len();
    vertices.extend(std::iter::repeat_n(
        Quad::from_rect(shape, TEXT_ACCENT),
        65_537,
    ));
    extra.finish_detail(&mut vertices, start);
    assert_eq!(extra.detail_primitives, 65_537);
    let offset = fallback.viewport_overlay_vertices.len() as u32;
    for span in extra.screen.iter_mut() {
        span.range.start += offset;
        span.range.end += offset;
    }
    for range in extra.detail.iter_mut() {
        range.start += offset;
        range.end += offset;
    }
    fallback
        .pane_cues
        .push((pane, std::sync::Arc::new(extra.freeze().unwrap())));
    // Capture fresh request ownership: cloning a PreparedScene must not permit
    // mutating shared intent from a previously rendered frame.
    let SelectionTarget::Native(view) = &state.selection else {
        panic!("fixture selection")
    };
    let mut request = crate::native_selection_overlay::mask::Request::capture(
        view,
        &state.scene,
        datum_gui_protocol::PaneContent::Board,
    )
    .unwrap()
    .unwrap();
    request
        .set_screen(
            fallback
                .pane_cues
                .iter()
                .filter(|(id, _)| *id == pane)
                .map(|(_, plan)| plan.clone()),
        )
        .unwrap();
    fallback.selection_masks[0].2 = request;
    fallback
        .viewport_overlay_vertices
        .extend(quads_to_vertices(&vertices));
    let image = capture_retained(&mut host, &fallback, &retained);
    assert!(host.renderer.selection_mask.detailed_overflow(pane));
    let center = (shape.x + shape.width * 0.5, shape.y + shape.height * 0.5);
    let pixel = image.get_pixel(center.0 as u32, center.1 as u32).0;
    assert_eq!(
        pixel[..3],
        PROPOSAL_BASE.map(|c| (c * 255.0).round() as u8),
        "mask-only emphasis preserves the original base material without a lift or black overwrite"
    );
    assert_eq!(fallback.selection_strokes, prepared.selection_strokes);
    assert_eq!(image, capture_retained(&mut host, &fallback, &retained));
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
}

#[path = "selection_mask_gpu_tests/artifact.rs"]
mod artifact;

#[test]
#[ignore = "requires local hardware GPU: original Vertex20 pipeline versus compact viewport44"]
fn viewport_quad_encoding_matches_original_vertex_pipeline_and_refuses_atomically() {
    use crate::text_gpu::{
        budget::Budget,
        upload::plan::{Source, Uploads},
    };
    let (_fixture, mut state) = selected_track_state(2_000_000);
    state.selection = SelectionTarget::None;
    state.ui.filters.show_authored = false;
    state.ui.filters.show_proposed = false;
    let retained = RetainedScene::empty();
    let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
    let mut compact = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
    compact.surface_passes.clear();
    compact.text_runs.clear();
    compact.panel_vertices.clear();
    compact.viewport_underlay_vertices.clear();
    compact.board_interaction_vertices.clear();
    compact.console_overlay_vertices.clear();
    compact.menu_overlay_vertices.clear();
    compact.schematic_overlay_vertices.clear();
    compact.viewport_overlay_vertices = quads_to_vertices(&[
        Quad {
            points: [
                (400.25, 310.5),
                (610.75, 317.25),
                (593.5, 425.75),
                (387.25, 418.5),
            ],
            color: [0.13, 0.58, 0.89],
        },
        Quad {
            points: [
                (520.5, 365.25),
                (653.25, 383.75),
                (581.75, 472.5),
                (581.75, 472.5),
            ],
            color: [0.93, 0.42, 0.17],
        },
    ]);
    compact.selection_overlay_split = 0;
    let mut original = compact.clone();
    original.panel_vertices = std::mem::take(&mut original.viewport_overlay_vertices);
    let mut host = hardware_renderer(1280, 800);
    let old_pixels = capture_retained(&mut host, &original, &retained);
    let compact_pixels = capture_retained(&mut host, &compact, &retained);
    assert_eq!(
        old_pixels, compact_pixels,
        "exact fractional/degenerate quads and sRGB fragment law match the original20B pipeline"
    );

    // RGB starts after the CURRENT coordinate plane, never allocation capacity.
    // Warm growth/shrink and distinct colors independently check both bindings.
    let source = compact.viewport_overlay_vertices.clone();
    for count in [1, 3, 2] {
        compact.viewport_overlay_vertices = (0..count)
            .flat_map(|i| source[(i % 2) * 6..(i % 2 + 1) * 6].iter().copied())
            .collect();
        let mut original = compact.clone();
        original.panel_vertices = std::mem::take(&mut original.viewport_overlay_vertices);
        let expected = capture_retained(&mut host, &original, &retained);
        assert_eq!(expected, capture_retained(&mut host, &compact, &retained));
    }
    compact.viewport_overlay_vertices = source;

    let staging = Budget::new(8192);
    let screen = Budget::new(128);
    let mut gpu = crate::gpu_data::screen_buffer::ScreenBuffer::with_budget(screen.clone())
        .with_staging_budget(staging.clone());
    let vertices = &compact.viewport_overlay_vertices[..6];
    let bytes = crate::gpu_vertex_upload::screen_quads::encode(vertices, &staging).unwrap();
    let preparation = gpu
        .prepare_owned(&host.device, "quad-refusal-baseline", bytes, 44)
        .unwrap();
    gpu.commit_owned(preparation);
    fn pending(gpu: &crate::gpu_data::screen_buffer::ScreenBuffer) -> Vec<(u64, Vec<u8>)> {
        let mut sources: Vec<Source<'_>> = Vec::new();
        gpu.append_sources(&mut sources);
        Uploads::sources(&sources)
            .map(|u| (u.offset, u.bytes.to_vec()))
            .collect()
    }
    let before = pending(&gpu);
    let target = gpu.submission_ref().unwrap().allocation_id;
    let reserved = staging.used();
    let oversized = vertices.repeat(20);
    let encoded = crate::gpu_vertex_upload::screen_quads::encode(&oversized, &staging).unwrap();
    assert!(
        gpu.prepare_owned(&host.device, "quad-postencode-refusal", encoded, 44)
            .is_err()
    );
    assert_eq!(pending(&gpu), before);
    assert_eq!(gpu.submission_ref().unwrap().allocation_id, target);
    assert_eq!(staging.used(), reserved);
    let mut changed = vertices.to_vec();
    for v in &mut changed {
        v.color[0] = 0.37;
    }
    let bytes = crate::gpu_vertex_upload::screen_quads::encode(&changed, &staging).unwrap();
    let retry = gpu
        .prepare_owned(&host.device, "quad-retry", bytes, 44)
        .unwrap();
    gpu.commit_owned(retry);
    assert_ne!(pending(&gpu), before);
    assert_eq!(gpu.submission_ref().unwrap().allocation_id, target);
    let mut sources = Vec::new();
    gpu.append_sources(&mut sources);
    let uploads = Uploads::sources(&sources).collect::<Vec<_>>();
    crate::text_gpu::upload::submit_buffers_for_test(&host.device, &host.queue, &uploads);
    drop(uploads);
    drop(sources);
    gpu.finish_uploads();
    assert!(pending(&gpu).is_empty());
    let unchanged = crate::gpu_vertex_upload::screen_quads::encode(&changed, &staging).unwrap();
    let unchanged = gpu
        .prepare_owned(&host.device, "quad-completed-reuse", unchanged, 44)
        .unwrap();
    gpu.commit_owned(unchanged);
    assert!(pending(&gpu).is_empty());
    drop(gpu);
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    assert_eq!(staging.used(), 0);
    assert_eq!(screen.used(), 0);
}
