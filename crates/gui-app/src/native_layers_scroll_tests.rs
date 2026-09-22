//! Actual Board Runtime wheel routing and presented Layers capacity.
use crate::{GuiArgs, Runtime};
use clap::Parser;
use datum_gui_render::HitTarget;
use std::{sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_layers_route_preserves_preparation_camera_and_boundary_damage() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--board", &board, "--initial-layout", "single"])
            .unwrap();
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = event_loop.create_proxy();
    let window = Arc::new(
        event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let layout = runtime.current_layout();
    let camera = format!("{:?}", runtime.camera);
    runtime.prepared_scene = None;
    runtime.last_cursor_pos = Some((layout.viewport.x + 20.0, layout.viewport.y + 20.0));
    assert_eq!(runtime.handle_layer_scroll(-1.0), None);
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
    assert_eq!(runtime.handle_layer_scroll(-1000.0), Some(true));
    assert_eq!(
        runtime.workspace().ui.filters.layer_scroll_offset,
        total - visible
    );
    assert!(runtime.scene_dirty);
    assert!(
        runtime.prepared_scene.is_none(),
        "scroll routing must not prepare a scene"
    );
    runtime.scene_dirty = false;
    assert_eq!(runtime.handle_layer_scroll(-1000.0), Some(false));
    assert!(
        !runtime.scene_dirty,
        "boundary input must not dirty the scene"
    );
    assert_eq!(runtime.handle_layer_scroll(1.0), Some(true));
    assert!(runtime.workspace().ui.filters.layer_scroll_offset < total - visible);
    assert_eq!(
        format!("{:?}", runtime.camera),
        camera,
        "Layers must not mutate camera"
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
