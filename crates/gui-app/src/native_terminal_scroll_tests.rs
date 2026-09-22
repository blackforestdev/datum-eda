//! Actual terminal scrollback bounds and changed-only damage.
use crate::{GuiArgs, Runtime};
use clap::Parser;
use std::{sync::Arc, time::Duration};
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

#[test]
#[ignore = "requires native Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_terminal_scrollback_boundaries_do_not_redraw() {
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
    runtime.session.workspace_mut().ui.active_dock_tab =
        Some(datum_gui_protocol::DockTab::Terminal);
    runtime.resize_terminal_to_dock();
    // Produce real PTY history instead of conditionally skipping reversal when
    // the startup screen happens to contain no scrollback.
    runtime.terminal_sessions.active().write_bytes(
        b"i=0; while [ $i -lt 128 ]; do printf 'PM045 scroll row %s\\n' \"$i\"; i=$((i+1)); done\n"
    ).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(6);
    while runtime.terminal_sessions.active_render_row_count() < 128 {
        runtime.poll_terminal_output();
        assert!(
            std::time::Instant::now() < deadline,
            "real PTY history must arrive"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
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
    // History/capacity changes can leave an old offset beyond today's extent.
    // An outward wheel still changes that state when it clamps back into range.
    runtime.session.workspace_mut().ui.terminal.scroll_offset = maximum + 1;
    assert!(
        runtime.handle_dock_scroll(1.0),
        "clamping stale scrollback must redraw"
    );
    assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum);
    assert!(runtime.scene_dirty);
    runtime.scene_dirty = false;
    for delta in [0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(!runtime.handle_dock_scroll(delta));
        assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum);
        assert!(!runtime.scene_dirty);
    }
    assert!(maximum > 2, "nonempty history is mandatory");
    assert!(runtime.handle_dock_scroll(-1.0));
    assert_eq!(runtime.workspace().ui.terminal.scroll_offset, maximum - 1);
    assert!(runtime.scene_dirty);

    let screen = runtime.terminal_screen_geometry().screen;
    runtime.last_cursor_pos = Some((
        screen.x + screen.width * 0.5,
        screen.y + screen.height * 0.5,
    ));
    let camera = format!("{:?}", runtime.camera);
    runtime.prepared_scene = None;
    for lines in [-8.0_f32, -0.25, 0.25, 8.0] {
        for delta in [
            MouseScrollDelta::LineDelta(0.0, lines),
            MouseScrollDelta::PixelDelta(PhysicalPosition::new(
                0.0,
                f64::from(lines * 20.0 * scale),
            )),
        ] {
            let start = if lines > 0.0 { 0 } else { maximum };
            runtime.session.workspace_mut().ui.terminal.scroll_offset = start;
            runtime.scene_dirty = false;
            assert!(runtime.handle_native_wheel(delta));
            assert_eq!(
                runtime.workspace().ui.terminal.scroll_offset,
                if lines > 0.0 { 1 } else { maximum - 1 },
                "terminal must scroll exactly one row per event"
            );
            assert!(runtime.scene_dirty);
            assert_eq!(format!("{:?}", runtime.camera), camera);
            assert!(runtime.prepared_scene.is_none());
            // Outward input at either boundary stays local and makes no frame.
            runtime.session.workspace_mut().ui.terminal.scroll_offset = maximum - start;
            runtime.scene_dirty = false;
            assert!(!runtime.handle_native_wheel(delta));
            assert!(!runtime.scene_dirty);
            assert_eq!(format!("{:?}", runtime.camera), camera);
            assert!(runtime.prepared_scene.is_none());
        }
    }
    eprintln!(
        "backend={backend} scale={scale} accepted={}x{} rows={rows} visible={visible} maximum={maximum} physical_line_one_row_and_boundary_match=true",
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
