use super::*;
use crate::{
    terminal_core_adapter::TerminalCoreSessionAdapter,
    terminal_session::{TerminalEvent, TerminalLaunchContext, spawn_terminal_session},
};
use datum_gui_protocol::TerminalLaneState;
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

struct CompatibilityProbe {
    name: &'static str,
    program: PathBuf,
    args: Vec<OsString>,
    expected: &'static str,
    input_after_start: &'static [u8],
    input_after_expected: &'static [u8],
    expected_exit_code: Option<i32>,
}

struct ProbeResult {
    name: &'static str,
    version: String,
    rendered_text: String,
    exit: String,
}

fn unique_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("datum-terminal-p28-{}-{nonce}", std::process::id()))
}

fn tool_path(name: &str) -> PathBuf {
    if let Some(root) = std::env::var_os("DATUM_P28_TOOL_ROOT") {
        let candidate = PathBuf::from(root).join("usr/bin").join(name);
        if candidate.is_file() {
            return candidate;
        }
    }
    let output = Command::new("/bin/sh")
        .args(["-c", "command -v -- \"$1\"", "datum-p28", name])
        .output()
        .unwrap_or_else(|error| panic!("resolve DTC-P28 tool {name}: {error}"));
    assert!(
        output.status.success(),
        "DTC-P28 requires installed black-box witness `{name}`; set DATUM_P28_TOOL_ROOT to a disposable extracted package root"
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("tool path is UTF-8")
            .trim(),
    )
}

fn version(program: &Path, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("read {} version: {error}", program.display()));
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn profile(program: &Path, args: Vec<OsString>) -> TerminalLaunchProfile {
    TerminalLaunchProfile {
        name: "dtc-p28-witness".to_string(),
        executable: Some(program.as_os_str().to_owned()),
        args,
        cwd: TerminalCwdTemplate::Project,
        environment: vec![
            (OsString::from("NO_COLOR"), None),
            (OsString::from("PAGER"), None),
            (
                OsString::from("NVIM_LOG_FILE"),
                Some(OsString::from("/dev/null")),
            ),
        ],
        ..TerminalLaunchProfile::default()
    }
}

fn run_probe(root: &Path, probe: CompatibilityProbe, version: String) -> ProbeResult {
    let mut context = TerminalLaunchContext::for_project_root(root);
    context.terminal_profile = profile(&probe.program, probe.args);
    let session = spawn_terminal_session(&context)
        .unwrap_or_else(|error| panic!("spawn {} compatibility witness: {error:#}", probe.name));
    let mut adapter = TerminalCoreSessionAdapter::new_with_profile(
        session.session_id(),
        "dtc-p28",
        120,
        36,
        &context.terminal_profile,
    )
    .expect("create production TerminalCore adapter");
    let mut lane = TerminalLaneState::default();
    if !probe.input_after_start.is_empty() {
        session
            .write_bytes(probe.input_after_start)
            .unwrap_or_else(|error| panic!("write {} startup input: {error:#}", probe.name));
    }
    let deadline = Instant::now() + PROBE_TIMEOUT;
    let mut expected_seen = false;
    let mut exit = None;
    while Instant::now() < deadline {
        match session.recv_event_timeout(Duration::from_millis(50)) {
            Ok(TerminalEvent::Output(bytes)) => {
                let update = adapter
                    .apply_output(&mut lane, &bytes)
                    .unwrap_or_else(|error| panic!("apply {} output: {error}", probe.name));
                for reply in update.replies {
                    session
                        .write_bytes(&reply)
                        .unwrap_or_else(|error| panic!("reply to {}: {error:#}", probe.name));
                }
                if !expected_seen && adapter.test_plain_text().contains(probe.expected) {
                    expected_seen = true;
                    if !probe.input_after_expected.is_empty() {
                        session
                            .write_bytes(probe.input_after_expected)
                            .unwrap_or_else(|error| {
                                panic!("write {} completion input: {error:#}", probe.name)
                            });
                    }
                }
            }
            Ok(TerminalEvent::Exited(status)) => {
                exit = Some(match status {
                    crate::terminal_transport::TerminalExitStatus::Code(code) => {
                        if let Some(expected) = probe.expected_exit_code {
                            assert_eq!(code, expected, "{} exit status", probe.name);
                        }
                        format!("code:{code}")
                    }
                    crate::terminal_transport::TerminalExitStatus::Signal { signal, .. } => {
                        panic!("{} unexpectedly exited by signal {signal}", probe.name)
                    }
                });
                break;
            }
            Ok(TerminalEvent::Error(error)) => {
                panic!("{} transport error: {error:?}", probe.name)
            }
            Err(_) => {}
        }
    }
    let rendered_text = adapter.test_plain_text();
    assert!(
        expected_seen || rendered_text.contains(probe.expected),
        "{} did not render {:?}; screen was {:?}",
        probe.name,
        probe.expected,
        rendered_text
    );
    if exit.is_none() {
        let _ = session.terminate();
    }
    ProbeResult {
        name: probe.name,
        version,
        rendered_text,
        exit: exit.unwrap_or_else(|| "terminated-after-proof".to_string()),
    }
}

fn shell_probe(
    root: &Path,
    name: &'static str,
    binary: &str,
    version_args: &[&str],
    args: &[&str],
    marker: &'static str,
) -> ProbeResult {
    let program = tool_path(binary);
    let version = version(&program, version_args);
    run_probe(
        root,
        CompatibilityProbe {
            name,
            program,
            args: args.iter().map(OsString::from).collect(),
            expected: marker,
            input_after_start: format!("printf '\\033[38;5;46m{marker}\\033[0m\\n'\nexit\n")
                .leak()
                .as_bytes(),
            input_after_expected: b"",
            expected_exit_code: Some(0),
        },
        version,
    )
}

#[test]
#[ignore = "DTC-P28 black-box witness: run through scripts/run_terminal_compatibility_proof.sh"]
fn production_pty_proves_named_shell_tui_and_tool_compatibility() {
    let root = unique_root();
    fs::create_dir_all(&root).expect("create DTC-P28 root");
    fs::write(
        root.join("fixture.txt"),
        (0..80)
            .map(|line| format!("DTC-P28-LESS-{line:02}\n"))
            .collect::<String>(),
    )
    .expect("write less/vim fixture");

    let mut results = vec![
        shell_probe(
            &root,
            "bash",
            "bash",
            &["--version"],
            &["--noprofile", "--norc"],
            "DTC-P28-BASH",
        ),
        shell_probe(&root, "zsh", "zsh", &["--version"], &["-f"], "DTC-P28-ZSH"),
        shell_probe(
            &root,
            "fish",
            "fish",
            &["--version"],
            &["--no-config"],
            "DTC-P28-FISH",
        ),
    ];

    let ssh = tool_path("ssh");
    results.push(run_probe(
        &root,
        CompatibilityProbe {
            name: "ssh",
            program: ssh.clone(),
            args: [
                "-F",
                "/dev/null",
                "-oBatchMode=yes",
                "-oConnectTimeout=1",
                "-p",
                "1",
                "127.0.0.1",
            ]
            .into_iter()
            .map(OsString::from)
            .collect(),
            expected: "connect to host 127.0.0.1 port 1",
            input_after_start: b"",
            input_after_expected: b"",
            expected_exit_code: Some(255),
        },
        version(&ssh, &["-V"]),
    ));

    for (name, binary, marker, quit) in [
        ("vim", "vim.tiny", "DTC-P28-LESS-00", b":qa!\r".as_slice()),
        ("neovim", "nvim", "DTC-P28-LESS-00", b":qa!\r".as_slice()),
        ("htop", "htop", "CPU", b"q".as_slice()),
        ("btop", "btop", "CPU", b"q".as_slice()),
    ] {
        let program = tool_path(binary);
        let (args, version_args) = match name {
            "vim" => (
                vec!["-Nu", "NONE", "-n", "-i", "NONE", "fixture.txt"],
                vec!["--version"],
            ),
            "neovim" => (
                vec!["-u", "NONE", "-n", "-i", "NONE", "fixture.txt"],
                vec!["--version"],
            ),
            "htop" => (vec!["-d", "10"], vec!["--version"]),
            _ => (vec!["--utf-force"], vec!["--version"]),
        };
        results.push(run_probe(
            &root,
            CompatibilityProbe {
                name,
                program: program.clone(),
                args: args.into_iter().map(OsString::from).collect(),
                expected: marker,
                input_after_start: b"",
                input_after_expected: quit,
                expected_exit_code: Some(0),
            },
            version(&program, &version_args),
        ));
    }

    let less = tool_path("less");
    results.push(run_probe(
        &root,
        CompatibilityProbe {
            name: "less",
            program: less.clone(),
            args: ["-R", "-X", "fixture.txt"]
                .into_iter()
                .map(OsString::from)
                .collect(),
            expected: "DTC-P28-LESS-79",
            input_after_start: b"G",
            input_after_expected: b"q",
            expected_exit_code: Some(0),
        },
        version(&less, &["--version"]),
    ));

    for (name, binary, version_args, args, expected, input) in [
        (
            "python",
            "python3",
            &["--version"][..],
            vec!["-q"],
            "DTC-P28-PYTHON",
            b"print('DTC-P28-PYTHON')\nexit()\n".as_slice(),
        ),
        (
            "git",
            "git",
            &["--version"][..],
            vec!["--version"],
            "git version",
            b"".as_slice(),
        ),
        (
            "cargo",
            "cargo",
            &["--version"][..],
            vec!["--version"],
            "cargo ",
            b"".as_slice(),
        ),
    ] {
        let program = tool_path(binary);
        results.push(run_probe(
            &root,
            CompatibilityProbe {
                name,
                program: program.clone(),
                args: args.into_iter().map(OsString::from).collect(),
                expected,
                input_after_start: input,
                input_after_expected: b"",
                expected_exit_code: Some(0),
            },
            version(&program, version_args),
        ));
    }

    let tmux = tool_path("tmux");
    let socket = format!("datum-p28-{}", std::process::id());
    results.push(run_probe(
        &root,
        CompatibilityProbe {
            name: "tmux",
            program: tmux.clone(),
            args: ["-L", &socket, "-f", "/dev/null", "new-session", "/bin/sh"]
                .into_iter()
                .map(OsString::from)
                .collect(),
            expected: "DTC-P28-TMUX",
            input_after_start: b"printf 'DTC-P28-TMUX\\n'\n",
            input_after_expected: b"exit\n",
            expected_exit_code: Some(0),
        },
        version(&tmux, &["-V"]),
    ));

    for result in &results {
        assert!(!result.version.is_empty(), "{} version", result.name);
        assert!(
            !result.rendered_text.is_empty(),
            "{} rendered output",
            result.name
        );
        eprintln!(
            "DTC-P28 {} | {} | {}",
            result.name, result.version, result.exit
        );
    }
    assert_eq!(results.len(), 13);
    if let Some(path) = std::env::var_os("DATUM_P28_EVIDENCE") {
        let payload = serde_json::json!({
            "schema": "datum-terminal-compatibility-v1",
            "revision": std::env::var("DATUM_P28_REVISION").unwrap_or_else(|_| "unrecorded".to_string()),
            "platform": format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            "transport": "production TerminalTransportSession + TerminalCoreSessionAdapter",
            "results": results.iter().map(|result| serde_json::json!({
                "name": result.name,
                "version": result.version,
                "exit": result.exit,
                "rendered_nonempty": !result.rendered_text.is_empty(),
                "status": "passed",
            })).collect::<Vec<_>>(),
            "failures": [],
        });
        fs::write(
            path,
            serde_json::to_vec_pretty(&payload).expect("serialize DTC-P28 evidence"),
        )
        .expect("write DTC-P28 evidence");
    }
    fs::remove_dir_all(root).expect("remove DTC-P28 root");
}
