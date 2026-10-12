//! Engine-produced artifact rendering and actual projection transition controls.
use super::*;

#[test]
#[ignore = "requires governed engine-created draft Project and local hardware GPU"]
fn genuine_artifact_only_overflow_preserves_base_and_admits_complete_frame() {
    use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
    use datum_gui_protocol::{PreparedProjectOpen, SelectionSubject};
    let root = std::env::var_os("DATUM_SHARED_CUE_PROJECT").expect("governed draft Project path");
    let opened = PreparedProjectOpen::prepare(std::path::Path::new(&root), false).unwrap();
    assert_eq!(
        opened.model.model_revision.0,
        "554096a8a1ead41134ae5c1e61a7c6df1b5b20228b9d5f4cb4ea923306217e6f"
    );
    let mut state = opened.workspace;
    let proposal = state
        .production
        .proposals
        .iter()
        .find(|p| p.proposal_id == "83000000-0000-4000-8000-000000000001")
        .unwrap();
    assert_eq!(proposal.operation_count, 8000);
    assert_eq!(proposal.preview.as_ref().unwrap().render_deltas.len(), 8000);
    let key = proposal.action_identity.as_ref().unwrap().selection_key();
    let authority = NativeSelectionResolution::capture(&opened.model).unwrap();
    let view = datum_gui_protocol::selection_artifacts::build_workspace_selection_view(
        &opened.model,
        &authority,
        SelectionSubject::Proposal(key.clone()),
        &state,
    )
    .unwrap();
    assert!(view.members.is_empty());
    assert!(
        view.projection_keys(&state.scene, datum_gui_protocol::PaneContent::Board)
            .unwrap()
            .is_empty()
    );
    state.ui.filters.show_authored = false;
    state.ui.filters.show_proposed = true;
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
    state.selection = SelectionTarget::None;
    let mut baseline = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
    baseline.append_native_selection(&state).unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    let mut selected = PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
    assert!(
        selected
            .hit_regions
            .iter()
            .any(|h| matches!(&h.target, crate::HitTarget::ProposalAction(id) if id == &key))
    );
    selected.append_native_selection(&state).unwrap();
    let count: usize = selected
        .pane_cues
        .iter()
        .map(|(_, p)| p.detail_primitives)
        .sum();
    assert!(count > crate::native_selection_overlay::cue_plan::DETAILED_PRIMITIVES);
    assert!(selected.pane_cues.iter().all(|(_, p)| p.detail.is_empty()));
    let selected_base = selected
        .pane_cues
        .iter()
        .flat_map(|(_, p)| p.screen.iter())
        .flat_map(|span| {
            selected.viewport_overlay_vertices[span.range.start as usize..span.range.end as usize]
                .iter()
        });
    let baseline_base = baseline
        .pane_cues
        .iter()
        .flat_map(|(_, p)| p.screen.iter())
        .flat_map(|span| {
            baseline.viewport_overlay_vertices[span.range.start as usize..span.range.end as usize]
                .iter()
        });
    assert!(
        selected_base.eq(baseline_base),
        "exact admitted base vertex sequence is independent of range fragmentation"
    );
    assert!(
        selected
            .selection_masks
            .iter()
            .any(|(_, _, r)| !r.authored() && r.detailed_overflow(0))
    );
    eprintln!(
        "genuine draft emitted detail={count}; baseline vertices={}",
        baseline.viewport_overlay_vertices.len()
    );
    let mut cold = hardware_renderer(1280, 800);
    let selected_pixels = capture_retained(&mut cold, &selected, &retained);
    assert!(cold.renderer.selection_mask.active());
    let mut warm = hardware_renderer(1280, 800);
    let baseline_pixels = capture_retained(&mut warm, &baseline, &retained);
    let warm_pixels = capture_retained(&mut warm, &selected, &retained);
    assert_eq!(
        selected_pixels, warm_pixels,
        "warm producer state cannot reuse an earlier unselected mask"
    );
    assert_ne!(
        selected_pixels, baseline_pixels,
        "typed Proposal palette remains visible over preserved base"
    );
    // A later-band, pane-clipped selected fallback glyph must contribute only
    // in its current prepared frame, even when Draw retains the old allocation.
    let pane = selected.surface_passes[0].pane_id;
    let viewport = selected.surface_passes[0].scene_viewport;
    let mut no_text = selected.clone();
    no_text.text_runs.clear();
    let mut with_glyph = no_text.clone();
    crate::draw_text(
        "I",
        viewport.x + 80.0,
        viewport.y + 230.0,
        36.0,
        TEXT_ACCENT,
        crate::TextFace::Ui,
        &mut with_glyph.text_runs,
    );
    let run = with_glyph.text_runs.last_mut().unwrap();
    run.layer = crate::TextLayer::Selection(pane);
    run.origin = crate::TextOrigin::Viewport(pane);
    run.clip_bounds = Some(RectPx {
        x: viewport.x + 80.0,
        y: viewport.y + 230.0,
        width: 5.0,
        height: 36.0,
    });
    let glyph_pixels = capture_retained(&mut warm, &with_glyph, &retained);
    assert!(warm.renderer.text_renderer.selected_instances(pane) > 0);
    let cleared_pixels = capture_retained(&mut warm, &no_text, &retained);
    let mut empty_cold = hardware_renderer(1280, 800);
    let cold_empty_pixels = capture_retained(&mut empty_cold, &no_text, &retained);
    assert_eq!(
        cleared_pixels, cold_empty_pixels,
        "warm selected glyphs cannot leak into a no-text frame"
    );
    assert_ne!(
        glyph_pixels, cleared_pixels,
        "actual clipped later-band glyph silhouette is visible"
    );
}

#[test]
#[ignore = "requires preserved native Draft Project and hardware; reservation localization only"]
fn genuine_artifact_split_to_zoom_reservation_diagnostic() {
    let root = std::env::var_os("DATUM_SHARED_CUE_PROJECT").unwrap();
    let opened =
        datum_gui_protocol::PreparedProjectOpen::prepare(std::path::Path::new(&root), false)
            .unwrap();
    assert_eq!(
        opened.model.project.project_id.to_string(),
        "20000000-0000-4000-8000-000000000001"
    );
    assert_eq!(
        opened.model.model_revision.0,
        "554096a8a1ead41134ae5c1e61a7c6df1b5b20228b9d5f4cb4ea923306217e6f"
    );
    let mut state = opened.workspace;
    let proposal = state
        .production
        .proposals
        .iter()
        .find(|p| p.proposal_id == "83000000-0000-4000-8000-000000000001")
        .unwrap();
    assert_eq!(proposal.preview.as_ref().unwrap().render_deltas.len(), 8000);
    state.selection = SelectionTarget::None;
    state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
    assert_eq!(state.ui.layout.focused, datum_gui_protocol::PaneId(0));
    assert!(state.ui.filters.show_authored && state.ui.filters.show_proposed);
    let width = 1333;
    let height = 800;
    let scale = 1.0416666269302368;
    let board = RetainedScene::from_workspace_for_surface(&state, width, height, scale);
    let schematic =
        RetainedScene::try_from_workspace_schematic_for_surface(&state, width, height, scale)
            .unwrap()
            .unwrap();
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();
    eprintln!(
        "transition adapter: {:?}; headless surface qualification",
        adapter.get_info()
    );
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    assert!(
        adapter
            .get_texture_format_features(format)
            .flags
            .sample_count_supported(8)
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            & adapter.features(),
        ..Default::default()
    }))
    .unwrap();
    let mut renderer = Renderer::new(&device, &queue, format, 8).unwrap();
    let observer = renderer.local_reservation_observer();
    let target = crate::capture_resource::CaptureTarget::new(
        &device,
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        format,
    )
    .unwrap();
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    assert_eq!(
        (camera.center_x_nm, camera.center_y_nm, camera.zoom),
        (25000000., 0., 1.)
    );
    let prepare = |state: &ReviewWorkspaceState, renderer: &mut Renderer, camera: CameraState| {
        let _host = renderer.resource_host.enter();
        let mut prepared = crate::text_metrics::measurement_owner::with_owner(
            &mut renderer.measurement_fonts,
            || {
                PreparedScene::from_workspace_with_controls(
                    state,
                    width,
                    height,
                    scale,
                    camera,
                    &board,
                    &[],
                    None,
                    false,
                    &mut renderer.control_meshes,
                )
            },
        )
        .unwrap();
        prepared.set_schematic_camera(CameraState::fit_to_bounds(
            &state.schematic_scene.as_ref().unwrap().bounds,
        ));
        prepared.append_native_selection(state).unwrap();
        prepared
    };
    let split = prepare(&state, &mut renderer, camera);
    assert_eq!(split.surface_passes().len(), 2);
    renderer
        .render(
            &device,
            &queue,
            &target_view,
            &split,
            &board,
            Some(&schematic),
            width,
            height,
        )
        .unwrap();
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    eprintln!("transition baseline usage: {:?}", observer.usage());
    state.ui.layout.toggle_zoom();
    let zoomed = prepare(&state, &mut renderer, camera);
    assert_eq!(zoomed.surface_passes().len(), 1);
    assert_eq!(
        zoomed.surface_passes()[0].pane_id,
        datum_gui_protocol::PaneId(0)
    );
    let answer = renderer.render(
        &device,
        &queue,
        &target_view,
        &zoomed,
        &board,
        Some(&schematic),
        width,
        height,
    );
    eprintln!("transition actual render result: {answer:?}");
    eprintln!("transition after usage: {:?}", observer.usage());
    for (name, stream) in [
        ("viewport overlay", &renderer.viewport_overlay_gpu),
        ("panel", &renderer.panel_gpu),
        ("underlay", &renderer.viewport_underlay_gpu),
        ("grid", &renderer.surface_grid_gpu),
    ] {
        let mut count = crate::text_gpu::upload::UploadCount::default();
        stream.append_sources(&mut count);
        eprintln!(
            "transition producer {name}: fragments={}; bytes={}; snapshots={}; ranges={}",
            count.entries,
            count.bytes,
            stream.snapshot_bytes(),
            stream.pending_metadata_bytes()
        );
    }
    assert!(
        answer.is_ok(),
        "complete actual split-to-zoom batch must fit unchanged allowances"
    );
    let mut camera = camera;
    camera.center_x_nm += 1_000_000.;
    let panned = prepare(&state, &mut renderer, camera);
    let panned_answer = renderer.render(
        &device,
        &queue,
        &target_view,
        &panned,
        &board,
        Some(&schematic),
        width,
        height,
    );
    eprintln!(
        "transition warm horizontal pan result: {panned_answer:?}; usage={:?}",
        observer.usage()
    );
    drop(renderer);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let retired = observer.usage();
    eprintln!("transition retirement usage: {retired:?}");
    assert_eq!(retired.staging.reserved_bytes, 0);
    assert_eq!(retired.screen.reserved_bytes, 0);
    assert!(
        panned_answer.is_ok(),
        "same-count camera change must retain exact complete admission"
    );
    // Controlled whole transition fit is distinct from ordinary native delivery.
}
