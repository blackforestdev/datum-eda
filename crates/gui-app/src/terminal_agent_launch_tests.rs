use super::*;
use crate::terminal_session::{TerminalEvent, TerminalLaunchContext, spawn_terminal_session};
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
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
adapter=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --probe-adapter) adapter=$2; shift 2 ;;
    *) shift ;;
  esac
done
test -t 0
test -t 1
test -t 2
test "$PWD" = "$DATUM_EXPECT_PROJECT"
test "$DATUM_AGENT_DISCOVERY" = "$DATUM_EXPECT_DISCOVERY"
test "$SSH_AUTH_SOCK" = "$DATUM_EXPECT_AUTH"
printf 'AGENT_PTY_OK:%s:%s\n' "$adapter" "$DATUM_AGENT_DISCOVERY"
"#,
    )
    .expect("write agent probe");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("probe mode");
}

fn profile(
    cli: &Path,
    project: &Path,
    discovery: &Path,
    probe: &Path,
    adapter: &str,
) -> TerminalLaunchProfile {
    let mut args = vec![
        OsString::from("agent"),
        OsString::from("launch"),
        OsString::from(adapter),
        OsString::from("--project-root"),
        project.as_os_str().to_owned(),
        OsString::from("--discovery"),
        discovery.as_os_str().to_owned(),
        OsString::from("--binary"),
        probe.as_os_str().to_owned(),
    ];
    if adapter == "cursor-cli" {
        args.push(OsString::from("--approve-project-config"));
    }
    args.extend([
        OsString::from("--"),
        OsString::from("--probe-adapter"),
        OsString::from(adapter),
    ]);
    TerminalLaunchProfile {
        name: format!("agent-{adapter}"),
        executable: Some(cli.as_os_str().to_owned()),
        args,
        cwd: TerminalCwdTemplate::Project,
        environment: vec![
            (
                OsString::from("DATUM_EXPECT_PROJECT"),
                Some(project.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_EXPECT_DISCOVERY"),
                Some(discovery.as_os_str().to_owned()),
            ),
            (
                OsString::from("DATUM_EXPECT_AUTH"),
                Some(OsString::from("/tmp/datum-agent-auth-fixture.sock")),
            ),
            (
                OsString::from("SSH_AUTH_SOCK"),
                Some(OsString::from("/tmp/datum-agent-auth-fixture.sock")),
            ),
        ],
        ..TerminalLaunchProfile::default()
    }
}

fn run_adapter(cli: &Path, root: &Path, probe: &Path, adapter: &str) {
    let project = root.join(adapter);
    fs::create_dir_all(&project).expect("project root");
    let discovery = project.join("discovery.json");
    fs::write(&discovery, br#"{"schema":"datum_agent_discovery_v1"}"#).expect("discovery document");
    let mut context = TerminalLaunchContext::for_project_root(&project);
    context.terminal_profile = profile(cli, &project, &discovery, probe, adapter);
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
        "{adapter} did not exit successfully; output={rendered:?}"
    );
    assert!(
        rendered.contains(&format!("AGENT_PTY_OK:{adapter}:{}", discovery.display())),
        "{adapter} did not preserve PTY/cwd/auth/discovery identity; output={rendered:?}"
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
fn governed_agents_launch_through_owned_pty_with_context_intact() {
    let cli = PathBuf::from(
        std::env::var_os("DATUM_AGENT_CLI_PROOF_BIN")
            .expect("proof runner provides DATUM_AGENT_CLI_PROOF_BIN"),
    );
    assert!(cli.is_file(), "missing CLI proof binary: {}", cli.display());
    let root = unique_root();
    fs::create_dir_all(&root).expect("fixture root");
    let probe = root.join("agent-probe.sh");
    write_probe(&probe);
    for adapter in ["codex", "claude-code", "cursor-cli", "local-generic"] {
        run_adapter(&cli, &root, &probe, adapter);
    }
    fs::remove_dir_all(root).expect("remove fixture root");
}
