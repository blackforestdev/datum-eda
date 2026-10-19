//! Real publication with prepared native sources and owned PTYs on Wayland.
use super::*;
use clap::Parser;
use std::sync::Arc;
use winit::platform::wayland::EventLoopBuilderExtWayland;

struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn prepared_answer(runtime: &Runtime, root: &std::path::Path) -> Answer {
    let prepared = prepare_project_directory(root.to_str().unwrap(), false).unwrap();
    let origin = if runtime.workspace().project_unopened {
        ProjectOpenOrigin::capture_unopened(runtime.project_open.session_generation)
    } else {
        ProjectOpenOrigin::capture(&prepared.root, runtime.project_open.session_generation).unwrap()
    };
    let context = crate::terminal_session::terminal_launch_context_from_state_with_cursor(
        &prepared.root,
        &prepared.workspace,
        None,
    );
    let terminal = runtime
        .terminal_sessions
        .reserve_project_terminal()
        .unwrap()
        .prepare(context.clone())
        .unwrap();
    let admission = prepared.admit().unwrap();
    Answer {
        origin,
        ticket: runtime.project_open.generation,
        prepared,
        terminal,
        context,
        _admission: admission,
    }
}

#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn project_terminal_handoff_preserves_dock_and_original_session_on_publication_and_refusal() {
    let root = Scratch(std::env::temp_dir().join(format!(
        "datum-handoff-{}",
        eda_engine::preferences::new_product_id()
    )));
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
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
                Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let args = GuiArgs::try_parse_from(["datum-gui"]).unwrap();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let original = runtime.terminal_sessions.active().session_id().to_owned();
    runtime.session.workspace_mut().ui.terminal.title = Some("original agent".into());
    runtime.session.workspace_mut().ui.terminal.scroll_offset = 17;
    for (tab, height, maximized) in [
        (Some(DockTab::Terminal), 287, false),
        (None, 313, false),
        (Some(DockTab::Terminal), 369, true),
    ] {
        {
            let ui = &mut runtime.session.workspace_mut().ui;
            ui.active_dock_tab = tab;
            ui.dock_height_px = height;
            ui.terminal_maximized = maximized;
        }
        runtime.open_project();
        let previous = runtime.terminal_sessions.active().session_id().to_owned();
        // Cancellation exposes no staged session and never resets geometry.
        runtime.close_project_open();
        assert_eq!(runtime.terminal_sessions.active().session_id(), previous);
        assert_eq!(
            (
                runtime.workspace().ui.active_dock_tab,
                runtime.workspace().ui.dock_height_px,
                runtime.workspace().ui.terminal_maximized
            ),
            (tab, height, maximized)
        );
        runtime.open_project();
        if height == 287 {
            let mut stale = prepared_answer(&runtime, &root.0);
            stale.ticket = stale.ticket.wrapping_add(1);
            assert!(runtime.publish_project_open(stale).is_err());
            assert_eq!(runtime.terminal_sessions.active().session_id(), previous);
            assert_eq!(
                (
                    runtime.workspace().ui.active_dock_tab,
                    runtime.workspace().ui.dock_height_px,
                    runtime.workspace().ui.terminal_maximized
                ),
                (tab, height, maximized)
            );
        }
        let answer = prepared_answer(&runtime, &root.0);
        runtime.publish_project_open(answer).unwrap();
        // This is before opening/focusing/resizing the panel: UI actions must
        // not repair a publication reset and hide the regression.
        assert_eq!(
            (
                runtime.workspace().ui.active_dock_tab,
                runtime.workspace().ui.dock_height_px,
                runtime.workspace().ui.terminal_maximized
            ),
            (tab, height, maximized)
        );
        assert_ne!(runtime.terminal_sessions.active().session_id(), previous);
        assert!(matches!(
            runtime.workspace().ui.focus,
            ApplicationFocus::Editor(_)
        ));
        assert_eq!(
            runtime
                .terminal_sessions
                .active_origin_context()
                .unwrap()
                .project_root
                .as_deref(),
            Some(root.0.as_path())
        );
        assert_eq!(
            runtime
                .workspace()
                .ui
                .terminal
                .session_context
                .as_ref()
                .unwrap()
                .binding_label(),
            format!("Project: {}", runtime.workspace().scene.project_name)
        );
        if height == 287 {
            runtime.refresh_terminal_accessibility();
            let cells = runtime
                .terminal_accessibility
                .current()
                .unwrap()
                .text
                .clone();
            assert!(runtime.close_active_terminal_session());
            runtime.refresh_terminal_accessibility();
            let snapshot = runtime.terminal_accessibility.current().unwrap();
            assert!(
                snapshot
                    .title
                    .contains("Status: close terminal? Enter confirms; Escape cancels")
            );
            assert!(snapshot.title.contains(&format!(
                "Project: {}",
                runtime.workspace().scene.project_name
            )));
            assert!(snapshot.title.contains(root.0.to_str().unwrap()));
            assert_eq!(
                snapshot.text, cells,
                "lifecycle accessibility must not consume PTY rows"
            );
            runtime.handle_terminal_lifecycle_target(&HitTarget::TerminalShutdownCancel);
        }
    }
    runtime.activate_terminal_session(&original);
    assert_eq!(
        runtime.workspace().ui.terminal.title.as_deref(),
        Some("original agent")
    );
    assert_eq!(runtime.workspace().ui.terminal.scroll_offset, 17);
    assert!(
        runtime
            .terminal_sessions
            .active_origin_context()
            .unwrap()
            .project_root
            .is_none()
    );
    assert_eq!(
        runtime
            .workspace()
            .ui
            .terminal
            .session_context
            .as_ref()
            .unwrap()
            .binding_label(),
        "Independent shell"
    );
    let until = std::time::Instant::now()
        + std::time::Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    runtime.terminal_sessions.terminate_all_by(until);
    while !runtime.terminal_sessions.all_sessions_closed() && std::time::Instant::now() < until {
        runtime
            .terminal_sessions
            .drain_all(&mut runtime.session.workspace_mut().ui.terminal);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(runtime.terminal_sessions.all_sessions_closed());
}
