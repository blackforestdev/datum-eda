use super::*;
use eda_engine::substrate::ProjectResolver;
fn unique_context_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn context_get_returns_gui_terminal_discovery_envelope() {
    let root = unique_context_root("datum-eda-cli-context-get");
    let datum_dir = root.join(".datum");
    std::fs::create_dir_all(&datum_dir).expect("context dir should exist");
    std::fs::write(
        datum_dir.join("gui-terminal-context.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "project_root": "/tmp/example",
  "project_id": "project-test",
  "model_revision": "model-rev-test",
  "context_id": "context-test",
  "session_id": "session-test",
  "terminal_session_id": "session-test",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Get(ContextGetArgs {
                session: Some("session-test".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context get should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context get output should be JSON");
    assert_eq!(value["contract"], "datum_terminal_context_v1");
    assert_eq!(value["session_id"], "session-test");
    assert_eq!(value["datum_cli"], "datum-eda");
    assert_eq!(value["actor_type"], "ExternalAgent");
    assert_eq!(
        value["capabilities"],
        serde_json::json!(["read", "check", "artifact", "propose", "apply-approved"])
    );
    assert_eq!(value["visible_output_job_ids"], serde_json::json!([]));
    assert_eq!(value["visible_check_run_ids"], serde_json::json!([]));
    assert_eq!(value["selection_context"]["kind"], "none");
    assert_eq!(value["cursor_context"]["active_tool"], "select");
    assert_eq!(value["projection_context"]["scene_id"], "unknown-scene");
    assert_eq!(value["agent_commands"]["codex"], "codex");
    assert_eq!(
        value["agent_commands"]["inspect_context"],
        "python3 -m json.tool \"$DATUM_DISCOVERY\""
    );
    assert_eq!(
        value["agent_commands"]["refresh_context"],
        "datum-eda context refresh --session \"$DATUM_SESSION_ID\""
    );
    assert_eq!(
        value["agent_commands"]["session_activity"],
        "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20"
    );
    assert_eq!(
        value["query_commands"]["sheets"],
        "datum-eda query sheets \"$DATUM_PROJECT_ROOT\""
    );
    assert_eq!(
        value["query_commands"]["hierarchy"],
        "datum-eda query hierarchy \"$DATUM_PROJECT_ROOT\""
    );
    assert!(
        value["provenance_seed"]
            .as_str()
            .unwrap()
            .contains("session-test")
    );
    assert_eq!(value["expires_at"], serde_json::Value::Null);
    assert_eq!(value["session_lifecycle"], "running");
    assert_eq!(value["session"]["lifecycle"], "running");
    assert_eq!(value["process_exit_code"], serde_json::Value::Null);

    let mismatch = execute_with_exit_code(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Get(ContextGetArgs {
                session: Some("other-session".to_string()),
                path: None,
                project_root: Some(root),
            }),
        },
    });
    assert!(mismatch.is_err(), "session mismatch should fail");
}

#[test]
fn context_get_enriches_discovery_from_project_resolver() {
    let root = unique_context_root("datum-eda-cli-context-get-resolver");
    create_native_project(&root, Some("Context Resolver Demo".to_string()))
        .expect("native project should be created");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    std::fs::create_dir_all(root.join(".datum")).expect("datum dir should exist");
    std::fs::write(
        root.join(".datum/gui-terminal-context.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "session-resolver",
  "context_id": "context-resolver",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("session-resolver".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    assert_eq!(value["project_root"], root.display().to_string());
    assert_eq!(value["project_id"], model.project.project_id.to_string());
    assert_eq!(value["project_name"], "Context Resolver Demo");
    assert_eq!(value["model_revision"], model.model_revision.0);
    assert_eq!(value["accepted_transaction_tip"], serde_json::Value::Null);
    assert_eq!(value["visible_artifact_ids"], serde_json::json!([]));
    assert_eq!(value["visible_output_job_ids"], serde_json::json!([]));
    assert_eq!(value["visible_check_run_ids"], serde_json::json!([]));
    assert_eq!(value["storage"]["schema_version"], 1);
    assert!(
        value["refresh_command"]
            .as_str()
            .unwrap()
            .contains("context refresh")
    );
    assert!(value["source_shard_status"]["total"].as_u64().unwrap() >= 4);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_refresh_persists_enriched_session_context_without_losing_gui_fields() {
    let root = unique_context_root("datum-eda-cli-context-refresh-persists");
    create_native_project(&root, Some("Context Refresh Demo".to_string()))
        .expect("native project should be created");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let session_dir = root.join(".datum/terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    let context_path = session_dir.join("terminal-live.json");
    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    std::fs::write(
        &context_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-live",
  "context_id": "context-live",
  "terminal_session_id": "terminal-live",
  "datum_cli": "datum-eda",
  "selection_context": {
    "kind": "authored_object",
    "id": "object-live"
  },
  "cursor_context": {
    "screen_px": [42, 84],
    "hovered_object_id": "hover-live",
    "active_dock_tab": "terminal",
    "active_tool": "select"
  },
  "projection_context": {
    "scene_id": "scene-live",
    "board_id": "board-live",
    "board_name": "Board Live",
    "scene_bounds_nm": {
      "min": [1, 2],
      "max": [3, 4]
    },
    "active_projection_id": "projection-live"
  },
  "session": {
    "session_id": "terminal-live",
    "context_id": "context-live",
    "actor_type": "ExternalAgent",
    "capabilities": ["read", "check"],
    "created_model_revision": "old-revision"
  }
}"#,
    )
    .expect("session context envelope should be written");
    std::fs::write(
        &latest_context_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-stale",
  "context_id": "context-stale",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("stale latest context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("terminal-live".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let output_value: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    let persisted: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&context_path).expect("refreshed context should be readable"),
    )
    .expect("persisted context should be JSON");
    let latest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&latest_context_path)
            .expect("latest context alias should be readable"),
    )
    .expect("latest context alias should be JSON");

    assert_eq!(persisted, output_value);
    assert_eq!(latest, output_value);
    assert_eq!(persisted["session_id"], "terminal-live");
    assert_eq!(persisted["context_id"], "context-live");
    assert_eq!(
        persisted["project_id"],
        model.project.project_id.to_string()
    );
    assert_eq!(persisted["project_name"], "Context Refresh Demo");
    assert_eq!(persisted["model_revision"], model.model_revision.0);
    assert_eq!(persisted["selection_context"]["kind"], "authored_object");
    assert_eq!(persisted["selection_context"]["id"], "object-live");
    assert_eq!(
        persisted["cursor_context"]["screen_px"],
        serde_json::json!([42, 84])
    );
    assert_eq!(persisted["projection_context"]["scene_id"], "scene-live");
    assert_eq!(persisted["session"]["session_id"], "terminal-live");
    assert_eq!(persisted["session_lifecycle"], "running");
    assert_eq!(persisted["session"]["lifecycle"], "running");
    assert_eq!(persisted["agent_commands"]["claude"], "claude");
    assert_eq!(persisted["agent_commands"]["aider"], "aider");
    assert_eq!(
        persisted["agent_commands"]["session_activity"],
        "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20"
    );
    assert_eq!(
        persisted["storage"]["context_path"],
        context_path.display().to_string()
    );
    assert_eq!(
        persisted["storage"]["latest_context_path"],
        latest_context_path.display().to_string()
    );
    assert_eq!(
        persisted["storage"]["compatibility_context_path"],
        latest_context_path.display().to_string()
    );
    assert_eq!(
        persisted["storage"]["legacy_context_path"],
        latest_context_path.display().to_string()
    );
    assert!(
        persisted["provenance_seed"]
            .as_str()
            .unwrap()
            .contains("terminal-live")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_get_prefers_matching_session_context_under_project_root() {
    let root = unique_context_root("datum-eda-cli-context-get-session-specific");
    let datum_dir = root.join(".datum");
    let session_dir = datum_dir.join("terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    std::fs::write(
        datum_dir.join("gui-terminal-context.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-latest",
  "context_id": "context-latest",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("latest context envelope should be written");
    std::fs::write(
        session_dir.join("terminal-pinned.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-pinned",
  "context_id": "context-pinned",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("pinned context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Get(ContextGetArgs {
                session: Some("terminal-pinned".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context get should read pinned session context");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context get output should be JSON");
    assert_eq!(value["session_id"], "terminal-pinned");
    assert_eq!(value["context_id"], "context-pinned");
    assert!(
        value["storage"]["context_path"]
            .as_str()
            .unwrap()
            .contains(".datum/terminal-contexts/terminal-pinned.json")
    );
    assert!(
        value["storage"]["legacy_context_path"]
            .as_str()
            .unwrap()
            .contains(".datum/gui-terminal-context.json")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_get_rejects_explicit_path_session_mismatch_without_fallback() {
    let root = unique_context_root("datum-eda-cli-context-get-explicit-mismatch");
    let datum_dir = root.join(".datum");
    let session_dir = datum_dir.join("terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    let wrong_path = session_dir.join("terminal-wrong.json");
    let latest_path = datum_dir.join("gui-terminal-context.json");
    std::fs::write(
        &wrong_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-wrong",
  "context_id": "context-wrong",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("wrong context envelope should be written");
    std::fs::write(
        session_dir.join("terminal-requested.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-requested",
  "context_id": "context-requested",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("requested context envelope should be written");
    std::fs::write(
        &latest_path,
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-latest",
  "context_id": "context-latest",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("latest context envelope should be written");

    let mismatch = execute_with_exit_code(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Get(ContextGetArgs {
                session: Some("terminal-requested".to_string()),
                path: Some(wrong_path),
                project_root: Some(root.clone()),
            }),
        },
    });
    assert!(
        mismatch.is_err(),
        "explicit wrong path should not fall back"
    );
    let latest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&latest_path).expect("latest context should still be readable"),
    )
    .expect("latest context should be JSON");
    assert_eq!(latest["session_id"], "terminal-latest");
    assert_eq!(latest["context_id"], "context-latest");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_session_events_returns_tool_session_event_log() {
    let root = unique_context_root("datum-eda-cli-context-session-events");
    let session_dir = root.join(".datum/terminal-contexts");
    let tool_session_dir = root.join(".datum/tool-sessions");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    std::fs::create_dir_all(&tool_session_dir).expect("tool session dir should exist");
    std::fs::write(
        session_dir.join("terminal-live.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-live",
  "context_id": "context-live",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("session context envelope should be written");
    std::fs::write(
        tool_session_dir.join("terminal-live.events.jsonl"),
        r#"{"event":"terminal_command_handoff","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.artifact.generate","mcp_alias":"datum.artifact.generate","command":"datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job job-1","occurred_unix_ms":1}
{"event":"terminal_command_handoff","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.proposal.reject","mcp_alias":"datum.proposal.reject","command":"datum-eda proposal reject \"$DATUM_PROJECT_ROOT\" --proposal prop-1","occurred_unix_ms":2}
{"event":"terminal_command_handoff","schema_version":1,"session_id":"terminal-live","origin":"board_text_terminal_command","command_id":"datum.gui.board_text.edit_prefill","mcp_alias":null,"command":"datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text txt-1 --value REF**","occurred_unix_ms":3}
{"event":"terminal_io","schema_version":1,"session_id":"terminal-live","direction":"input","byte_count":7,"text_preview":"ls -al\r","truncated":false,"occurred_unix_ms":4}
{"event":"terminal_io","schema_version":1,"session_id":"terminal-live","direction":"output","byte_count":12,"text_preview":"total 8\n","truncated":false,"occurred_unix_ms":5}
{"event":"terminal_lifecycle","schema_version":1,"session_id":"terminal-live","lifecycle":"exited","process_exit_code":0,"occurred_unix_ms":6}
"#,
    )
    .expect("event log should be written");

    let parsed = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-events",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
    ])
    .expect("context session-events should parse");
    let output = execute(parsed).expect("context session-events should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("session-events output should be JSON");

    assert_eq!(value["contract"], "datum_tool_session_events_v1");
    assert_eq!(value["session_id"], "terminal-live");
    assert_eq!(value["event_count"], 6);
    assert_eq!(value["total_event_count"], 6);
    assert_eq!(value["matched_event_count"], 6);
    assert!(
        value["context_path"]
            .as_str()
            .unwrap()
            .contains(".datum/terminal-contexts/terminal-live.json")
    );
    assert!(
        value["event_log_path"]
            .as_str()
            .unwrap()
            .contains(".datum/tool-sessions/terminal-live.events.jsonl")
    );
    assert_eq!(value["events"][0]["event"], "terminal_command_handoff");
    assert_eq!(value["events"][0]["command_id"], "datum.artifact.generate");
    assert_eq!(value["events"][0]["mcp_alias"], "datum.artifact.generate");
    assert_eq!(value["events"][1]["command_id"], "datum.proposal.reject");
    assert_eq!(value["events"][2]["origin"], "board_text_terminal_command");
    assert_eq!(value["events"][3]["event"], "terminal_io");
    assert_eq!(value["events"][3]["direction"], "input");
    assert_eq!(value["events"][3]["byte_count"], 7);
    assert_eq!(value["events"][4]["direction"], "output");
    assert_eq!(value["events"][4]["text_preview"], "total 8\n");
    assert_eq!(value["events"][5]["event"], "terminal_lifecycle");
    assert_eq!(value["events"][5]["lifecycle"], "exited");

    let filtered = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-events",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
        "--event-kind",
        "terminal_command_handoff",
        "--origin",
        "board_text_terminal_command",
        "--command-id",
        "datum.gui.board_text.edit_prefill",
    ])
    .expect("filtered context session-events should parse");
    let output = execute(filtered).expect("filtered context session-events should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("filtered session-events output should be JSON");
    assert_eq!(value["event_count"], 1);
    assert_eq!(value["total_event_count"], 6);
    assert_eq!(value["filters"]["event_kind"], "terminal_command_handoff");
    assert_eq!(value["filters"]["origin"], "board_text_terminal_command");
    assert_eq!(
        value["filters"]["command_id"],
        "datum.gui.board_text.edit_prefill"
    );
    assert_eq!(
        value["events"][0]["command_id"],
        "datum.gui.board_text.edit_prefill"
    );

    let limited = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-events",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
        "--origin",
        "production_terminal_command",
        "--limit",
        "1",
    ])
    .expect("limited context session-events should parse");
    let output = execute(limited).expect("limited context session-events should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("limited session-events output should be JSON");
    assert_eq!(value["event_count"], 1);
    assert_eq!(value["matched_event_count"], 2);
    assert_eq!(value["total_event_count"], 6);
    assert_eq!(value["limit"], 1);
    assert_eq!(value["events"][0]["command_id"], "datum.proposal.reject");

    let activity = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-activity",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
        "--origin",
        "production_terminal_command",
        "--limit",
        "2",
    ])
    .expect("context session-activity should parse");
    let output = execute(activity).expect("context session-activity should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("session-activity output should be JSON");
    assert_eq!(value["contract"], "datum_tool_session_activity_summary_v1");
    assert_eq!(value["session_lifecycle"], "running");
    assert_eq!(value["process_exit_code"], serde_json::Value::Null);
    assert_eq!(value["activity_event_count"], 2);
    assert_eq!(value["matched_event_count"], 2);
    assert_eq!(value["total_event_count"], 6);
    assert_eq!(value["first_occurred_unix_ms"], 1);
    assert_eq!(value["last_occurred_unix_ms"], 2);
    assert_eq!(value["event_kinds"]["terminal_command_handoff"], 2);
    assert_eq!(value["origins"]["production_terminal_command"], 2);
    assert_eq!(value["commands"].as_array().unwrap().len(), 2);
    assert_eq!(
        value["commands"][0]["command_id"],
        "datum.artifact.generate"
    );
    assert_eq!(value["commands"][1]["command_id"], "datum.proposal.reject");
    assert_eq!(value["terminal_io"]["input_event_count"], 0);
    assert_eq!(value["terminal_io"]["output_event_count"], 0);

    let full_activity = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-activity",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
    ])
    .expect("full terminal activity should parse");
    let output = execute(full_activity).expect("full terminal activity should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("full terminal activity output should be JSON");
    assert_eq!(value["activity_spans"].as_array().unwrap().len(), 3);
    assert_eq!(value["activity_spans"][0]["span_id"], "span-000001");
    assert_eq!(value["activity_spans"][0]["span_kind"], "command");
    assert_eq!(value["activity_spans"][0]["end_reason"], "next_handoff");
    assert_eq!(
        value["activity_spans"][0]["handoff"]["command_id"],
        "datum.artifact.generate"
    );
    assert_eq!(value["activity_spans"][2]["span_kind"], "command");
    assert_eq!(value["activity_spans"][2]["end_reason"], "lifecycle");
    assert_eq!(
        value["activity_spans"][2]["handoff"]["command_id"],
        "datum.gui.board_text.edit_prefill"
    );
    assert_eq!(
        value["activity_spans"][2]["terminal_io"]["input_event_count"],
        1
    );
    assert_eq!(
        value["activity_spans"][2]["terminal_io"]["output_event_count"],
        1
    );
    assert_eq!(
        value["activity_spans"][2]["terminal_io"]["input_byte_count"],
        7
    );
    assert_eq!(
        value["activity_spans"][2]["terminal_io"]["output_byte_count"],
        12
    );
    assert_eq!(value["activity_spans"][2]["start_occurred_unix_ms"], 3);
    assert_eq!(value["activity_spans"][2]["end_occurred_unix_ms"], 6);
    assert_eq!(
        value["activity_spans"][2]["lifecycle"]["lifecycle"],
        "exited"
    );
    assert_eq!(
        value["activity_spans"][2]["lifecycle"]["process_exit_code"],
        0
    );

    let io_activity = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-activity",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
        "--event-kind",
        "terminal_io",
    ])
    .expect("terminal I/O activity should parse");
    let output = execute(io_activity).expect("terminal I/O activity should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("terminal I/O activity output should be JSON");
    assert_eq!(value["activity_event_count"], 2);
    assert_eq!(value["event_kinds"]["terminal_io"], 2);
    assert_eq!(value["terminal_io"]["input_event_count"], 1);
    assert_eq!(value["terminal_io"]["output_event_count"], 1);
    assert_eq!(value["terminal_io"]["input_byte_count"], 7);
    assert_eq!(value["terminal_io"]["output_byte_count"], 12);
    assert_eq!(value["terminal_io"]["last_input_preview"], "ls -al\r");
    assert_eq!(value["terminal_io"]["last_output_preview"], "total 8\n");
    assert_eq!(value["activity_spans"].as_array().unwrap().len(), 1);
    assert_eq!(value["activity_spans"][0]["span_id"], "span-000001");
    assert_eq!(value["activity_spans"][0]["span_kind"], "terminal_io");
    assert_eq!(
        value["activity_spans"][0]["handoff"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["input_event_count"],
        1
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["output_event_count"],
        1
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["input_byte_count"],
        7
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["output_byte_count"],
        12
    );
    assert_eq!(value["activity_spans"][0]["start_occurred_unix_ms"], 4);
    assert_eq!(value["activity_spans"][0]["end_occurred_unix_ms"], 5);
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["last_input_preview"],
        "ls -al\r"
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["last_output_preview"],
        "total 8\n"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_session_events_returns_empty_log_when_session_has_no_events() {
    let root = unique_context_root("datum-eda-cli-context-session-events-empty");
    let session_dir = root.join(".datum/terminal-contexts");
    std::fs::create_dir_all(&session_dir).expect("session context dir should exist");
    std::fs::write(
        session_dir.join("terminal-empty.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "terminal-empty",
  "context_id": "context-empty",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("session context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::SessionEvents(ContextSessionEventsArgs {
                session: Some("terminal-empty".to_string()),
                path: None,
                project_root: Some(root.clone()),
                event_kind: None,
                origin: None,
                command_id: None,
                execution_id: None,
                limit: None,
            }),
        },
    })
    .expect("context session-events should succeed with empty log");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("session-events output should be JSON");
    assert_eq!(value["contract"], "datum_tool_session_events_v1");
    assert_eq!(value["event_count"], 0);
    assert_eq!(value["events"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}
