//! Compare the dialog's single-pass output with the general renderer on a GPU.
use super::*;

fn hardware_renderer(width: u32, height: u32) -> OffscreenRenderer {
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
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
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
