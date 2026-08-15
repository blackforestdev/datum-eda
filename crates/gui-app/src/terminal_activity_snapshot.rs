use anyhow::{Context, Result};
use serde_json::Value;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Completed spans retained by [`TerminalActivitySummaryCache`]. Must be at
/// least the largest `max_spans` any caller formats (4 today) so the cached
/// tail always covers the requested window; older spans are dropped but keep
/// their global numbering through the drop counter.
const CACHE_RETAINED_SPANS: usize = 16;

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

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn load_terminal_activity_summary_lines(
    event_log_path: &Path,
    max_spans: usize,
) -> Result<Vec<String>> {
    let events = read_event_log(event_log_path)?;
    let mut builder = ActivitySpanBuilder::unbounded();
    for event in &events {
        builder.ingest(event);
    }
    Ok(builder.summary_lines(max_spans))
}

/// Incrementally maintained activity summary over ONE session's append-only
/// event log (terminal performance slice, dat-pan-trace-terminal-pollution-0j0):
/// the previous per-refresh full-log reload was O(history) per drained PTY
/// chunk and quadratic over the session lifetime. The cache remembers the byte
/// offset of the last fully consumed line plus the folded span state, so a
/// refresh costs O(new bytes). Summary output is byte-identical to
/// [`load_terminal_activity_summary_lines`] because both fold events through
/// the same [`ActivitySpanBuilder`]. The span data itself is unchanged — it
/// still feeds the future Command Console.
#[derive(Default)]
pub(crate) struct TerminalActivitySummaryCache {
    path: Option<PathBuf>,
    /// Byte offset of the first unconsumed log byte (always a line start).
    offset: u64,
    /// Total physical lines consumed (for one-shot-parity error line numbers).
    total_lines: usize,
    /// Non-empty lines consumed (parity with the tab activity event count).
    nonempty_lines: usize,
    builder: ActivitySpanBuilder,
    /// First malformed-line error; sticky, matching the one-shot loader which
    /// fails on that same line on every reload of an append-only log.
    parse_error: Option<String>,
    /// Last I/O error; cleared by the next successful refresh.
    read_error: Option<String>,
}

impl TerminalActivitySummaryCache {
    /// Ingest any event-log bytes appended since the previous refresh.
    pub(crate) fn refresh(&mut self, path: &Path) {
        if self.path.as_deref() != Some(path) {
            self.reset(Some(path.to_path_buf()));
        }
        self.read_error = None;
        let Ok(mut file) = std::fs::File::open(path) else {
            // Missing log reads as empty, exactly like the one-shot loader.
            self.reset(Some(path.to_path_buf()));
            return;
        };
        let len = file.metadata().map(|meta| meta.len()).unwrap_or(0);
        if len < self.offset {
            // Truncated or replaced in place: rebuild from the start.
            self.reset(Some(path.to_path_buf()));
        }
        if len == self.offset {
            return;
        }
        let mut new_bytes = Vec::new();
        if file.seek(SeekFrom::Start(self.offset)).is_err()
            || file.read_to_end(&mut new_bytes).is_err()
        {
            self.read_error = Some(format!("read terminal activity log {}", path.display()));
            return;
        }
        // Consume complete lines only; a torn trailing line is re-read whole
        // on the next refresh once its writer has finished appending it.
        let Some(consumable) = new_bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map(|position| position + 1)
        else {
            return;
        };
        let text = String::from_utf8_lossy(&new_bytes[..consumable]);
        for line in text.lines() {
            self.total_lines += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            self.nonempty_lines += 1;
            if self.parse_error.is_some() {
                continue;
            }
            match serde_json::from_str::<Value>(trimmed) {
                Ok(event) => self.builder.ingest(&event),
                Err(_) => {
                    self.parse_error = Some(format!(
                        "parse terminal activity log {} line {}",
                        path.display(),
                        self.total_lines
                    ));
                }
            }
        }
        self.offset += consumable as u64;
    }

    /// Formatted summary window, or the same persistent error the one-shot
    /// loader reports for an unreadable or malformed log.
    pub(crate) fn summary_lines(&self, max_spans: usize) -> Result<Vec<String>> {
        if let Some(error) = self.read_error.as_deref().or(self.parse_error.as_deref()) {
            anyhow::bail!("{error}");
        }
        Ok(self.builder.summary_lines(max_spans))
    }

    /// Count of non-empty event-log lines (the tab activity event count).
    pub(crate) fn event_count(&self) -> usize {
        self.nonempty_lines
    }

    fn reset(&mut self, path: Option<PathBuf>) {
        *self = Self {
            path,
            builder: ActivitySpanBuilder::bounded(CACHE_RETAINED_SPANS),
            ..Self::default()
        };
    }
}

/// The one activity-span fold shared by the one-shot loader and the
/// incremental cache. A bounded builder drops the oldest completed spans past
/// its retention limit while preserving global span numbering.
#[derive(Default)]
struct ActivitySpanBuilder {
    completed: Vec<ActivitySpan>,
    dropped: usize,
    retention: Option<usize>,
    current: Option<ActivitySpan>,
}

impl ActivitySpanBuilder {
    #[cfg_attr(not(test), allow(dead_code))]
    fn unbounded() -> Self {
        Self::default()
    }

    fn bounded(retention: usize) -> Self {
        Self {
            retention: Some(retention),
            ..Self::default()
        }
    }

    fn finish_current(&mut self) {
        if let Some(span) = self.current.take() {
            self.completed.push(span);
            if let Some(retention) = self.retention {
                while self.completed.len() > retention {
                    self.completed.remove(0);
                    self.dropped += 1;
                }
            }
        }
    }

    fn summary_lines(&self, max_spans: usize) -> Vec<String> {
        let visible = self.completed.len() + usize::from(self.current.is_some());
        if visible == 0 {
            return vec!["no terminal activity spans yet".to_string()];
        }
        let total = self.dropped + visible;
        let start = total.saturating_sub(max_spans.max(1));
        let skip = start.saturating_sub(self.dropped);
        self.completed
            .iter()
            .chain(self.current.iter())
            .skip(skip)
            .enumerate()
            .map(|(offset, span)| format_activity_span(self.dropped + skip + offset + 1, span))
            .collect()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
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

impl ActivitySpanBuilder {
    fn ingest(&mut self, event: &Value) {
        match event.get("event").and_then(Value::as_str) {
            Some("terminal_command_handoff") => {
                if let Some(span) = &mut self.current {
                    span.end_reason = "next_handoff";
                }
                self.finish_current();
                self.current = Some(ActivitySpan {
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
                if matches!(
                    event.get("direction").and_then(Value::as_str),
                    Some("input" | "input_accepted")
                ) =>
            {
                if self
                    .current
                    .as_ref()
                    .is_some_and(|span| span.kind == "terminal_io" && span.input_bytes > 0)
                {
                    if let Some(span) = &mut self.current {
                        span.end_reason = "next_input";
                    }
                    self.finish_current();
                }
                if self.current.is_none() {
                    self.current = Some(ActivitySpan::new("terminal_io"));
                }
                if let Some(span) = &mut self.current {
                    add_terminal_io(span, event);
                }
            }
            Some("terminal_io") => {
                let span = self
                    .current
                    .get_or_insert_with(|| ActivitySpan::new("terminal_io"));
                add_terminal_io(span, event);
            }
            Some("terminal_command_lifecycle") => {
                let span = self
                    .current
                    .get_or_insert_with(|| ActivitySpan::new("command"));
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
                    self.finish_current();
                }
            }
            Some("terminal_lifecycle") => {
                let span = self
                    .current
                    .get_or_insert_with(|| ActivitySpan::new("lifecycle"));
                span.lifecycle = event
                    .get("lifecycle")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                span.end_reason = "lifecycle";
                self.finish_current();
            }
            _ => {}
        }
    }
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
        Some("input" | "input_accepted") => {
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
    fn incremental_cache_matches_one_shot_loader_at_every_append_step() {
        // Terminal performance slice: the O(new bytes) cache must produce
        // byte-identical summary windows and event counts to the full-log
        // loader after every append, including window overflow past
        // `max_spans` (global `#N` numbering) and torn trailing lines.
        let path = std::env::temp_dir().join(format!(
            "datum-terminal-activity-cache-parity-{}.jsonl",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let mut cache = TerminalActivitySummaryCache::default();
        cache.refresh(&path);
        assert_eq!(
            cache.summary_lines(4).expect("missing log reads as empty"),
            vec!["no terminal activity spans yet".to_string()]
        );
        assert_eq!(cache.event_count(), 0);

        // 20 completed spans crosses the cache retention boundary
        // (CACHE_RETAINED_SPANS = 16), proving dropped spans keep global
        // numbering intact.
        let events: Vec<String> = (1..=20)
            .flat_map(|index| {
                vec![
                    format!(
                        r#"{{"event":"terminal_command_handoff","command_id":"datum.check.run","execution_id":"exec-{index}","origin":"production_terminal_command","occurred_unix_ms":{index}}}"#
                    ),
                    format!(
                        r#"{{"event":"terminal_io","direction":"output","byte_count":8,"text_preview":"run {index}\n","occurred_unix_ms":{index}}}"#
                    ),
                    format!(
                        r#"{{"event":"terminal_command_lifecycle","command_id":"datum.check.run","execution_id":"exec-{index}","origin":"production_terminal_command","lifecycle":"finished","process_exit_code":0,"occurred_unix_ms":{index}}}"#
                    ),
                ]
            })
            .collect();
        let mut appended = String::new();
        for (step, event) in events.iter().enumerate() {
            // Torn write: half the line first — the cache must defer it.
            let (head, tail) = event.split_at(event.len() / 2);
            appended.push_str(head);
            std::fs::write(&path, &appended).expect("write torn event log");
            cache.refresh(&path);
            appended.push_str(tail);
            appended.push('\n');
            std::fs::write(&path, &appended).expect("write event log");
            cache.refresh(&path);
            for max_spans in [1, 2, 4] {
                assert_eq!(
                    cache.summary_lines(max_spans).expect("cache summary"),
                    load_terminal_activity_summary_lines(&path, max_spans)
                        .expect("one-shot summary"),
                    "summary diverged at step {step} (max_spans {max_spans})"
                );
            }
            assert_eq!(cache.event_count(), step + 1, "event count at step {step}");
        }
        // Truncation (session log replaced) resets and re-folds.
        std::fs::write(&path, "").expect("truncate event log");
        cache.refresh(&path);
        assert_eq!(
            cache.summary_lines(4).expect("summary after truncation"),
            vec!["no terminal activity spans yet".to_string()]
        );
        assert_eq!(cache.event_count(), 0);
        let _ = std::fs::remove_file(&path);
    }

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
