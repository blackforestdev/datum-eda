use serde_json::{Map, Value};

pub(super) fn insert_context_defaults(object: &mut Map<String, Value>, source_shard_status: Value) {
    insert_default(
        object,
        "actor_type",
        Value::String("ExternalAgent".to_string()),
    );
    insert_default(
        object,
        "capabilities",
        Value::Array(
            ["read", "check", "artifact", "propose", "apply-approved"]
                .into_iter()
                .map(|capability| Value::String(capability.to_string()))
                .collect(),
        ),
    );
    insert_default(object, "visible_artifact_ids", Value::Array(Vec::new()));
    insert_default(object, "visible_output_job_ids", Value::Array(Vec::new()));
    insert_default(
        object,
        "visible_artifact_file_paths",
        Value::Array(Vec::new()),
    );
    insert_default(object, "latest_output_job_id", Value::Null);
    insert_default(object, "latest_output_job_run_id", Value::Null);
    insert_default(object, "latest_output_job_artifact_id", Value::Null);
    insert_default(object, "latest_artifact_id", Value::Null);
    insert_default(object, "latest_artifact_run_id", Value::Null);
    insert_default(object, "previous_artifact_id", Value::Null);
    insert_default(object, "visible_proposal_ids", Value::Array(Vec::new()));
    insert_default(object, "latest_proposal_id", Value::Null);
    insert_default(object, "focused_artifact_id", Value::Null);
    insert_default(object, "focused_artifact_file_path", Value::Null);
    insert_default(object, "source_shard_status", source_shard_status);
    insert_default(object, "latest_check_run_id", Value::Null);
    insert_default(object, "latest_profile_id", Value::Null);
    insert_default(
        object,
        "profile_latest_check_runs",
        Value::Array(Vec::new()),
    );
    insert_default(object, "visible_check_run_ids", Value::Array(Vec::new()));
    insert_default(object, "accepted_transaction_tip", Value::Null);
    insert_default(object, "expires_at", Value::Null);
    insert_default(
        object,
        "session_lifecycle",
        Value::String("running".to_string()),
    );
    insert_default(object, "created_unix_ms", Value::Null);
    insert_default(object, "updated_unix_ms", Value::Null);
    insert_default(object, "process_group_id", Value::Null);
    insert_default(object, "process_exit_code", Value::Null);
    insert_default(
        object,
        "selection_context",
        serde_json::json!({"kind": "none", "id": null}),
    );
    insert_default(
        object,
        "cursor_context",
        serde_json::json!({
            "screen_px": null,
            "hovered_object_id": null,
            "active_dock_tab": null,
            "active_tool": "select"
        }),
    );
    insert_default(
        object,
        "projection_context",
        serde_json::json!({
            "scene_id": object
                .get("scene_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown-scene"),
            "board_id": object.get("board_id").and_then(Value::as_str),
            "board_name": object.get("board_name").and_then(Value::as_str),
            "scene_bounds_nm": null,
            "active_projection_id": null
        }),
    );
}

pub(super) fn insert_command_defaults(object: &mut Map<String, Value>) {
    insert_default(
        object,
        "refresh_command",
        Value::String("datum-eda context refresh --session \"$DATUM_SESSION_ID\"".to_string()),
    );
    insert_default(
        object,
        "agent_commands",
        serde_json::json!({
            "codex": "codex",
            "claude": "claude",
            "aider": "aider",
            "codex_with_context": "codex 'You are running inside Datum EDA. Read the Datum context from $DATUM_DISCOVERY before acting, use datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 for recent GUI/terminal activity, and use datum-eda CLI commands for project-aware work.'",
            "claude_with_context": "claude 'You are running inside Datum EDA. Read $DATUM_DISCOVERY and inspect recent activity with datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 before acting.'",
            "context_prompt": "You are running inside Datum EDA. Read the Datum context from $DATUM_DISCOVERY before acting, use datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 for recent GUI/terminal activity, and use datum-eda CLI commands for project-aware work.",
            "inspect_context": "python3 -m json.tool \"$DATUM_DISCOVERY\"",
            "refresh_context": "datum-eda context refresh --session \"$DATUM_SESSION_ID\"",
            "session_activity": "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20"
        }),
    );
    insert_default(
        object,
        "query_commands",
        serde_json::json!({
            "resolve_debug": "datum-eda project query \"$DATUM_PROJECT_ROOT\" resolve-debug",
            "sheets": "datum-eda query sheets \"$DATUM_PROJECT_ROOT\"",
            "symbols": "datum-eda query symbols \"$DATUM_PROJECT_ROOT\"",
            "labels": "datum-eda query labels \"$DATUM_PROJECT_ROOT\"",
            "ports": "datum-eda query ports \"$DATUM_PROJECT_ROOT\"",
            "buses": "datum-eda query buses \"$DATUM_PROJECT_ROOT\"",
            "bus_entries": "datum-eda query bus-entries \"$DATUM_PROJECT_ROOT\"",
            "noconnects": "datum-eda query noconnects \"$DATUM_PROJECT_ROOT\"",
            "hierarchy": "datum-eda query hierarchy \"$DATUM_PROJECT_ROOT\"",
            "schematic_nets": "datum-eda query schematic-nets \"$DATUM_PROJECT_ROOT\"",
            "connectivity_diagnostics": "datum-eda query connectivity-diagnostics \"$DATUM_PROJECT_ROOT\"",
            "import_map": "datum-eda query import-map \"$DATUM_PROJECT_ROOT\"",
            "relationships": "datum-eda query relationships \"$DATUM_PROJECT_ROOT\"",
            "variants": "datum-eda query variants \"$DATUM_PROJECT_ROOT\"",
            "zone_fills": "datum-eda query zone-fills \"$DATUM_PROJECT_ROOT\""
        }),
    );
}

fn insert_default(object: &mut Map<String, Value>, key: &str, default: Value) {
    object.entry(key.to_string()).or_insert(default);
}
