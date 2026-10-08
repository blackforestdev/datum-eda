//! Typed native adapter for the already-public component translation verb.
use super::*;
use crate::api::native_write::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveParams {
    component: Uuid,
    x_nm: i64,
    y_nm: i64,
}
pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.pcb.move_component",
    build: build_move,
}];
fn build_move(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: MoveParams = parse_verb_params("datum.pcb.move_component", params)?;
    build_edit_board_package(
        context.model,
        provenance,
        params.component,
        BoardPackageEdit::Position {
            x: params.x_nm,
            y: params.y_nm,
        },
    )
}
