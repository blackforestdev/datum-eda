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
