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
            &RetainedScene::empty(),
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
            assert!(renderer.renderer.last_text_prepare_signature.is_none());
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
            assert!(
                actual == expected,
                "dialog pixels differ at row {row}, scale {scale}"
            );
        }
    }
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn rounded_control_fans_match_scanline_pixels() {
    let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    let mut renderer = hardware_renderer(960, 720);
    let mut prepared =
        PreparedScene::from_native_preferences(&state.ui.global_preferences, 960, 720, 1.0);
    prepared.menu_overlay_text_runs.clear();
    for (width, height, radius) in [
        (140.0, 28.0, 4.0),
        (24.0, 24.0, 12.0),
        (37.5, 19.5, 3.0),
        (200.0, 50.0, 0.0),
    ] {
        for offset in [0.0, 0.25, 0.5] {
            let rect = crate::RectPx {
                x: 40.0 + offset,
                y: 40.0 + offset,
                width,
                height,
            };
            let mut old = Vec::new();
            crate::push_projected_polygon_fill(
                &mut old,
                &crate::global_preferences_primitives::rounded_rect_points(rect, radius),
                [0.3, 0.5, 0.7],
            );
            prepared.menu_overlay_vertices = crate::gpu_data::quads_to_vertices(&old);
            let expected = capture(&mut renderer, &prepared);
            let mut new = Vec::new();
            crate::global_preferences_primitives::push_rounded_rect_fill(
                &mut new,
                rect,
                [0.3, 0.5, 0.7],
                radius,
            );
            prepared.menu_overlay_vertices = crate::gpu_data::quads_to_vertices(&new);
            let actual = capture(&mut renderer, &prepared);
            assert!(
                actual == expected,
                "rounded control changed for {rect:?}, radius {radius}"
            );
        }
    }
}

#[test]
#[ignore = "requires local GPU; run explicitly with the visual feature"]
fn fractional_dialog_scroll_preserves_chrome_and_reuses_shaped_text() {
    let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
    state.ui.global_preferences.open_choice_key = Some("datum.console.feedback_duration".into());
    state.ui.global_preferences.explanation_key = Some("datum.console.feedback_duration".into());
    let dialog = &state.ui.global_preferences;
    let mut renderer = hardware_renderer(960, 300);
    let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
    let initial =
        PreparedScene::from_native_preferences_scrolled(dialog, 960, 300, 1.0, &mut scroll, None);
    let first = capture(&mut renderer, &initial);
    let header_bottom = scroll.viewport.y as u32;
    for cycle in 0..2 {
        for offset in [0.25, 40.0, 100.0, 75.0, 0.0] {
            scroll.set_offset(offset);
            let scene = PreparedScene::from_native_preferences_scrolled(
                dialog,
                960,
                300,
                1.0,
                &mut scroll,
                None,
            );
            assert!(scene.is_overlay_only());
            if cycle == 1 {
                let (_, stats) = renderer.renderer.cached_text_buffer_indices(
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
