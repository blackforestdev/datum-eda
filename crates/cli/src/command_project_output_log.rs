use std::path::PathBuf;

use eda_engine::substrate::{
    ArtifactProductionProjection, OutputJobLogEntry, OutputJobLogLevel, OutputJobRunLauncher,
    OutputJobRunProvenance,
};

pub(crate) fn append_production_projection_log_entries(
    log: &mut Vec<OutputJobLogEntry>,
    production_projections: &[ArtifactProductionProjection],
) {
    let mut sequence = log.last().map(|entry| entry.sequence + 1).unwrap_or(1);
    for projection in production_projections {
        log.push(OutputJobLogEntry {
            sequence,
            level: OutputJobLogLevel::Info,
            message: format!(
                "production projection {} {} {} bytes {}",
                projection.projection_kind,
                projection.projection_contract,
                projection.byte_count,
                projection.sha256
            ),
        });
        sequence += 1;
    }
}

pub(crate) fn terminal_origin_log_entries_from(
    env: &std::collections::BTreeMap<String, String>,
    first_sequence: u64,
) -> Vec<OutputJobLogEntry> {
    let mut sequence = first_sequence;
    let mut entries = Vec::new();
    if let Some(session_id) = non_empty_env(env, "DATUM_TERMINAL_SESSION_ID") {
        entries.push(OutputJobLogEntry {
            sequence,
            level: OutputJobLogLevel::Info,
            message: format!("launched from Datum terminal session {session_id}"),
        });
        sequence += 1;
    }
    if let Some(context_path) = non_empty_env(env, "DATUM_TERMINAL_CONTEXT") {
        entries.push(OutputJobLogEntry {
            sequence,
            level: OutputJobLogLevel::Info,
            message: format!("Datum terminal context {context_path}"),
        });
    }
    entries
}

pub(crate) fn terminal_origin_provenance_from(
    env: &std::collections::BTreeMap<String, String>,
) -> Option<OutputJobRunProvenance> {
    let terminal_session_id = non_empty_env(env, "DATUM_TERMINAL_SESSION_ID").map(str::to_string);
    let terminal_context_path = non_empty_env(env, "DATUM_TERMINAL_CONTEXT").map(PathBuf::from);
    if terminal_session_id.is_none() && terminal_context_path.is_none() {
        return None;
    }
    Some(OutputJobRunProvenance {
        launcher: OutputJobRunLauncher::GuiTerminal,
        terminal_session_id,
        terminal_context_path,
        project_root: non_empty_env(env, "DATUM_PROJECT_ROOT").map(PathBuf::from),
        source_revision: non_empty_env(env, "DATUM_SOURCE_REVISION").map(str::to_string),
    })
}

fn non_empty_env<'a>(
    env: &'a std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    env.get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}
