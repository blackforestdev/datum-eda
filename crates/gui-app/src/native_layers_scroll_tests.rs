//! Actual Board Runtime wheel routing and presented Layers capacity.
use crate::{GuiArgs, Runtime};
use clap::Parser;
use datum_gui_render::HitTarget;
use std::{sync::Arc, time::Duration};
use winit::event::MouseScrollDelta;

#[test]
#[ignore = "requires native Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_layers_route_preserves_preparation_camera_and_boundary_damage() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--board", &board, "--initial-layout", "single"])
            .unwrap();
    let backend = std::env::var("DATUM_NATIVE_TEST_BACKEND").unwrap_or_else(|_| "x11".into());
    let scale: f32 = std::env::var("DATUM_NATIVE_TEST_SCALE")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .unwrap();
    assert!([1.0, 1.5, 2.0].contains(&scale));
    let mut builder = winit::event_loop::EventLoop::<()>::with_user_event();
    match backend.as_str() {
        "x11" => {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_x11().with_any_thread(true);
        }
        "wayland" => {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            builder.with_wayland().with_any_thread(true);
        }
        _ => panic!("unsupported native test backend: {backend}"),
    }
    let event_loop = builder.build().unwrap();
    let wake = event_loop.create_proxy();
    let window = Arc::new(
        event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(
                        ((1280.0 * scale) as u32).min(1918),
                        ((800.0 * scale) as u32).min(1050),
                    )),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(scale), wake)).unwrap();
    let layout = runtime.current_layout();
    let initial_camera = runtime.camera;
    let camera = format!("{:?}", initial_camera);
    runtime.prepared_scene = None;
    runtime.last_cursor_pos = Some((0.0, 0.0));
    assert!(!runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, -1.0)));
    assert!(
        runtime.prepared_scene.is_none(),
        "outside Layers must not prepare a scene"
    );
    assert_eq!(format!("{:?}", runtime.camera), camera);

    let _ = runtime.prepared_scene();
    let mut prepared = runtime.prepared_scene.take().unwrap();
    runtime.presented_hits.mark_pending();
    runtime.presented_hits.present(&mut prepared);
    let region = runtime
        .presented_hits
        .regions()
        .iter()
        .find(|r| r.target == HitTarget::LayerScrollRegion)
        .expect("real Board Layers region")
        .rect;
    let visible = runtime
        .presented_hits
        .regions()
        .iter()
        .filter(|r| matches!(r.target, HitTarget::ToggleLayer(_)))
        .count();
    let total = runtime.workspace().scene.layers.len();
    assert!(visible > 0);
    assert!(total > visible, "real fixture must exercise scrolling");
    runtime.last_cursor_pos = Some((
        region.x + region.width * 0.5,
        region.y + region.height * 0.5,
    ));
    runtime.scene_dirty = false;
    assert!(runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, -1000.0)));
    assert_eq!(
        runtime.workspace().ui.filters.layer_scroll_offset,
        total - visible
    );
    assert!(runtime.scene_dirty);
    assert_eq!(
        format!("{:?}", runtime.camera),
        camera,
        "changing Layers wheel must not fall through into board zoom"
    );
    assert!(
        runtime.prepared_scene.is_none(),
        "scroll routing must not prepare a scene"
    );
    runtime.scene_dirty = false;
    assert!(!runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, -1000.0)));
    assert!(
        !runtime.scene_dirty,
        "boundary input must not dirty the scene"
    );
    assert!(runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, 1.0)));
    assert!(runtime.workspace().ui.filters.layer_scroll_offset < total - visible);
    assert_eq!(
        format!("{:?}", runtime.camera),
        camera,
        "Layers must not mutate camera"
    );

    for lines in [-1.25_f32, -0.25, 0.25, 1.25] {
        let reset = if lines < 0.0 { 1000.0 } else { -1000.0 };
        runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, reset));
        runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, lines));
        let line_offset = runtime.workspace().ui.filters.layer_scroll_offset;
        let step = lines.abs().ceil() as usize;
        let expected = if lines < 0.0 {
            step.min(total - visible)
        } else {
            (total - visible).saturating_sub(step)
        };
        assert_eq!(
            line_offset, expected,
            "Layers must preserve discrete row policy"
        );
        runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, reset));
        runtime.handle_native_wheel(MouseScrollDelta::PixelDelta(
            winit::dpi::PhysicalPosition::new(0.0, f64::from(lines * 20.0 * scale)),
        ));
        assert_eq!(
            runtime.workspace().ui.filters.layer_scroll_offset,
            line_offset,
            "Layers physical/line disagreement at scale={scale} lines={lines}"
        );
        assert_eq!(format!("{:?}", runtime.camera), camera);
        assert!(runtime.prepared_scene.is_none());
    }

    // The same native routing entry point must still reach the editor camera.
    runtime.last_cursor_pos = Some((
        layout.viewport.x + layout.viewport.width * 0.5,
        layout.viewport.y + layout.viewport.height * 0.5,
    ));
    for lines in [-1.0_f32, -0.25, 0.25, 1.0] {
        runtime.camera = initial_camera;
        assert!(runtime.handle_native_wheel(MouseScrollDelta::LineDelta(0.0, lines)));
        let line_camera = format!("{:?}", runtime.camera);
        assert_ne!(line_camera, camera);
        runtime.camera = initial_camera;
        assert!(runtime.handle_native_wheel(MouseScrollDelta::PixelDelta(
            winit::dpi::PhysicalPosition::new(0.0, f64::from(lines * 20.0 * scale)),
        )));
        assert_eq!(
            format!("{:?}", runtime.camera),
            line_camera,
            "camera physical/line disagreement at scale={scale} lines={lines}"
        );
        assert!(
            runtime.prepared_scene.is_none(),
            "camera route must not prepare Layers"
        );
    }
    eprintln!(
        "backend={backend} scale={scale} accepted={}x{} visible_layers={visible} total_layers={total} physical_line_rows_camera_match=true",
        runtime.config.width, runtime.config.height
    );

    runtime.begin_application_terminal_shutdown();
    let deadline = std::time::Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        runtime.poll_terminal_output();
        assert!(
            std::time::Instant::now() < deadline,
            "owned terminal shutdown"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
