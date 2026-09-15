use super::*;

fn unique_context_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn context_session_activity_reports_terminal_command_lifecycle() {
    let root = unique_context_root("datum-eda-cli-context-command-lifecycle");
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
        r#"{"event":"terminal_command_handoff","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-1","mcp_alias":"datum.check.run","command":"datum-eda check run \"$DATUM_PROJECT_ROOT\"","occurred_unix_ms":1}
{"event":"terminal_command_lifecycle","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-1","command":"datum-eda check run \"$DATUM_PROJECT_ROOT\"","lifecycle":"started","process_exit_code":null,"occurred_unix_ms":2}
{"event":"terminal_io","schema_version":1,"session_id":"terminal-live","execution_id":"exec-1","direction":"output","byte_count":8,"text_preview":"first\n","truncated":false,"occurred_unix_ms":3}
{"event":"terminal_command_lifecycle","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-1","command":null,"lifecycle":"finished","process_exit_code":7,"occurred_unix_ms":4}
{"event":"terminal_command_handoff","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-2","mcp_alias":"datum.check.run","command":"datum-eda check run \"$DATUM_PROJECT_ROOT\"","occurred_unix_ms":5}
{"event":"terminal_command_lifecycle","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-2","command":"datum-eda check run \"$DATUM_PROJECT_ROOT\"","lifecycle":"started","process_exit_code":null,"occurred_unix_ms":6}
{"event":"terminal_io","schema_version":1,"session_id":"terminal-live","execution_id":"exec-2","direction":"output","byte_count":9,"text_preview":"second\n","truncated":false,"occurred_unix_ms":7}
{"event":"terminal_command_lifecycle","schema_version":1,"session_id":"terminal-live","origin":"production_terminal_command","command_id":"datum.check.run","execution_id":"exec-2","command":null,"lifecycle":"finished","process_exit_code":0,"occurred_unix_ms":8}
"#,
    )
    .expect("event log should be written");

    let parsed = Cli::try_parse_from([
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
    .expect("context session-activity should parse");
    let output = execute(parsed).expect("context session-activity should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("session-activity output should be JSON");

    assert_eq!(value["activity_event_count"], 8);
    assert_eq!(value["context_provenance"]["context_id"], "context-live");
    assert_eq!(value["context_provenance"]["session_id"], "terminal-live");
    assert_eq!(value["context_provenance"]["actor_type"], "ExternalAgent");
    assert!(
        value["context_provenance"]["context_path"]
            .as_str()
            .unwrap()
            .contains("terminal-live.json")
    );
    assert!(
        value["context_provenance"]["event_log_path"]
            .as_str()
            .unwrap()
            .contains("terminal-live.events.jsonl")
    );
    assert_eq!(value["event_kinds"]["terminal_command_lifecycle"], 4);
    assert_eq!(value["commands"].as_array().unwrap().len(), 1);
    assert_eq!(value["commands"][0]["count"], 2);
    assert_eq!(value["executions"].as_array().unwrap().len(), 2);
    assert_eq!(value["executions"][0]["execution_id"], "exec-1");
    assert_eq!(
        value["executions"][0]["context_provenance"]["context_id"],
        "context-live"
    );
    assert_eq!(value["executions"][0]["event_count"], 4);
    assert_eq!(value["executions"][0]["start_occurred_unix_ms"], 1);
    assert_eq!(value["executions"][0]["end_occurred_unix_ms"], 4);
    assert_eq!(value["executions"][0]["duration_ms"], 3);
    assert_eq!(value["executions"][0]["lifecycle"], "finished");
    assert_eq!(value["executions"][0]["process_exit_code"], 7);
    assert_eq!(value["executions"][0]["event_kinds"]["terminal_io"], 1);
    assert_eq!(
        value["executions"][0]["terminal_io"]["last_output_preview"],
        "first\n"
    );
    assert_eq!(value["executions"][1]["execution_id"], "exec-2");
    assert_eq!(value["executions"][1]["duration_ms"], 3);
    assert_eq!(value["executions"][1]["process_exit_code"], 0);
    assert_eq!(
        value["executions"][1]["terminal_io"]["output_byte_count"],
        9
    );
    assert_eq!(value["activity_spans"].as_array().unwrap().len(), 2);
    assert_eq!(value["activity_spans"][0]["span_kind"], "command");
    assert_eq!(value["activity_spans"][0]["end_reason"], "command_finished");
    assert_eq!(value["activity_spans"][0]["execution_id"], "exec-1");
    assert_eq!(value["activity_spans"][1]["execution_id"], "exec-2");
    assert_eq!(
        value["activity_spans"][0]["handoff"]["command_id"],
        "datum.check.run"
    );
    assert_eq!(
        value["activity_spans"][0]["handoff"]["context_provenance"]["session_id"],
        "terminal-live"
    );
    assert_eq!(
        value["activity_spans"][0]["command_lifecycle"]["lifecycle"],
        "finished"
    );
    assert_eq!(
        value["activity_spans"][0]["command_lifecycle"]["process_exit_code"],
        7
    );
    assert_eq!(
        value["activity_spans"][1]["command_lifecycle"]["process_exit_code"],
        0
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["output_event_count"],
        1
    );

    let filtered = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-activity",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-live",
        "--execution-id",
        "exec-2",
    ])
    .expect("execution-filtered context session-activity should parse");
    let output = execute(filtered).expect("execution-filtered session-activity should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("filtered session-activity output should be JSON");
    assert_eq!(value["activity_event_count"], 4);
    assert_eq!(value["filters"]["execution_id"], "exec-2");
    assert_eq!(value["executions"].as_array().unwrap().len(), 1);
    assert_eq!(value["executions"][0]["execution_id"], "exec-2");
    assert_eq!(value["executions"][0]["duration_ms"], 3);
    assert_eq!(
        value["executions"][0]["terminal_io"]["last_output_preview"],
        "second\n"
    );
    assert_eq!(value["activity_spans"].as_array().unwrap().len(), 1);
    assert_eq!(value["activity_spans"][0]["execution_id"], "exec-2");
    assert_eq!(
        value["activity_spans"][0]["command_lifecycle"]["process_exit_code"],
        0
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["output_event_count"],
        1
    );
    assert_eq!(
        value["activity_spans"][0]["terminal_io"]["last_output_preview"],
        "second\n"
    );

    let events = Cli::try_parse_from([
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
    let output = execute(events).expect("context session-events should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("session-events output should be JSON");
    assert_eq!(value["context_provenance"]["context_id"], "context-live");
    assert_eq!(value["context_provenance"]["session_id"], "terminal-live");

    let _ = std::fs::remove_dir_all(&root);
}
