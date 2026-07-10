use crate::terminal_active_context::TerminalActiveContextCommands;
use crate::terminal_check_context::profile_latest_check_runs_context;
use crate::terminal_context_contract::{
    TerminalAgentCommands, TerminalCheckCommands, TerminalContextCommands, TerminalContextEnvelope,
    TerminalContextStorage, TerminalJournalCommands, TerminalLibraryCommands,
    TerminalProductionCommands, TerminalProposalCommands,
};
use crate::terminal_context_io::atomic_write_text;
use crate::terminal_journal_context::accepted_transaction_tip;
use crate::terminal_proposal_context::{latest_proposal_id, visible_proposal_ids};
use crate::{ASSISTANT_ACTIVITY_COMMAND, terminal_session::TerminalLaunchContext};
use anyhow::{Context, Result};
use datum_gui_protocol::{
    DatumToolSessionLifecycle, DatumToolSessionMetadata, ProductionStatus,
    TERMINAL_COMMAND_CATALOG_VERSION, terminal_command_catalog,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const DATUM_CLI: &str = "datum-eda";
pub(super) const DATUM_LEGACY_CLI: &str = "eda";

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
    let production_visibility = production_visibility_context(&context.production_status);
    let selected_finding_fingerprint = (context.selection_context.kind == "check_finding")
        .then_some(context.selection_context.id.as_deref())
        .flatten();
    let latest_proposal_id =
        latest_proposal_id(&context.production_status, &context.selection_context);
    let accepted_transaction_tip = accepted_transaction_tip(context);
    let active_context_commands = TerminalActiveContextCommands::from_focus(
        &context.project_root,
        production_visibility
            .focused_artifact_id
            .as_deref()
            .or(production_visibility.latest_artifact_id.as_deref()),
        production_visibility.previous_artifact_id.as_deref(),
        production_visibility.focused_artifact_file_path.as_deref(),
        production_visibility.latest_output_job_id.as_deref(),
        production_visibility.latest_output_job_run_id.as_deref(),
        latest_proposal_id.as_deref(),
        accepted_transaction_tip.as_deref(),
        context.check_status.check_run_id.as_deref(),
        selected_finding_fingerprint,
    );
    let profile_latest_check_runs = profile_latest_check_runs_context(&context.check_status);
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
        accepted_transaction_tip,
        visible_artifact_ids: production_visibility.visible_artifact_ids,
        visible_output_job_ids: production_visibility.visible_output_job_ids,
        visible_artifact_file_paths: production_visibility.visible_artifact_file_paths,
        latest_output_job_id: production_visibility.latest_output_job_id,
        latest_output_job_run_id: production_visibility.latest_output_job_run_id,
        latest_output_job_artifact_id: production_visibility.latest_output_job_artifact_id,
        latest_artifact_id: production_visibility.latest_artifact_id,
        latest_artifact_run_id: production_visibility.latest_artifact_run_id,
        previous_artifact_id: production_visibility.previous_artifact_id,
        visible_proposal_ids: visible_proposal_ids(&context.production_status),
        latest_proposal_id,
        focused_artifact_id: production_visibility.focused_artifact_id,
        focused_artifact_file_path: production_visibility.focused_artifact_file_path,
        latest_check_run_id: context.check_status.check_run_id.clone(),
        latest_profile_id: context.check_status.profile_id.clone(),
        profile_latest_check_runs,
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
        source_shard_status: context.source_shard_status.clone(),
        provenance_seed,
        expires_at: None,
        refresh_command: "datum-eda context refresh --session \"$DATUM_SESSION_ID\"",
        command_catalog_version: TERMINAL_COMMAND_CATALOG_VERSION,
        handoff_commands: terminal_command_catalog(),
        active_context_commands,
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
        terminal_sessions: context.terminal_sessions.clone(),
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
        library_commands: TerminalLibraryCommands {
            validate_pool: "datum-eda project validate \"$DATUM_PROJECT_ROOT\"",
            list_objects: "datum-eda query pool-library-objects \"$DATUM_PROJECT_ROOT\" --pool pool",
            show_object: "datum-eda query pool-library-objects \"$DATUM_PROJECT_ROOT\" --pool pool --kind <kind> --object <uuid> --include-payload",
            create_pin_pad_map: "datum-eda project create-pool-pin-pad-map \"$DATUM_PROJECT_ROOT\" --pool pool --map <uuid> --part <uuid> --entry <pad_uuid>:<gate_uuid>:<pin_uuid> [--footprint <uuid>] [--set-default]",
            set_pin_pad_map: "datum-eda project set-pool-pin-pad-map \"$DATUM_PROJECT_ROOT\" --pool pool --map <uuid> --mode merge --entry <pad_uuid>:<gate_uuid>:<pin_uuid>",
            propose_create_pin_pad_map: "datum-eda proposal create-pool-pin-pad-map \"$DATUM_PROJECT_ROOT\" --pool pool --map <uuid> --part <uuid> --entry <pad_uuid>:<gate_uuid>:<pin_uuid> [--footprint <uuid>] [--set-default] --rationale <text>",
            propose_set_pin_pad_map: "datum-eda proposal set-pool-pin-pad-map \"$DATUM_PROJECT_ROOT\" --pool pool --map <uuid> --mode merge --entry <pad_uuid>:<gate_uuid>:<pin_uuid> --rationale <text>",
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
            list: None,
            show: None,
            undo: None,
            redo: None,
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
    atomic_write_text(&terminal_context.context_path, &format!("{text}\n")).with_context(|| {
        format!(
            "write terminal context {}",
            terminal_context.context_path.display()
        )
    })?;
    atomic_write_text(&terminal_context.latest_context_path, &format!("{text}\n")).with_context(
        || {
            format!(
                "write latest terminal context {}",
                terminal_context.latest_context_path.display()
            )
        },
    )?;
    let session_text =
        serde_json::to_string_pretty(&session).context("serialize tool session metadata")?;
    atomic_write_text(&terminal_context.session_path, &format!("{session_text}\n")).with_context(
        || {
            format!(
                "write tool session metadata {}",
                terminal_context.session_path.display()
            )
        },
    )?;
    Ok(())
}

struct ProductionVisibilityContext {
    visible_artifact_ids: Vec<String>,
    visible_output_job_ids: Vec<String>,
    visible_artifact_file_paths: Vec<String>,
    latest_output_job_id: Option<String>,
    latest_output_job_run_id: Option<String>,
    latest_output_job_artifact_id: Option<String>,
    latest_artifact_id: Option<String>,
    latest_artifact_run_id: Option<String>,
    previous_artifact_id: Option<String>,
    focused_artifact_id: Option<String>,
    focused_artifact_file_path: Option<String>,
}

fn production_visibility_context(status: &ProductionStatus) -> ProductionVisibilityContext {
    let mut artifact_ids = BTreeSet::new();
    let mut artifact_file_paths = BTreeSet::new();
    let visible_output_job_ids = status
        .output_jobs
        .iter()
        .map(|job| job.id.clone())
        .collect::<Vec<_>>();

    let latest_job = status
        .output_jobs
        .iter()
        .filter_map(|job| {
            job.latest_run_id.as_ref().map(|run_id| {
                (
                    job.id.clone(),
                    run_id.clone(),
                    job.latest_run_artifact_id.clone(),
                )
            })
        })
        .max_by(|(a_job_id, a_run_id, _), (b_job_id, b_run_id, _)| {
            a_run_id.cmp(b_run_id).then_with(|| a_job_id.cmp(b_job_id))
        });

    for job in &status.output_jobs {
        if let Some(artifact_id) = &job.latest_run_artifact_id {
            artifact_ids.insert(artifact_id.clone());
        }
        for artifact in &job.artifacts {
            artifact_ids.insert(artifact.artifact_id.clone());
            for file in &artifact.files {
                artifact_file_paths.insert(file.path.clone());
            }
        }
    }
    for run in &status.artifact_runs {
        artifact_ids.insert(run.artifact_id.clone());
    }
    let focused_artifact_id = status
        .focused_artifact
        .as_ref()
        .map(|artifact| artifact.artifact_id.clone());
    let focused_artifact_file_path = status
        .focused_artifact
        .as_ref()
        .and_then(|artifact| artifact.focused_file.as_ref())
        .map(|file| file.path.clone());
    if let Some(artifact) = &status.focused_artifact {
        artifact_ids.insert(artifact.artifact_id.clone());
        for file in &artifact.files {
            artifact_file_paths.insert(file.path.clone());
        }
    }

    let fallback_output_job_run = status
        .latest_output_job_run_id
        .as_ref()
        .or(status.latest_run_id.as_ref())
        .and_then(|run_id| {
            status
                .artifact_runs
                .iter()
                .find(|run| run.run_id == *run_id && run.output_job_id.is_some())
        })
        .or_else(|| {
            status
                .artifact_runs
                .iter()
                .rev()
                .find(|run| run.output_job_id.is_some())
        });
    let (latest_output_job_id, latest_output_job_run_id, latest_output_job_artifact_id) =
        latest_job
            .map(|(job_id, run_id, artifact_id)| (Some(job_id), Some(run_id), artifact_id))
            .unwrap_or_else(|| {
                (
                    fallback_output_job_run.and_then(|run| run.output_job_id.clone()),
                    fallback_output_job_run
                        .map(|run| run.run_id.clone())
                        .or(status.latest_output_job_run_id.clone())
                        .or(status.latest_run_id.clone()),
                    fallback_output_job_run.map(|run| run.artifact_id.clone()),
                )
            });
    let latest_artifact_id = status
        .latest_artifact_id
        .clone()
        .or(latest_output_job_artifact_id.clone())
        .or_else(|| {
            status
                .artifact_runs
                .last()
                .map(|run| run.artifact_id.clone())
        })
        .or(focused_artifact_id.clone());
    let latest_artifact_run_id = status
        .latest_artifact_run_id
        .clone()
        .or_else(|| status.artifact_runs.last().map(|run| run.run_id.clone()));
    let previous_artifact_id = latest_artifact_id.as_deref().and_then(|latest| {
        status
            .artifact_runs
            .iter()
            .rev()
            .find(|run| run.artifact_id != latest)
            .map(|run| run.artifact_id.clone())
    });

    ProductionVisibilityContext {
        visible_artifact_ids: artifact_ids.into_iter().collect(),
        visible_output_job_ids,
        visible_artifact_file_paths: artifact_file_paths.into_iter().collect(),
        latest_output_job_id,
        latest_output_job_run_id,
        latest_output_job_artifact_id,
        latest_artifact_id,
        latest_artifact_run_id,
        previous_artifact_id,
        focused_artifact_id,
        focused_artifact_file_path,
    }
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
    atomic_write_text(path, &format!("{refreshed}\n"))
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

#[cfg(test)]
#[path = "terminal_active_context_tests.rs"]
mod terminal_active_context_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_text_keeps_terminal_context_json_parseable_across_rewrites() {
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-context-atomic-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("atomic context test dir should create");
        let path = root.join("context.json");

        for index in 0..20 {
            atomic_write_text(
                &path,
                &format!("{{\"contract\":\"datum_terminal_context_v1\",\"index\":{index}}}\n"),
            )
            .expect("atomic context write should succeed");
            let value: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&path).expect("context json should read"))
                    .expect("context json should remain parseable after rewrite");
            assert_eq!(value["index"], index);
        }

        let leaked_temp = fs::read_dir(&root)
            .expect("atomic context test dir should list")
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(".context.json.tmp-")
            });
        assert!(
            !leaked_temp,
            "successful atomic writes should not leak temp files"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn lifecycle_update_rewrites_context_atomically_and_preserves_valid_json() {
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-lifecycle-atomic-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("lifecycle context test dir should create");
        let path = root.join("context.json");
        atomic_write_text(
            &path,
            r#"{
  "contract": "datum_terminal_context_v1",
  "session_lifecycle": "running",
  "session": {
    "lifecycle": "running"
  }
}
"#,
        )
        .expect("seed context should write");

        update_terminal_lifecycle_file(
            &path,
            DatumToolSessionLifecycle::Exited,
            Some(0),
            Some(1234),
        )
        .expect("lifecycle update should rewrite context");

        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path).expect("updated lifecycle context should read"),
        )
        .expect("updated lifecycle context should remain parseable");
        assert_eq!(value["session_lifecycle"], "exited");
        assert_eq!(value["session"]["lifecycle"], "exited");
        assert_eq!(value["process_exit_code"], 0);
        assert_eq!(value["process_group_id"], 1234);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn production_visibility_prefers_canonical_latest_output_job_run_id() {
        let status = ProductionStatus {
            latest_run_id: Some("legacy-run".to_string()),
            latest_output_job_run_id: Some("canonical-output-run".to_string()),
            artifact_runs: vec![datum_gui_protocol::ProductionArtifactRunSummary {
                run_id: "canonical-output-run".to_string(),
                artifact_id: "artifact-from-run".to_string(),
                run_source: "output_job_run".to_string(),
                output_job_id: Some("job-from-run".to_string()),
                run_sequence: 7,
                status: "succeeded".to_string(),
                exit_code: Some(0),
            }],
            ..ProductionStatus::default()
        };

        let visibility = production_visibility_context(&status);

        assert_eq!(
            visibility.latest_output_job_id.as_deref(),
            Some("job-from-run")
        );
        assert_eq!(
            visibility.latest_output_job_run_id.as_deref(),
            Some("canonical-output-run")
        );
        assert_eq!(
            visibility.latest_output_job_artifact_id.as_deref(),
            Some("artifact-from-run")
        );
    }
}
