use crate::{ASSISTANT_ACTIVITY_COMMAND, terminal_session::TerminalLaunchContext};
use anyhow::{Context, Result};
use datum_gui_protocol::{
    CheckRunReviewState, DatumToolSessionLifecycle, DatumToolSessionMetadata, ProductionStatus,
    TERMINAL_COMMAND_CATALOG_VERSION, TerminalCommandCatalogEntry, terminal_command_catalog,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const DATUM_CLI: &str = "datum-eda";
pub(super) const DATUM_LEGACY_CLI: &str = "eda";

#[derive(Debug, Serialize)]
struct TerminalContextEnvelope {
    contract: &'static str,
    project_root: String,
    project_id: Option<String>,
    project_name: Option<String>,
    board_id: Option<String>,
    board_name: Option<String>,
    scene_id: Option<String>,
    model_revision: Option<String>,
    source_revision: Option<String>,
    context_id: String,
    session_id: String,
    terminal_session_id: String,
    session_lifecycle: DatumToolSessionLifecycle,
    actor_type: &'static str,
    capabilities: Vec<&'static str>,
    created_unix_ms: u128,
    updated_unix_ms: u128,
    process_group_id: Option<i32>,
    process_exit_code: Option<i32>,
    accepted_transaction_tip: Option<String>,
    visible_artifact_ids: Vec<String>,
    visible_check_run_ids: Vec<String>,
    visible_finding_fingerprints: Vec<String>,
    check_status: CheckRunReviewState,
    provenance_seed: String,
    expires_at: Option<String>,
    refresh_command: &'static str,
    command_catalog_version: &'static str,
    handoff_commands: std::collections::BTreeMap<String, TerminalCommandCatalogEntry>,
    storage: TerminalContextStorage,
    session: DatumToolSessionMetadata,
    selection_context: datum_gui_protocol::DatumSelectionContext,
    cursor_context: datum_gui_protocol::DatumCursorContext,
    projection_context: datum_gui_protocol::DatumProjectionContext,
    datum_cli: &'static str,
    legacy_cli: &'static str,
    discovery: String,
    discovery_command: &'static str,
    context_commands: TerminalContextCommands,
    agent_commands: TerminalAgentCommands,
    check_commands: TerminalCheckCommands,
    proposal_commands: TerminalProposalCommands,
    production_status: ProductionStatus,
    production_commands: TerminalProductionCommands,
    journal_commands: TerminalJournalCommands,
    query_commands: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TerminalContextStorage {
    context_path: String,
    latest_context_path: String,
    compatibility_context_path: String,
    legacy_context_path: String,
    session_path: String,
    event_log_path: String,
    schema_version: u64,
}

#[derive(Debug, Serialize)]
struct TerminalContextCommands {
    get: &'static str,
    refresh: &'static str,
    legacy_resolve_debug: &'static str,
}

#[derive(Debug, Serialize)]
struct TerminalAgentCommands {
    codex: &'static str,
    claude: &'static str,
    aider: &'static str,
    codex_with_context: &'static str,
    claude_with_context: &'static str,
    context_prompt: &'static str,
    inspect_context: &'static str,
    refresh_context: &'static str,
    session_activity: &'static str,
}

#[derive(Debug, Serialize)]
struct TerminalCheckCommands {
    run_current: &'static str,
    run_profile: &'static str,
    list: &'static str,
    show_current: &'static str,
    profiles: &'static str,
    generate_standards_repairs: &'static str,
    waive: &'static str,
    accept_deviation: &'static str,
}

#[derive(Debug, Serialize)]
struct TerminalProposalCommands {
    query: &'static str,
    show: &'static str,
    preview: &'static str,
    validate: &'static str,
    defer: &'static str,
    review_accept: &'static str,
    review_reject: &'static str,
    reject: &'static str,
    accept_apply: &'static str,
    apply: &'static str,
}

#[derive(Debug, Serialize)]
struct TerminalProductionCommands {
    query_output_jobs: &'static str,
    create_gerber_output_job: &'static str,
    update_output_job: &'static str,
    run_output_job: &'static str,
    start_output_job_run: &'static str,
    cancel_output_job_run: &'static str,
    export_manufacturing_set: &'static str,
    validate_manufacturing_set: &'static str,
    delete_output_job: &'static str,
    query_manufacturing_plans: &'static str,
    create_manufacturing_plan: &'static str,
    update_manufacturing_plan: &'static str,
    delete_manufacturing_plan: &'static str,
    query_panel_projections: &'static str,
    create_panel_projection: &'static str,
    update_panel_projection: &'static str,
    delete_panel_projection: &'static str,
}

#[derive(Debug, Serialize)]
struct TerminalJournalCommands {
    list: &'static str,
    show: &'static str,
    undo: &'static str,
    redo: &'static str,
}

pub(super) struct TerminalContext {
    pub(super) project_root: PathBuf,
    pub(super) context_path: PathBuf,
    pub(super) latest_context_path: PathBuf,
    pub(super) session_path: PathBuf,
    pub(super) context_id: String,
    pub(super) session_id: String,
    pub(super) project_id: Option<String>,
    pub(super) model_revision: Option<String>,
    pub(super) created_unix_ms: u128,
    pub(super) process_group_id: Option<i32>,
}

pub(super) fn write_terminal_context(context: &TerminalLaunchContext) -> Result<TerminalContext> {
    let datum_dir = context.project_root.join(".datum");
    fs::create_dir_all(&datum_dir)
        .with_context(|| format!("create terminal context dir {}", datum_dir.display()))?;
    let terminal_context_dir = datum_dir.join("terminal-contexts");
    fs::create_dir_all(&terminal_context_dir).with_context(|| {
        format!(
            "create terminal session context dir {}",
            terminal_context_dir.display()
        )
    })?;
    let tool_session_dir = datum_dir.join("tool-sessions");
    fs::create_dir_all(&tool_session_dir).with_context(|| {
        format!(
            "create tool session metadata dir {}",
            tool_session_dir.display()
        )
    })?;
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("terminal context timestamp")?
            .as_nanos()
    );
    let session_id = format!("terminal-{unique_suffix}");
    let context_id = format!("context-{unique_suffix}");
    let context_path = terminal_context_dir.join(format!("{session_id}.json"));
    let latest_context_path = datum_dir.join("gui-terminal-context.json");
    let session_path = tool_session_dir.join(format!("{session_id}.json"));
    let terminal_context = TerminalContext {
        project_root: context.project_root.clone(),
        context_path,
        latest_context_path,
        session_path,
        context_id,
        session_id,
        project_id: context.project_id.clone(),
        model_revision: context.source_revision.clone(),
        created_unix_ms: unix_time_ms()?,
        process_group_id: None,
    };
    write_terminal_context_files(&terminal_context, context)?;
    Ok(terminal_context)
}

pub(super) fn write_terminal_context_files(
    terminal_context: &TerminalContext,
    context: &TerminalLaunchContext,
) -> Result<()> {
    let updated_unix_ms = unix_time_ms()?;
    let provenance_seed = format!(
        "datum-context:{}:{}:{}",
        terminal_context.session_id,
        terminal_context.context_id,
        context
            .source_revision
            .as_deref()
            .unwrap_or("unknown-revision")
    );
    let session = DatumToolSessionMetadata {
        session_id: terminal_context.session_id.clone(),
        context_id: terminal_context.context_id.clone(),
        actor_type: "ExternalAgent".to_string(),
        capabilities: vec![
            "read".to_string(),
            "check".to_string(),
            "artifact".to_string(),
            "propose".to_string(),
            "apply-approved".to_string(),
        ],
        created_model_revision: context.source_revision.clone(),
        lifecycle: DatumToolSessionLifecycle::Running,
        created_unix_ms: terminal_context.created_unix_ms,
        updated_unix_ms,
        expires_unix_ms: None,
        process_group_id: terminal_context.process_group_id,
        process_exit_code: None,
    };
    let envelope = TerminalContextEnvelope {
        contract: "datum_terminal_context_v1",
        project_root: context.project_root.display().to_string(),
        project_id: context.project_id.clone(),
        project_name: context.project_name.clone(),
        board_id: context.board_id.clone(),
        board_name: context.board_name.clone(),
        scene_id: context.scene_id.clone(),
        model_revision: context.source_revision.clone(),
        source_revision: context.source_revision.clone(),
        context_id: terminal_context.context_id.clone(),
        session_id: terminal_context.session_id.clone(),
        terminal_session_id: terminal_context.session_id.clone(),
        session_lifecycle: DatumToolSessionLifecycle::Running,
        actor_type: "ExternalAgent",
        capabilities: vec!["read", "check", "artifact", "propose", "apply-approved"],
        created_unix_ms: terminal_context.created_unix_ms,
        updated_unix_ms,
        process_group_id: terminal_context.process_group_id,
        process_exit_code: None,
        accepted_transaction_tip: None,
        visible_artifact_ids: Vec::new(),
        visible_check_run_ids: context.check_status.check_run_id.iter().cloned().collect(),
        visible_finding_fingerprints: context
            .check_status
            .findings
            .iter()
            .filter_map(|finding| {
                (!finding.fingerprint.is_empty()).then_some(finding.fingerprint.clone())
            })
            .collect(),
        check_status: context.check_status.clone(),
        provenance_seed,
        expires_at: None,
        refresh_command: "datum-eda context refresh --session \"$DATUM_SESSION_ID\"",
        command_catalog_version: TERMINAL_COMMAND_CATALOG_VERSION,
        handoff_commands: terminal_command_catalog(),
        storage: TerminalContextStorage {
            context_path: terminal_context.context_path.display().to_string(),
            latest_context_path: terminal_context.latest_context_path.display().to_string(),
            compatibility_context_path: terminal_context.latest_context_path.display().to_string(),
            legacy_context_path: terminal_context.latest_context_path.display().to_string(),
            session_path: terminal_context.session_path.display().to_string(),
            event_log_path: tool_session_event_log_path(&terminal_context.session_path)
                .display()
                .to_string(),
            schema_version: 1,
        },
        session: session.clone(),
        selection_context: context.selection_context.clone(),
        cursor_context: context.cursor_context.clone(),
        projection_context: context.projection_context.clone(),
        datum_cli: DATUM_CLI,
        legacy_cli: DATUM_LEGACY_CLI,
        discovery: terminal_context.context_path.display().to_string(),
        discovery_command: "datum-eda context get --session \"$DATUM_SESSION_ID\"",
        context_commands: TerminalContextCommands {
            get: "datum-eda context get --session \"$DATUM_SESSION_ID\"",
            refresh: "datum-eda context refresh --session \"$DATUM_SESSION_ID\"",
            legacy_resolve_debug: "datum-eda project query \"$DATUM_PROJECT_ROOT\" resolve-debug",
        },
        agent_commands: TerminalAgentCommands {
            codex: "codex",
            claude: "claude",
            aider: "aider",
            codex_with_context: "codex 'You are running inside Datum EDA. Read the Datum context from $DATUM_DISCOVERY before acting, use datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 for recent GUI/terminal activity, and use datum-eda CLI commands for project-aware work.'",
            claude_with_context: "claude 'You are running inside Datum EDA. Read $DATUM_DISCOVERY and inspect recent activity with datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 before acting.'",
            context_prompt: "You are running inside Datum EDA. Read the Datum context from $DATUM_DISCOVERY before acting, use datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 for recent GUI/terminal activity, and use datum-eda CLI commands for project-aware work.",
            inspect_context: "python3 -m json.tool \"$DATUM_DISCOVERY\"",
            refresh_context: "datum-eda context refresh --session \"$DATUM_SESSION_ID\"",
            session_activity: ASSISTANT_ACTIVITY_COMMAND,
        },
        check_commands: TerminalCheckCommands {
            run_current: "datum-eda check run \"$DATUM_PROJECT_ROOT\"",
            run_profile: "datum-eda check run \"$DATUM_PROJECT_ROOT\" --profile <profile>",
            list: "datum-eda check list \"$DATUM_PROJECT_ROOT\"",
            show_current: "datum-eda check show \"$DATUM_PROJECT_ROOT\" --check-run <uuid>",
            profiles: "datum-eda check profiles \"$DATUM_PROJECT_ROOT\"",
            generate_standards_repairs: "datum-eda check repair-standards \"$DATUM_PROJECT_ROOT\"",
            waive: "datum-eda check waive \"$DATUM_PROJECT_ROOT\" --fingerprint <sha256:...> --rationale <text>",
            accept_deviation: "datum-eda check accept-deviation \"$DATUM_PROJECT_ROOT\" --fingerprint <sha256:...> --rationale <text>",
        },
        proposal_commands: TerminalProposalCommands {
            query: "datum-eda proposal list \"$DATUM_PROJECT_ROOT\"",
            show: "datum-eda proposal show \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            preview: "datum-eda proposal preview \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            validate: "datum-eda proposal validate \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            defer: "datum-eda proposal defer \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            review_accept: "datum-eda proposal review \"$DATUM_PROJECT_ROOT\" --proposal <uuid> --status accepted",
            review_reject: "datum-eda proposal review \"$DATUM_PROJECT_ROOT\" --proposal <uuid> --status rejected",
            reject: "datum-eda proposal reject \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            accept_apply: "datum-eda proposal accept-apply \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
            apply: "datum-eda proposal apply \"$DATUM_PROJECT_ROOT\" --proposal <uuid>",
        },
        production_status: context.production_status.clone(),
        production_commands: TerminalProductionCommands {
            query_output_jobs: "datum-eda query output-jobs \"$DATUM_PROJECT_ROOT\"",
            create_gerber_output_job: "datum-eda proposal create-output-job \"$DATUM_PROJECT_ROOT\" --prefix <prefix> --include gerber-set [--name <name>] [--manufacturing-plan <uuid>] [--output-dir <dir>] [--rationale <text>]",
            update_output_job: "datum-eda proposal update-output-job \"$DATUM_PROJECT_ROOT\" --output-job <uuid> [--name <name>] [--manufacturing-plan <uuid>|--clear-manufacturing-plan] [--rationale <text>]",
            run_output_job: "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job <uuid> [--output-dir <dir>]",
            start_output_job_run: "datum-eda artifact start-output-job-run \"$DATUM_PROJECT_ROOT\" --output-job <uuid>",
            cancel_output_job_run: "datum-eda artifact cancel-output-job-run \"$DATUM_PROJECT_ROOT\" --run <uuid>",
            export_manufacturing_set: "datum-eda artifact export-manufacturing-set \"$DATUM_PROJECT_ROOT\" --output-dir <dir> [--prefix <prefix>] [--include <scope>[,<scope>...]] [--output-job <uuid>|--job <name>] [--variant <uuid>]",
            validate_manufacturing_set: "datum-eda artifact validate-manufacturing-set \"$DATUM_PROJECT_ROOT\" --output-dir <dir> [--prefix <prefix>] [--include <scope>[,<scope>...]] [--output-job <uuid>|--job <name>] [--variant <uuid>]",
            delete_output_job: "datum-eda proposal delete-output-job \"$DATUM_PROJECT_ROOT\" --output-job <uuid> [--rationale <text>]",
            query_manufacturing_plans: "datum-eda query manufacturing-plans \"$DATUM_PROJECT_ROOT\"",
            create_manufacturing_plan: "datum-eda proposal create-manufacturing-plan \"$DATUM_PROJECT_ROOT\" --prefix <prefix> [--name <name>] [--variant <uuid>] [--panel-projection <uuid>] [--rationale <text>]",
            update_manufacturing_plan: "datum-eda proposal update-manufacturing-plan \"$DATUM_PROJECT_ROOT\" --manufacturing-plan <uuid> [--name <name>] [--prefix <prefix>] [--variant <uuid>|--clear-variant] [--panel-projection <uuid>|--clear-panel-projection] [--rationale <text>]",
            delete_manufacturing_plan: "datum-eda proposal delete-manufacturing-plan \"$DATUM_PROJECT_ROOT\" --manufacturing-plan <uuid> [--rationale <text>]",
            query_panel_projections: "datum-eda query panel-projections \"$DATUM_PROJECT_ROOT\"",
            create_panel_projection: "datum-eda proposal create-panel-projection \"$DATUM_PROJECT_ROOT\" --key <key> [--name <name>] [--board <uuid>] [--x-nm <nm>] [--y-nm <nm>] [--rotation-deg <deg>] [--rationale <text>]",
            update_panel_projection: "datum-eda proposal update-panel-projection \"$DATUM_PROJECT_ROOT\" --panel-projection <uuid> [--name <name>] [--board <uuid>] [--x-nm <nm>] [--y-nm <nm>] [--rotation-deg <deg>] [--rationale <text>]",
            delete_panel_projection: "datum-eda proposal delete-panel-projection \"$DATUM_PROJECT_ROOT\" --panel-projection <uuid> [--rationale <text>]",
        },
        journal_commands: TerminalJournalCommands {
            list: "datum-eda journal list \"$DATUM_PROJECT_ROOT\"",
            show: "datum-eda journal show \"$DATUM_PROJECT_ROOT\" --transaction <uuid>",
            undo: "datum-eda journal undo \"$DATUM_PROJECT_ROOT\"",
            redo: "datum-eda journal redo \"$DATUM_PROJECT_ROOT\"",
        },
        query_commands: serde_json::json!({
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
    };
    let text = serde_json::to_string_pretty(&envelope).context("serialize terminal context")?;
    fs::write(&terminal_context.context_path, format!("{text}\n")).with_context(|| {
        format!(
            "write terminal context {}",
            terminal_context.context_path.display()
        )
    })?;
    fs::write(&terminal_context.latest_context_path, format!("{text}\n")).with_context(|| {
        format!(
            "write latest terminal context {}",
            terminal_context.latest_context_path.display()
        )
    })?;
    let session_text =
        serde_json::to_string_pretty(&session).context("serialize tool session metadata")?;
    fs::write(&terminal_context.session_path, format!("{session_text}\n")).with_context(|| {
        format!(
            "write tool session metadata {}",
            terminal_context.session_path.display()
        )
    })?;
    Ok(())
}

pub(super) fn update_terminal_lifecycle_file(
    path: &Path,
    lifecycle: DatumToolSessionLifecycle,
    process_exit_code: Option<i32>,
    process_group_id: Option<i32>,
) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parse terminal lifecycle file {}", path.display()))?;
    let updated_unix_ms = unix_time_ms()?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "session_lifecycle".to_string(),
            serde_json::json!(lifecycle.as_str()),
        );
        object.insert(
            "updated_unix_ms".to_string(),
            serde_json::json!(updated_unix_ms),
        );
        object.insert(
            "process_exit_code".to_string(),
            serde_json::to_value(process_exit_code)?,
        );
        object.insert(
            "process_group_id".to_string(),
            serde_json::to_value(process_group_id)?,
        );
        if let Some(session) = object
            .get_mut("session")
            .and_then(serde_json::Value::as_object_mut)
        {
            session.insert(
                "lifecycle".to_string(),
                serde_json::json!(lifecycle.as_str()),
            );
            session.insert(
                "updated_unix_ms".to_string(),
                serde_json::json!(updated_unix_ms),
            );
            session.insert(
                "process_exit_code".to_string(),
                serde_json::to_value(process_exit_code)?,
            );
            session.insert(
                "process_group_id".to_string(),
                serde_json::to_value(process_group_id)?,
            );
        }
    }
    let refreshed = serde_json::to_string_pretty(&value)
        .with_context(|| format!("serialize terminal lifecycle file {}", path.display()))?;
    fs::write(path, format!("{refreshed}\n"))
        .with_context(|| format!("write terminal lifecycle file {}", path.display()))
}

pub(super) fn read_session_created_unix_ms(path: &Path) -> Option<u128> {
    let text = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value
        .get("created_unix_ms")
        .or_else(|| value.pointer("/session/created_unix_ms"))
        .and_then(serde_json::Value::as_u64)
        .map(u128::from)
}

pub(super) fn unix_time_ms() -> Result<u128> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal context timestamp")?
        .as_millis())
}

pub(super) fn tool_session_event_log_path(session_path: &Path) -> PathBuf {
    session_path.with_extension("events.jsonl")
}
