//! Qualified compensating native history through the engine facade.
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance,
        history::{HistoryGuards, commit_history},
    },
    substrate::DesignModel,
};
pub(super) fn history_response(
    id: Value,
    params: NativeWriteParams,
    mut model: DesignModel,
    provenance: WriteProvenance,
) -> JsonRpcResponse {
    if params.dry_run {
        return error_response(
            id,
            -32602,
            "history is an explicit commit; dry_run unavailable",
        );
    }
    let guards: HistoryGuards = match serde_json::from_value(params.params) {
        Ok(guards) => guards,
        Err(error) => return error_response(id, -32602, &error.to_string()),
    };
    match commit_history(
        &mut model,
        &params.project_root,
        guards,
        params.verb == "datum.journal.redo",
        provenance,
    ) {
        Ok(report) => success_response(
            id,
            json!({"status":"applied","verb":params.verb,
            "transaction_id":report.transaction.transaction_id,"batch_id":report.transaction.batch_id,
            "before_model_revision":report.transaction.before_model_revision.0,
            "after_model_revision":report.transaction.after_model_revision.0,"journal_len":report.journal_len}),
        ),
        Err(error) => error_response(id, -32060, &error.to_string()),
    }
}
