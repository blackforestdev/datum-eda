use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::substrate::*;
use uuid::Uuid;

use super::command_project_native_inspect::{
    NativeProjectJournalMutationGuardView, NativeProjectJournalMutationView,
    journal_tip_availability,
};
use crate::{OutputFormat, render_output};

use crate::command_project::cli_commit_source;

pub(crate) fn execute_native_project_journal_undo(
    format: &OutputFormat,
    root: &Path,
    expected_model_revision: Option<&str>,
    expected_tip_transaction: Option<Uuid>,
) -> Result<(String, i32)> {
    execute_native_project_journal_mutation(
        format,
        root,
        "undo",
        expected_model_revision,
        expected_tip_transaction,
    )
}

pub(crate) fn execute_native_project_journal_redo(
    format: &OutputFormat,
    root: &Path,
    expected_model_revision: Option<&str>,
    expected_tip_transaction: Option<Uuid>,
) -> Result<(String, i32)> {
    execute_native_project_journal_mutation(
        format,
        root,
        "redo",
        expected_model_revision,
        expected_tip_transaction,
    )
}

fn execute_native_project_journal_mutation(
    format: &OutputFormat,
    root: &Path,
    action: &'static str,
    expected_model_revision: Option<&str>,
    expected_tip_transaction: Option<Uuid>,
) -> Result<(String, i32)> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let project_id = model.project.project_id.to_string();
    let before_model_revision = model.model_revision.0.clone();
    let current_tip_transaction = model
        .journal
        .last()
        .map(|transaction| transaction.transaction_id);
    validate_journal_cursor_health(&model, action)?;
    validate_journal_mutation_guard(
        &model,
        action,
        expected_model_revision,
        expected_tip_transaction,
    )?;
    let cursor_before = model.journal_cursor.applied_transaction_count;
    let provenance = CommitProvenance {
        actor: "datum-eda-cli".to_string(),
        source: cli_commit_source()?,
        reason: format!("journal {action} requested from CLI"),
    };
    let report = match action {
        "undo" => model.commit_journal_undo(root, provenance)?,
        "redo" => model.commit_journal_redo(root, provenance)?,
        _ => unreachable!("journal mutation action should be known"),
    };
    let (can_undo, can_redo) = journal_tip_availability(&model);
    let view = NativeProjectJournalMutationView {
        contract: "project_transaction_journal_mutation_v1",
        action,
        status: "applied",
        project_id,
        before_model_revision: before_model_revision.clone(),
        after_model_revision: report.transaction.after_model_revision.0.clone(),
        journal_path: eda_engine::substrate::JOURNAL_RELATIVE_PATH,
        cursor_path: ".datum/journal/cursor.json",
        journal_len: report.journal_len,
        cursor_before,
        cursor_after: model.journal_cursor.applied_transaction_count,
        guard: NativeProjectJournalMutationGuardView {
            checked: expected_model_revision.is_some() || expected_tip_transaction.is_some(),
            current_model_revision: before_model_revision.clone(),
            expected_model_revision: expected_model_revision.map(str::to_string),
            current_tip_transaction,
            expected_tip_transaction,
        },
        can_undo,
        can_redo,
        transaction: report.transaction,
    };
    Ok((render_output(format, &view), 0))
}

fn validate_journal_cursor_health(model: &DesignModel, action: &str) -> Result<()> {
    const UNHEALTHY_CURSOR_CODES: &[&str] = &[
        "journal_cursor_read_error",
        "journal_cursor_parse_error",
        "journal_cursor_out_of_range",
        "journal_cursor_behind",
    ];
    if let Some(diagnostic) = model
        .diagnostics
        .iter()
        .find(|diagnostic| UNHEALTHY_CURSOR_CODES.contains(&diagnostic.code.as_str()))
    {
        return Err(anyhow!(
            "journal {action} refused: unhealthy journal cursor {}{}",
            diagnostic.code,
            diagnostic
                .path
                .as_ref()
                .map(|path| format!(" at {}", path.display()))
                .unwrap_or_default()
        ));
    }
    Ok(())
}

fn validate_journal_mutation_guard(
    model: &DesignModel,
    action: &str,
    expected_model_revision: Option<&str>,
    expected_tip_transaction: Option<Uuid>,
) -> Result<()> {
    if let Some(expected) = expected_model_revision {
        if expected != model.model_revision.0 {
            return Err(anyhow!(
                "journal {action} refused: expected model revision {}, current {}",
                expected,
                model.model_revision.0
            ));
        }
    }
    if let Some(expected) = expected_tip_transaction {
        let current = model
            .journal
            .last()
            .map(|transaction| transaction.transaction_id);
        if current != Some(expected) {
            let current = current
                .map(|transaction_id| transaction_id.to_string())
                .unwrap_or_else(|| "none".to_string());
            return Err(anyhow!(
                "journal {action} refused: expected tip transaction {}, current {}",
                expected,
                current
            ));
        }
    }
    Ok(())
}
