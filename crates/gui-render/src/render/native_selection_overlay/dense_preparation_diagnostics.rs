//! Actual governed dense preparation and shared reservation diagnosis.
use super::*;
use std::time::Instant;

fn stage<T>(name: &str, work: impl FnOnce() -> T) -> T {
    eprintln!("dense stage begin: {name}");
    let start = Instant::now();
    let result = work();
    eprintln!(
        "dense stage end: {name}; elapsed_ms={}",
        start.elapsed().as_millis()
    );
    result
}

fn prepare(
    state: &ReviewWorkspaceState,
    board: &RetainedScene,
    renderer: &mut crate::Renderer,
) -> PreparedScene {
    let _host = renderer.resource_host.enter();
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut prepared = stage("base prepared frame", || {
        crate::text_metrics::measurement_owner::with_owner(&mut renderer.measurement_fonts, || {
            PreparedScene::from_workspace_with_controls(
                state,
                1920,
                1051,
                1.0416666269302368,
                camera,
                board,
                &[],
                None,
                false,
                &mut renderer.control_meshes,
            )
        })
        .unwrap()
    });
    let mut schematic_camera =
        CameraState::fit_to_bounds(&state.schematic_scene.as_ref().unwrap().bounds);
    schematic_camera.zoom = 0.6274223327636719;
    schematic_camera.center_x_nm = 399141216.0;
    schematic_camera.center_y_nm = 249014848.0;
    prepared.set_schematic_camera(schematic_camera);
    assert_eq!(prepared.surface_passes().len(), 1);
    assert_eq!(
        prepared.surface_passes()[0].pane_id,
        datum_gui_protocol::PaneId(1)
    );
    assert_eq!(
        prepared.surface_passes()[0].surface,
        SceneSurface::Schematic
    );
    stage("append native selection", || {
        prepared.append_native_selection(state).unwrap()
    });
    prepared
}

#[test]
#[ignore = "actual preserved dense fixture and local GPU; diagnostic, not native acceptance"]
fn governed_dense_preparation_and_reservation_owners() {
    let root = std::path::PathBuf::from(std::env::var("S5A_DENSE_NATIVE_ROOT").unwrap());
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        kicad_board_source: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
    };
    let (mut state, model) = stage("public workspace", || {
        datum_gui_protocol::load_board_editor_workspace_with_model(&request).unwrap()
    });
    let files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| s.path.clone())
        .chain(std::iter::once(
            eda_engine::substrate::transaction_journal_path(&root),
        ))
        .collect();
    let before: Vec<_> = files.iter().map(|p| std::fs::read(p).unwrap()).collect();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let members: std::collections::BTreeSet<_> = authority
        .authored_identities()
        .iter()
        .filter(|id| id.class == Class::SchematicText)
        .cloned()
        .collect();
    assert_eq!(members.len(), 100_000);
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(members.clone(), None).unwrap(),
    );
    let view = stage("complete SelectionView", || {
        build_selection_view(
            &model,
            &authority,
            subject,
            &state.scene,
            state.schematic_scene.as_ref(),
            |_| true,
        )
        .unwrap()
    });
    assert_eq!(view.members, members);
    state.selection = SelectionTarget::Native(Box::new(view));
    state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
    state.ui.layout.focus_next();
    state.ui.layout.toggle_zoom();
    assert_eq!(state.ui.layout.focused, datum_gui_protocol::PaneId(1));
    let board = stage("retained board", || {
        RetainedScene::try_from_workspace_for_surface(&state, 1920, 1051, 1.0416666269302368)
            .unwrap()
    });
    let schematic = stage("retained schematic", || {
        RetainedScene::try_from_workspace_schematic_for_surface(
            &state,
            1920,
            1051,
            1.0416666269302368,
        )
        .unwrap()
        .unwrap()
    });
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();
    eprintln!("dense headless adapter: {:?}", adapter.get_info());
    let required_features =
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES & adapter.features();
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    assert!(
        adapter
            .get_texture_format_features(format)
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8)
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("datum-dense-preparation-diagnostic"),
        required_features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .unwrap();
    let mut renderer = crate::Renderer::new(&device, &queue, format, 8).unwrap();
    let observer = renderer.local_reservation_observer();
    assert_eq!(observer.usage().staging.limit_bytes, 16 * 1024 * 1024);
    assert_eq!(observer.usage().screen.limit_bytes, 16 * 1024 * 1024);
    let prior = Id {
        class: Class::SchematicText,
        id: uuid::Uuid::parse_str("50000000-0000-4000-8000-00000000d287").unwrap(),
        instance_path: vec![],
    };
    let prior_view = build_selection_view(
        &model,
        &authority,
        SelectionSubject::Object(prior),
        &state.scene,
        state.schematic_scene.as_ref(),
        |_| true,
    )
    .unwrap();
    let complete = std::mem::replace(
        &mut state.selection,
        SelectionTarget::Native(Box::new(prior_view)),
    );
    let target = crate::capture_resource::CaptureTarget::new(
        &device,
        wgpu::Extent3d {
            width: 1920,
            height: 1051,
            depth_or_array_layers: 1,
        },
        format,
    )
    .unwrap();
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let warm = prepare(&state, &board, &mut renderer);
    stage("warm singleton render", || {
        renderer.render(
            &device,
            &queue,
            &target_view,
            &warm,
            &board,
            Some(&schematic),
            1920,
            1051,
        )
    })
    .unwrap();
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    eprintln!("dense warm reservation baseline: {:?}", observer.usage());
    drop(warm);
    state.selection = complete;
    let prepared = prepare(&state, &board, &mut renderer);
    eprintln!(
        "dense prepared: strokes={}; masks={}; lod={:?}",
        prepared.selection_strokes.len(),
        prepared.selection_masks.len(),
        prepared.selection_visual_lod()
    );
    eprintln!("dense render before: {:?}", observer.usage());
    let result = stage("full renderer", || {
        renderer.render(
            &device,
            &queue,
            &target_view,
            &prepared,
            &board,
            Some(&schematic),
            1920,
            1051,
        )
    });
    eprintln!("dense render result: {result:?}");
    eprintln!(
        "dense render after: {:?}; mask_cpu={}; snapshots={}; ranges={}; glyph_cpu={}",
        observer.usage(),
        renderer.selection_mask_cpu_storage_bytes(),
        renderer.screen_upload_snapshot_bytes(),
        renderer.screen_upload_metadata_bytes(),
        renderer.glyph_upload_storage_bytes()
    );
    for (name, stream) in [
        ("panel", &renderer.panel_gpu),
        ("viewport underlay", &renderer.viewport_underlay_gpu),
        ("viewport overlay", &renderer.viewport_overlay_gpu),
        ("board interaction", &renderer.board_interaction_gpu),
        ("console", &renderer.console_gpu.vertices),
        ("menu overlay", &renderer.menu_overlay_gpu),
        ("schematic overlay", &renderer.schematic_overlay_gpu),
        ("mask table", &renderer.selection_mask.table_gpu),
        ("selection strokes", &renderer.selection_stroke_gpu),
        (
            "selection exact clips",
            &renderer.selection_stroke_clips.gpu,
        ),
        ("surface grid", &renderer.surface_grid_gpu),
    ] {
        let mut count = crate::text_gpu::upload::UploadCount::default();
        stream.append_sources(&mut count);
        eprintln!(
            "dense producer {name}: fragments={}; payload_bytes={}; snapshots={}; pending_metadata={}",
            count.entries,
            count.bytes,
            stream.snapshot_bytes(),
            stream.pending_metadata_bytes()
        );
    }
    drop(renderer);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    eprintln!("dense after renderer retirement: {:?}", observer.usage());
    for (path, bytes) in files.iter().zip(before) {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    // This diagnoses reached owners and preserves refusal; success is not native delivery.
}
