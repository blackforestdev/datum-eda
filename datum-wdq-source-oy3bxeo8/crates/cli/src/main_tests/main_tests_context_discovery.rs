use super::*;

#[test]
fn context_get_follows_agent_discovery_to_mutable_live_context() {
    let root = std::env::temp_dir().join(format!(
        "datum-eda-cli-agent-discovery-live-{}",
        Uuid::new_v4()
    ));
    let datum_dir = root.join(".datum");
    std::fs::create_dir_all(&datum_dir).expect("context dir should exist");
    let live_path = datum_dir.join("live.json");
    let pinned_path = datum_dir.join("pinned.json");
    let discovery_path = datum_dir.join("discovery.json");
    std::fs::write(
        &live_path,
        format!(
            r#"{{"contract":"datum_terminal_context_v1","project_root":"{}","session_id":"terminal-split","terminal_session_id":"terminal-split","context_id":"context-pinned","live_context_id":"live-terminal-split","pinned_context_id":"context-pinned","context_kind":"live","selection_context":{{"id":"live-selection"}}}}"#,
            root.display()
        ),
    )
    .expect("live context should be written");
    std::fs::write(&pinned_path, r#"{"context_kind":"pinned"}"#)
        .expect("pinned context should be written");
    std::fs::write(
        &discovery_path,
        format!(
            r#"{{"schema":"datum_agent_discovery_v1","project_root":"{}","terminal_session_id":"terminal-split","live_context_id":"live-terminal-split","live_context_path":"{}","pinned_context_id":"context-pinned","pinned_context_path":"{}"}}"#,
            root.display(),
            live_path.display(),
            pinned_path.display()
        ),
    )
    .expect("agent discovery should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Get(ContextGetArgs {
                session: Some("terminal-split".to_string()),
                path: Some(discovery_path),
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context get should follow agent discovery to live context");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context get output should be JSON");
    assert_eq!(value["context_kind"], "live");
    assert_eq!(value["selection_context"]["id"], "live-selection");
    assert_eq!(value["pinned_context_id"], "context-pinned");

    let _ = std::fs::remove_dir_all(&root);
}
