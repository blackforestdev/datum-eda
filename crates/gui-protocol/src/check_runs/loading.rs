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
    {
        return Ok(state);
    }
    // Loading/reopening a workspace is read-only. Missing terminal context must
    // never generate a canonical CheckRun transaction. Use the existing history
    // owner's latest record instead; explicit check commands remain separate.
    let history = run_cli_json::<Value>(cli, &["check", "list", &project_root])?;
    let Some(id) = history.get("latest_check_run_id").and_then(Value::as_str) else {
        return Ok(CheckRunReviewState::default());
    };
    let record = run_cli_json::<Value>(cli, &["check", "show", &project_root, "--check-run", id])?;
    check_run_review_state_from_json(&serde_json::to_string(&record)?)
}
