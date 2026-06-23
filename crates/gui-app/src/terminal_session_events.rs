use crate::terminal_session::TerminalSession;
use anyhow::{Context, Result};
use datum_gui_protocol::{DatumToolSessionLifecycle, TerminalCommandHandoff};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

static TERMINAL_COMMAND_EXECUTION_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize)]
struct TerminalCommandHandoffEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    origin: &'a str,
    command_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_id: Option<&'a str>,
    mcp_alias: Option<&'a str>,
    handoff_mode: &'a str,
    command: &'a str,
    occurred_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct TerminalCommandLifecycleEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    origin: &'a str,
    command_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_id: Option<&'a str>,
    command: &'a str,
    lifecycle: &'a str,
    process_exit_code: Option<i32>,
    occurred_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct TerminalLifecycleEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    lifecycle: &'static str,
    process_exit_code: Option<i32>,
    occurred_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct TerminalIoEvent<'a> {
    event: &'static str,
    schema_version: u64,
    session_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_id: Option<&'a str>,
    direction: &'static str,
    byte_count: usize,
    text_preview: String,
    truncated: bool,
    occurred_unix_ms: u128,
}

pub(super) fn prepare_terminal_command_execution(
    session: &TerminalSession,
    origin: &str,
    handoff: &TerminalCommandHandoff,
) -> Result<String> {
    let execution_id = next_terminal_command_execution_id(session.session_id());
    append_terminal_command_handoff_event(
        &session.event_log_path(),
        session.session_id(),
        origin,
        "execute",
        handoff,
        Some(&execution_id),
    )?;
    append_terminal_command_lifecycle_event(
        &session.event_log_path(),
        session.session_id(),
        origin,
        handoff,
        Some(&execution_id),
        "started",
        None,
    )?;
    session.set_active_execution_id(execution_id.clone());
    Ok(terminal_command_lifecycle_shell_wrapper(
        origin,
        handoff,
        &execution_id,
    ))
}

pub(super) fn terminal_command_lifecycle_shell_wrapper(
    origin: &str,
    handoff: &TerminalCommandHandoff,
    execution_id: &str,
) -> String {
    format!(
        "__datum_command_id={command_id}; __datum_origin={origin}; __datum_execution_id={execution_id}; {{ {command}; }}; __datum_exit=$?; python3 -c {python} \"$DATUM_TOOL_SESSION_EVENT_LOG\" \"$DATUM_SESSION_ID\" \"$__datum_origin\" \"$__datum_command_id\" \"$__datum_execution_id\" \"$__datum_exit\"; unset __datum_command_id __datum_origin __datum_execution_id __datum_exit",
        command_id = shell_single_quote(&handoff.command_id),
        origin = shell_single_quote(origin),
        execution_id = shell_single_quote(execution_id),
        command = handoff.command,
        python = shell_single_quote(TERMINAL_COMMAND_FINISH_PYTHON),
    )
}

pub(super) fn record_manual_terminal_command_handoff(
    session: &TerminalSession,
    origin: &str,
    command_id: &str,
    handoff_mode: &str,
    command: &str,
) -> Result<()> {
    append_terminal_command_handoff_event(
        &session.event_log_path(),
        session.session_id(),
        origin,
        handoff_mode,
        &TerminalCommandHandoff {
            command_id: command_id.to_string(),
            mcp_alias: None,
            command: command.to_string(),
        },
        None,
    )
}

const TERMINAL_COMMAND_FINISH_PYTHON: &str = r#"import json,sys,time
p,s,o,c,e,x=sys.argv[1:7]
open(p,"a",encoding="utf-8").write(json.dumps({"event":"terminal_command_lifecycle","schema_version":1,"session_id":s,"origin":o,"command_id":c,"execution_id":e,"command":None,"lifecycle":"finished","process_exit_code":int(x),"occurred_unix_ms":int(time.time()*1000)},separators=(",",":"))+"\n")"#;

pub(super) fn record_terminal_lifecycle_event(
    session: &TerminalSession,
    lifecycle: DatumToolSessionLifecycle,
    process_exit_code: Option<i32>,
) -> Result<()> {
    append_terminal_lifecycle_event(
        &session.event_log_path(),
        session.session_id(),
        lifecycle,
        process_exit_code,
    )
}

pub(super) fn record_terminal_input_event(session: &TerminalSession, bytes: &[u8]) -> Result<()> {
    let execution_id = session.active_execution_id();
    append_terminal_io_event(
        &session.event_log_path(),
        session.session_id(),
        execution_id.as_deref(),
        "input",
        bytes,
    )
}

pub(super) fn record_terminal_output_event(session: &TerminalSession, bytes: &[u8]) -> Result<()> {
    let execution_id = session.active_execution_id();
    let result = append_terminal_io_event(
        &session.event_log_path(),
        session.session_id(),
        execution_id.as_deref(),
        "output",
        bytes,
    );
    if let Some(execution_id) = execution_id
        && terminal_command_execution_finished(session, &execution_id)
    {
        session.clear_active_execution_id(&execution_id);
    }
    result
}

fn append_terminal_io_event(
    path: &std::path::Path,
    session_id: &str,
    execution_id: Option<&str>,
    direction: &'static str,
    bytes: &[u8],
) -> Result<()> {
    let occurred_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal I/O event timestamp")?
        .as_millis();
    let (text_preview, truncated) = terminal_text_preview(bytes);
    let event = TerminalIoEvent {
        event: "terminal_io",
        schema_version: 1,
        session_id,
        execution_id,
        direction,
        byte_count: bytes.len(),
        text_preview,
        truncated,
        occurred_unix_ms,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open terminal I/O event log {}", path.display()))?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&event).context("serialize terminal I/O event")?
    )
    .with_context(|| format!("append terminal I/O event {}", path.display()))
}

fn terminal_command_execution_finished(session: &TerminalSession, execution_id: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(session.event_log_path()) else {
        return false;
    };
    text.lines().rev().any(|line| {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            return false;
        };
        event.get("event").and_then(serde_json::Value::as_str) == Some("terminal_command_lifecycle")
            && event
                .get("execution_id")
                .and_then(serde_json::Value::as_str)
                == Some(execution_id)
            && event.get("lifecycle").and_then(serde_json::Value::as_str) == Some("finished")
    })
}

fn terminal_text_preview(bytes: &[u8]) -> (String, bool) {
    const MAX_PREVIEW_CHARS: usize = 512;
    let text = String::from_utf8_lossy(bytes);
    let mut preview = String::new();
    let mut truncated = false;
    for (index, ch) in text.chars().enumerate() {
        if index == MAX_PREVIEW_CHARS {
            truncated = true;
            break;
        }
        preview.push(ch);
    }
    (preview, truncated || text.len() != bytes.len())
}

fn append_terminal_lifecycle_event(
    path: &std::path::Path,
    session_id: &str,
    lifecycle: DatumToolSessionLifecycle,
    process_exit_code: Option<i32>,
) -> Result<()> {
    let occurred_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal lifecycle event timestamp")?
        .as_millis();
    let event = TerminalLifecycleEvent {
        event: "terminal_lifecycle",
        schema_version: 1,
        session_id,
        lifecycle: lifecycle.as_str(),
        process_exit_code,
        occurred_unix_ms,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open terminal lifecycle event log {}", path.display()))?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&event).context("serialize terminal lifecycle event")?
    )
    .with_context(|| format!("append terminal lifecycle event {}", path.display()))
}

fn append_terminal_command_handoff_event(
    path: &std::path::Path,
    session_id: &str,
    origin: &str,
    handoff_mode: &str,
    handoff: &TerminalCommandHandoff,
    execution_id: Option<&str>,
) -> Result<()> {
    let occurred_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal handoff event timestamp")?
        .as_millis();
    let event = TerminalCommandHandoffEvent {
        event: "terminal_command_handoff",
        schema_version: 1,
        session_id,
        origin,
        command_id: &handoff.command_id,
        execution_id,
        mcp_alias: handoff.mcp_alias.as_deref(),
        handoff_mode,
        command: &handoff.command,
        occurred_unix_ms,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open terminal handoff event log {}", path.display()))?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&event).context("serialize terminal handoff event")?
    )
    .with_context(|| format!("append terminal handoff event {}", path.display()))
}

fn append_terminal_command_lifecycle_event(
    path: &std::path::Path,
    session_id: &str,
    origin: &str,
    handoff: &TerminalCommandHandoff,
    execution_id: Option<&str>,
    lifecycle: &str,
    process_exit_code: Option<i32>,
) -> Result<()> {
    let occurred_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal command lifecycle event timestamp")?
        .as_millis();
    let event = TerminalCommandLifecycleEvent {
        event: "terminal_command_lifecycle",
        schema_version: 1,
        session_id,
        origin,
        command_id: &handoff.command_id,
        execution_id,
        command: &handoff.command,
        lifecycle,
        process_exit_code,
        occurred_unix_ms,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| {
            format!(
                "open terminal command lifecycle event log {}",
                path.display()
            )
        })?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&event).context("serialize terminal command lifecycle event")?
    )
    .with_context(|| format!("append terminal command lifecycle event {}", path.display()))
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn next_terminal_command_execution_id(session_id: &str) -> String {
    let sequence = TERMINAL_COMMAND_EXECUTION_SEQ.fetch_add(1, Ordering::Relaxed);
    let occurred_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{session_id}:cmd:{occurred_unix_ms}:{sequence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_events_append_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-handoff-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append_terminal_command_handoff_event(
            &path,
            "terminal-test",
            "production_terminal_command",
            "execute",
            &TerminalCommandHandoff {
                command_id: "datum.artifact.generate".to_string(),
                mcp_alias: Some("datum.artifact.generate".to_string()),
                command: "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job job-1"
                    .to_string(),
            },
            Some("terminal-test:cmd:1"),
        )
        .expect("append handoff event");
        let line = std::fs::read_to_string(&path).expect("read handoff event log");
        let event: serde_json::Value = serde_json::from_str(line.trim()).expect("parse event");
        assert_eq!(event["event"], "terminal_command_handoff");
        assert_eq!(event["schema_version"], 1);
        assert_eq!(event["session_id"], "terminal-test");
        assert_eq!(event["origin"], "production_terminal_command");
        assert_eq!(event["command_id"], "datum.artifact.generate");
        assert_eq!(event["execution_id"], "terminal-test:cmd:1");
        assert_eq!(event["mcp_alias"], "datum.artifact.generate");
        assert_eq!(event["handoff_mode"], "execute");
        assert!(
            event["command"]
                .as_str()
                .unwrap()
                .contains("datum-eda artifact generate")
        );
        assert!(event["occurred_unix_ms"].as_u64().is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn manual_handoff_events_record_null_mcp_alias() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-manual-handoff-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append_terminal_command_handoff_event(
            &path,
            "terminal-test",
            "board_text_terminal_command",
            "prefill",
            &TerminalCommandHandoff {
                command_id: "datum.gui.board_text.edit_prefill".to_string(),
                mcp_alias: None,
                command: "datum-eda project edit-board-text \"$DATUM_PROJECT_ROOT\" --text text-1"
                    .to_string(),
            },
            None,
        )
        .expect("append manual handoff event");
        let line = std::fs::read_to_string(&path).expect("read manual handoff event log");
        let event: serde_json::Value = serde_json::from_str(line.trim()).expect("parse event");
        assert_eq!(event["origin"], "board_text_terminal_command");
        assert_eq!(event["command_id"], "datum.gui.board_text.edit_prefill");
        assert_eq!(event["mcp_alias"], serde_json::Value::Null);
        assert_eq!(event["handoff_mode"], "prefill");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lifecycle_events_append_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-lifecycle-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append_terminal_lifecycle_event(
            &path,
            "terminal-test",
            DatumToolSessionLifecycle::Exited,
            Some(0),
        )
        .expect("append lifecycle event");
        let line = std::fs::read_to_string(&path).expect("read lifecycle event log");
        let event: serde_json::Value = serde_json::from_str(line.trim()).expect("parse event");
        assert_eq!(event["event"], "terminal_lifecycle");
        assert_eq!(event["schema_version"], 1);
        assert_eq!(event["session_id"], "terminal-test");
        assert_eq!(event["lifecycle"], "exited");
        assert_eq!(event["process_exit_code"], 0);
        assert!(event["occurred_unix_ms"].as_u64().is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn command_lifecycle_events_append_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-command-lifecycle-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append_terminal_command_lifecycle_event(
            &path,
            "terminal-test",
            "production_terminal_command",
            &TerminalCommandHandoff {
                command_id: "datum.check.run".to_string(),
                mcp_alias: Some("datum.check.run".to_string()),
                command: "datum-eda check run \"$DATUM_PROJECT_ROOT\"".to_string(),
            },
            Some("terminal-test:cmd:2"),
            "started",
            None,
        )
        .expect("append command lifecycle event");
        let line = std::fs::read_to_string(&path).expect("read command lifecycle event log");
        let event: serde_json::Value = serde_json::from_str(line.trim()).expect("parse event");
        assert_eq!(event["event"], "terminal_command_lifecycle");
        assert_eq!(event["schema_version"], 1);
        assert_eq!(event["session_id"], "terminal-test");
        assert_eq!(event["origin"], "production_terminal_command");
        assert_eq!(event["command_id"], "datum.check.run");
        assert_eq!(event["execution_id"], "terminal-test:cmd:2");
        assert_eq!(event["lifecycle"], "started");
        assert_eq!(event["process_exit_code"], serde_json::Value::Null);
        assert!(
            event["command"]
                .as_str()
                .unwrap()
                .contains("datum-eda check run")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn terminal_command_lifecycle_wrapper_preserves_command_and_records_finish() {
        let wrapper = terminal_command_lifecycle_shell_wrapper(
            "production_terminal_command",
            &TerminalCommandHandoff {
                command_id: "datum.check.run's".to_string(),
                mcp_alias: None,
                command: "datum-eda check run \"$DATUM_PROJECT_ROOT\"".to_string(),
            },
            "terminal-test:cmd:quoted",
        );
        assert!(wrapper.contains("datum-eda check run \"$DATUM_PROJECT_ROOT\""));
        assert!(wrapper.contains("DATUM_TOOL_SESSION_EVENT_LOG"));
        assert!(wrapper.contains("terminal_command_lifecycle"));
        assert!(wrapper.contains("'datum.check.run'\"'\"'s'"));
        assert!(wrapper.contains("'terminal-test:cmd:quoted'"));
        assert!(wrapper.contains("__datum_exit=$?"));
    }

    #[test]
    fn terminal_command_lifecycle_wrapper_writes_finish_event_from_shell() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-wrapper-finish-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let wrapper = terminal_command_lifecycle_shell_wrapper(
            "production_terminal_command",
            &TerminalCommandHandoff {
                command_id: "datum.check.run".to_string(),
                mcp_alias: None,
                command: "true".to_string(),
            },
            "terminal-test:cmd:shell",
        );
        let status = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(wrapper)
            .env("DATUM_TOOL_SESSION_EVENT_LOG", &path)
            .env("DATUM_SESSION_ID", "terminal-test")
            .status()
            .expect("run lifecycle wrapper shell");
        assert!(status.success());
        let line = std::fs::read_to_string(&path).expect("read wrapper finish event log");
        let event: serde_json::Value = serde_json::from_str(line.trim()).expect("parse event");
        assert_eq!(event["event"], "terminal_command_lifecycle");
        assert_eq!(event["session_id"], "terminal-test");
        assert_eq!(event["origin"], "production_terminal_command");
        assert_eq!(event["command_id"], "datum.check.run");
        assert_eq!(event["execution_id"], "terminal-test:cmd:shell");
        assert_eq!(event["command"], serde_json::Value::Null);
        assert_eq!(event["lifecycle"], "finished");
        assert_eq!(event["process_exit_code"], 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn io_events_append_bounded_jsonl_records() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-io-events-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        append_terminal_io_event(
            &path,
            "terminal-test",
            Some("terminal-test:cmd:io"),
            "input",
            b"ls -al\r",
        )
        .expect("append input event");
        append_terminal_io_event(&path, "terminal-test", None, "output", &vec![b'a'; 600])
            .expect("append output event");
        let lines = std::fs::read_to_string(&path).expect("read I/O event log");
        let events = lines
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("parse event"))
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "terminal_io");
        assert_eq!(events[0]["schema_version"], 1);
        assert_eq!(events[0]["session_id"], "terminal-test");
        assert_eq!(events[0]["execution_id"], "terminal-test:cmd:io");
        assert_eq!(events[0]["direction"], "input");
        assert_eq!(events[0]["byte_count"], 7);
        assert_eq!(events[0]["text_preview"], "ls -al\r");
        assert_eq!(events[0]["truncated"], false);
        assert_eq!(events[1]["direction"], "output");
        assert_eq!(events[1].get("execution_id"), None);
        assert_eq!(events[1]["byte_count"], 600);
        assert_eq!(events[1]["text_preview"].as_str().unwrap().len(), 512);
        assert_eq!(events[1]["truncated"], true);
        let _ = std::fs::remove_file(&path);
    }
}
