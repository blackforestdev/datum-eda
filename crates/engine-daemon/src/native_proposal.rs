//! GUI-native typed Proposal lifecycle, with the same resolver/journal authority.
use super::*;
use eda_engine::api::native_write::schematic_delete_proposal::{NativeProposalRequest, execute};

pub(super) fn response(engine: &mut Engine, id: Value, params: Value) -> JsonRpcResponse {
    if params.get("expected_transaction_tip").is_none() {
        return error_response(
            id,
            -32602,
            "native.proposal requires explicit expected_transaction_tip (null for an empty journal)",
        );
    }
    let request: NativeProposalRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(error) => return error_response(id, -32602, &error.to_string()),
    };
    let mut model = match engine.read_native_project(&request.project_root) {
        Ok(model) => std::sync::Arc::unwrap_or_clone(model),
        Err(error) => return error_response(id, -32060, &error.to_string()),
    };
    match execute(&mut model, &request) {
        Ok(response) => serialized_success_response(id, response),
        Err(error) => error_response(id, -32060, &error.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/main_tests_native_proposal.rs"]
mod tests;
