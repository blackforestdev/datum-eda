//! Canonical native writer request ownership; source classification preserves proposal policy.
use super::dispatch::operation_kind;
use super::*;
use eda_engine::api::native_write::{
    WriteProvenance,
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::CommitSource;
/// `native.write`: verb-addressed native mutation through the engine's
/// native-write verb registry. Resolves the project per request, enforces the
/// optional `expected_model_revision` stale guard, builds via the registered
/// verb, then either previews (`dry_run`) or commits through the one
/// journaled commit path.
///
/// Error codes: -32602 invalid params (bad source, empty reason),
/// -32060 engine resolve/build/commit failure, -32061 stale
/// `expected_model_revision`, -32062 unknown verb.
pub(super) fn native_write_response(
    engine: &mut Engine,
    request_id: Value,
    params: NativeWriteParams,
) -> JsonRpcResponse {
    let reason = params.reason.trim().to_string();
    if reason.is_empty() {
        return error_response(
            request_id,
            -32602,
            "invalid params: native.write requires a non-empty reason",
        );
    }
    let source = match params.source.as_deref() {
        None | Some("tool") => CommitSource::Tool,
        Some("assistant") => CommitSource::Assistant,
        Some("manual") => CommitSource::Manual,
        Some(other) => {
            return error_response(
                request_id,
                -32602,
                &format!(
                    "invalid params: native.write source must be \"tool\", \"assistant\" or \"manual\", got \"{other}\""
                ),
            );
        }
    };
    let actor = params
        .actor
        .clone()
        .unwrap_or_else(|| "datum-eda-daemon".to_string());

    let mut model = match engine.read_native_project(&params.project_root) {
        Ok(model) => std::sync::Arc::unwrap_or_clone(model),
        Err(err) => return error_response(request_id, -32060, &err.to_string()),
    };
    if let Some(expected) = params.expected_model_revision.as_deref()
        && expected != model.model_revision.0
    {
        return error_response(
            request_id,
            -32061,
            &format!(
                "stale expected_model_revision: expected {expected}, current {}",
                model.model_revision.0
            ),
        );
    }
    if matches!(
        params.verb.as_str(),
        "datum.pcb.move_component"
            | "datum.journal.undo"
            | "datum.journal.redo"
            | "datum.schematic.place_symbol"
            | "datum.schematic.draw_wire"
    ) && params.expected_model_revision.is_none()
    {
        return error_response(
            request_id,
            -32602,
            "native board editing requires expected_model_revision",
        );
    }
    if matches!(
        params.verb.as_str(),
        "datum.pcb.move_component"
            | "datum.journal.undo"
            | "datum.journal.redo"
            | "datum.schematic.place_symbol"
            | "datum.schematic.draw_wire"
    ) && eda_engine::revision::transaction_tip(&model.journal) != params.expected_transaction_tip
    {
        return error_response(
            request_id,
            -32061,
            "stale expected transaction tip; native edit refused",
        );
    }
    if matches!(
        params.verb.as_str(),
        "datum.journal.undo" | "datum.journal.redo"
    ) {
        return super::native_history::history_response(
            request_id,
            params,
            model,
            WriteProvenance::new(actor, source, reason),
        );
    }
    let Some(verb) = find_native_write_verb(&params.verb) else {
        return error_response(
            request_id,
            -32062,
            &format!("unknown native write verb: {}", params.verb),
        );
    };

    let provenance = WriteProvenance::new(actor, source, reason);
    let mut prepared = {
        let context = NativeWriteContext {
            model: &model,
            project_root: &params.project_root,
        };
        match (verb.build)(&context, provenance, params.params) {
            Ok(prepared) => prepared,
            Err(err) => return error_response(request_id, -32060, &err.to_string()),
        }
    };

    if params.dry_run {
        let operation_kinds: Vec<String> = prepared
            .batch
            .operations
            .iter()
            .map(operation_kind)
            .collect();
        return success_response(
            request_id,
            json!({
                "verb": params.verb,
                "status": "dry_run",
                "operation_kinds": operation_kinds,
                "operation_count": prepared.batch.operations.len(),
                "primary_object_id": prepared.primary_object_id,
                "expected_model_revision": model.model_revision.0,
            }),
        );
    }

    if let Some(batch_id) = params.batch_id {
        prepared.batch.batch_id = batch_id;
    }
    let primary_object_id = prepared.primary_object_id;
    let operation_count = prepared.batch.operations.len();
    match engine.commit_native_prepared(&mut model, &params.project_root, prepared) {
        Ok(report) => success_response(
            request_id,
            json!({
                "verb": params.verb,
                "status": "applied",
                "transaction_id": report.transaction.transaction_id,
                "before_model_revision": report.transaction.before_model_revision.0,
                "after_model_revision": report.transaction.after_model_revision.0,
                "operation_count": operation_count,
                "primary_object_id": primary_object_id,
                "journal_len": report.journal_len,
            }),
        ),
        Err(err) => error_response(request_id, -32060, &err.to_string()),
    }
}
