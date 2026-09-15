use super::*;
use crate::terminal_agent_authority::TerminalAgentAuthority;
use crate::terminal_session::{TerminalEvent, TerminalLaunchContext, spawn_terminal_session};
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const AGENT_PROBE_TIMEOUT: Duration = Duration::from_secs(12);

fn unique_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "datum-agent-pty-proof-{}-{nonce}",
        std::process::id()
    ))
}

fn write_probe(path: &Path) {
    fs::write(
        path,
        br#"#!/bin/sh
set -eu
test "$PWD" = "$DATUM_EXPECT_PROJECT"
test "$DATUM_AGENT_DISCOVERY" = "$DATUM_EXPECT_DISCOVERY"
test "$SSH_AUTH_SOCK" = "$DATUM_EXPECT_AUTH"
exec python3 "$DATUM_AGENT_MCP_PROBE" "$@"
"#,
    )
    .expect("write agent probe");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("probe mode");
}

struct WorkflowPhase<'a> {
    cli: &'a Path,
    project: &'a Path,
    discovery: &'a Path,
    probe: &'a Path,
    adapter: &'a str,
    phase: &'a str,
    proposal_id: &'a str,
    resume: bool,
}

fn profile(workflow: &WorkflowPhase<'_>) -> TerminalLaunchProfile {
    let mut args = vec![
        OsString::from("agent"),
        OsString::from("launch"),
        OsString::from(workflow.adapter),
        OsString::from("--project-root"),
        workflow.project.as_os_str().to_owned(),
        OsString::from("--discovery"),
        workflow.discovery.as_os_str().to_owned(),
        OsString::from("--binary"),
        workflow.probe.as_os_str().to_owned(),
    ];
    if workflow.resume {
        args.push(OsString::from("--resume"));
    }
    if workflow.adapter == "cursor-cli" {
        args.push(OsString::from("--approve-project-config"));
    }
    args.extend([
        OsString::from("--"),
        OsString::from("--probe-adapter"),
        OsString::from(workflow.adapter),
        OsString::from("--probe-phase"),
        OsString::from(workflow.phase),
    ]);
    let mut profile = TerminalLaunchProfile {
        name: format!("agent-{}", workflow.adapter),
        executable: Some(workflow.cli.as_os_str().to_owned()),
        args,
        cwd: TerminalCwdTemplate::Project,
        environment: vec![
            (
                OsString::from("DATUM_EXPECT_PROJECT"),
                Some(workflow.project.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_EXPECT_DISCOVERY"),
                Some(workflow.discovery.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_EXPECT_AUTH"),
                Some(OsString::from("/tmp/datum-agent-auth-fixture.sock")),
            ),
            (
                OsString::from("SSH_AUTH_SOCK"),
                Some(OsString::from("/tmp/datum-agent-auth-fixture.sock")),
            ),
            (
                OsString::from("DATUM_EXPECT_CLI"),
                Some(workflow.cli.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_CLI_BIN"),
                Some(workflow.cli.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_AGENT_MCP_PROBE"),
                Some(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../scripts/agent_mcp_adapter_probe.py")
                        .into_os_string(),
                ),
            ),
            (
                OsString::from("DATUM_PROOF_PROPOSAL_ID"),
                Some(OsString::from(workflow.proposal_id)),
            ),
            (OsString::from("DATUM_MCP_ENDPOINT"), None),
        ],
        ..TerminalLaunchProfile::default()
    };
    profile.agent_authority = TerminalAgentAuthority::ApplyApproved;
    profile
}

fn cli_json(cli: &Path, arguments: &[&str]) -> serde_json::Value {
    let output = Command::new(cli)
        .arg("--format")
        .arg("json")
        .args(arguments)
        .output()
        .expect("run production Datum CLI");
    assert!(
        output.status.success(),
        "production Datum CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("production Datum CLI JSON")
}

fn project_context(cli: &Path, project: &Path) -> (String, Option<String>) {
    let project_arg = project.to_str().expect("UTF-8 proof path");
    let journal = cli_json(cli, &["journal", "list", project_arg]);
    let revision = journal["model_revision"]
        .as_str()
        .expect("journal model revision")
        .to_string();
    let cursor = journal["cursor_index"].as_u64().expect("journal cursor") as usize;
    let tip = cursor.checked_sub(1).and_then(|index| {
        journal["transactions"][index]["transaction_id"]
            .as_str()
            .map(str::to_string)
    });
    (revision, tip)
}

fn run_phase(
    cli: &Path,
    project: &Path,
    probe: &Path,
    adapter: &str,
    phase: &str,
    proposal_id: &str,
    resume: bool,
) {
    let discovery = project.join("discovery.json");
    fs::write(&discovery, b"{}\n").expect("agent launch discovery placeholder");
    let (revision, accepted_transaction_tip) = project_context(cli, project);
    let mut context = TerminalLaunchContext::for_project_root(project);
    context.project_name = Some(format!("Agent Workflow {adapter}"));
    context.source_revision = Some(revision);
    context.accepted_transaction_tip = accepted_transaction_tip;
    context.terminal_profile = profile(&WorkflowPhase {
        cli,
        project,
        discovery: &discovery,
        probe,
        adapter,
        phase,
        proposal_id,
        resume,
    });
    let session = spawn_terminal_session(&context).expect("spawn agent CLI through owned PTY");
    let deadline = Instant::now() + AGENT_PROBE_TIMEOUT;
    let mut output = Vec::new();
    let mut exit = None;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(50)) {
            Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
            Ok(TerminalEvent::Exited(status)) => {
                exit = Some(status);
                break;
            }
            Ok(TerminalEvent::Error(error)) => panic!("{adapter} transport error: {error:?}"),
            Err(_) => {}
        }
    }
    let rendered = String::from_utf8_lossy(&output);
    assert_eq!(
        exit,
        Some(crate::terminal_transport::TerminalExitStatus::Code(0)),
        "{adapter} {phase} phase did not exit successfully; output={rendered:?}"
    );
    assert!(
        rendered.contains(&format!("AGENT_WORKFLOW_OK:{adapter}:{phase}")),
        "{adapter} did not complete {phase} workflow phase; output={rendered:?}"
    );
}

fn run_adapter(cli: &Path, root: &Path, probe: &Path, adapter: &str, ordinal: u128) {
    let project = root.join(adapter);
    let project_arg = project.to_str().expect("UTF-8 proof path");
    cli_json(
        cli,
        &[
            "project",
            "new",
            project_arg,
            "--name",
            &format!("Agent {adapter}"),
        ],
    );
    let proposal_id = format!("00000000-0000-4000-8000-{ordinal:012x}");
    run_phase(
        cli,
        &project,
        probe,
        adapter,
        "propose",
        &proposal_id,
        false,
    );
    run_phase(
        cli,
        &project,
        probe,
        adapter,
        "resume",
        &proposal_id,
        adapter != "local-generic",
    );
    assert!(
        !project
            .join(".datum/runtime")
            .read_dir()
            .is_ok_and(|mut entries| entries.next().is_some()),
        "{adapter} retained session runtime configuration"
    );
    assert!(
        !project.join(".cursor/mcp.json").exists(),
        "{adapter} retained the temporary Cursor project overlay"
    );
}

#[test]
#[ignore = "run with scripts/run_agent_launch_pty_proof.sh"]
fn governed_agents_complete_production_workflow_through_owned_pty() {
    let cli = PathBuf::from(
        std::env::var_os("DATUM_AGENT_CLI_PROOF_BIN")
            .expect("proof runner provides DATUM_AGENT_CLI_PROOF_BIN"),
    );
    assert!(cli.is_file(), "missing CLI proof binary: {}", cli.display());
    let root = unique_root();
    fs::create_dir_all(&root).expect("fixture root");
    let probe = root.join("agent-probe.sh");
    write_probe(&probe);
    for (index, adapter) in ["codex", "claude-code", "cursor-cli", "local-generic"]
        .into_iter()
        .enumerate()
    {
        run_adapter(&cli, &root, &probe, adapter, index as u128 + 1);
    }
    fs::remove_dir_all(root).expect("remove fixture root");
}
