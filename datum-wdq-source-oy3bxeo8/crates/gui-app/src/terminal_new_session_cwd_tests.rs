use crate::{
    terminal_session::{TerminalEvent, TerminalLaunchContext, spawn_terminal_session},
    terminal_transport::TerminalExitStatus,
    terminal_working_directory::context_for_new_terminal,
};
use std::{fs, time::Duration};

#[test]
fn production_shell_starts_in_active_local_cwd_while_project_identity_stays_stable() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-new-cwd-production-{}",
        std::process::id()
    ));
    let project = root.join("project");
    let active_cwd = root.join("agent work");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&active_cwd).unwrap();
    let base = TerminalLaunchContext::for_project_root(&project);
    let reported = format!(
        "file://{}",
        active_cwd.display().to_string().replace(' ', "%20")
    );
    let context = context_for_new_terminal(&base, Some(&reported));
    let session = spawn_terminal_session(&context).unwrap();

    session
        .write_bytes(b"printf 'DTC-P27-CWD:%s|%s\\n' \"$PWD\" \"$DATUM_PROJECT_ROOT\"\nexit\n")
        .unwrap();
    let mut output = Vec::new();
    let mut exit = None;
    for _ in 0..100 {
        match session.recv_event_timeout(Duration::from_millis(100)) {
            Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
            Ok(TerminalEvent::Exited(status)) => exit = Some(status),
            Ok(TerminalEvent::Error(error)) => panic!("unexpected terminal error: {error:?}"),
            Err(_) => {}
        }
        if exit.is_some() && String::from_utf8_lossy(&output).contains("DTC-P27-CWD:") {
            break;
        }
    }

    let output = String::from_utf8_lossy(&output);
    assert!(
        output.contains(&format!(
            "DTC-P27-CWD:{}|{}",
            active_cwd.display(),
            project.display()
        )),
        "shell launch did not preserve CWD/project separation: {output}"
    );
    assert_eq!(exit, Some(TerminalExitStatus::Code(0)));
    let _ = fs::remove_dir_all(root);
}
