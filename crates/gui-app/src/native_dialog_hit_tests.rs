//! Actual owned surface publication and invalidated coordinate-frame rejection.
use super::*;
use clap::Parser;
use std::{sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_preferences_hits_reject_old_scale_and_extent_until_presented() {
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--board", &board, "--open-global-preferences"])
            .unwrap();
    let events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = events.create_proxy();
    let window = Arc::new(
        events
            .create_window(
                Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(960, 720)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let mut stale_scale_hits = Vec::new();
    for project in [false, true] {
        if project {
            runtime.open_project_preferences();
        }
        let window = Arc::new(
            events
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_inner_size(winit::dpi::PhysicalSize::new(960, 720)),
                )
                .unwrap(),
        );
        let mut surface = GlobalPreferencesWindowSurface::new(&runtime, window, Some(1.0)).unwrap();
        let present = |surface: &mut GlobalPreferencesWindowSurface| {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !surface.render(&runtime, project, false).unwrap() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "native presentation timeout"
                );
                runtime.device.poll(wgpu::PollType::Poll).unwrap();
                std::thread::sleep(Duration::from_millis(10));
            }
        };
        let target = |surface: &mut GlobalPreferencesWindowSurface| {
            let rect = surface
                .presented_hits
                .regions()
                .iter()
                .find(|r| r.target == HitTarget::GlobalPreferencesSearch)
                .unwrap()
                .rect;
            surface.cursor_position = Some((rect.x + rect.width * 0.5, rect.y + rect.height * 0.5));
            assert_eq!(
                surface.hit_target(),
                Some(HitTarget::GlobalPreferencesSearch)
            );
        };
        present(&mut surface);
        target(&mut surface);
        surface.invalidate();
        assert_eq!(
            surface.hit_target(),
            Some(HitTarget::GlobalPreferencesSearch),
            "ordinary damage keeps visible hits"
        );
        present(&mut surface);
        for scale in [1.5, 2.0, 1.0] {
            target(&mut surface);
            surface.set_scale_factor(scale);
            if surface.hit_target().is_some() {
                stale_scale_hits.push((if project { "PROJECT" } else { "GLOBAL" }, scale));
            }
            present(&mut surface);
            target(&mut surface);
        }
        surface.resize(&runtime, 900, 680);
        assert!(
            surface.hit_target().is_none(),
            "old-extent hits must be rejected before presentation"
        );
        present(&mut surface);
        target(&mut surface);
        surface.resize(&runtime, 900, 680);
        assert_eq!(
            surface.hit_target(),
            Some(HitTarget::GlobalPreferencesSearch),
            "unchanged extent retains presented hits"
        );
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
    // Complete both host paths and owned PTY teardown before reporting a defect.
    assert!(
        stale_scale_hits.is_empty(),
        "old-scale hits survived before presentation: {stale_scale_hits:?}"
    );
}
