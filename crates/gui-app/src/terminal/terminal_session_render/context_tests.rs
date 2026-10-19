use super::*;
use clap::Parser;
use datum_gui_protocol::TerminalProgressState;

#[test]
fn terminal_identity_actual_profile_launch_directory_survives_refresh_restart_and_final_close() {
    for relative in [false, true] {
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-cwd-{}",
            eda_engine::preferences::new_product_id()
        ));
        let actual = root.join("actual-start");
        std::fs::create_dir_all(&actual).unwrap();
        let configured = if relative {
            "actual-start".into()
        } else {
            actual.display().to_string()
        };
        let args = crate::GuiArgs::try_parse_from([
            "datum-gui",
            "--terminal-session-profile",
            "custom",
            "--terminal-program",
            "/bin/sh",
            "--terminal-arg",
            "-i",
            "--terminal-cwd",
            &configured,
        ])
        .unwrap();
        let mut origin = TerminalLaunchContext::for_project_root(&root);
        origin.project_name = Some("Actual cwd Project".into());
        origin.terminal_profile =
            crate::terminal_profile::TerminalProfileCatalog::from_args(&args.terminal_profile)
                .unwrap()
                .selected()
                .clone();
        let mut registry = TerminalSessionRegistry::spawn(&origin).unwrap();
        let mut lane = TerminalLaneState::default();
        for generation in 0..2 {
            registry.sync_lane_tabs(&mut lane);
            let context = lane.session_context.as_ref().unwrap();
            assert_eq!(context.directory_label(), "Started in");
            assert_eq!(context.directory, actual.display().to_string());
            assert_eq!(context.binding_label(), "Project: Actual cwd Project");
            let marker = root.join(format!("cwd-{generation}.txt"));
            registry
                .active()
                .write_bytes(format!("pwd -P > '{}'\r", marker.display()).as_bytes())
                .unwrap();
            let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while !std::fs::read_to_string(&marker).is_ok_and(|v| !v.trim().is_empty())
                && std::time::Instant::now() < until
            {
                registry.drain_all(&mut lane);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert_eq!(
                std::fs::read_to_string(marker).unwrap().trim(),
                actual.canonicalize().unwrap().to_str().unwrap()
            );
            // The existing semantic refresh can reconstruct the launch context
            // from Project state; it is not a launch-directory authority.
            registry.update_active_origin_context(origin.clone());
            lane.current_working_directory = Some("file:///old/reported/directory".into());
            if generation == 0 {
                let old = registry.active().session_id().to_owned();
                registry.restart_active(&mut lane, &origin).unwrap();
                let until = std::time::Instant::now()
                    + std::time::Duration::from_millis(
                        crate::terminal_transport::GLOBAL_SHUTDOWN_MS,
                    );
                while registry.active().session_id() == old && std::time::Instant::now() < until {
                    registry.drain_all(&mut lane);
                    registry
                        .complete_pending_restarts(&mut lane, &origin)
                        .unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                assert_ne!(registry.active().session_id(), old);
                assert!(lane.current_working_directory.is_none());
            }
        }
        let until = std::time::Instant::now()
            + std::time::Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
        registry.terminate_all_by(until);
        while !registry.all_sessions_closed() && std::time::Instant::now() < until {
            registry.drain_all(&mut lane);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(registry.all_sessions_closed());
        registry.close_active(&mut lane).unwrap();
        registry.sync_lane_tabs(&mut lane);
        assert!(lane.tabs.is_empty() && lane.active_session_id.is_none());
        assert!(
            lane.session_context.is_none(),
            "closed final session must not project a binding"
        );
        drop(registry);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn terminal_identity_keeps_binding_separate_from_cwd_title_and_user_name() {
    let mut origin = TerminalLaunchContext::for_project_root(Path::new("/project/original"));
    origin.project_name = Some("Original Project".into());
    let context = session_context(
        &origin,
        &origin.launch_working_directory,
        Some("file://localhost/home/user/elsewhere%20now"),
    );
    assert_eq!(context.binding_label(), "Project: Original Project");
    assert_eq!(context.directory, "/home/user/elsewhere now");
    assert_eq!(context.directory_label(), "Cwd");
    assert!(
        context
            .description()
            .contains("Project root: /project/original")
    );
    let label = session_tab_label(
        "shell 2",
        false,
        Some(&context),
        Some("vim"),
        TerminalProgressState::Clear,
        false,
    );
    assert_eq!(label, "vim");
    assert_eq!(
        session_tab_label(
            "shell 2",
            false,
            Some(&context),
            None,
            TerminalProgressState::Clear,
            false
        ),
        "Original Project"
    );
    let renamed = session_tab_label(
        "My agent",
        true,
        Some(&context),
        Some("vim"),
        TerminalProgressState::Set { percent: 35 },
        false,
    );
    assert_eq!(renamed, "My agent · 35%");
    assert_eq!(context.binding_label(), "Project: Original Project");
    let independent =
        TerminalLaunchContext::unbound(PathBuf::from("/home/user"), Default::default());
    let context = session_context(
        &independent,
        &independent.launch_working_directory,
        Some("file:///project/original"),
    );
    assert_eq!(context.binding_label(), "Independent shell");
    assert_eq!(context.directory, "/project/original");
}

#[test]
fn terminal_identity_invalid_reports_show_started_in_without_fabricating_current_cwd() {
    let origin = TerminalLaunchContext::for_project_root(Path::new("/project/start"));
    for report in [
        None,
        Some("file://remote.example/project/other"),
        Some("relative/path"),
        Some("file:///bad%XX"),
        Some("file:///path?query"),
    ] {
        let context = session_context(&origin, &origin.launch_working_directory, report);
        assert_eq!(context.directory_label(), "Started in");
        assert_eq!(context.directory, "/project/start");
    }
}

#[test]
fn terminal_identity_live_publication_and_pending_switch_never_mix_origins() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-identity-{}",
        eda_engine::preferences::new_product_id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let independent = TerminalLaunchContext::unbound(root.clone(), Default::default());
    let mut registry = TerminalSessionRegistry::spawn(&independent).unwrap();
    let old = registry.active().session_id().to_owned();
    let mut lane = TerminalLaneState {
        title: Some("old agent".into()),
        current_working_directory: Some(root.display().to_string()),
        ..Default::default()
    };
    let mut project = TerminalLaunchContext::for_project_root(&root);
    project.project_name = Some("Bound Project".into());
    let prepared = registry
        .reserve_project_terminal()
        .unwrap()
        .prepare(project)
        .unwrap();
    registry
        .publish_project_terminal(prepared, &mut lane)
        .unwrap();
    assert_eq!(
        lane.session_context.as_ref().unwrap().binding_label(),
        "Project: Bound Project"
    );
    assert_eq!(
        lane.session_context.as_ref().unwrap().directory_label(),
        "Started in"
    );
    assert_eq!(
        registry.sessions[0]
            .parked_lane
            .session_context
            .as_ref()
            .unwrap()
            .binding_label(),
        "Independent shell"
    );
    registry
        .rename(
            registry.active().session_id().to_owned().as_str(),
            "my shell",
        )
        .unwrap();
    registry.sync_lane_tabs(&mut lane);
    assert_eq!(
        lane.tabs.iter().find(|t| t.active).unwrap().label,
        "my shell"
    );
    registry.activate_with_lane(&old, &mut lane).unwrap();
    registry.sync_lane_tabs(&mut lane);
    assert_eq!(lane.title.as_deref(), Some("old agent"));
    assert_eq!(
        lane.session_context.as_ref().unwrap().binding_label(),
        "Independent shell"
    );
    assert_eq!(
        lane.session_context.as_ref().unwrap().directory_label(),
        "Cwd"
    );
    registry.active_pending_id = Some("pending-proof".into());
    registry.sync_session_contexts(&mut lane);
    assert!(lane.session_context.is_none());
    registry.active_pending_id = None;
    registry.restart_active(&mut lane, &independent).unwrap();
    let until = std::time::Instant::now()
        + std::time::Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while registry.active().session_id() == old && std::time::Instant::now() < until {
        registry.drain_all(&mut lane);
        registry
            .complete_pending_restarts(&mut lane, &independent)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_ne!(registry.active().session_id(), old);
    assert!(lane.current_working_directory.is_none());
    assert_eq!(
        lane.session_context.as_ref().unwrap().directory_label(),
        "Started in"
    );
    assert_eq!(
        lane.session_context.as_ref().unwrap().binding_label(),
        "Independent shell"
    );
    let until = std::time::Instant::now()
        + std::time::Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    registry.terminate_all_by(until);
    while !registry.all_sessions_closed() && std::time::Instant::now() < until {
        registry.drain_all(&mut lane);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(registry.all_sessions_closed());
    drop(registry);
    std::fs::remove_dir_all(root).unwrap();
}
