use super::*;
use std::{ffi::OsString, os::unix::fs::PermissionsExt};

fn fixture_root(label: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("datum-agent-acceptance-{label}-{}", Uuid::new_v4()));
    fs::create_dir_all(&root).expect("fixture root");
    root
}

fn write_executable(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write executable fixture");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("fixture mode");
}

fn launch_args(adapter: &str, project: &Path, discovery: &Path) -> AgentLaunchArgs {
    AgentLaunchArgs {
        adapter: adapter.to_string(),
        project_root: project.to_path_buf(),
        discovery: Some(discovery.to_path_buf()),
        binary: None,
        resume: false,
        resume_id: None,
        approve_project_config: false,
        native_args: Vec::new(),
    }
}

fn command_args(command: &Command) -> Vec<OsString> {
    command.get_args().map(OsString::from).collect()
}

#[test]
fn doctor_reports_probe_version_and_missing_or_nonexecutable_clients() {
    let root = fixture_root("doctor");
    let versioned = root.join("versioned-agent");
    write_executable(&versioned, b"#!/bin/sh\nprintf 'fixture-agent 1.2.3\\n'\n");
    let report = doctor_agent(&AgentDoctorArgs {
        adapter: "codex".to_string(),
        binary: Some(versioned),
    })
    .expect("successful doctor");
    assert!(report.available);
    assert!(report.launch_ready);
    assert_eq!(report.version.as_deref(), Some("fixture-agent 1.2.3"));

    let missing = doctor_agent(&AgentDoctorArgs {
        adapter: "claude-code".to_string(),
        binary: Some(root.join("missing-agent")),
    })
    .expect("missing doctor remains structured");
    assert!(!missing.available);
    assert!(!missing.launch_ready);
    assert_eq!(missing.diagnostics, ["client executable was not found"]);

    let nonexecutable = root.join("nonexecutable-agent");
    fs::write(&nonexecutable, b"#!/bin/sh\nexit 0\n").expect("nonexecutable fixture");
    let report = doctor_agent(&AgentDoctorArgs {
        adapter: "cursor-cli".to_string(),
        binary: Some(nonexecutable),
    })
    .expect("nonexecutable doctor remains structured");
    assert!(!report.available);
    assert!(!report.launch_ready);
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn native_resume_arguments_are_exact_and_generic_resume_refuses() {
    let root = fixture_root("resume");
    let discovery = root.join("discovery.json");
    fs::write(&discovery, b"{}").expect("discovery");
    for (adapter_id, latest, identity) in [
        (
            "codex",
            vec![OsString::from("resume"), OsString::from("--last")],
            vec![OsString::from("resume"), OsString::from("opaque-id")],
        ),
        (
            "claude-code",
            vec![OsString::from("--continue")],
            vec![OsString::from("--resume"), OsString::from("opaque-id")],
        ),
        (
            "cursor-cli",
            vec![OsString::from("resume")],
            vec![OsString::from("--resume"), OsString::from("opaque-id")],
        ),
    ] {
        let adapter = agent_adapter(adapter_id).expect("governed adapter");
        let mut latest_args = launch_args(adapter_id, &root, &discovery);
        latest_args.resume = true;
        let mut command = Command::new("/bin/true");
        apply_resume(adapter, &latest_args, &mut command).expect("latest resume");
        assert_eq!(command_args(&command), latest, "{adapter_id} latest");

        let mut identity_args = launch_args(adapter_id, &root, &discovery);
        identity_args.resume_id = Some("opaque-id".to_string());
        let mut command = Command::new("/bin/true");
        apply_resume(adapter, &identity_args, &mut command).expect("identity resume");
        assert_eq!(command_args(&command), identity, "{adapter_id} identity");
    }

    let adapter = agent_adapter("local-generic").expect("generic adapter");
    let mut args = launch_args(adapter.id, &root, &discovery);
    args.resume = true;
    let error = apply_resume(adapter, &args, &mut Command::new("/bin/true"))
        .expect_err("generic resume must remain unsupported");
    assert!(error.to_string().contains("does not support native resume"));
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn failed_cursor_spawn_restores_config_and_removes_runtime() {
    let project = fixture_root("spawn-failure");
    let discovery = project.join("discovery.json");
    fs::write(&discovery, b"{}").expect("discovery");
    let config = project.join(".cursor/mcp.json");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config parent");
    let original = br#"{"mcpServers":{"owner":{"command":"owner"}}}"#;
    fs::write(&config, original).expect("original config");
    let invalid = project.join("invalid-exec-format");
    write_executable(&invalid, b"not an executable image\n");
    let mut args = launch_args("cursor-cli", &project, &discovery);
    args.binary = Some(invalid);
    args.approve_project_config = true;
    let error = launch_agent(&OutputFormat::Text, args).expect_err("spawn must fail");
    assert!(error.to_string().contains("failed to launch cursor-cli"));
    assert_eq!(fs::read(&config).expect("restored config"), original);
    let runtime = project.join(".datum/runtime");
    assert!(
        !runtime.exists()
            || fs::read_dir(runtime)
                .expect("runtime directory")
                .next()
                .is_none()
    );
    fs::remove_dir_all(project).expect("remove fixture");
}
