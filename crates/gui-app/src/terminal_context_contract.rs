use crate::terminal_active_context::TerminalActiveContextCommands;
use datum_gui_protocol::{
    CheckRunReviewState, DatumToolSessionLifecycle, DatumToolSessionMetadata, ProductionStatus,
    TerminalCommandCatalogEntry,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct TerminalContextEnvelope {
    pub(super) contract: &'static str,
    pub(super) project_root: String,
    pub(super) project_id: Option<String>,
    pub(super) project_name: Option<String>,
    pub(super) board_id: Option<String>,
    pub(super) board_name: Option<String>,
    pub(super) scene_id: Option<String>,
    pub(super) model_revision: Option<String>,
    pub(super) source_revision: Option<String>,
    pub(super) context_id: String,
    pub(super) session_id: String,
    pub(super) terminal_session_id: String,
    pub(super) session_lifecycle: DatumToolSessionLifecycle,
    pub(super) actor_type: &'static str,
    pub(super) capabilities: Vec<&'static str>,
    pub(super) created_unix_ms: u128,
    pub(super) updated_unix_ms: u128,
    pub(super) process_group_id: Option<i32>,
    pub(super) process_exit_code: Option<i32>,
    pub(super) accepted_transaction_tip: Option<String>,
    pub(super) visible_artifact_ids: Vec<String>,
    pub(super) visible_output_job_ids: Vec<String>,
    pub(super) visible_artifact_file_paths: Vec<String>,
    pub(super) latest_output_job_id: Option<String>,
    pub(super) latest_output_job_run_id: Option<String>,
    pub(super) latest_output_job_artifact_id: Option<String>,
    pub(super) latest_artifact_id: Option<String>,
    pub(super) latest_artifact_run_id: Option<String>,
    pub(super) previous_artifact_id: Option<String>,
    pub(super) visible_proposal_ids: Vec<String>,
    pub(super) latest_proposal_id: Option<String>,
    pub(super) focused_artifact_id: Option<String>,
    pub(super) focused_artifact_file_path: Option<String>,
    pub(super) latest_check_run_id: Option<String>,
    pub(super) latest_profile_id: Option<String>,
    pub(super) profile_latest_check_runs: Vec<TerminalCheckRunProfileLatest>,
    pub(super) visible_check_run_ids: Vec<String>,
    pub(super) visible_finding_fingerprints: Vec<String>,
    pub(super) check_status: CheckRunReviewState,
    pub(super) source_shard_status: datum_gui_protocol::SourceShardStatusSummary,
    pub(super) provenance_seed: String,
    pub(super) expires_at: Option<String>,
    pub(super) refresh_command: &'static str,
    pub(super) command_catalog_version: &'static str,
    pub(super) handoff_commands: std::collections::BTreeMap<String, TerminalCommandCatalogEntry>,
    pub(super) active_context_commands: TerminalActiveContextCommands,
    pub(super) storage: TerminalContextStorage,
    pub(super) session: DatumToolSessionMetadata,
    pub(super) terminal_sessions: crate::terminal_session_context::TerminalSessionContextSummary,
    pub(super) selection_context: datum_gui_protocol::DatumSelectionContext,
    pub(super) cursor_context: datum_gui_protocol::DatumCursorContext,
    pub(super) projection_context: datum_gui_protocol::DatumProjectionContext,
    pub(super) datum_cli: &'static str,
    pub(super) legacy_cli: &'static str,
    pub(super) discovery: String,
    pub(super) discovery_command: &'static str,
    pub(super) context_commands: TerminalContextCommands,
    pub(super) agent_commands: TerminalAgentCommands,
    pub(super) check_commands: TerminalCheckCommands,
    pub(super) library_commands: TerminalLibraryCommands,
    pub(super) proposal_commands: TerminalProposalCommands,
    pub(super) production_status: ProductionStatus,
    pub(super) production_commands: TerminalProductionCommands,
    pub(super) journal_commands: TerminalJournalCommands,
    pub(super) query_commands: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalCheckRunProfileLatest {
    pub(super) profile_id: String,
    pub(super) check_run_id: String,
    pub(super) model_revision: Option<String>,
    pub(super) status: Option<String>,
    pub(super) finding_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalContextStorage {
    pub(super) context_path: String,
    pub(super) latest_context_path: String,
    pub(super) compatibility_context_path: String,
    pub(super) legacy_context_path: String,
    pub(super) session_path: String,
    pub(super) event_log_path: String,
    pub(super) schema_version: u64,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalContextCommands {
    pub(super) get: &'static str,
    pub(super) refresh: &'static str,
    pub(super) legacy_resolve_debug: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalAgentCommands {
    pub(super) codex: &'static str,
    pub(super) claude: &'static str,
    pub(super) aider: &'static str,
    pub(super) codex_with_context: &'static str,
    pub(super) claude_with_context: &'static str,
    pub(super) context_prompt: &'static str,
    pub(super) inspect_context: &'static str,
    pub(super) refresh_context: &'static str,
    pub(super) session_activity: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalCheckCommands {
    pub(super) run_current: &'static str,
    pub(super) run_profile: &'static str,
    pub(super) list: &'static str,
    pub(super) show_current: &'static str,
    pub(super) profiles: &'static str,
    pub(super) generate_standards_repairs: &'static str,
    pub(super) waive: &'static str,
    pub(super) accept_deviation: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalLibraryCommands {
    pub(super) validate_pool: &'static str,
    pub(super) list_objects: &'static str,
    pub(super) show_object: &'static str,
    pub(super) create_pin_pad_map: &'static str,
    pub(super) set_pin_pad_map: &'static str,
    pub(super) propose_create_pin_pad_map: &'static str,
    pub(super) propose_set_pin_pad_map: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalProposalCommands {
    pub(super) query: &'static str,
    pub(super) show: &'static str,
    pub(super) preview: &'static str,
    pub(super) validate: &'static str,
    pub(super) defer: &'static str,
    pub(super) review_accept: &'static str,
    pub(super) review_reject: &'static str,
    pub(super) reject: &'static str,
    pub(super) accept_apply: &'static str,
    pub(super) apply: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalProductionCommands {
    pub(super) query_output_jobs: &'static str,
    pub(super) create_gerber_output_job: &'static str,
    pub(super) update_output_job: &'static str,
    pub(super) run_output_job: &'static str,
    pub(super) start_output_job_run: &'static str,
    pub(super) cancel_output_job_run: &'static str,
    pub(super) export_manufacturing_set: &'static str,
    pub(super) validate_manufacturing_set: &'static str,
    pub(super) delete_output_job: &'static str,
    pub(super) query_manufacturing_plans: &'static str,
    pub(super) create_manufacturing_plan: &'static str,
    pub(super) update_manufacturing_plan: &'static str,
    pub(super) delete_manufacturing_plan: &'static str,
    pub(super) query_panel_projections: &'static str,
    pub(super) create_panel_projection: &'static str,
    pub(super) update_panel_projection: &'static str,
    pub(super) delete_panel_projection: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TerminalJournalCommands {
    pub(super) list: Option<&'static str>,
    pub(super) show: Option<&'static str>,
    pub(super) undo: Option<&'static str>,
    pub(super) redo: Option<&'static str>,
}
