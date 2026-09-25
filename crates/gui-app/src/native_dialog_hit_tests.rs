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

/// S4 adapter proof: one device, serial native hosts, and real hidden PTY work.
/// This does not qualify desktop input, displayed pixels, resize or endurance.
#[test]
#[ignore = "requires X11/Vulkan, isolated config and DATUM_NATIVE_TEST_BOARD; run serially"]
#[allow(deprecated)]
fn native_shared_device_hosts_preserve_hidden_terminal_and_console_work() {
    use datum_gui_protocol::{ConsoleFeedbackDraft, ConsoleFeedbackSource, DockTab};
    let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("owned real board fixture");
    let args = GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
    let events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let make_window = || {
        Arc::new(
            events
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_inner_size(winit::dpi::PhysicalSize::new(960, 720)),
                )
                .unwrap(),
        )
    };
    let wake = events.create_proxy();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime =
        pollster::block_on(Runtime::new(make_window(), launch, Some(1.0), wake)).unwrap();
    runtime.session.workspace_mut().ui.active_dock_tab = None;
    assert!(runtime.ensure_retained_scene());
    runtime.terminal_sessions.active().write_bytes(
        b"for i in 1 2 3 4 5; do printf 'S4-row-%s\\n' \"$i\"; done; printf 'PM045_S4_%s\\n' DONE\n"
    ).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    let before = loop {
        runtime.poll_terminal_output();
        let core = runtime.terminal_sessions.active_core_mut();
        let (snapshot, damage) = core.take_render_state().unwrap();
        core.merge_render_damage(&damage);
        let text: String = snapshot
            .rows()
            .flat_map(|row| row.cells())
            .filter_map(|cell| match &cell.content {
                datum_terminal_core::CellContent::Cluster(cluster) => Some(cluster.text()),
                _ => None,
            })
            .collect();
        if text.contains("PM045_S4_DONE") {
            break snapshot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "hidden PTY output must reach TerminalCore"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    runtime
        .session
        .workspace_mut()
        .ui
        .console
        .publish(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Viewport,
            42,
            "S4 retained Console feedback",
        ));
    let closed = runtime.build_terminal_prepared_scene().unwrap();
    assert!(
        closed
            .console_overlay_layout()
            .is_none_or(|layout| layout.history_panel.is_none())
    );
    let core = runtime.terminal_sessions.active_core_mut();
    let (after, damage) = core.take_render_state().unwrap();
    assert_eq!(
        before, after,
        "closed preparation preserves terminal bytes and state"
    );
    assert!(
        !damage.is_empty(),
        "closed preparation must not consume a render snapshot"
    );
    // A deliberate hidden snapshot consumes the same observable damage. This
    // control makes the absence assertion sensitive to that forbidden work.
    assert!(core.take_render_state().unwrap().1.is_empty());
    core.merge_render_damage(&damage);
    runtime.session.workspace_mut().ui.active_dock_tab = Some(DockTab::Terminal);
    runtime
        .session
        .workspace_mut()
        .ui
        .console
        .set_history_expanded(true);
    runtime.resize_terminal_to_dock();
    runtime.prepared_scene = None;
    let present_main = |runtime: &mut Runtime| {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !runtime.render().unwrap() {
            assert!(
                std::time::Instant::now() < deadline,
                "main native presentation"
            );
            runtime.device.poll(wgpu::PollType::Poll).unwrap();
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    present_main(&mut runtime);
    assert!(
        runtime
            .prepared_scene
            .as_ref()
            .unwrap()
            .console_overlay_layout()
            .unwrap()
            .history_panel
            .is_some(),
        "accumulated Console history reopens"
    );
    assert!(
        runtime
            .terminal_sessions
            .active_core_mut()
            .take_render_state()
            .unwrap()
            .1
            .is_empty(),
        "visible preparation consumed the pending terminal update"
    );
    for host in ["GLOBAL", "PROJECT", "NEW"] {
        match host {
            "GLOBAL" => {
                runtime.open_global_preferences();
            }
            "PROJECT" => {
                runtime.open_project_preferences();
            }
            _ => {
                runtime.open_new_project();
            }
        }
        // Every owned host takes this same Runtime device and queue. No second
        // adapter/device is created by this serial multiwindow scenario.
        let mut surface =
            GlobalPreferencesWindowSurface::new(&runtime, make_window(), Some(1.0)).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !surface
            .render(&runtime, host == "PROJECT", host == "NEW")
            .unwrap()
        {
            assert!(
                std::time::Instant::now() < deadline,
                "{host} native presentation"
            );
            runtime.device.poll(wgpu::PollType::Poll).unwrap();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !surface.presented_hits.regions().is_empty(),
            "{host} published hits"
        );
        drop(surface);
        match host {
            "GLOBAL" => {
                runtime.close_global_preferences();
            }
            "PROJECT" => {
                runtime.close_project_preferences();
            }
            _ => {
                runtime.close_new_project();
            }
        }
        present_main(&mut runtime);
        eprintln!("S4 serial native shared-device host {host}: presented, closed, Main resumed");
    }
    runtime.begin_application_terminal_shutdown();
    let deadline = std::time::Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        runtime.poll_terminal_output();
        assert!(std::time::Instant::now() < deadline, "owned PTY shutdown");
        std::thread::sleep(Duration::from_millis(10));
    }
}
