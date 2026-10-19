//! Held real spawn-worker/Runtime controls for truthful pending accessibility.
use super::*;
use crate::{GuiArgs, Runtime};
use clap::Parser;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn terminal_pending_accessibility_never_retains_previous_project_and_restores_attach_or_cancel() {
    let root = Scratch(std::env::temp_dir().join(format!(
        "datum-pending-a11y-{}",
        eda_engine::preferences::new_product_id()
    )));
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    for rel in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ] {
        let path = root.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(rel), path).unwrap();
    }
    let events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = events.create_proxy();
    let window = Arc::new(
        events
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--project-root", root.0.to_str().unwrap()]).unwrap();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    runtime.set_active_dock(datum_gui_protocol::DockTab::Terminal);
    runtime.set_application_focus(datum_gui_protocol::ApplicationFocus::Terminal);
    for cancel in [true, false] {
        runtime.sync_terminal_tabs();
        assert!(
            runtime
                .terminal_accessibility
                .current()
                .unwrap()
                .title
                .contains(root.0.to_str().unwrap())
        );
        let original = runtime.terminal_sessions.active().session_id().to_owned();
        let origin = runtime.context_for_active_terminal_launch().unwrap();
        let (entered, waiting) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel::<bool>();
        runtime
            .terminal_sessions
            .begin_spawn_and_activate_using(
                &origin,
                &mut runtime.session.workspace_mut().ui.terminal,
                move |context, wake| {
                    entered.send(()).unwrap();
                    if !gate.recv().unwrap() {
                        anyhow::bail!("held worker canceled before child preparation");
                    }
                    spawn_terminal_session_with_wake(&context, wake)
                },
            )
            .unwrap();
        waiting.recv_timeout(Duration::from_secs(3)).unwrap();
        // The already-open/Terminal-focused New Tab route has no focus or
        // output wake to repair stale metadata: its normal sync must publish.
        runtime.set_active_dock(datum_gui_protocol::DockTab::Terminal);
        runtime.sync_terminal_tabs();
        let pending = runtime.terminal_accessibility.current().unwrap();
        assert!(pending.title.contains("starting terminal session"));
        assert!(!pending.title.contains(root.0.to_str().unwrap()));
        assert!(!pending.title.contains(&format!(
            "Project: {}",
            runtime.workspace().scene.project_name
        )));
        assert!(pending.text.is_empty() && pending.selection.is_none() && !pending.focused);
        if cancel {
            assert!(runtime.close_active_terminal_session());
            assert!(
                runtime
                    .terminal_accessibility
                    .current()
                    .unwrap()
                    .title
                    .contains(root.0.to_str().unwrap())
            );
            assert_eq!(runtime.terminal_sessions.active().session_id(), original);
        }
        release.send(!cancel).unwrap();
        let until = Instant::now() + Duration::from_secs(5);
        while !runtime.terminal_sessions.pending_spawns.is_empty() && Instant::now() < until {
            runtime
                .terminal_sessions
                .complete_pending_spawns(&mut runtime.session.workspace_mut().ui.terminal);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(runtime.terminal_sessions.pending_spawns.is_empty());
        runtime.sync_terminal_tabs();
        let attached = runtime.terminal_accessibility.current().unwrap();
        assert!(attached.title.contains(root.0.to_str().unwrap()));
        assert_eq!(
            attached.session_id,
            runtime.terminal_sessions.active().session_id()
        );
        assert_eq!(
            runtime.terminal_sessions.active().session_id() == original,
            cancel
        );
    }
    let until =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    runtime.terminal_sessions.terminate_all_by(until);
    while !runtime.terminal_sessions.all_sessions_closed() && Instant::now() < until {
        runtime
            .terminal_sessions
            .drain_all(&mut runtime.session.workspace_mut().ui.terminal);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(runtime.terminal_sessions.all_sessions_closed());
}
