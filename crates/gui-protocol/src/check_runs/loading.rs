//! Read-only workspace check projection consumes existing context/history.
use super::{
    CheckRunReviewState, check_run_review_state_from_context_value,
    check_run_review_state_from_json,
};
use crate::{LiveReviewRequest, run_cli_json};
use anyhow::Result;
use serde_json::Value;

pub(crate) fn load(cli: &[String], request: &LiveReviewRequest) -> Result<CheckRunReviewState> {
    if request.board_file.is_some() {
        return Ok(CheckRunReviewState::default());
    }
    let project_root = request.project_root.display().to_string();
    if let Ok(context) = run_cli_json::<Value>(
        cli,
        &["context", "refresh", "--project-root", &project_root],
    ) && let Some(state) = check_run_review_state_from_context_value(&context)
        && state.check_run_id.is_some()
    {
        return Ok(state);
    }
    // Loading is read-only; history order does not confer current authority.
    let history = run_cli_json::<Value>(cli, &["check", "list", &project_root])?;
    let (empty, id) = history_candidate(&history);
    let Some(candidate) = id else {
        return Ok(empty);
    };
    let record = run_cli_json::<Value>(
        cli,
        &["check", "show", &project_root, "--check-run", &candidate.id],
    )?;
    admit_record(empty, &candidate, &record)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryCandidate {
    id: String,
    revision: String,
}

fn history_candidate(history: &Value) -> (CheckRunReviewState, Option<HistoryCandidate>) {
    let project = history
        .get("project_id")
        .and_then(Value::as_str)
        .filter(|id| uuid::Uuid::parse_str(id).is_ok());
    let revision = history
        .get("model_revision")
        .and_then(Value::as_str)
        .filter(|revision| !revision.is_empty());
    let empty = CheckRunReviewState {
        project_id: project.map(str::to_owned),
        model_revision: revision.map(str::to_owned),
        ..Default::default()
    };
    if history.get("contract").and_then(Value::as_str) != Some("check_run_list_v1")
        || project.is_none()
        || revision.is_none()
    {
        return (CheckRunReviewState::default(), None);
    }
    let runs = history
        .get("check_runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let valid = |run: &&Value| {
        run.get("project_id").and_then(Value::as_str) == project
            && run
                .get("model_revision")
                .and_then(Value::as_str)
                .is_some_and(|r| !r.is_empty())
            && run
                .get("check_run_id")
                .and_then(Value::as_str)
                .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
    };
    // Retain actual historical evidence separately from currentness. Stable
    // history order is not represented as chronological or current authority.
    let selected = runs
        .iter()
        .rev()
        .filter(valid)
        .find(|run| run.get("model_revision").and_then(Value::as_str) == revision)
        .or_else(|| {
            runs.iter()
                .rev()
                .filter(valid)
                .find(|run| run.get("profile_id").and_then(Value::as_str) == Some("erc"))
        })
        .or_else(|| runs.iter().rev().find(valid));
    let id = selected.map(|run| HistoryCandidate {
        id: run["check_run_id"]
            .as_str()
            .expect("validated identity")
            .into(),
        revision: run["model_revision"]
            .as_str()
            .expect("validated revision")
            .into(),
    });
    (empty, id)
}

fn admit_record(
    empty: CheckRunReviewState,
    candidate: &HistoryCandidate,
    record: &Value,
) -> Result<CheckRunReviewState> {
    let run = record.get("check_run");
    if record.get("contract").and_then(Value::as_str) != Some("check_run_record_v1")
        || record.get("project_id").and_then(Value::as_str) != empty.project_id.as_deref()
        || record.get("model_revision").and_then(Value::as_str) != empty.model_revision.as_deref()
        || run
            .and_then(|run| run.get("project_id"))
            .and_then(Value::as_str)
            != empty.project_id.as_deref()
        || run
            .and_then(|run| run.get("model_revision"))
            .and_then(Value::as_str)
            != Some(candidate.revision.as_str())
        || run
            .and_then(|run| run.get("check_run_id"))
            .and_then(Value::as_str)
            != Some(candidate.id.as_str())
    {
        return Ok(empty);
    }
    check_run_review_state_from_json(&serde_json::to_string(record)?)
}

#[cfg(test)]
mod tests;

/// Compare only canonical disposition source, not ordinary geometry changes.
pub fn disposition_basis(model: &eda_engine::substrate::DesignModel) -> Result<Value> {
    use eda_engine::substrate::SourceShardKind;
    if !model
        .source_shards
        .iter()
        .any(|s| s.kind == SourceShardKind::SchematicRoot)
    {
        return Ok(Value::Null);
    }
    let root = model.materialized_source_shard_value(SourceShardKind::SchematicRoot)?;
    Ok(serde_json::json!({"waivers":root.get("waivers"),"deviations":root.get("deviations")}))
}
pub(crate) fn has_dispositions(model: &eda_engine::substrate::DesignModel) -> Result<bool> {
    let basis = disposition_basis(model)?;
    Ok(["waivers", "deviations"]
        .iter()
        .any(|key| basis[*key].as_array().is_some_and(|v| !v.is_empty())))
}
/// The existing evaluator runs read-only: no new journal tail can steal Undo.
/// Persisted current evidence wins; a computed report never authorizes a disposition.
pub fn refresh_disposition_report(
    state: &mut crate::ReviewWorkspaceState,
    request: &LiveReviewRequest,
) -> Result<()> {
    if current_persisted(state) {
        return Ok(());
    }
    let cli = crate::cli_bridge::cli_prefix()?;
    let value: Value = run_cli_json(
        &cli,
        &[
            "project",
            "query",
            &request.project_root.display().to_string(),
            "erc",
        ],
    )?;
    let run = check_run_review_state_from_json(&serde_json::to_string(&value)?)?;
    admit_projection(state, &run)?;
    state.checks = run;
    sync_report_summary(state);
    Ok(())
}

fn sync_report_summary(state: &mut crate::ReviewWorkspaceState) {
    state.supervision.checks.check_run_id = state.checks.check_run_id.clone();
    state.supervision.checks.model_revision = state.checks.model_revision.clone();
    state.supervision.checks.profile_id = state.checks.profile_id.clone();
    state.supervision.checks.status = state.checks.status.clone();
    state.supervision.checks.finding_count = state.checks.finding_count;
    state.supervision.checks.proposal_ref_count = state.checks.proposal_refs.len();
}

fn admit_projection(state: &crate::ReviewWorkspaceState, run: &CheckRunReviewState) -> Result<()> {
    anyhow::ensure!(
        !run.persisted
            && run.profile_id.as_deref() == Some("erc")
            && run.project_id.as_deref() == Some(state.scene.project_uuid.as_str())
            && run.model_revision.as_deref() == Some(state.scene.source_revision.as_str()),
        "Read-only ERC projection has a foreign source basis"
    );
    Ok(())
}

fn current_persisted(state: &crate::ReviewWorkspaceState) -> bool {
    state.checks.persisted
        && state.checks.profile_id.as_deref() == Some("erc")
        && state.checks.project_id.as_deref() == Some(state.scene.project_uuid.as_str())
        && state.checks.model_revision.as_deref() == Some(state.scene.source_revision.as_str())
}
/// Carry an existing read-only projection across Save or metadata-only refresh.
/// A changed source revision invalidates it; a new current persisted run wins.
pub fn retain_current_check_projection(
    state: &mut crate::ReviewWorkspaceState,
    previous: &CheckRunReviewState,
) {
    if !current_persisted(state) && admit_projection(state, previous).is_ok() {
        state.checks = previous.clone();
        sync_report_summary(state);
    }
}
