//! Compare the dialog's single-pass output with the general renderer on a GPU.
use super::*;

fn hardware_renderer(width: u32, height: u32) -> OffscreenRenderer {
    hardware_renderer_with_features(width, height, wgpu::Features::empty())
}

fn hardware_renderer_with_features(
    width: u32,
    height: u32,
    required_features: wgpu::Features,
) -> OffscreenRenderer {
    hardware_renderer_with_atlas_limit(width, height, required_features, None)
}

fn hardware_renderer_with_atlas_limit(
    width: u32,
    height: u32,
    required_features: wgpu::Features,
    max_dimension: Option<u32>,
) -> OffscreenRenderer {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();
    let info = adapter.get_info();
    assert_ne!(
        info.device_type,
        wgpu::DeviceType::Cpu,
        "explicit local hardware test requires a GPU"
    );
    eprintln!(
        "pixel parity adapter: {} ({:?}, {:?})",
        info.name, info.device_type, info.backend
    );
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features,
        required_limits: wgpu::Limits {
            max_texture_dimension_2d:
                max_dimension.unwrap_or(wgpu::Limits::default().max_texture_dimension_2d),
            ..Default::default()
        },
        ..Default::default()
    }))
    .unwrap();
    let renderer = Renderer::new(&device, &queue, OUTPUT_FORMAT, DEFAULT_MSAA_SAMPLES);
    OffscreenRenderer {
        device,
        queue,
        renderer,
        width,
        height,
    }
}

fn capture(renderer: &mut OffscreenRenderer, prepared: &PreparedScene) -> RgbaImage {
    capture_retained(renderer, prepared, &RetainedScene::empty())
}

fn capture_retained(
    renderer: &mut OffscreenRenderer,
    prepared: &PreparedScene,
    retained: &RetainedScene,
) -> RgbaImage {
    let target = renderer.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dialog-parity-target"),
        size: renderer.extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: OUTPUT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    renderer
        .renderer
        .render(
            &renderer.device,
            &renderer.queue,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            prepared,
            retained,
            None,
            renderer.width,
            renderer.height,
        )
        .unwrap();
    renderer.read_texture(&target).unwrap()
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn dialog_single_pass_matches_general_renderer_pixels() {
    let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    for (width, height, scale) in [(960, 720, 1.0), (1050, 810, 1.5)] {
        let mut renderer = hardware_renderer(width, height);
        let retained = RetainedScene::from_workspace_for_surface(&state, width, height, scale);
        for row in [0, 1, 2, 0] {
            state.ui.global_preferences.scroll_row = row;
            let prepared = PreparedScene::from_native_preferences(
                &state.ui.global_preferences,
                width,
                height,
                scale,
            );
            assert!(prepared.is_overlay_only());
            let actual = capture(&mut renderer, &prepared);
            // A retirement/reindex may need one fresh preparation before reuse.
            assert!(actual == capture(&mut renderer, &prepared));
            let prepares = renderer.renderer.text_preparation.overlay_prepares;
            assert!(actual == capture(&mut renderer, &prepared));
            assert_eq!(
                renderer.renderer.text_preparation.overlay_prepares, prepares,
                "unchanged dialog must not prepare/upload glyphs again"
            );
            assert!(renderer.renderer.text_preparation.is_invalid());
            let mut general = prepared.clone();
            // An off-screen quad selects the general renderer without changing
            // any output pixel, so both paths consume the identical dialog.
            general.panel_vertices = crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
                crate::RectPx {
                    x: -100.0,
                    y: -100.0,
                    width: 1.0,
                    height: 1.0,
                },
                [0.0; 3],
            )]);
            assert!(!general.is_overlay_only());
            let expected = capture(&mut renderer, &general);
            let warm_general = capture(&mut renderer, &general);
            assert!(expected == warm_general);
            assert_eq!(renderer.renderer.panel_gpu.last_upload_bytes, 0);
            assert_eq!(renderer.renderer.menu_overlay_gpu.last_upload_bytes, 0);
            assert!(
                actual == expected,
                "dialog pixels differ at row {row}, scale {scale}"
            );
            let legacy = PreparedScene::from_workspace_with_terminal_renderer(
                &state,
                width,
                height,
                scale,
                CameraState::fit_to_bounds(&state.scene.bounds),
                &retained,
                &[],
                None,
                true,
            );
            assert!(
                actual == capture_retained(&mut renderer, &legacy, &retained),
                "native surface clipping must preserve legacy dialog pixels"
            );
        }
    }
}

#[path = "control_gpu_tests.rs"]
mod control_gpu_tests;

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn fractional_dialog_scroll_preserves_chrome_and_reuses_shaped_text() {
    let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    state.ui.global_preferences.open_choice_key = Some("datum.console.feedback_duration".into());
    state.ui.global_preferences.explanation_key = Some("datum.console.feedback_duration".into());
    let dialog = &state.ui.global_preferences;
    let mut renderer = hardware_renderer(960, 300);
    let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
    let initial = renderer.renderer.prepare_native_preferences_scrolled(
        dialog,
        960,
        300,
        1.0,
        &mut scroll,
        None,
    );
    let first = capture(&mut renderer, &initial);
    let header_bottom = scroll.viewport.y as u32;
    for cycle in 0..2 {
        for offset in [0.25, 40.0, 100.0, 75.0, 0.0] {
            scroll.set_offset(offset);
            let previous_builds = renderer.renderer.control_meshes.builds;
            let scene = renderer.renderer.prepare_native_preferences_scrolled(
                dialog,
                960,
                300,
                1.0,
                &mut scroll,
                None,
            );
            assert!(scene.is_overlay_only());
            if cycle == 1 {
                assert_eq!(
                    renderer.renderer.control_meshes.builds, previous_builds,
                    "warm scroll must reuse control meshes"
                );
                let (_, stats) = renderer.renderer.text_buffers.indices(
                    &mut renderer.renderer.font_system,
                    scene.menu_overlay_text_runs(),
                    960,
                    300,
                );
                assert_eq!(
                    stats.misses, 0,
                    "unchanged labels must not be reshaped on scrolling"
                );
            }
            let frame = capture(&mut renderer, &scene);
            for y in 0..header_bottom {
                for x in 0..960 {
                    assert_eq!(
                        frame.get_pixel(x, y),
                        first.get_pixel(x, y),
                        "scroll changed pinned chrome at {x},{y}"
                    );
                }
            }
            if offset == 0.0 {
                assert!(
                    frame == first,
                    "returning to top must restore identical pixels"
                );
            }
            if cycle == 1
                && offset == 40.0
                && let Ok(path) = std::env::var("DATUM_SCROLL_CAPTURE")
            {
                frame.save(path).unwrap();
            }
        }
    }
}

#[test]
#[ignore = "requires timestamp-capable hardware; production measurement parity"]
fn gpu_measurements_preserve_production_workspace_and_dialog_pixels() {
    let mut renderer = hardware_renderer_with_features(960, 720, wgpu::Features::TIMESTAMP_QUERY);
    let workspace = datum_gui_protocol::load_fixture_workspace_state();
    let dialog_state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let dialog =
        PreparedScene::from_native_preferences(&dialog_state.ui.global_preferences, 960, 720, 1.0);
    assert!(renderer.renderer.measurements.is_none());
    assert!(renderer.renderer.gpu_measurement_poll_deadline().is_none());
    let expected_workspace = renderer.render_workspace(&workspace, None).unwrap();
    let expected_dialog = capture(&mut renderer, &dialog);
    let mut menu_workspace = workspace.clone();
    menu_workspace.ui.active_menu = Some("View".to_owned());
    let expected_menu = renderer.render_workspace(&menu_workspace, None).unwrap();
    let mut terminal_workspace = workspace.clone();
    terminal_workspace.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    terminal_workspace.ui.dock_height_px = 220;
    let snapshot = super::tests::sixel_snapshot(true);
    let expected_terminal = renderer
        .render_workspace_with_terminal_snapshot(&terminal_workspace, &snapshot, 1.0)
        .unwrap();
    renderer
        .renderer
        .enable_gpu_measurements(
            &renderer.device,
            &renderer.queue,
            101,
            1,
            Box::new(|receipt| panic!("unexpected cancellation: {receipt:?}")),
        )
        .unwrap();
    let measured_workspace = renderer.render_workspace(&workspace, None).unwrap();
    assert!(
        expected_workspace == measured_workspace,
        "measurement changed workspace pixels"
    );
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene", "text"]
    );
    let measured_dialog = capture(&mut renderer, &dialog);
    assert!(
        expected_dialog == measured_dialog,
        "measurement changed dialog pixels"
    );
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["dialog"]
    );
    let measured_terminal = renderer
        .render_workspace_with_terminal_snapshot(&terminal_workspace, &snapshot, 1.0)
        .unwrap();
    assert!(
        expected_terminal == measured_terminal,
        "measurement changed terminal pixels"
    );
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert!(
        samples[0]
            .passes_ns
            .iter()
            .any(|p| p.0 == "datum-terminal-foreground-graphics-pass")
    );
    assert!(
        samples[0]
            .passes_ns
            .iter()
            .any(|p| p.0 == "datum-terminal-background-graphics-pass")
    );
    let measured_menu = renderer.render_workspace(&menu_workspace, None).unwrap();
    assert!(
        expected_menu == measured_menu,
        "measurement changed menu pixels"
    );
    let samples = renderer
        .renderer
        .poll_gpu_measurements(&renderer.device)
        .unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].passes_ns.iter().map(|p| p.0).collect::<Vec<_>>(),
        ["scene", "text", "menu-background", "menu-text"]
    );
    assert!(renderer.renderer.gpu_measurement_poll_deadline().is_none());
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn production_dialog_upload_reuses_content_and_matches_evicted_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let cold = capture(&mut renderer, &prepared);
    assert!(renderer.renderer.menu_overlay_gpu.last_upload_bytes > 0);
    let warm = capture(&mut renderer, &prepared);
    assert_eq!(renderer.renderer.menu_overlay_gpu.last_upload_bytes, 0);
    assert!(cold == warm);
    // Forced eviction is the negative path: it must upload, with identical pixels.
    renderer.renderer.menu_overlay_gpu = Default::default();
    let evicted = capture(&mut renderer, &prepared);
    assert!(renderer.renderer.menu_overlay_gpu.last_upload_bytes > 0);
    assert!(warm == evicted);
    let mut replacement = prepared.clone();
    let red = &mut replacement.menu_overlay_vertices[0].color[0];
    *red = if *red == 1.0 { 0.0 } else { 1.0 };
    let changed = capture(&mut renderer, &replacement);
    assert_eq!(renderer.renderer.menu_overlay_gpu.last_upload_bytes, 4);
    renderer.renderer.menu_overlay_gpu = Default::default();
    assert!(changed == capture(&mut renderer, &replacement));
    let restored = capture(&mut renderer, &prepared);
    assert_eq!(renderer.renderer.menu_overlay_gpu.last_upload_bytes, 4);
    assert!(restored == cold);
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn shared_atlas_retry_and_cache_reindex_refresh_workspace_glyphs() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut workspace = prepared.menu_overlay_text_runs[0].clone();
    workspace.text = "Workspace glyph residency".into();
    workspace.rich_spans.clear();
    workspace.x = 20.0;
    workspace.y = 680.0;
    workspace.clip_bounds = None;
    let mut overlay = workspace.clone();
    overlay.text = "Overlay glyph residency".into();
    overlay.x = 600.0;
    overlay.y = 20.0;
    prepared.text_runs = vec![workspace.clone()];
    prepared.menu_overlay_text_runs = vec![overlay.clone()];
    prepared.menu_overlay_vertices = crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
        crate::RectPx {
            x: 500.0,
            y: 0.0,
            width: 460.0,
            height: 720.0,
        },
        [0.05; 3],
    )]);
    let cold = capture(&mut renderer, &prepared);
    let count = renderer.renderer.text_preparation.workspace_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares, count,
        "warm workspace preparation stays skipped"
    );
    renderer.renderer.text_preparation.force_overlay_errors(1);
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1,
        "retry must re-protect workspace glyphs before overlay allocation"
    );

    renderer.renderer.text_preparation.force_overlay_errors(2);
    let failed = renderer.renderer.prepare_frame_text(
        &renderer.device,
        &renderer.queue,
        &prepared,
        960,
        720,
        false,
    );
    assert!(failed.is_err());
    assert!(renderer.renderer.text_preparation.is_invalid());
    let count = renderer.renderer.text_preparation.workspace_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1
    );

    // Evict entry zero but retain entry one, which moves into the same index.
    // The new workspace hits that existing buffer with the old placement key.
    renderer
        .renderer
        .text_buffers
        .begin_frame(crate::text_buffer_cache::Profile::Workspace);
    renderer.renderer.text_buffers.indices(
        &mut renderer.renderer.font_system,
        &[overlay],
        960,
        720,
    );
    renderer
        .renderer
        .text_buffers
        .begin_frame(crate::text_buffer_cache::Profile::Workspace);
    prepared.text_runs[0].text = "Overlay glyph residency".into();
    let (indices, stats) = renderer.renderer.text_buffers.indices(
        &mut renderer.renderer.font_system,
        &prepared.text_runs,
        960,
        720,
    );
    assert_eq!(indices, vec![0]);
    assert_eq!(
        stats.misses, 0,
        "collision is a retained cache hit, not new shaping"
    );
    let count = renderer.renderer.text_preparation.workspace_prepares;
    let reindexed = capture(&mut renderer, &prepared);
    assert_eq!(
        renderer.renderer.text_preparation.workspace_prepares,
        count + 1
    );
    assert!(
        reindexed != cold,
        "workspace changed to the retained overlay label"
    );
    renderer.renderer.text_preparation = Default::default();
    assert!(
        reindexed == capture(&mut renderer, &prepared),
        "reindex reuse matches forced fresh glyph preparation"
    );
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn shared_text_preparation_survives_real_atlas_pressure() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer =
        hardware_renderer_with_atlas_limit(192, 192, wgpu::Features::empty(), Some(256));
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 192, 192, 1.0);
    let mut run = prepared.menu_overlay_text_runs[0].clone();
    run.rich_spans.clear();
    run.clip_bounds = None;
    run.x = 10.0;
    run.y = 10.0;
    run.text = "Workspace".into();
    run.size = 14.0;
    prepared.text_runs = vec![run.clone()];
    prepared.menu_overlay_vertices = crate::gpu_data::quads_to_vertices(&[crate::Quad::from_rect(
        crate::RectPx {
            x: 0.0,
            y: 80.0,
            width: 192.0,
            height: 112.0,
        },
        [0.05; 3],
    )]);
    for size in 24..56 {
        prepared.menu_overlay_text_runs = (b'A'..=b'L')
            .map(|ch| {
                let mut label = run.clone();
                label.text = (ch as char).to_string();
                label.size = size as f32;
                label.y = 85.0;
                label
            })
            .collect();
        capture(&mut renderer, &prepared);
    }
    assert!(
        renderer.renderer.text_preparation.atlas_retries > 0,
        "limited atlas must exercise real pressure, without injection"
    );
    let pressure = capture(&mut renderer, &prepared);
    let retries = renderer.renderer.text_preparation.atlas_retries;
    let mut fresh =
        hardware_renderer_with_atlas_limit(192, 192, wgpu::Features::empty(), Some(256));
    assert!(
        pressure == capture(&mut fresh, &prepared),
        "pressure result differs from fresh atlas"
    );
    assert!(pressure == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.atlas_retries, retries,
        "unchanged pressure recovery remains warm"
    );
    eprintln!("real bounded-atlas retries: {retries}");
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn cached_shape_relayout_matches_fresh_dialog_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut fresh = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    let mut label = prepared.menu_overlay_text_runs[0].clone();
    label.text = "Wrap widths preserve shaped text: µm and Ω".into();
    label.rich_spans.clear();
    label.x = 30.0;
    label.y = 30.0;
    prepared.menu_overlay_text_runs = vec![label];
    for (step, width) in [240.0, 100.0, 180.0, 100.0].into_iter().enumerate() {
        prepared.menu_overlay_text_runs[0].clip_bounds = Some(crate::RectPx {
            x: 30.0,
            y: 30.0,
            width,
            height: 150.0,
        });
        let actual = capture(&mut renderer, &prepared);
        fresh.renderer.text_buffers = Default::default();
        assert!(
            actual == capture(&mut fresh, &prepared),
            "cached shape differs at width {width}"
        );
        assert_eq!(renderer.renderer.text_buffers.shape_reuses, step);
        assert_eq!(renderer.renderer.text_buffers.entries().len(), 1);
    }
}

#[path = "gpu_pass_tests.rs"]
mod pass_tests;

#[test]
#[ignore = "requires local GPU; renderer-owned control preparation proof"]
fn renderer_owned_preferences_meshes_stay_warm_and_match_fresh_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
    let prepared = renderer.renderer.prepare_native_preferences_scrolled(
        &state.ui.global_preferences,
        960,
        720,
        1.0,
        &mut scroll,
        Some(0),
    );
    let cold = capture(&mut renderer, &prepared);
    let builds = renderer.renderer.control_meshes.builds;
    assert!(builds > 0);
    let warm = renderer.renderer.prepare_native_preferences_scrolled(
        &state.ui.global_preferences,
        960,
        720,
        1.0,
        &mut scroll,
        None,
    );
    assert_eq!(renderer.renderer.control_meshes.builds, builds);
    assert!(cold == capture(&mut renderer, &warm));
    let scaled = renderer.renderer.prepare_native_preferences_scrolled(
        &state.ui.global_preferences,
        960,
        720,
        1.5,
        &mut scroll,
        Some(0),
    );
    assert!(renderer.renderer.control_meshes.builds > builds);
    let expected =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.5);
    assert!(capture(&mut renderer, &scaled) == capture(&mut renderer, &expected));
    renderer.renderer = Renderer::new(
        &renderer.device,
        &renderer.queue,
        OUTPUT_FORMAT,
        DEFAULT_MSAA_SAMPLES,
    );
    assert_eq!(
        renderer.renderer.control_meshes.builds, 0,
        "renderer replacement starts without old control retention"
    );
    let restored = renderer.renderer.prepare_native_preferences_scrolled(
        &state.ui.global_preferences,
        960,
        720,
        1.0,
        &mut scroll,
        Some(0),
    );
    assert!(renderer.renderer.control_meshes.builds > 0);
    assert!(cold == capture(&mut renderer, &restored));
}

#[path = "gpu_control_tests.rs"]
mod control_tests;

#[path = "gpu_uniform_tests.rs"]
mod uniform_tests;

#[test]
#[ignore = "requires local GPU; bounded overlay signature proof"]
fn oversized_overlay_signature_bypasses_reuse_without_omitting_text() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    prepared.menu_overlay_text_runs = vec![prepared.menu_overlay_text_runs[0].clone(); 129];
    let cold = capture(&mut renderer, &prepared);
    let prepares = renderer.renderer.text_preparation.overlay_prepares;
    assert!(cold == capture(&mut renderer, &prepared));
    assert_eq!(
        renderer.renderer.text_preparation.overlay_prepares,
        prepares + 1
    );
    let mut fresh = hardware_renderer(960, 720);
    assert!(cold == capture(&mut fresh, &prepared));
}
