use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Default)]
struct ActivitySpan {
    kind: &'static str,
    command_id: Option<String>,
    execution_id: Option<String>,
    origin: Option<String>,
    action_label: Option<String>,
    input_bytes: u64,
    output_bytes: u64,
    last_input_preview: Option<String>,
    last_output_preview: Option<String>,
    lifecycle: Option<String>,
    command_lifecycle: Option<String>,
    process_exit_code: Option<i64>,
    end_reason: &'static str,
}

impl ActivitySpan {
    fn new(kind: &'static str) -> Self {
        Self {
            kind,
            end_reason: "end_of_window",
            ..Self::default()
        }
    }
}

pub(super) fn load_terminal_activity_summary_lines(
    event_log_path: &Path,
    max_spans: usize,
) -> Result<Vec<String>> {
    let events = read_event_log(event_log_path)?;
    let spans = build_activity_spans(&events);
    Ok(format_activity_spans(&spans, max_spans))
}

fn read_event_log(path: &Path) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read terminal activity log {}", path.display()))?;
    let mut events = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        events.push(serde_json::from_str::<Value>(trimmed).with_context(|| {
            format!(
                "parse terminal activity log {} line {}",
                path.display(),
                index + 1
            )
        })?);
    }
    Ok(events)
}

fn build_activity_spans(events: &[Value]) -> Vec<ActivitySpan> {
    let mut spans = Vec::new();
    let mut current: Option<ActivitySpan> = None;
    for event in events {
        match event.get("event").and_then(Value::as_str) {
            Some("terminal_command_handoff") => {
                if let Some(mut span) = current.take() {
                    span.end_reason = "next_handoff";
                    spans.push(span);
                }
                current = Some(ActivitySpan {
                    kind: classify_command_kind(event),
                    end_reason: "end_of_window",
                    command_id: event
                        .get("command_id")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    execution_id: event
                        .get("execution_id")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    origin: event
                        .get("origin")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    action_label: classify_command_action(event),
                    ..ActivitySpan::default()
                });
            }
            Some("terminal_io")
                if event.get("direction").and_then(Value::as_str) == Some("input") =>
            {
                if current
                    .as_ref()
                    .is_some_and(|span| span.kind == "terminal_io" && span.input_bytes > 0)
                    && let Some(mut span) = current.take() {
                        span.end_reason = "next_input";
                        spans.push(span);
                    }
                if current.is_none() {
                    current = Some(ActivitySpan::new("terminal_io"));
                }
                if let Some(span) = &mut current {
                    add_terminal_io(span, event);
                }
            }
            Some("terminal_io") => {
                let span = current.get_or_insert_with(|| ActivitySpan::new("terminal_io"));
                add_terminal_io(span, event);
            }
            Some("terminal_command_lifecycle") => {
                let span = current.get_or_insert_with(|| ActivitySpan::new("command"));
                if span.command_id.is_none() {
                    span.command_id = event
                        .get("command_id")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if span.origin.is_none() {
                    span.origin = event
                        .get("origin")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                if span.execution_id.is_none() {
                    span.execution_id = event
                        .get("execution_id")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                span.command_lifecycle = event
                    .get("lifecycle")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                span.process_exit_code = event.get("process_exit_code").and_then(Value::as_i64);
                if span.command_lifecycle.as_deref() == Some("finished") {
                    span.end_reason = "command_finished";
                    if let Some(span) = current.take() {
                        spans.push(span);
                    }
                }
            }
            Some("terminal_lifecycle") => {
                let span = current.get_or_insert_with(|| ActivitySpan::new("lifecycle"));
                span.lifecycle = event
                    .get("lifecycle")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                span.end_reason = "lifecycle";
                if let Some(span) = current.take() {
                    spans.push(span);
                }
            }
            _ => {}
        }
    }
    if let Some(span) = current {
        spans.push(span);
    }
    spans
}

fn classify_command_kind(event: &Value) -> &'static str {
    match event.get("command_id").and_then(Value::as_str) {
        Some(command_id) if command_id.starts_with("datum.proposal.") => "proposal",
        Some(command_id) if command_id.starts_with("datum.check.") => "check",
        Some(command_id) if command_id.starts_with("datum.artifact.") => "artifact",
        Some(command_id) if command_id.starts_with("datum.journal.") => "journal",
        Some(command_id) if command_id.starts_with("datum.query.") => "query",
        _ => "command",
    }
}

fn classify_command_action(event: &Value) -> Option<String> {
    let command_id = event.get("command_id").and_then(Value::as_str)?;
    command_id
        .strip_prefix("datum.proposal.")
        .or_else(|| command_id.strip_prefix("datum.check."))
        .or_else(|| command_id.strip_prefix("datum.artifact."))
        .or_else(|| command_id.strip_prefix("datum.journal."))
        .or_else(|| command_id.strip_prefix("datum.query."))
        .map(|action| action.replace('_', "-"))
}

fn add_terminal_io(span: &mut ActivitySpan, event: &Value) {
    let byte_count = event.get("byte_count").and_then(Value::as_u64).unwrap_or(0);
    let preview = event
        .get("text_preview")
        .and_then(Value::as_str)
        .map(str::to_string);
    if span.execution_id.is_none() {
        span.execution_id = event
            .get("execution_id")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    match event.get("direction").and_then(Value::as_str) {
        Some("input") => {
            span.input_bytes += byte_count;
            span.last_input_preview = preview;
        }
        Some("output") => {
            span.output_bytes += byte_count;
            span.last_output_preview = preview;
        }
        _ => {}
    }
}

fn format_activity_spans(spans: &[ActivitySpan], max_spans: usize) -> Vec<String> {
    if spans.is_empty() {
        return vec!["no terminal activity spans yet".to_string()];
    }
    let start = spans.len().saturating_sub(max_spans.max(1));
    spans[start..]
        .iter()
        .enumerate()
        .map(|(offset, span)| format_activity_span(start + offset + 1, span))
        .collect()
}

fn format_activity_span(index: usize, span: &ActivitySpan) -> String {
    let subject = span
        .command_id
        .as_deref()
        .or(span.origin.as_deref())
        .unwrap_or(span.kind);
    let mut line = format!(
        "#{index} {kind} {subject} in:{input}B out:{output}B",
        kind = span.kind,
        input = span.input_bytes,
        output = span.output_bytes
    );
    if let Some(lifecycle) = &span.lifecycle {
        line.push_str(&format!(" lifecycle:{lifecycle}"));
    }
    if let Some(lifecycle) = &span.command_lifecycle {
        line.push_str(&format!(" command:{lifecycle}"));
    }
    if let Some(execution_id) = &span.execution_id {
        line.push_str(&format!(" exec:{}", truncate(execution_id, 32)));
    }
    if let Some(exit_code) = span.process_exit_code {
        line.push_str(&format!(" exit:{exit_code}"));
    }
    if let Some(action) = &span.action_label {
        line.push_str(&format!(" action:{action}"));
    }
    if span.end_reason != "end_of_window" {
        line.push_str(&format!(" end:{}", span.end_reason));
    }
    if let Some(output) = &span.last_output_preview {
        let compact = output.replace(['\r', '\n'], " ");
        if !compact.trim().is_empty() {
            line.push_str(&format!(" | {}", truncate(&compact, 48)));
        }
    }
    line
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index == max_chars {
            output.push_str("...");
            break;
        }
        output.push(ch);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_command_span_from_existing_event_log() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-activity-summary-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::fs::write(
            &path,
            r#"{"event":"terminal_command_handoff","command_id":"datum.gui.board_text.edit_prefill","origin":"board_text_terminal_command","occurred_unix_ms":1}
{"event":"terminal_io","direction":"input","byte_count":7,"text_preview":"ls -al\r","occurred_unix_ms":2}
{"event":"terminal_io","direction":"output","byte_count":12,"text_preview":"total 8\n","occurred_unix_ms":3}
{"event":"terminal_lifecycle","lifecycle":"exited","process_exit_code":0,"occurred_unix_ms":4}
"#,
        )
        .expect("write event log");

        let lines = load_terminal_activity_summary_lines(&path, 4).expect("load summary");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("#1 command datum.gui.board_text.edit_prefill"));
        assert!(lines[0].contains("in:7B out:12B"));
        assert!(lines[0].contains("lifecycle:exited"));
        assert!(lines[0].contains("total 8"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn summarizes_orphan_io_without_handoff() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-activity-orphan-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::fs::write(
            &path,
            r#"{"event":"terminal_io","execution_id":"exec-raw","direction":"input","byte_count":4,"text_preview":"pwd\r","occurred_unix_ms":1}
{"event":"terminal_io","execution_id":"exec-raw","direction":"output","byte_count":9,"text_preview":"/tmp\n","occurred_unix_ms":2}
"#,
        )
        .expect("write orphan event log");

        let lines = load_terminal_activity_summary_lines(&path, 4).expect("load summary");
        assert_eq!(
            lines,
            vec!["#1 terminal_io terminal_io in:4B out:9B exec:exec-raw | /tmp ".to_string()]
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn classifies_proposal_handoff_as_proposal_activity() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-activity-proposal-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::fs::write(
            &path,
            r#"{"event":"terminal_command_handoff","command_id":"datum.proposal.preview","origin":"production_terminal_command","occurred_unix_ms":1}
{"event":"terminal_io","direction":"input","byte_count":72,"text_preview":"datum-eda proposal preview /tmp/project --proposal abc\r","occurred_unix_ms":2}
{"event":"terminal_io","direction":"output","byte_count":28,"text_preview":"{\"contract\":\"proposal_preview_v1\"}\n","occurred_unix_ms":3}
"#,
        )
        .expect("write proposal event log");

        let lines = load_terminal_activity_summary_lines(&path, 4).expect("load summary");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("#1 proposal datum.proposal.preview"));
        assert!(lines[0].contains("action:preview"));
        assert!(lines[0].contains("proposal_preview_v1"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn summarizes_command_lifecycle_completion() {
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-activity-command-lifecycle-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::fs::write(
            &path,
            r#"{"event":"terminal_command_handoff","command_id":"datum.check.run","execution_id":"exec-1","origin":"production_terminal_command","occurred_unix_ms":1}
{"event":"terminal_command_lifecycle","command_id":"datum.check.run","execution_id":"exec-1","origin":"production_terminal_command","lifecycle":"started","process_exit_code":null,"occurred_unix_ms":2}
{"event":"terminal_io","direction":"output","byte_count":8,"text_preview":"running\n","occurred_unix_ms":3}
{"event":"terminal_command_lifecycle","command_id":"datum.check.run","execution_id":"exec-1","origin":"production_terminal_command","lifecycle":"finished","process_exit_code":7,"occurred_unix_ms":4}
{"event":"terminal_command_handoff","command_id":"datum.check.run","execution_id":"exec-2","origin":"production_terminal_command","occurred_unix_ms":5}
{"event":"terminal_command_lifecycle","command_id":"datum.check.run","execution_id":"exec-2","origin":"production_terminal_command","lifecycle":"started","process_exit_code":null,"occurred_unix_ms":6}
{"event":"terminal_command_lifecycle","command_id":"datum.check.run","execution_id":"exec-2","origin":"production_terminal_command","lifecycle":"finished","process_exit_code":0,"occurred_unix_ms":7}
"#,
        )
        .expect("write command lifecycle event log");

        let lines = load_terminal_activity_summary_lines(&path, 4).expect("load summary");
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("#1 check datum.check.run"));
        assert!(lines[0].contains("command:finished"));
        assert!(lines[0].contains("exec:exec-1"));
        assert!(lines[0].contains("exit:7"));
        assert!(lines[0].contains("end:command_finished"));
        assert!(lines[0].contains("running"));
        assert!(lines[1].contains("#2 check datum.check.run"));
        assert!(lines[1].contains("exec:exec-2"));
        assert!(lines[1].contains("exit:0"));
        let _ = std::fs::remove_file(&path);
    }
}
