use super::*;
use crate::terminal_session::TerminalEvent;
use datum_gui_protocol::ApplicationFocus as KeyboardFocus;
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
fn accepted_input_returns_scrollback_to_live_cursor_and_clears_selection() {
    let mut state = datum_gui_protocol::TerminalLaneState::default();
    *state.pty_grid_mut().lines = (0..40).map(|row| format!("line {row}")).collect();
    state.scroll_offset = 18;
    state.set_text_selection((2, 1), (4, 3));

    follow_live_terminal_input(&mut state);

    assert_eq!(state.scroll_offset, 0);
    assert_eq!(state.text_selection_ordered(), None);
}

#[test]
fn terminal_focus_has_one_attached_pty_input_recipient() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-input-mode-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create terminal input-mode root");
    let context = TerminalLaunchContext::for_project_root(&root);
    let registry = TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
    let baseline = recorded_input_bytes(&registry);
    assert_eq!(
        keyboard_focus::terminal_input_owner(KeyboardFocus::Terminal, true),
        keyboard_focus::TerminalInputOwner::AttachedPty
    );
    assert_eq!(recorded_input_bytes(&registry), baseline);

    let payload = b"printf 'ti03-shell-proof'\r";
    assert!(write_attached_terminal_bytes(&registry, payload).unwrap());
    assert_eq!(
        recorded_input_bytes(&registry),
        baseline + payload.len(),
        "shell payload must be recorded exactly once"
    );

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut output = Vec::new();
    while Instant::now() < deadline
        && !String::from_utf8_lossy(&output).contains("ti03-shell-proof")
    {
        if let Ok(TerminalEvent::Output(bytes)) = registry
            .active()
            .recv_event_timeout(Duration::from_millis(25))
        {
            output.extend(bytes);
        }
    }
    assert!(
        String::from_utf8_lossy(&output).contains("ti03-shell-proof"),
        "shell must produce the payload"
    );
    let _ = fs::remove_dir_all(&root);
}
