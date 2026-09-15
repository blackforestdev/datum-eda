use super::*;

#[test]
fn context_session_events_merges_rotated_io_segments_chronologically() {
    let root = std::env::temp_dir().join(format!(
        "datum-cli-context-rotated-terminal-{}",
        Uuid::new_v4()
    ));
    let context_dir = root.join(".datum/terminal-contexts");
    let session_dir = root.join(".datum/tool-sessions");
    std::fs::create_dir_all(&context_dir).expect("create context directory");
    std::fs::create_dir_all(&session_dir).expect("create session directory");
    std::fs::write(
        context_dir.join("terminal-rotated.json"),
        r#"{"contract":"datum_terminal_context_v1","session_id":"terminal-rotated","context_id":"context-rotated","datum_cli":"datum-eda"}"#,
    )
    .expect("write context envelope");
    let metadata = session_dir.join("terminal-rotated.events.jsonl");
    std::fs::write(
        &metadata,
        concat!(
            r#"{"event":"terminal_command_handoff","session_id":"terminal-rotated","command_id":"datum.check.run","occurred_unix_ms":1}"#,
            "\n",
            r#"{"event":"terminal_lifecycle","session_id":"terminal-rotated","lifecycle":"exited","occurred_unix_ms":4}"#,
            "\n"
        ),
    )
    .expect("write durable metadata");
    std::fs::write(
        metadata.with_file_name("terminal-rotated.events.jsonl.io.1.jsonl"),
        concat!(
            r#"{"event":"terminal_io","session_id":"terminal-rotated","direction":"input","byte_count":3,"occurred_unix_ms":2}"#,
            "\n"
        ),
    )
    .expect("write retained I/O segment");
    std::fs::write(
        metadata.with_file_name("terminal-rotated.events.jsonl.io.0.jsonl"),
        concat!(
            r#"{"event":"terminal_io","session_id":"terminal-rotated","direction":"output","byte_count":7,"occurred_unix_ms":3}"#,
            "\n"
        ),
    )
    .expect("write current I/O segment");

    let parsed = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "context",
        "session-events",
        "--project-root",
        root.to_str().unwrap(),
        "--session",
        "terminal-rotated",
    ])
    .expect("parse rotated session-events query");
    let value: serde_json::Value =
        serde_json::from_str(&execute(parsed).expect("query rotated terminal event family"))
            .expect("session-events JSON");
    assert_eq!(value["event_count"], 4);
    assert_eq!(value["events"][0]["event"], "terminal_command_handoff");
    assert_eq!(value["events"][1]["direction"], "input");
    assert_eq!(value["events"][2]["direction"], "output");
    assert_eq!(value["events"][3]["event"], "terminal_lifecycle");

    let _ = std::fs::remove_dir_all(root);
}
