use super::*;
use crate::terminal_session::TerminalEvent;
use datum_gui_protocol::TerminalLaneState;
use std::fs;
use std::time::{Duration, Instant};

fn recorded_input_bytes(registry: &TerminalSessionRegistry) -> usize {
    fs::read_to_string(registry.active_event_log_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "terminal_io" && event["direction"] == "input_accepted")
        .map(|event| event["byte_count"].as_u64().unwrap_or(0) as usize)
        .sum()
}

#[test]
fn detached_and_rename_modes_write_zero_bytes_then_reattach_writes_once() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-input-mode-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create terminal input-mode root");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
    let session_id = registry.active().session_id().to_string();
    let mut state = TerminalLaneState::default();
    let baseline = recorded_input_bytes(&registry);

    registry
        .detach_active(&mut state)
        .expect("detach active terminal session");
    assert_eq!(
        keyboard_focus::terminal_input_owner(KeyboardFocus::Terminal, true, false, false),
        keyboard_focus::TerminalInputOwner::DetachedReadOnly
    );
    assert!(!write_attached_terminal_bytes(&registry, b"detached").unwrap());
    assert_eq!(recorded_input_bytes(&registry), baseline);

    state.rename_input = "chrome only".to_string();
    state.rename_cursor = state.rename_input.chars().count();
    assert_eq!(
        keyboard_focus::terminal_input_owner(KeyboardFocus::Terminal, true, false, true),
        keyboard_focus::TerminalInputOwner::RenameChrome
    );
    assert_eq!(recorded_input_bytes(&registry), baseline);

    registry
        .activate(&session_id)
        .expect("reattach terminal session");
    let payload = b"printf 'ti03-reattach-proof'\r";
    assert!(write_attached_terminal_bytes(&registry, payload).unwrap());
    assert_eq!(
        recorded_input_bytes(&registry),
        baseline + payload.len(),
        "reattached payload must be recorded exactly once"
    );

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut output = Vec::new();
    while Instant::now() < deadline
        && !String::from_utf8_lossy(&output).contains("ti03-reattach-proof")
    {
        if let Ok(TerminalEvent::Output(bytes)) = registry
            .active()
            .recv_event_timeout(Duration::from_millis(25))
        {
            output.extend(bytes);
        }
    }
    assert!(
        String::from_utf8_lossy(&output).contains("ti03-reattach-proof"),
        "reattached shell must produce the recovery payload"
    );
    let _ = fs::remove_dir_all(&root);
}
