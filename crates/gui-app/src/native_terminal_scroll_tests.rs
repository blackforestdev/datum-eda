//! Actual terminal scrollback bounds and changed-only damage.
use crate::{GuiArgs, Runtime};
use clap::Parser;
use std::{sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_terminal_scrollback_boundaries_do_not_redraw() {
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
    runtime.session.workspace_mut().ui.active_dock_tab =
        Some(datum_gui_protocol::DockTab::Terminal);
    let rows = runtime.terminal_sessions.active_render_row_count();
    let visible = usize::from(runtime.terminal_screen_geometry().rows);
    let maximum = rows.saturating_sub(visible);
    runtime.session.workspace_mut().ui.terminal.scroll_offset = 0;
    runtime.scene_dirty = false;
    assert!(
        !runtime.handle_dock_scroll(-1.0),
        "bottom boundary must not redraw"
    );
    assert!(!runtime.scene_dirty);
    runtime.session.workspace_mut().ui.terminal.scroll_offset = maximum;
    assert!(
        !runtime.handle_dock_scroll(1.0),
        "top boundary must not redraw or overscroll"
    );
    assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum);
    assert!(!runtime.scene_dirty);
    for delta in [0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(!runtime.handle_dock_scroll(delta));
        assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum);
        assert!(!runtime.scene_dirty);
    }
    if maximum > 0 {
        assert!(runtime.handle_dock_scroll(-1.0));
        assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum - 1);
        assert!(runtime.scene_dirty);
    }
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
