use super::*;
use crate::terminal_session::TerminalSessionRegistry;
use clap::Parser;
use std::time::{Duration, Instant};

fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "datum-unbound-test-{}",
        eda_engine::preferences::new_product_id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn profile(extra: &[&str]) -> TerminalLaunchProfile {
    let mut argv = vec![
        "datum-gui",
        "--terminal-session-profile",
        "custom",
        "--terminal-program",
        "/bin/sh",
    ];
    argv.extend(extra);
    let args = crate::GuiArgs::parse_from(argv);
    crate::terminal_profile::TerminalProfileCatalog::from_args(&args.terminal_profile)
        .unwrap()
        .selected()
        .clone()
}

fn drain_until(
    registry: &mut TerminalSessionRegistry,
    lane: &mut datum_gui_protocol::TerminalLaneState,
    expected: &str,
) {
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until {
        registry.drain_all(lane);
        if registry.test_active_text().contains(expected) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!(
        "missing terminal result {expected:?}: {}",
        registry.test_active_text()
    );
}

fn finish(
    registry: &mut TerminalSessionRegistry,
    lane: &mut datum_gui_protocol::TerminalLaneState,
) {
    let until = Instant::now() + Duration::from_secs(7);
    registry.terminate_all_by(until);
    while Instant::now() < until {
        registry.drain_all(lane);
        if registry.all_sessions_closed() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("unbound terminal did not complete guarded shutdown");
}

#[test]
fn real_unbound_shell_clears_profile_authority_keeps_agent_environment_and_resolves_cohort_cli() {
    let cwd = root();
    let profile = profile(&[
        "--terminal-env",
        "DATUM_PROJECT_ROOT=/stale",
        "--terminal-env",
        "DATUM_DISCOVERY=/stale",
        "--terminal-env",
        "DATUM_AGENT_CREDENTIAL_FILE=/stale",
        "--terminal-env",
        "DATUM_ENGINE_SOCKET=/stale",
        "--terminal-env",
        "EDA_ENGINE_SOCKET=/stale",
        "--terminal-env",
        "OPENAI_API_KEY=owned-test-value",
        "--terminal-env",
        "DATUM_AGENT_PROVENANCE=stale-parent-provenance",
        "--terminal-env",
        "PATH=/usr/bin:/bin",
    ]);
    let context = TerminalLaunchContext::unbound(cwd.clone(), profile);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let runtime = registry
        .active()
        .context_path
        .parent()
        .unwrap()
        .to_path_buf();
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry.active().write_bytes(b"printf 'UNBOUND:%s:%s:%s:%s:%s:%s:%s\\n' \"${DATUM_PROJECT_ROOT-unset}\" \"${DATUM_DISCOVERY-unset}\" \"${DATUM_AGENT_CREDENTIAL_FILE-unset}\" \"${DATUM_ENGINE_SOCKET-unset}\" \"${EDA_ENGINE_SOCKET-unset}\" \"${OPENAI_API_KEY+present}\" \"${DATUM_AGENT_PROVENANCE-unset}\"\ncommand -v datum-eda\npwd\n").unwrap();
    drain_until(
        &mut registry,
        &mut lane,
        "UNBOUND:unset:unset:unset:unset:unset:present:unset",
    );
    drain_until(
        &mut registry,
        &mut lane,
        cli_companion().unwrap().to_str().unwrap(),
    );
    assert!(!cwd.join(".datum").exists());
    assert!(
        registry
            .active_origin_context()
            .unwrap()
            .project_root
            .is_none()
    );
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry.active().context_path).unwrap()).unwrap();
    assert_eq!(document["context_kind"], "unbound");
    assert!(document.get("project_root").is_none());
    assert_eq!(fs::metadata(&runtime).unwrap().mode() & 0o777, 0o700);
    assert!(!fs::read_dir(&runtime).unwrap().any(|p| {
        p.unwrap()
            .file_name()
            .to_string_lossy()
            .contains("credential")
    }));
    finish(&mut registry, &mut lane);
    drop(registry);
    assert!(!runtime.exists());
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn unbound_tabs_splits_restart_and_exit_keep_their_origin_after_project_publication() {
    let cwd = root();
    let context = TerminalLaunchContext::unbound(cwd.clone(), profile(&[]));
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry.sync_lane_tabs(&mut lane);
    let original = registry.active().session_id().to_string();
    registry
        .active()
        .write_bytes(b"export C08_SESSION_SENTINEL=preserved\nprintf 'C08_%s_READY\\n' ORIGIN\n")
        .unwrap();
    drain_until(&mut registry, &mut lane, "C08_ORIGIN_READY");
    let destination = cwd.join("bound");
    fs::create_dir(&destination).unwrap();
    let bound = TerminalLaunchContext::for_project_root(&destination);
    let prepared = registry
        .reserve_project_terminal()
        .unwrap()
        .prepare(bound.clone())
        .unwrap();
    registry
        .publish_project_terminal(prepared, &mut lane)
        .unwrap();
    registry.activate_with_lane(&original, &mut lane).unwrap();
    registry.active().write_bytes(b"printf 'SURVIVED:%s:%s\\n' \"$C08_SESSION_SENTINEL\" \"${DATUM_PROJECT_ROOT-unset}\"\n").unwrap();
    drain_until(&mut registry, &mut lane, "SURVIVED:preserved:unset");
    assert_eq!(
        registry
            .active_origin_context()
            .unwrap()
            .launch_working_directory,
        cwd
    );
    let origin = registry.active_origin_context().unwrap().clone();
    registry
        .begin_split_and_activate(
            &origin,
            &mut lane,
            datum_gui_protocol::TerminalSplitDirection::SideBySide,
        )
        .unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while registry.len() < 3 && Instant::now() < until {
        registry.complete_pending_spawns(&mut lane);
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(registry.len(), 3);
    assert!(
        registry
            .active_origin_context()
            .unwrap()
            .project_root
            .is_none()
    );
    let before = registry.active().session_id().to_string();
    registry.restart_active(&mut lane, &bound).unwrap();
    let until = Instant::now() + Duration::from_secs(7);
    while registry.active().session_id() == before && Instant::now() < until {
        registry.drain_all(&mut lane);
        registry
            .complete_pending_restarts(&mut lane, &bound)
            .unwrap();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_ne!(registry.active().session_id(), before);
    assert!(
        registry
            .active_origin_context()
            .unwrap()
            .project_root
            .is_none()
    );
    registry
        .active()
        .write_bytes(b"printf 'RESTART:%s\\n' \"${DATUM_PROJECT_ROOT-unset}\"\nexit 7\n")
        .unwrap();
    drain_until(&mut registry, &mut lane, "RESTART:unset");
    finish(&mut registry, &mut lane);
    drop(registry);
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn project_relative_profile_refuses_without_manufacturing_a_root() {
    let cwd = root();
    let configured = profile(&["--terminal-cwd", "project"]);
    assert!(
        configured
            .resolve_unbound(&cwd)
            .unwrap_err()
            .to_string()
            .contains("requires a Project")
    );
    assert!(!cwd.join(".datum").exists());
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn dropping_unbound_session_keeps_runtime_until_resistant_child_is_reaped() {
    let cwd = root();
    let context = TerminalLaunchContext::unbound(cwd.clone(), profile(&[]));
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let runtime = registry
        .active()
        .context_path
        .parent()
        .unwrap()
        .to_path_buf();
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry
        .active()
        .write_bytes(
            b"trap '' HUP TERM\nprintf 'C08_%s_READY\\n' DROP\nwhile :; do sleep 1; done\n",
        )
        .unwrap();
    drain_until(&mut registry, &mut lane, "C08_DROP_READY");
    drop(registry);
    assert!(
        runtime.exists(),
        "runtime must survive asynchronous teardown"
    );
    let until = Instant::now() + Duration::from_secs(7);
    while runtime.exists() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !runtime.exists(),
        "supervised closure must release unbound runtime"
    );
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn sanitization_preserves_configuration_and_foreign_agent_authentication() {
    for key in [
        "DATUM_PROJECT_ROOT",
        "DATUM_DISCOVERY",
        "DATUM_AGENT_CREDENTIAL_FILE",
        "DATUM_MCP_ENDPOINT",
        "DATUM_ENGINE_SOCKET",
        "EDA_ENGINE_SOCKET",
    ] {
        assert!(authoritative_environment_key(std::ffi::OsStr::new(key)));
    }
    for key in [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "SSH_AUTH_SOCK",
        "DATUM_GUI_PREFERENCES_PATH",
        "DATUM_AGENT_MCP_PROBE",
        "DATUM_PREFERENCES_ROOT",
        "XDG_CONFIG_HOME",
    ] {
        assert!(!authoritative_environment_key(std::ffi::OsStr::new(key)));
    }
}

#[test]
fn unbound_new_tab_reports_exact_inactive_exit_without_a_project() {
    let cwd = root();
    let context = TerminalLaunchContext::unbound(cwd.clone(), profile(&[]));
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry.sync_lane_tabs(&mut lane);
    let first = registry.active().session_id().to_string();
    registry
        .begin_spawn_and_activate(&context, &mut lane)
        .unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while registry.len() < 2 && Instant::now() < until {
        registry.complete_pending_spawns(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(registry.len(), 2);
    let second = registry.active().session_id().to_string();
    assert!(
        registry
            .active_origin_context()
            .unwrap()
            .project_root
            .is_none()
    );
    registry.active().write_bytes(b"exit 7\n").unwrap();
    registry.activate_with_lane(&first, &mut lane).unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until {
        registry.drain_all(&mut lane);
        registry.sync_lane_tabs(&mut lane);
        if lane
            .tabs
            .iter()
            .any(|t| t.session_id == second && t.status == "exited 7")
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        lane.tabs
            .iter()
            .any(|t| t.session_id == second && t.status == "exited 7")
    );
    assert!(!cwd.join(".datum").exists());
    finish(&mut registry, &mut lane);
    drop(registry);
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn absolute_profile_cwd_is_the_real_origin_and_all_persisted_metadata() {
    let cwd = root();
    let actual = cwd.join("explicit cwd");
    fs::create_dir(&actual).unwrap();
    let configured = profile(&["--terminal-cwd", actual.to_str().unwrap()]);
    let context = TerminalLaunchContext::unbound(cwd.clone(), configured);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry.resize_active_surface(200, 24, 1600, 480).unwrap();
    registry
        .active()
        .write_bytes(b"printf 'ACTUAL_CWD:%s\\n' \"$PWD\"\n")
        .unwrap();
    drain_until(
        &mut registry,
        &mut lane,
        &format!("ACTUAL_CWD:{}", actual.display()),
    );
    assert_eq!(
        registry
            .active_origin_context()
            .unwrap()
            .launch_working_directory,
        actual
    );
    for path in [
        &registry.active().context_path,
        &registry.active().latest_context_path,
        &registry.active().session_path,
    ] {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(value["working_directory"], actual.to_str().unwrap());
    }
    finish(&mut registry, &mut lane);
    drop(registry);
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn unavailable_cli_companion_does_not_prevent_an_ordinary_shell_process() {
    let cwd = root();
    let resolved = profile(&[]).resolve_unbound(&cwd).unwrap();
    let request = TerminalTransportRequest::new(&resolved.executable, cwd.clone())
        .args(["-c", "printf shell-without-cli"]);
    let request = cli_environment(
        request,
        &resolved,
        Err(anyhow::anyhow!("owned missing companion")),
    )
    .unwrap();
    let (mut command, _, _) = request.into_command();
    let output = command.output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"shell-without-cli");
    assert!(!cwd.join(".datum").exists());
    fs::remove_dir_all(cwd).unwrap();
}
