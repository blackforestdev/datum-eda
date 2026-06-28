use super::*;
#[path = "command_context_active.rs"]
mod command_context_active;
#[path = "command_context_activity.rs"]
mod command_context_activity;
#[path = "command_context_checks.rs"]
mod command_context_checks;
#[path = "command_context_defaults.rs"]
mod command_context_defaults;
#[path = "command_context_production.rs"]
mod command_context_production;
#[path = "command_context_proposals.rs"]
mod command_context_proposals;
#[path = "command_context_source_shards.rs"]
mod command_context_source_shards;
use command_context_active::update_active_context_commands;
use command_context_activity::{
    command_activity_summaries, command_execution_summaries, count_string_field, occurrence_time,
    terminal_activity_spans, terminal_io_activity_summary,
};
use command_context_checks::{
    check_context_summary, check_status_from_context, visible_check_run_ids_from_context,
    visible_finding_fingerprints_from_context,
};
use command_context_defaults::{insert_command_defaults, insert_context_defaults};
use command_context_production::update_production_visibility;
use command_context_proposals::{latest_proposal_id_from_context, visible_proposal_ids};
use command_context_source_shards::{empty_source_shard_status, update_source_shard_status};
use eda_engine::substrate::ProjectResolver;
use serde_json::{Map, Value};

const GUI_TERMINAL_CONTEXT_PATH: &str = ".datum/gui-terminal-context.json";
const GUI_TERMINAL_CONTEXT_DIR: &str = ".datum/terminal-contexts";

pub(crate) fn query_context_envelope(args: &ContextGetArgs) -> Result<serde_json::Value> {
    let context_path = resolve_context_path(args)?;
    let mut value = read_context_envelope(&context_path)?;
    validate_context_session(args, &value)?;
    enrich_context_envelope(&context_path, args, &mut value);
    Ok(value)
}

pub(crate) fn refresh_context_envelope(args: &ContextGetArgs) -> Result<serde_json::Value> {
    let context_path = resolve_context_path(args)?;
    let mut value = read_context_envelope(&context_path)?;
    validate_context_session(args, &value)?;
    enrich_context_envelope(&context_path, args, &mut value);
    let text =
        serde_json::to_string_pretty(&value).context("serialize refreshed context envelope")?;
    let text = format!("{text}\n");
    std::fs::write(&context_path, &text).with_context(|| {
        format!(
            "write refreshed context envelope {}",
            context_path.display()
        )
    })?;
    mirror_latest_context_alias(&context_path, args, &value, &text)?;
    Ok(value)
}

pub(crate) fn query_context_session_events(
    args: &ContextSessionEventsArgs,
) -> Result<serde_json::Value> {
    let context_args = ContextGetArgs {
        session: args.session.clone(),
        path: args.path.clone(),
        project_root: args.project_root.clone(),
    };
    let context_path = resolve_context_path(&context_args)?;
    let mut context = read_context_envelope(&context_path)?;
    validate_context_session(&context_args, &context)?;
    enrich_context_envelope(&context_path, &context_args, &mut context);
    let session_id = context_session_id(&context)
        .ok_or_else(|| anyhow::anyhow!("context session id is required for session events"))?;
    let event_log_path = resolve_session_event_log_path(&context_args, &context, &session_id)?;
    let events = read_session_event_log(&event_log_path)?;
    let total_event_count = events.len();
    let events = filter_session_events(events, args);
    let matched_event_count = events.len();
    let events = limit_session_events(events, args.limit);
    let context_provenance =
        context_provenance(&context, &context_path, &event_log_path, &session_id);
    Ok(serde_json::json!({
        "contract": "datum_tool_session_events_v1",
        "session_id": session_id,
        "context_provenance": context_provenance,
        "context_path": context_path.display().to_string(),
        "event_log_path": event_log_path.display().to_string(),
        "filters": {
            "event_kind": args.event_kind.clone(),
            "origin": args.origin.clone(),
            "command_id": args.command_id.clone(),
            "execution_id": args.execution_id.clone()
        },
        "limit": args.limit,
        "total_event_count": total_event_count,
        "matched_event_count": matched_event_count,
        "event_count": events.len(),
        "events": events
    }))
}

pub(crate) fn query_context_session_activity(
    args: &ContextSessionActivityArgs,
) -> Result<serde_json::Value> {
    let context_args = ContextGetArgs {
        session: args.session.clone(),
        path: args.path.clone(),
        project_root: args.project_root.clone(),
    };
    let context_path = resolve_context_path(&context_args)?;
    let mut context = read_context_envelope(&context_path)?;
    validate_context_session(&context_args, &context)?;
    enrich_context_envelope(&context_path, &context_args, &mut context);
    let session_id = context_session_id(&context)
        .ok_or_else(|| anyhow::anyhow!("context session id is required for session activity"))?;
    let event_log_path = resolve_session_event_log_path(&context_args, &context, &session_id)?;
    let events = read_session_event_log(&event_log_path)?;
    let total_event_count = events.len();
    let events = filter_activity_events(events, args);
    let matched_event_count = events.len();
    let events = limit_session_events(events, args.limit);
    let context_provenance =
        context_provenance(&context, &context_path, &event_log_path, &session_id);
    Ok(serde_json::json!({
        "contract": "datum_tool_session_activity_summary_v1",
        "session_id": session_id,
        "context_provenance": context_provenance,
        "session_lifecycle": context
            .get("session_lifecycle")
            .or_else(|| context.pointer("/session/lifecycle"))
            .cloned()
            .unwrap_or(Value::Null),
        "process_exit_code": context
            .get("process_exit_code")
            .or_else(|| context.pointer("/session/process_exit_code"))
            .cloned()
            .unwrap_or(Value::Null),
        "context_path": context_path.display().to_string(),
        "event_log_path": event_log_path.display().to_string(),
        "filters": {
            "event_kind": args.event_kind.clone(),
            "origin": args.origin.clone(),
            "command_id": args.command_id.clone(),
            "execution_id": args.execution_id.clone()
        },
        "limit": args.limit,
        "total_event_count": total_event_count,
        "matched_event_count": matched_event_count,
        "activity_event_count": events.len(),
        "first_occurred_unix_ms": occurrence_time(events.first()),
        "last_occurred_unix_ms": occurrence_time(events.last()),
        "event_kinds": count_string_field(&events, "event"),
        "origins": count_string_field(&events, "origin"),
        "handoff_modes": count_string_field(&events, "handoff_mode"),
        "terminal_io": terminal_io_activity_summary(&events),
        "activity_spans": terminal_activity_spans(&events, &context_provenance),
        "executions": command_execution_summaries(&events, &context_provenance),
        "commands": command_activity_summaries(&events)
    }))
}

fn context_provenance(
    context: &Value,
    context_path: &Path,
    event_log_path: &Path,
    session_id: &str,
) -> Value {
    serde_json::json!({
        "context_id": context.get("context_id").cloned().unwrap_or(Value::Null),
        "session_id": session_id,
        "provenance_seed": context.get("provenance_seed").cloned().unwrap_or(Value::Null),
        "project_id": context.get("project_id").cloned().unwrap_or(Value::Null),
        "project_root": context.get("project_root").cloned().unwrap_or(Value::Null),
        "model_revision": context.get("model_revision").cloned().unwrap_or(Value::Null),
        "source_revision": context.get("source_revision").cloned().unwrap_or(Value::Null),
        "actor_type": context.get("actor_type").cloned().unwrap_or(Value::Null),
        "context_path": context_path.display().to_string(),
        "event_log_path": event_log_path.display().to_string()
    })
}

fn read_context_envelope(context_path: &Path) -> Result<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&context_path)
            .with_context(|| format!("read context envelope {}", context_path.display()))?,
    )
    .with_context(|| format!("parse context envelope {}", context_path.display()))?;
    Ok(value)
}

fn validate_context_session(args: &ContextGetArgs, value: &serde_json::Value) -> Result<()> {
    if let Some(expected_session) = &args.session {
        let session = value
            .get("session_id")
            .or_else(|| value.get("terminal_session_id"))
            .or_else(|| value.pointer("/session/session_id"))
            .and_then(|value| value.as_str());
        if session != Some(expected_session.as_str()) {
            bail!(
                "context session mismatch: expected {}, found {}",
                expected_session,
                session.unwrap_or("<missing>")
            );
        }
    }
    Ok(())
}

fn context_session_id(value: &serde_json::Value) -> Option<String> {
    value
        .get("session_id")
        .or_else(|| value.get("terminal_session_id"))
        .or_else(|| value.pointer("/session/session_id"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn resolve_context_path(args: &ContextGetArgs) -> Result<PathBuf> {
    if let Some(path) = &args.path {
        return Ok(path.clone());
    }
    if let Ok(path) = std::env::var("DATUM_DISCOVERY") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    if let Ok(path) = std::env::var("DATUM_TERMINAL_CONTEXT") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    if let Some(root) = &args.project_root {
        if let Some(session) = &args.session {
            let session_path = root
                .join(GUI_TERMINAL_CONTEXT_DIR)
                .join(format!("{session}.json"));
            if session_path.exists() {
                return Ok(session_path);
            }
        }
        return Ok(root.join(GUI_TERMINAL_CONTEXT_PATH));
    }
    bail!("context path required: pass --path, --project-root, or set DATUM_DISCOVERY")
}

fn enrich_context_envelope(context_path: &Path, args: &ContextGetArgs, value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    insert_context_defaults(object, empty_source_shard_status());
    insert_command_defaults(object);
    let project_root = context_project_root(args, object);
    if let Some(root) = &project_root {
        object.insert(
            "project_root".to_string(),
            Value::String(root.display().to_string()),
        );
        if let Ok(model) = ProjectResolver::new(&root).resolve() {
            object.insert(
                "project_id".to_string(),
                Value::String(model.project.project_id.to_string()),
            );
            object.insert(
                "project_name".to_string(),
                Value::String(model.project.name.clone()),
            );
            object.insert(
                "model_revision".to_string(),
                Value::String(model.model_revision.0.clone()),
            );
            object.insert(
                "accepted_transaction_tip".to_string(),
                model
                    .journal_cursor
                    .applied_transaction_count
                    .checked_sub(1)
                    .and_then(|index| model.journal.get(index))
                    .map(|transaction| Value::String(transaction.transaction_id.to_string()))
                    .unwrap_or(Value::Null),
            );
            update_production_visibility(object, &model);
            object.insert(
                "visible_proposal_ids".to_string(),
                visible_proposal_ids(&model),
            );
            object.insert(
                "latest_proposal_id".to_string(),
                latest_proposal_id_from_context(object, &model)
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            );
            update_source_shard_status(object, &model);
            let check_context = check_context_summary(&model);
            object.insert(
                "visible_check_run_ids".to_string(),
                visible_check_run_ids_from_context(&check_context),
            );
            object.insert(
                "latest_check_run_id".to_string(),
                check_context
                    .get("latest_check_run_id")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            object.insert(
                "latest_profile_id".to_string(),
                check_context
                    .get("latest_profile_id")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            object.insert(
                "profile_latest_check_runs".to_string(),
                check_context
                    .get("profile_latest_check_runs")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            );
            object.insert(
                "visible_finding_fingerprints".to_string(),
                visible_finding_fingerprints_from_context(&check_context),
            );
            object.insert(
                "check_status".to_string(),
                check_status_from_context(&check_context, &model),
            );
            object.insert("check_context".to_string(), check_context);
            object.insert(
                "output_context".to_string(),
                serde_json::json!({
                    "output_job_ids": model.output_jobs.keys().map(ToString::to_string).collect::<Vec<_>>(),
                    "manufacturing_plan_ids": model.manufacturing_plans.keys().map(ToString::to_string).collect::<Vec<_>>(),
                    "panel_projection_ids": model.panel_projections.keys().map(ToString::to_string).collect::<Vec<_>>()
                }),
            );
        }
    }
    update_storage(object, context_path, project_root.as_deref());
    update_session_metadata(object);
    update_active_context_commands(object);
    let seed = format!(
        "datum-context:{}:{}:{}",
        object
            .get("session_id")
            .or_else(|| object.get("terminal_session_id"))
            .and_then(Value::as_str)
            .unwrap_or("unknown-session"),
        object
            .get("context_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown-context"),
        object
            .get("model_revision")
            .and_then(Value::as_str)
            .unwrap_or("unknown-revision")
    );
    object.insert("provenance_seed".to_string(), Value::String(seed));
}

fn update_session_metadata(object: &mut Map<String, Value>) {
    let session_id = object
        .get("session_id")
        .or_else(|| object.get("terminal_session_id"))
        .and_then(Value::as_str)
        .unwrap_or("unknown-session")
        .to_string();
    let context_id = object
        .get("context_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown-context")
        .to_string();
    let mut session = object
        .get("session")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    session
        .entry("session_id".to_string())
        .or_insert(Value::String(session_id));
    session
        .entry("context_id".to_string())
        .or_insert(Value::String(context_id));
    session
        .entry("actor_type".to_string())
        .or_insert(Value::String("ExternalAgent".to_string()));
    session
        .entry("capabilities".to_string())
        .or_insert_with(|| {
            Value::Array(
                ["read", "check", "artifact", "propose", "apply-approved"]
                    .into_iter()
                    .map(|capability| Value::String(capability.to_string()))
                    .collect(),
            )
        });
    for key in [
        "lifecycle",
        "created_unix_ms",
        "updated_unix_ms",
        "expires_unix_ms",
        "process_group_id",
        "process_exit_code",
    ] {
        let value = object
            .get(match key {
                "lifecycle" => "session_lifecycle",
                "expires_unix_ms" => "expires_unix_ms",
                other => other,
            })
            .cloned()
            .unwrap_or(Value::Null);
        session.entry(key.to_string()).or_insert(value);
    }
    object.insert("session".to_string(), Value::Object(session));
}

fn context_project_root(args: &ContextGetArgs, object: &Map<String, Value>) -> Option<PathBuf> {
    args.project_root.clone().or_else(|| {
        object
            .get("project_root")
            .and_then(Value::as_str)
            .map(PathBuf::from)
    })
}

fn update_storage(object: &mut Map<String, Value>, context_path: &Path, root: Option<&Path>) {
    let mut storage = object
        .get("storage")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    storage.insert(
        "context_path".to_string(),
        Value::String(context_path.display().to_string()),
    );
    storage.insert(
        "schema_version".to_string(),
        Value::Number(serde_json::Number::from(1)),
    );
    if let Some(root) = root {
        let latest_context_path = root.join(GUI_TERMINAL_CONTEXT_PATH).display().to_string();
        storage.insert(
            "latest_context_path".to_string(),
            Value::String(latest_context_path.clone()),
        );
        storage.insert(
            "compatibility_context_path".to_string(),
            Value::String(latest_context_path.clone()),
        );
        storage.insert(
            "legacy_context_path".to_string(),
            Value::String(latest_context_path),
        );
        if let Some(session_id) = object
            .get("session_id")
            .or_else(|| object.get("terminal_session_id"))
            .or_else(|| {
                object
                    .get("session")
                    .and_then(Value::as_object)
                    .and_then(|session| session.get("session_id"))
            })
            .and_then(Value::as_str)
        {
            storage
                .entry("event_log_path".to_string())
                .or_insert_with(|| {
                    Value::String(
                        root.join(".datum/tool-sessions")
                            .join(format!("{session_id}.events.jsonl"))
                            .display()
                            .to_string(),
                    )
                });
        }
    }
    object.insert("storage".to_string(), Value::Object(storage));
}

fn mirror_latest_context_alias(
    context_path: &Path,
    args: &ContextGetArgs,
    value: &Value,
    text: &str,
) -> Result<()> {
    let Some(path) = latest_context_alias_path(context_path, args, value) else {
        return Ok(());
    };
    if path == context_path {
        return Ok(());
    }
    std::fs::write(&path, text)
        .with_context(|| format!("write refreshed latest context alias {}", path.display()))
}

fn latest_context_alias_path(
    context_path: &Path,
    args: &ContextGetArgs,
    value: &Value,
) -> Option<PathBuf> {
    for pointer in [
        "/storage/latest_context_path",
        "/storage/compatibility_context_path",
        "/storage/legacy_context_path",
    ] {
        if let Some(path) = value
            .pointer(pointer)
            .and_then(Value::as_str)
            .filter(|path| !path.is_empty())
        {
            return Some(PathBuf::from(path));
        }
    }
    args.project_root
        .clone()
        .or_else(|| {
            value
                .get("project_root")
                .and_then(Value::as_str)
                .map(PathBuf::from)
        })
        .map(|root| root.join(GUI_TERMINAL_CONTEXT_PATH))
        .filter(|path| path != context_path)
}

fn resolve_session_event_log_path(
    args: &ContextGetArgs,
    context: &serde_json::Value,
    session_id: &str,
) -> Result<PathBuf> {
    if let Some(path) = context
        .pointer("/storage/event_log_path")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
    {
        return Ok(PathBuf::from(path));
    }
    let root = args
        .project_root
        .clone()
        .or_else(|| {
            context
                .get("project_root")
                .and_then(Value::as_str)
                .map(PathBuf::from)
        })
        .ok_or_else(|| anyhow::anyhow!("project root required to resolve session event log"))?;
    Ok(root
        .join(".datum/tool-sessions")
        .join(format!("{session_id}.events.jsonl")))
}

fn read_session_event_log(path: &Path) -> Result<Vec<serde_json::Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read tool-session event log {}", path.display()))?;
    let mut events = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let event: serde_json::Value = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "parse tool-session event log {} line {}",
                path.display(),
                index + 1
            )
        })?;
        events.push(event);
    }
    Ok(events)
}

fn filter_session_events(
    events: Vec<serde_json::Value>,
    args: &ContextSessionEventsArgs,
) -> Vec<serde_json::Value> {
    events
        .into_iter()
        .filter(|event| session_event_matches(event, "event", args.event_kind.as_deref()))
        .filter(|event| session_event_matches(event, "origin", args.origin.as_deref()))
        .filter(|event| session_event_matches(event, "command_id", args.command_id.as_deref()))
        .filter(|event| session_event_matches(event, "execution_id", args.execution_id.as_deref()))
        .collect()
}

fn filter_activity_events(
    events: Vec<serde_json::Value>,
    args: &ContextSessionActivityArgs,
) -> Vec<serde_json::Value> {
    events
        .into_iter()
        .filter(|event| session_event_matches(event, "event", args.event_kind.as_deref()))
        .filter(|event| session_event_matches(event, "origin", args.origin.as_deref()))
        .filter(|event| session_event_matches(event, "command_id", args.command_id.as_deref()))
        .filter(|event| session_event_matches(event, "execution_id", args.execution_id.as_deref()))
        .collect()
}

fn session_event_matches(event: &serde_json::Value, field: &str, filter: Option<&str>) -> bool {
    match filter {
        Some(expected) => event.get(field).and_then(Value::as_str) == Some(expected),
        None => true,
    }
}

fn limit_session_events(
    mut events: Vec<serde_json::Value>,
    limit: Option<usize>,
) -> Vec<serde_json::Value> {
    let Some(limit) = limit else {
        return events;
    };
    if events.len() <= limit {
        return events;
    }
    events.split_off(events.len() - limit)
}
