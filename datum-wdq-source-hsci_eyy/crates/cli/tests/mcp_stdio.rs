use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

fn unique_fixture_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("datum-mcp-stdio-{}-{nonce}", std::process::id()))
}

#[test]
fn canonical_mcp_command_emits_protocol_only_on_stdout() {
    let root = unique_fixture_root();
    fs::create_dir_all(&root).expect("fixture root");
    let discovery = root.join("discovery.json");
    let credential = root.join(".agent-credential.json");
    let authority = root.join("agent-authority.json");
    let event_log = root.join(".datum/tool-sessions/terminal-events.jsonl");
    fs::create_dir_all(event_log.parent().expect("event log parent"))
        .expect("event log parent should create");
    fs::write(
        &credential,
        serde_json::to_vec(&json!({
            "schema": "datum_agent_credential_v1",
            "credential_id": "credential-mcp-test",
            "terminal_session_id": "terminal-mcp-test",
            "project_root": root,
            "secret": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }))
        .expect("serialize credential"),
    )
    .expect("write credential");
    fs::set_permissions(&credential, fs::Permissions::from_mode(0o600))
        .expect("protect credential");
    fs::write(
        &authority,
        serde_json::to_vec(&json!({
            "schema": "datum_agent_authority_v1",
            "credential_id": "credential-mcp-test",
            "terminal_session_id": "terminal-mcp-test",
            "project_root": root,
            "state": "active"
        }))
        .expect("serialize authority"),
    )
    .expect("write authority");
    fs::set_permissions(&authority, fs::Permissions::from_mode(0o600)).expect("protect authority");
    fs::write(
        &discovery,
        serde_json::to_vec(&json!({
            "schema": "datum_agent_discovery_v1",
            "project_root": root,
            "terminal_session_id": "terminal-mcp-test",
            "context_id": "context-mcp-test",
            "agent_launch_id": "agent-launch-mcp-test",
            "credential_descriptor": authority,
            "session_lifecycle": "running",
            "storage": {"event_log_path": event_log},
            "capability_profile": "datum_agent_capability_v1",
            "capabilities": ["inspect", "propose"],
            "approval_policy": "owner-review-required",
            "unattended_tools": []
        }))
        .expect("serialize discovery"),
    )
    .expect("write discovery");

    let mut child = Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(["mcp", "serve", "--discovery"])
        .arg(&discovery)
        .env("DATUM_AGENT_CREDENTIAL_FILE", &credential)
        .env("DATUM_AGENT_LAUNCH_ID", "agent-launch-mcp-test")
        .env("DATUM_AGENT_ADAPTER_ID", "integration-test")
        .env_remove("DATUM_PROJECT_ROOT")
        .env_remove("DATUM_TERMINAL_SESSION_ID")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn canonical MCP command");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"id\":41,\"method\":\"initialize\",\"params\":{}}\n\
{\"jsonrpc\":\"2.0\",\"id\":42,\"method\":\"resources/list\",\"params\":{}}\n\
{\"jsonrpc\":\"2.0\",\"id\":43,\"method\":\"resources/templates/list\",\"params\":{}}\n\
{\"jsonrpc\":\"2.0\",\"id\":44,\"method\":\"prompts/list\",\"params\":{}}\n",
        )
        .expect("write MCP capability requests");
    let output = child.wait_with_output().expect("wait for MCP broker");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "protocol success must not log: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("UTF-8 JSON-RPC stdout");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 4, "stdout must contain only protocol messages");
    let response: Value = serde_json::from_str(lines[0]).expect("JSON-RPC response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 41);
    assert_eq!(response["result"]["serverInfo"]["name"], "datum-eda");
    assert!(response["result"]["capabilities"]["resources"].is_object());
    assert!(response["result"]["capabilities"]["prompts"].is_object());
    let resources: Value = serde_json::from_str(lines[1]).expect("resources response");
    assert!(
        resources["result"]["resources"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["uri"] == "datum://context/live"))
    );
    let templates: Value = serde_json::from_str(lines[2]).expect("templates response");
    assert!(
        templates["result"]["resourceTemplates"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| { item["uriTemplate"] == "datum://objects/{kind}{?cursor,limit}" }))
    );
    let prompts: Value = serde_json::from_str(lines[3]).expect("prompts response");
    assert!(
        prompts["result"]["prompts"]
            .as_array()
            .is_some_and(|items| items
                .iter()
                .any(|item| item["name"] == "datum.prepare-proposal"))
    );

    fs::remove_dir_all(&root).expect("remove fixture root");
}

#[test]
fn invalid_discovery_reports_only_to_stderr() {
    let root = unique_fixture_root();
    fs::create_dir_all(&root).expect("fixture root");
    let discovery = root.join("discovery.json");
    fs::write(&discovery, br#"{"schema":"future_v99"}"#).expect("write discovery");
    let output = Command::new(env!("CARGO_BIN_EXE_datum-eda"))
        .args(["mcp", "serve", "--discovery"])
        .arg(&discovery)
        .output()
        .expect("run canonical MCP command");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("structured stderr log");
    assert_eq!(error["component"], "datum-mcp");
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| message.contains("unsupported discovery schema"))
    );
    fs::remove_dir_all(&root).expect("remove fixture root");
}
