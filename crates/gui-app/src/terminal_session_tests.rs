use super::*;
use crate::terminal_session_context::TerminalSessionContextSummary;
use datum_gui_protocol::{SelectionTarget, TERMINAL_COMMAND_CATALOG_VERSION};
use std::fs;
use std::time::Duration;

fn terminate_and_remove_active(
    registry: &mut TerminalSessionRegistry,
    state: &mut TerminalLaneState,
) {
    let initial_len = registry.len();
    registry
        .close_active(state)
        .expect("arm close confirmation");
    registry
        .confirm_close_active(state)
        .expect("confirm close termination");
    let deadline = std::time::Instant::now() + Duration::from_secs(7);
    while std::time::Instant::now() < deadline {
        registry.drain_all(state);
        if registry.len() < initial_len || state.tabs.is_empty() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("confirmed terminal close did not remove its presented tab");
}
impl TerminalLaunchContext {
    pub(crate) fn for_project_root(project_root: &std::path::Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            project_id: None,
            project_name: None,
            board_id: None,
            board_name: None,
            scene_id: None,
            source_revision: None,
            accepted_transaction_tip: None,
            production_status: ProductionStatus::default(),
            source_shard_status: datum_gui_protocol::SourceShardStatusSummary::default(),
            check_status: CheckRunReviewState::default(),
            selection_context: DatumSelectionContext::from_selection(&SelectionTarget::None),
            cursor_context: DatumCursorContext {
                screen_px: None,
                hovered_object_id: None,
                active_dock_tab: None,
                active_tool: "select".to_string(),
            },
            projection_context: DatumProjectionContext {
                scene_id: "test-scene".to_string(),
                board_id: None,
                board_name: None,
                scene_bounds_nm: None,
                active_projection_id: None,
            },
            terminal_sessions: TerminalSessionContextSummary::default(),
        }
    }
}

#[test]
fn terminal_session_registry_tracks_active_session_contexts() {
    let root = std::env::temp_dir().join(format!("datum-terminal-registry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal registry test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let first_session_id = registry.active().session_id().to_string();
    let first_context_path = registry.active().context_path.clone();
    let first_event_log_path = registry.active().event_log_path();
    assert_eq!(registry.len(), 1);
    let second_session_id = registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session")
        .to_string();
    let second_event_log_path = registry.active().event_log_path();
    assert_eq!(registry.len(), 2);
    assert_ne!(first_session_id, second_session_id);
    assert_ne!(first_context_path, registry.active().context_path);
    assert!(first_context_path.exists() && registry.active().context_path.exists());
    assert!(
        fs::read_to_string(&first_event_log_path).map_or(true, |contents| !contents
            .contains("\"lifecycle\":\"detached\""))
    );
    assert!(
        fs::read_to_string(&second_event_log_path).map_or(true, |contents| !contents
            .contains("\"lifecycle\":\"attached\""))
    );
    let mut terminal_state = TerminalLaneState::default();
    assert!(registry.resize_active(112, 31).is_ok());
    registry.sync_lane_tabs(&mut terminal_state);
    assert_eq!(
        terminal_state.active_session_id.as_deref(),
        Some(second_session_id.as_str())
    );
    assert_eq!((terminal_state.columns, terminal_state.rows), (112, 31));
    assert_eq!(terminal_state.tabs.len(), 2);
    assert_eq!(terminal_state.tabs[0].label, "shell 1");
    assert!(!terminal_state.tabs[0].active);
    assert!(terminal_state.tabs[0].attached);
    assert_eq!(terminal_state.tabs[1].label, "shell 2");
    assert!(terminal_state.tabs[1].active);
    assert!(terminal_state.tabs[1].attached);
    assert_eq!(
        terminal_state.tabs[0].event_log_path,
        first_event_log_path.display().to_string()
    );
    assert_eq!(
        terminal_state.tabs[1].event_log_path,
        second_event_log_path.display().to_string()
    );
    assert_eq!(
        terminal_state.tabs[0].activity_event_count, 0,
        "background ownership must not invent a detached lifecycle event"
    );
    assert_eq!(
        terminal_state.tabs[1].activity_event_count, 0,
        "creating a permanently owned session must not invent an attach event"
    );
    assert!(
        terminal_state.tabs[0]
            .activity_summary
            .iter()
            .all(|line| !line.contains("lifecycle:detached")),
        "background tab must not invent a detached lifecycle: {:?}",
        terminal_state.tabs[0].activity_summary
    );
    assert!(
        terminal_state.tabs[1]
            .activity_summary
            .iter()
            .all(|line| !line.contains("lifecycle:attached"))
    );
    registry
        .rename(&first_session_id, "layout shell")
        .expect("rename first terminal tab");
    registry
        .activate(&first_session_id)
        .expect("activate first terminal tab");
    registry.sync_lane_tabs(&mut terminal_state);
    assert_eq!(
        terminal_state.active_session_id.as_deref(),
        Some(first_session_id.as_str())
    );
    assert_eq!(terminal_state.tabs[0].label, "layout shell");
    assert!(terminal_state.tabs[0].active);
    assert!(terminal_state.tabs[0].attached);
    assert!(!terminal_state.tabs[1].active);
    assert!(terminal_state.tabs[1].attached);
    assert!(
        terminal_state
            .activity_summary
            .iter()
            .all(|line| !line.contains("lifecycle:attached"))
    );
    assert!(
        fs::read_to_string(&second_event_log_path).map_or(true, |contents| !contents
            .contains("\"lifecycle\":\"detached\""))
    );
    assert!(
        fs::read_to_string(&first_event_log_path).map_or(true, |contents| contents
            .lines()
            .all(|line| !line.contains("\"lifecycle\":\"attached\"")))
    );
    terminate_and_remove_active(&mut registry, &mut terminal_state);
    assert_eq!(registry.len(), 1);
    assert_eq!(
        terminal_state.active_session_id.as_deref(),
        Some(second_session_id.as_str())
    );
    assert_eq!(terminal_state.tabs.len(), 1);
    assert_eq!(terminal_state.tabs[0].session_id, second_session_id);
    assert!(terminal_state.tabs[0].attached);
    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    let latest_context: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(latest_context["session_id"], second_session_id);
    assert_eq!(
        latest_context["storage"]["latest_context_path"],
        latest_context_path.display().to_string()
    );
    let _ = fs::remove_dir_all(&root);
}
#[test]
fn terminal_session_registry_close_active_repoints_latest_context() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-close-latest-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal registry test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let first_session_id = registry.active().session_id().to_string();
    let second_session_id = registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session")
        .to_string();
    assert_ne!(first_session_id, second_session_id);
    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    let latest_before_close: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(latest_before_close["session_id"], second_session_id);
    let mut terminal_state = TerminalLaneState::default();
    terminate_and_remove_active(&mut registry, &mut terminal_state);
    assert_eq!(registry.len(), 1);
    assert_eq!(
        terminal_state.active_session_id.as_deref(),
        Some(first_session_id.as_str())
    );
    refresh_terminal_session_context(registry.active(), &context)
        .expect("refresh survivor terminal context");
    let latest_after_close: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(latest_after_close["session_id"], first_session_id);
    assert_eq!(
        latest_after_close["storage"]["latest_context_path"],
        latest_context_path.display().to_string()
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn live_close_requires_an_approved_confirmation_and_then_removes_automatically() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-close-confirmation-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();

    registry.close_active(&mut lane).unwrap();
    assert!(registry.active_close_confirmation_armed());
    registry.close_active(&mut lane).unwrap();
    assert!(
        registry
            .active()
            .write_bytes(b"printf still-running\\n\n")
            .is_ok()
    );

    registry.handle_close_confirmation_input(b"not-a-choice", &mut lane);
    assert!(registry.active_close_confirmation_armed());
    registry.handle_close_confirmation_input(b"\x1b", &mut lane);
    assert!(!registry.active_close_confirmation_armed());

    registry.close_active(&mut lane).unwrap();
    registry.handle_close_confirmation_input(b"\n", &mut lane);
    assert!(registry.active().write_bytes(b"must-not-enter\n").is_err());

    let deadline = std::time::Instant::now() + Duration::from_secs(7);
    while std::time::Instant::now() < deadline && !lane.tabs.is_empty() {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        lane.tabs.is_empty(),
        "confirmed close must remove the sole tab"
    );
    let _ = fs::remove_dir_all(root);
}
#[test]
fn background_tabs_remain_owned_without_detached_lifecycle() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-owned-tabs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal detach test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let first_session_id = registry.active().session_id().to_string();
    let first_event_log_path = registry.active().event_log_path();
    let mut terminal_state = TerminalLaneState::default();
    registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal tab");
    registry.sync_lane_tabs(&mut terminal_state);
    assert_eq!(terminal_state.status, "running");
    assert_eq!(terminal_state.tabs.len(), 2);
    assert!(terminal_state.tabs.iter().all(|tab| tab.attached));
    assert!(
        fs::read_to_string(&first_event_log_path).map_or(true, |contents| !contents
            .contains("\"lifecycle\":\"detached\""))
    );
    registry
        .activate(&first_session_id)
        .expect("select first tab");
    assert_eq!(registry.active().session_id(), first_session_id);
    let _ = fs::remove_dir_all(&root);
}
#[test]
fn terminal_session_restart_preserves_tab_and_reports_lineage() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-restart-lineage-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal restart test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let first_session_id = registry.active().session_id().to_string();
    registry
        .rename(&first_session_id, "layout shell")
        .expect("rename terminal session before restart");
    let mut terminal_state = TerminalLaneState::default();
    assert!(registry.resize_active(101, 29).is_ok());
    registry
        .restart_active(&mut terminal_state, &context)
        .expect("restart active terminal tab");
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while registry.active().session_id() == first_session_id && std::time::Instant::now() < deadline
    {
        registry.drain_all(&mut terminal_state);
        registry
            .complete_pending_restarts(&mut terminal_state, &context)
            .expect("complete verified terminal restart");
        std::thread::sleep(Duration::from_millis(5));
    }
    let restarted_session_id = registry.active().session_id().to_string();
    assert_ne!(first_session_id, restarted_session_id);
    assert_eq!(terminal_state.status, "running");
    assert_eq!((terminal_state.columns, terminal_state.rows), (101, 29));
    assert_eq!(
        terminal_state.active_session_id.as_deref(),
        Some(restarted_session_id.as_str())
    );
    assert_eq!(terminal_state.tabs.len(), 1);
    let tab = &terminal_state.tabs[0];
    assert_eq!(tab.session_id, restarted_session_id);
    assert_eq!(
        tab.previous_session_id.as_deref(),
        Some(first_session_id.as_str())
    );
    assert_eq!(tab.label, "layout shell");
    assert_eq!(tab.restart_count, 1);
    assert!(tab.active);
    assert!(tab.attached);
    let restarted_rows = terminal_state.grid_lines().join("\n");
    assert!(
        !restarted_rows.contains("terminal restarted"),
        "restart lifecycle message must not enter the terminal grid"
    );
    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    let latest_context: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(latest_context["session_id"], restarted_session_id);
    assert_eq!(
        latest_context["storage"]["latest_context_path"],
        latest_context_path.display().to_string()
    );
    let _ = fs::remove_dir_all(&root);
}
#[test]
fn terminal_session_spawns_real_pty_shell() {
    let root = std::env::temp_dir().join(format!("datum-terminal-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal test root should create");
    let mut context = TerminalLaunchContext::for_project_root(&root);
    context.project_id = Some("project-test".to_string());
    context.source_revision = Some("source-rev-test".to_string());
    context.production_status = ProductionStatus {
        output_job_count: 1,
        artifact_count: 3,
        artifact_run_count: 1,
        latest_status: Some("complete".to_string()),
        latest_run_id: Some("run-gerber-2".to_string()),
        latest_artifact_id: Some("artifact-bom".to_string()),
        latest_artifact_run_id: Some("run-bom-1".to_string()),
        latest_output_job_run_id: Some("run-gerber-2".to_string()),
        output_jobs: vec![datum_gui_protocol::ProductionOutputJobSummary {
            id: "job-gerber".to_string(),
            name: "Gerber Set".to_string(),
            include: vec!["gerber-set".to_string()],
            prefix: "doa2526".to_string(),
            output_dir: Some("build/fab".to_string()),
            family: "GERBER SET".to_string(),
            status: "complete".to_string(),
            execution_count: 2,
            artifact_count: 1,
            latest_run_id: Some("run-gerber-2".to_string()),
            latest_run_artifact_id: Some("artifact-gerber".to_string()),
            artifacts: vec![datum_gui_protocol::ProductionArtifactSummary {
                artifact_id: "artifact-gerber".to_string(),
                kind: "gerber_set".to_string(),
                project_id: Some("project-test".to_string()),
                model_revision: Some("source-rev-test".to_string()),
                output_job: Some("job-gerber".to_string()),
                variant: None,
                generator_version: Some("datum-test".to_string()),
                output_dir: Some("build/fab".to_string()),
                validation_state: Some("valid".to_string()),
                file_count: 1,
                files: vec![datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "build/fab/doa2526-F_Cu.gbr".to_string(),
                    sha256: "sha256-gerber".to_string(),
                }],
                production_projection_count: 0,
                production_projections: Vec::new(),
            }],
        }],
        artifact_runs: vec![datum_gui_protocol::ProductionArtifactRunSummary {
            run_id: "run-bom-1".to_string(),
            artifact_id: "artifact-bom".to_string(),
            run_source: "artifact_run".to_string(),
            output_job_id: None,
            run_sequence: 1,
            status: "complete".to_string(),
            exit_code: Some(0),
        }],
        focused_artifact: Some(datum_gui_protocol::ProductionArtifactDetail {
            artifact_id: "artifact-gerber".to_string(),
            kind: "gerber_set".to_string(),
            output_dir: Some("build/fab".to_string()),
            validation_state: "valid".to_string(),
            file_count: 2,
            files: vec![
                datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "build/fab/doa2526-F_Cu.gbr".to_string(),
                    sha256: "sha256-gerber".to_string(),
                },
                datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "build/fab/doa2526.drl".to_string(),
                    sha256: "sha256-drill".to_string(),
                },
            ],
            focused_file: Some(datum_gui_protocol::ProductionArtifactFileSummary {
                path: "build/fab/doa2526.drl".to_string(),
                sha256: "sha256-drill".to_string(),
            }),
            focused_preview: None,
            production_projection_count: 0,
            production_projections: Vec::new(),
        }),
        ..ProductionStatus::default()
    };
    context.check_status = CheckRunReviewState {
        check_run_id: Some("check-run-visible".to_string()),
        model_revision: Some("source-rev-test".to_string()),
        profile_id: Some("standards".to_string()),
        status: Some("succeeded".to_string()),
        finding_count: 1,
        findings: vec![datum_gui_protocol::CheckFindingSummary {
            fingerprint: "sha256:visible-finding".to_string(),
            rule_id: "zone_fill_state".to_string(),
            ..datum_gui_protocol::CheckFindingSummary::default()
        }],
        ..CheckRunReviewState::default()
    };
    let session = spawn_terminal_session(&context).expect("spawn PTY terminal session");
    assert!(session.context_path.exists());
    assert!(
        session
            .context_path
            .to_string_lossy()
            .contains(".datum/terminal-contexts/terminal-")
    );
    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    assert!(latest_context_path.exists());
    let session_context: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&session.context_path).unwrap()).unwrap();
    let latest_context: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(session_context["session_id"], latest_context["session_id"]);
    assert_eq!(
        session_context["storage"]["legacy_context_path"],
        latest_context_path.display().to_string()
    );
    assert_eq!(
        session_context["storage"]["latest_context_path"],
        latest_context_path.display().to_string()
    );
    assert_eq!(
        session_context["storage"]["compatibility_context_path"],
        latest_context_path.display().to_string()
    );
    assert_eq!(
        session_context["storage"]["event_log_path"],
        session.event_log_path().display().to_string()
    );
    assert!(session.session_path.exists());
    let session_metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&session.session_path).unwrap()).unwrap();
    assert_eq!(
        session_metadata["session_id"],
        session_context["session_id"]
    );
    for key in [
        "create_gerber_output_job",
        "update_output_job",
        "delete_output_job",
        "create_manufacturing_plan",
        "update_manufacturing_plan",
        "delete_manufacturing_plan",
        "create_panel_projection",
        "update_panel_projection",
        "delete_panel_projection",
    ] {
        let command = session_context["production_commands"][key]
            .as_str()
            .expect("production command template should be a string");
        assert!(
            command.starts_with("datum-eda proposal "),
            "production command template {key} should use canonical proposal CLI: {command}"
        );
        assert!(
            !command.contains("--as-proposal"),
            "canonical proposal template {key} should not need --as-proposal: {command}"
        );
    }
    assert_eq!(
        session_context["command_catalog_version"],
        TERMINAL_COMMAND_CATALOG_VERSION
    );
    assert_eq!(
        session_context["handoff_commands"]["datum.artifact.generate"]["mcp_alias"],
        "datum.artifact.generate"
    );
    assert_eq!(
        session_context["handoff_commands"]["datum.proposal.accept_apply"]["cli_argv_template"],
        serde_json::json!([
            "datum-eda",
            "proposal",
            "accept-apply",
            "{project_root}",
            "--proposal",
            "{proposal}"
        ])
    );
    assert_eq!(
        session_context["proposal_commands"]["preview"],
        "datum-eda proposal preview \"$DATUM_PROJECT_ROOT\" --proposal <uuid>"
    );
    assert_eq!(
        session_context["visible_check_run_ids"],
        serde_json::json!(["check-run-visible"])
    );
    assert_eq!(
        session_context["visible_finding_fingerprints"],
        serde_json::json!(["sha256:visible-finding"])
    );
    assert_eq!(
        session_context["visible_artifact_ids"],
        serde_json::json!(["artifact-bom", "artifact-gerber"])
    );
    assert_eq!(
        session_context["visible_output_job_ids"],
        serde_json::json!(["job-gerber"])
    );
    assert_eq!(
        session_context["visible_artifact_file_paths"],
        serde_json::json!(["build/fab/doa2526-F_Cu.gbr", "build/fab/doa2526.drl"])
    );
    assert_eq!(session_context["latest_output_job_id"], "job-gerber");
    assert_eq!(session_context["latest_output_job_run_id"], "run-gerber-2");
    assert_eq!(
        session_context["latest_output_job_artifact_id"],
        "artifact-gerber"
    );
    assert_eq!(session_context["latest_artifact_id"], "artifact-bom");
    assert_eq!(session_context["latest_artifact_run_id"], "run-bom-1");
    assert_eq!(session_context["latest_check_run_id"], "check-run-visible");
    assert_eq!(session_context["latest_profile_id"], "standards");
    assert_eq!(
        session_context["profile_latest_check_runs"][0]["check_run_id"],
        "check-run-visible"
    );
    assert_eq!(
        session_context["profile_latest_check_runs"][0]["profile_id"],
        "standards"
    );
    assert_eq!(session_context["focused_artifact_id"], "artifact-gerber");
    assert_eq!(
        session_context["focused_artifact_file_path"],
        "build/fab/doa2526.drl"
    );
    assert_eq!(
        session_context["check_status"]["findings"][0]["rule_id"],
        "zone_fill_state"
    );
    context.selection_context = DatumSelectionContext {
        kind: "authored_object".to_string(),
        id: Some("object-live".to_string()),
    };
    context.cursor_context.screen_px = Some([42, 84]);
    context.cursor_context.hovered_object_id = Some("hover-live".to_string());
    refresh_terminal_session_context(&session, &context)
        .expect("refresh existing terminal context");
    let refreshed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&session.context_path).unwrap()).unwrap();
    assert_eq!(refreshed["session_id"], session_context["session_id"]);
    assert_eq!(refreshed["selection_context"]["id"], "object-live");
    assert_eq!(
        refreshed["cursor_context"]["screen_px"],
        serde_json::json!([42, 84])
    );
    assert_eq!(
        refreshed["session"]["session_id"],
        session_context["session_id"]
    );
    assert_eq!(refreshed["contract"], "datum_terminal_context_v1");
    assert_eq!(refreshed["datum_cli"], "datum-eda");
    assert_eq!(refreshed["actor_type"], "ExternalAgent");
    assert_eq!(refreshed["selection_context"]["kind"], "authored_object");
    assert_eq!(
        refreshed["agent_commands"]["refresh_context"],
        "datum-eda context refresh --session \"$DATUM_SESSION_ID\""
    );
    assert!(
        refreshed["agent_commands"]["codex_with_context"]
            .as_str()
            .unwrap()
            .contains("$DATUM_DISCOVERY")
    );
    assert!(
        refreshed["agent_commands"]["context_prompt"]
            .as_str()
            .unwrap()
            .contains("context session-activity")
    );
    assert_eq!(
        refreshed["check_commands"]["run_current"],
        "datum-eda check run \"$DATUM_PROJECT_ROOT\""
    );
    assert_eq!(
        refreshed["proposal_commands"]["accept_apply"],
        "datum-eda proposal accept-apply \"$DATUM_PROJECT_ROOT\" --proposal <uuid>"
    );
    assert_eq!(
        refreshed["query_commands"]["import_map"],
        "datum-eda query import-map \"$DATUM_PROJECT_ROOT\""
    );
    assert_eq!(
        refreshed["query_commands"]["zone_fills"],
        "datum-eda query zone-fills \"$DATUM_PROJECT_ROOT\""
    );
    session.resize(100, 24).expect("resize PTY");
    session
        .write_bytes(
            b"printf 'datum-pty-ok:%s:%s:%s\\n' \"$DATUM_PROJECT_ROOT\" \"$DATUM_CLI\" \"$DATUM_SESSION_ID\"\nexit\n",
        )
        .expect("write command to PTY");
    let mut output = String::new();
    for _ in 0..80 {
        if let Ok(event) = session.recv_event_timeout(Duration::from_millis(100)) {
            match event {
                TerminalEvent::Output(bytes) => {
                    let _ = crate::terminal_session_events::record_terminal_output_event(
                        &session, &bytes,
                    );
                    output.push_str(&String::from_utf8_lossy(&bytes));
                    if output.contains("datum-pty-ok:") && output.contains("datum-eda") {
                        break;
                    }
                }
                TerminalEvent::Exited(code) => {
                    assert_eq!(code, crate::terminal_transport::TerminalExitStatus::Code(0));
                }
                TerminalEvent::Error(error) => panic!("unexpected transport error: {error:?}"),
            }
        }
    }
    for expected in ["datum-pty-ok:", "datum-eda"] {
        assert!(
            output.contains(expected),
            "missing {expected:?} in PTY output: {output}"
        );
    }
    let event_log = crate::terminal_session_events::io_event_log::read_event_log_family_text(
        &session.event_log_path(),
    );
    assert!(
        event_log.contains(r#""event":"terminal_io""#),
        "terminal event log should record I/O events: {event_log}"
    );
    assert!(
        event_log.contains(r#""direction":"input_accepted""#),
        "terminal event log should record PTY input: {event_log}"
    );
    assert!(
        event_log.contains(r#""direction":"output""#),
        "terminal event log should record PTY output: {event_log}"
    );
    assert!(
        event_log.contains(session.session_id()),
        "terminal event log should tie events to the session id: {event_log}"
    );
    let _ = fs::remove_dir_all(&root);
}
