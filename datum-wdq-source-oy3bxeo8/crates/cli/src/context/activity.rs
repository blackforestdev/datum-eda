use serde_json::Value;

pub(super) fn count_string_field(events: &[serde_json::Value], field: &str) -> serde_json::Value {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for event in events {
        if let Some(value) = event.get(field).and_then(Value::as_str) {
            *counts.entry(value.to_string()).or_insert(0) += 1;
        }
    }
    serde_json::to_value(counts).unwrap_or_else(|_| serde_json::json!({}))
}

pub(super) fn occurrence_time(event: Option<&serde_json::Value>) -> serde_json::Value {
    event
        .and_then(|event| event.get("occurred_unix_ms"))
        .cloned()
        .unwrap_or(Value::Null)
}

pub(super) fn command_activity_summaries(events: &[serde_json::Value]) -> serde_json::Value {
    let mut commands =
        std::collections::BTreeMap::<String, (usize, Value, Value, Value, Value, Value)>::new();
    for event in events {
        if event.get("event").and_then(Value::as_str) != Some("terminal_command_handoff") {
            continue;
        }
        let Some(command_id) = event.get("command_id").and_then(Value::as_str) else {
            continue;
        };
        let entry = commands.entry(command_id.to_string()).or_insert((
            0,
            event.get("mcp_alias").cloned().unwrap_or(Value::Null),
            event.get("origin").cloned().unwrap_or(Value::Null),
            event.get("handoff_mode").cloned().unwrap_or(Value::Null),
            Value::Null,
            Value::Null,
        ));
        entry.0 += 1;
        entry.4 = occurrence_time(Some(event));
        entry.5 = event.get("execution_id").cloned().unwrap_or(Value::Null);
    }
    Value::Array(
        commands
            .into_iter()
            .map(
                |(
                    command_id,
                    (
                        count,
                        mcp_alias,
                        origin,
                        handoff_mode,
                        last_occurred_unix_ms,
                        last_execution_id,
                    ),
                )| {
                    serde_json::json!({
                        "command_id": command_id,
                        "mcp_alias": mcp_alias,
                        "origin": origin,
                        "handoff_mode": handoff_mode,
                        "count": count,
                        "last_execution_id": last_execution_id,
                        "last_occurred_unix_ms": last_occurred_unix_ms
                    })
                },
            )
            .collect(),
    )
}

pub(super) fn command_execution_summaries(
    events: &[serde_json::Value],
    context_provenance: &Value,
) -> serde_json::Value {
    let mut executions = std::collections::BTreeMap::<String, CommandExecutionSummary>::new();
    let mut execution_order = Vec::<String>::new();
    for event in events {
        let Some(execution_id) = event.get("execution_id").and_then(Value::as_str) else {
            continue;
        };
        if !executions.contains_key(execution_id) {
            execution_order.push(execution_id.to_string());
        }
        executions
            .entry(execution_id.to_string())
            .or_insert_with(|| {
                CommandExecutionSummary::new(execution_id, event, context_provenance)
            })
            .add_event(event);
    }
    Value::Array(
        execution_order
            .into_iter()
            .filter_map(|execution_id| executions.remove(&execution_id))
            .map(CommandExecutionSummary::to_json)
            .collect(),
    )
}

#[derive(Debug)]
struct CommandExecutionSummary {
    execution_id: String,
    command_id: Value,
    origin: Value,
    command: Value,
    event_count: usize,
    event_kinds: std::collections::BTreeMap<String, usize>,
    start_occurred_unix_ms: Value,
    end_occurred_unix_ms: Value,
    lifecycle: Value,
    process_exit_code: Value,
    input_event_count: usize,
    output_event_count: usize,
    input_byte_count: u64,
    output_byte_count: u64,
    last_input_preview: Value,
    last_output_preview: Value,
    context_provenance: Value,
}

impl CommandExecutionSummary {
    fn new(execution_id: &str, event: &serde_json::Value, context_provenance: &Value) -> Self {
        let occurred = occurrence_time(Some(event));
        Self {
            execution_id: execution_id.to_string(),
            command_id: event.get("command_id").cloned().unwrap_or(Value::Null),
            origin: event.get("origin").cloned().unwrap_or(Value::Null),
            command: event.get("command").cloned().unwrap_or(Value::Null),
            event_count: 0,
            event_kinds: std::collections::BTreeMap::new(),
            start_occurred_unix_ms: occurred.clone(),
            end_occurred_unix_ms: occurred,
            lifecycle: Value::Null,
            process_exit_code: Value::Null,
            input_event_count: 0,
            output_event_count: 0,
            input_byte_count: 0,
            output_byte_count: 0,
            last_input_preview: Value::Null,
            last_output_preview: Value::Null,
            context_provenance: context_provenance.clone(),
        }
    }

    fn add_event(&mut self, event: &serde_json::Value) {
        self.event_count += 1;
        self.end_occurred_unix_ms = occurrence_time(Some(event));
        if let Some(kind) = event.get("event").and_then(Value::as_str) {
            *self.event_kinds.entry(kind.to_string()).or_insert(0) += 1;
        }
        if self.command_id.is_null() {
            self.command_id = event.get("command_id").cloned().unwrap_or(Value::Null);
        }
        if self.origin.is_null() {
            self.origin = event.get("origin").cloned().unwrap_or(Value::Null);
        }
        if self.command.is_null() {
            self.command = event.get("command").cloned().unwrap_or(Value::Null);
        }
        match event.get("event").and_then(Value::as_str) {
            Some("terminal_command_lifecycle") => {
                self.lifecycle = event.get("lifecycle").cloned().unwrap_or(Value::Null);
                self.process_exit_code = event
                    .get("process_exit_code")
                    .cloned()
                    .unwrap_or(Value::Null);
            }
            Some("terminal_io") => self.add_terminal_io(event),
            _ => {}
        }
    }

    fn add_terminal_io(&mut self, event: &serde_json::Value) {
        let byte_count = event.get("byte_count").and_then(Value::as_u64).unwrap_or(0);
        let preview = event.get("text_preview").cloned().unwrap_or(Value::Null);
        match event.get("direction").and_then(Value::as_str) {
            Some("input") => {
                self.input_event_count += 1;
                self.input_byte_count += byte_count;
                self.last_input_preview = preview;
            }
            Some("output") => {
                self.output_event_count += 1;
                self.output_byte_count += byte_count;
                self.last_output_preview = preview;
            }
            _ => {}
        }
    }

    fn duration_ms(&self) -> Value {
        match (
            self.start_occurred_unix_ms.as_u64(),
            self.end_occurred_unix_ms.as_u64(),
        ) {
            (Some(start), Some(end)) => serde_json::json!(end.saturating_sub(start)),
            _ => Value::Null,
        }
    }

    fn terminal_io_json(&self) -> Value {
        serde_json::json!({
            "input_event_count": self.input_event_count,
            "output_event_count": self.output_event_count,
            "input_byte_count": self.input_byte_count,
            "output_byte_count": self.output_byte_count,
            "last_input_preview": self.last_input_preview,
            "last_output_preview": self.last_output_preview
        })
    }

    // Intentionally consumes self to move owned fields into the JSON value without cloning.
    #[allow(clippy::wrong_self_convention)]
    fn to_json(self) -> Value {
        serde_json::json!({
            "execution_id": self.execution_id,
            "command_id": self.command_id,
            "origin": self.origin,
            "command": self.command,
            "event_count": self.event_count,
            "event_kinds": self.event_kinds,
            "start_occurred_unix_ms": self.start_occurred_unix_ms,
            "end_occurred_unix_ms": self.end_occurred_unix_ms,
            "duration_ms": self.duration_ms(),
            "lifecycle": self.lifecycle,
            "process_exit_code": self.process_exit_code,
            "terminal_io": self.terminal_io_json(),
            "context_provenance": self.context_provenance
        })
    }
}

pub(super) fn terminal_io_activity_summary(events: &[serde_json::Value]) -> serde_json::Value {
    let mut input_event_count = 0_usize;
    let mut output_event_count = 0_usize;
    let mut input_byte_count = 0_u64;
    let mut output_byte_count = 0_u64;
    let mut last_input_preview = Value::Null;
    let mut last_output_preview = Value::Null;
    for event in events {
        if event.get("event").and_then(Value::as_str) != Some("terminal_io") {
            continue;
        }
        let byte_count = event.get("byte_count").and_then(Value::as_u64).unwrap_or(0);
        let preview = event.get("text_preview").cloned().unwrap_or(Value::Null);
        match event.get("direction").and_then(Value::as_str) {
            Some("input") => {
                input_event_count += 1;
                input_byte_count += byte_count;
                last_input_preview = preview;
            }
            Some("output") => {
                output_event_count += 1;
                output_byte_count += byte_count;
                last_output_preview = preview;
            }
            _ => {}
        }
    }
    serde_json::json!({
        "input_event_count": input_event_count,
        "output_event_count": output_event_count,
        "input_byte_count": input_byte_count,
        "output_byte_count": output_byte_count,
        "last_input_preview": last_input_preview,
        "last_output_preview": last_output_preview
    })
}

#[derive(Debug, Default)]
struct TerminalActivitySpan {
    span_index: usize,
    span_kind: &'static str,
    session_id: Value,
    start_occurred_unix_ms: Value,
    end_occurred_unix_ms: Value,
    event_count: usize,
    event_kinds: std::collections::BTreeMap<String, usize>,
    execution_id: Value,
    handoff: Value,
    input_event_count: usize,
    output_event_count: usize,
    input_byte_count: u64,
    output_byte_count: u64,
    last_input_preview: Value,
    last_output_preview: Value,
    lifecycle: Value,
    command_lifecycle: Value,
    end_reason: &'static str,
    context_provenance: Value,
}

impl TerminalActivitySpan {
    fn new(
        span_index: usize,
        span_kind: &'static str,
        event: &serde_json::Value,
        context_provenance: &Value,
    ) -> Self {
        let occurred = occurrence_time(Some(event));
        Self {
            span_index,
            span_kind,
            session_id: event.get("session_id").cloned().unwrap_or(Value::Null),
            start_occurred_unix_ms: occurred.clone(),
            end_occurred_unix_ms: occurred,
            execution_id: event.get("execution_id").cloned().unwrap_or(Value::Null),
            handoff: Value::Null,
            last_input_preview: Value::Null,
            last_output_preview: Value::Null,
            lifecycle: Value::Null,
            command_lifecycle: Value::Null,
            end_reason: "end_of_window",
            context_provenance: context_provenance.clone(),
            ..Self::default()
        }
    }

    fn add_event_kind(&mut self, event: &serde_json::Value) {
        if let Some(kind) = event.get("event").and_then(Value::as_str) {
            *self.event_kinds.entry(kind.to_string()).or_insert(0) += 1;
        }
    }

    fn add_handoff(&mut self, event: &serde_json::Value) {
        self.event_count += 1;
        self.end_occurred_unix_ms = occurrence_time(Some(event));
        self.add_event_kind(event);
        self.handoff = serde_json::json!({
            "origin": event.get("origin").cloned().unwrap_or(Value::Null),
            "command_id": event.get("command_id").cloned().unwrap_or(Value::Null),
            "execution_id": event.get("execution_id").cloned().unwrap_or(Value::Null),
            "mcp_alias": event.get("mcp_alias").cloned().unwrap_or(Value::Null),
            "handoff_mode": event.get("handoff_mode").cloned().unwrap_or(Value::Null),
            "command": event.get("command").cloned().unwrap_or(Value::Null),
            "context_provenance": self.context_provenance.clone()
        });
    }

    fn add_terminal_io(&mut self, event: &serde_json::Value) {
        self.event_count += 1;
        self.end_occurred_unix_ms = occurrence_time(Some(event));
        self.add_event_kind(event);
        let byte_count = event.get("byte_count").and_then(Value::as_u64).unwrap_or(0);
        let preview = event.get("text_preview").cloned().unwrap_or(Value::Null);
        match event.get("direction").and_then(Value::as_str) {
            Some("input") => {
                self.input_event_count += 1;
                self.input_byte_count += byte_count;
                self.last_input_preview = preview;
            }
            Some("output") => {
                self.output_event_count += 1;
                self.output_byte_count += byte_count;
                self.last_output_preview = preview;
            }
            _ => {}
        }
    }

    fn add_lifecycle(&mut self, event: &serde_json::Value) {
        self.event_count += 1;
        self.end_occurred_unix_ms = occurrence_time(Some(event));
        self.add_event_kind(event);
        self.lifecycle = serde_json::json!({
            "lifecycle": event.get("lifecycle").cloned().unwrap_or(Value::Null),
            "process_exit_code": event.get("process_exit_code").cloned().unwrap_or(Value::Null)
        });
        self.end_reason = "lifecycle";
    }

    fn add_command_lifecycle(&mut self, event: &serde_json::Value) {
        self.event_count += 1;
        self.end_occurred_unix_ms = occurrence_time(Some(event));
        self.add_event_kind(event);
        if self.execution_id.is_null() {
            self.execution_id = event.get("execution_id").cloned().unwrap_or(Value::Null);
        }
        self.command_lifecycle = serde_json::json!({
            "origin": event.get("origin").cloned().unwrap_or(Value::Null),
            "command_id": event.get("command_id").cloned().unwrap_or(Value::Null),
            "execution_id": event.get("execution_id").cloned().unwrap_or(Value::Null),
            "command": event.get("command").cloned().unwrap_or(Value::Null),
            "lifecycle": event.get("lifecycle").cloned().unwrap_or(Value::Null),
            "process_exit_code": event.get("process_exit_code").cloned().unwrap_or(Value::Null)
        });
        if event.get("lifecycle").and_then(Value::as_str) == Some("finished") {
            self.end_reason = "command_finished";
        }
    }

    fn terminal_io_json(&self) -> serde_json::Value {
        serde_json::json!({
            "input_event_count": self.input_event_count,
            "output_event_count": self.output_event_count,
            "input_byte_count": self.input_byte_count,
            "output_byte_count": self.output_byte_count,
            "last_input_preview": self.last_input_preview,
            "last_output_preview": self.last_output_preview
        })
    }

    // Intentionally consumes self to move owned fields into the JSON value without cloning.
    #[allow(clippy::wrong_self_convention)]
    fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "span_id": format!("span-{index:06}", index = self.span_index),
            "span_kind": self.span_kind,
            "session_id": self.session_id,
            "start_occurred_unix_ms": self.start_occurred_unix_ms,
            "end_occurred_unix_ms": self.end_occurred_unix_ms,
            "event_count": self.event_count,
            "event_kinds": self.event_kinds,
            "execution_id": self.execution_id,
            "handoff": self.handoff,
            "terminal_io": self.terminal_io_json(),
            "lifecycle": self.lifecycle,
            "command_lifecycle": self.command_lifecycle,
            "end_reason": self.end_reason
        })
    }
}

pub(super) fn terminal_activity_spans(
    events: &[serde_json::Value],
    context_provenance: &Value,
) -> serde_json::Value {
    let mut spans = Vec::new();
    let mut current: Option<TerminalActivitySpan> = None;
    let mut next_span_index = 1_usize;

    for event in events {
        match event.get("event").and_then(Value::as_str) {
            Some("terminal_command_handoff") => {
                if let Some(mut span) = current.take() {
                    span.end_reason = "next_handoff";
                    spans.push(span.to_json());
                }
                let mut span = TerminalActivitySpan::new(
                    next_span_index,
                    "command",
                    event,
                    context_provenance,
                );
                next_span_index += 1;
                span.add_handoff(event);
                current = Some(span);
            }
            Some("terminal_io")
                if event.get("direction").and_then(Value::as_str) == Some("input") =>
            {
                if current.as_ref().is_some_and(|span| {
                    span.span_kind == "terminal_io" && span.input_event_count > 0
                }) && let Some(mut span) = current.take()
                {
                    span.end_reason = "next_input";
                    spans.push(span.to_json());
                }
                if current.is_none() {
                    let span = TerminalActivitySpan::new(
                        next_span_index,
                        "terminal_io",
                        event,
                        context_provenance,
                    );
                    next_span_index += 1;
                    current = Some(span);
                }
                if let Some(span) = &mut current {
                    span.add_terminal_io(event);
                }
            }
            Some("terminal_io") => {
                let span = current.get_or_insert_with(|| {
                    let span = TerminalActivitySpan::new(
                        next_span_index,
                        "terminal_io",
                        event,
                        context_provenance,
                    );
                    next_span_index += 1;
                    span
                });
                span.add_terminal_io(event);
            }
            Some("terminal_command_lifecycle") => {
                let span = current.get_or_insert_with(|| {
                    let span = TerminalActivitySpan::new(
                        next_span_index,
                        "command",
                        event,
                        context_provenance,
                    );
                    next_span_index += 1;
                    span
                });
                span.add_command_lifecycle(event);
                if span.end_reason == "command_finished"
                    && let Some(span) = current.take()
                {
                    spans.push(span.to_json());
                }
            }
            Some("terminal_lifecycle") => {
                let span = current.get_or_insert_with(|| {
                    let span = TerminalActivitySpan::new(
                        next_span_index,
                        "lifecycle",
                        event,
                        context_provenance,
                    );
                    next_span_index += 1;
                    span
                });
                span.add_lifecycle(event);
                if let Some(span) = current.take() {
                    spans.push(span.to_json());
                }
            }
            _ => {}
        }
    }

    if let Some(span) = current {
        spans.push(span.to_json());
    }
    Value::Array(spans)
}
