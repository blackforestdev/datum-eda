//! Runtime/PTY geometry proof for the shared native cancellation path.
use crate::{App, DockTab, GuiArgs, Runtime};
use clap::Parser;
use std::{sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)] // Hidden native fixture; production uses ActiveEventLoop.
fn native_cancellation_commits_terminal_dock_preview_geometry() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
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
    let runtime = pollster::block_on(Runtime::new(
        window.clone(),
        launch,
        Some(1.0),
        wake.clone(),
    ))
    .unwrap();
    let mut app = App::new(args, wake);
    app.frames.register(
        window.id(),
        runtime.measurements.epoch(),
        window.inner_size(),
        runtime.surface_transaction.recovery.clone(),
    );
    app.window = Some(window.clone());
    app.runtime = Some(runtime);
    let runtime = app.runtime.as_mut().unwrap();
    assert!(runtime.terminal_sessions.active_attached());
    runtime.set_active_dock(DockTab::Terminal);
    let before = runtime.workspace().ui.terminal.rows;
    let strip = runtime.current_layout().bottom_strip;
    assert!(runtime.begin_dock_resize_drag((500.0, strip.y + 2.0)));
    assert!(runtime.handle_dock_resize_drag((500.0, strip.y - 90.0)));
    let expected = runtime.terminal_screen_geometry().rows;
    assert_ne!(before, expected, "fixture must change terminal row count");
    assert_eq!(
        runtime.workspace().ui.terminal.rows,
        before,
        "drag is a preview"
    );

    // This is the production path used by failed acquisition/queue/device
    // recovery as well as native focus/DPI adapters, without a GPU fault shim.
    app.cancel_native_host_gestures(window.id());
    let runtime = app.runtime.as_mut().unwrap();
    let actual = runtime.workspace().ui.terminal.rows;
    let core_rows = runtime
        .terminal_sessions
        .active_core_mut()
        .test_render_snapshot()
        .size()
        .rows
        .get();
    let dragging = runtime.dock_drag_active;
    eprintln!("terminal rows initial={before} preview={expected} ui={actual} core={core_rows}");
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
    assert!(!dragging);
    assert_eq!(
        core_rows, expected,
        "terminal core follows retained geometry"
    );
    assert_eq!(
        actual, expected,
        "cancel must reconcile retained preview with PTY geometry"
    );
}
