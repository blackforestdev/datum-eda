#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use crate::terminal_session::TerminalLaunchContext;
use datum_gui_protocol::TerminalSplitDirection;
use std::{fs, time::Duration};

#[test]
fn restarting_split_retains_owned_slot_until_verified_replacement() {
    let root = development_workspace::path("open-project-split-restart-tests")
        .join(std::process::id().to_string());
    let origin = root.join("origin");
    let cwd = origin.join("board");
    let destination = root.join("destination");
    fs::create_dir_all(&cwd).unwrap();
    fs::create_dir_all(&destination).unwrap();
    let mut context = TerminalLaunchContext::for_project_root(&origin);
    context.launch_working_directory = cwd.clone();
    let current = TerminalLaunchContext::for_project_root(&destination);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let sibling = registry.active().session_id().to_owned();
    let sibling_context = registry.active().context_id.clone();
    let mut lane = TerminalLaneState::default();
    registry
        .begin_split_and_activate(&context, &mut lane, TerminalSplitDirection::SideBySide)
        .unwrap();
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while !registry.pending_spawns.is_empty() && Instant::now() < deadline {
        registry.complete_pending_spawns(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.pending_spawns.is_empty());
    let old = registry.active().session_id().to_owned();
    let old_context = registry.active().context_id.clone();
    let tab = registry.terminal_tabs[0].tab_id.clone();
    assert_eq!(registry.sessions.len(), 2);
    registry.restart_active(&mut lane, &current).unwrap();
    assert!(registry.sessions[registry.active_index].pending_restart);
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while registry.active().session_id() == old && Instant::now() < deadline {
        // Ordinary application order: drain/closed-tab removal BEFORE restart completion.
        registry.drain_all(&mut lane);
        assert_eq!(registry.sessions.len(), 2);
        assert!(
            registry
                .sessions
                .iter()
                .any(|slot| slot.session.session_id() == old)
        );
        registry
            .complete_pending_restarts(&mut lane, &current)
            .unwrap();
        std::thread::sleep(Duration::from_millis(5));
    }
    let new = registry.active().session_id().to_owned();
    assert_ne!(new, old);
    assert_ne!(registry.active().context_id, old_context);
    assert_eq!(registry.terminal_tabs[0].tab_id, tab);
    assert!(registry.terminal_tabs[0].root.contains_session(&sibling));
    assert!(registry.terminal_tabs[0].root.contains_session(&new));
    assert!(!registry.terminal_tabs[0].root.contains_session(&old));
    assert_eq!(
        registry
            .sessions
            .iter()
            .find(|slot| slot.session.session_id() == sibling)
            .unwrap()
            .session
            .context_id,
        sibling_context
    );
    let bound = registry.active_origin_context().unwrap();
    assert_eq!(bound.project_root, Some(origin.clone()));
    assert_eq!(bound.launch_working_directory, cwd);
    assert_eq!(lane.status, "running");
    assert!(registry.active_attached());
    let marker = root.join("restarted-shell.txt");
    let command = format!(
        "printf '%s\\n' \"$DATUM_PROJECT_ROOT\" \"$PWD\" \"$DATUM_SESSION_ID\" \"$DATUM_CONTEXT_ID\" > '{}'\r",
        marker.display()
    );
    registry.active().write_bytes(command.as_bytes()).unwrap();
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while !marker.exists() && Instant::now() < deadline {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    let values = fs::read_to_string(marker).unwrap();
    let values = values.lines().collect::<Vec<_>>();
    assert_eq!(values.len(), 4);
    assert_eq!(
        std::path::Path::new(values[0]).canonicalize().unwrap(),
        origin.canonicalize().unwrap()
    );
    assert_eq!(
        std::path::Path::new(values[1]).canonicalize().unwrap(),
        cwd.canonicalize().unwrap()
    );
    assert_eq!(values[2], new);
    assert_eq!(values[3], registry.active().context_id);
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    registry.terminate_all_by(deadline);
    while !registry.all_sessions_closed() && Instant::now() < deadline {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.all_sessions_closed());
}

#[test]
fn global_teardown_cancels_pending_split_restart_without_new_children() {
    let root = development_workspace::path("open-project-restart-shutdown-tests")
        .join(std::process::id().to_string());
    fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    registry
        .begin_split_and_activate(&context, &mut lane, TerminalSplitDirection::SideBySide)
        .unwrap();
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while !registry.pending_spawns.is_empty() && Instant::now() < deadline {
        registry.complete_pending_spawns(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.pending_spawns.is_empty());
    assert_eq!(registry.sessions.len(), 2);
    let ids = registry
        .sessions
        .iter()
        .map(|slot| slot.session.session_id().to_owned())
        .collect::<Vec<_>>();
    registry.restart_active(&mut lane, &context).unwrap();
    assert!(registry.sessions[registry.active_index].pending_restart);
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    registry.terminate_all_by(deadline);
    assert!(registry.sessions.iter().all(|slot| !slot.pending_restart));
    while !registry.all_sessions_closed() && Instant::now() < deadline {
        registry.drain_all(&mut lane);
        assert!(
            !registry
                .complete_pending_restarts(&mut lane, &context)
                .unwrap()
        );
        assert!(
            registry
                .sessions
                .iter()
                .all(|slot| ids.iter().any(|id| id == slot.session.session_id()))
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.all_sessions_closed());
    assert!(
        !registry
            .complete_pending_restarts(&mut lane, &context)
            .unwrap()
    );
    let eventlogs = fs::read_dir(root.join(".datum/tool-sessions"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".events.jsonl")
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        eventlogs.len(),
        2,
        "global shutdown allocated a replacement child"
    );
}
