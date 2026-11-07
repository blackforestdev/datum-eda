//! Local visible design placement/transforms through the existing Move verb.
//! Explicit reviewed automation continues through native.proposal; local edits
//! commit one canonical batch without synthetic Proposal metadata history.
use super::{
    context::{PreparedWrite, WriteProvenance},
    design_clipboard_edit::{self, DesignEditIntent},
    registry::{NativeWriteContext, parse_verb_params},
};
use crate::{error::EngineError, substrate::CommitSource};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignMoveParams {
    sheet: Uuid,
    instance_path: Vec<Uuid>,
    design_edit: DesignEditIntent,
}

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    if provenance.source != CommitSource::Manual {
        return Err(EngineError::Validation(
            "Direct design placement and transforms require manual provenance; automation uses a reviewed Proposal".into(),
        ));
    }
    let params: DesignMoveParams = parse_verb_params("datum.schematic.move", params)?;
    match &params.design_edit {
        DesignEditIntent::Cut { .. } => Err(EngineError::Validation(
            "Cut uses the reviewed deletion workflow".into(),
        )),
        DesignEditIntent::PlaceBundle { edit_seed, .. } => {
            // The native.write envelope uses this SAME retained gesture seed
            // as its final batch_id. Establish it before first-birth nomination,
            // never regenerate identity after a request ID overwrites the batch.
            design_clipboard_edit::build_with_id(
                context,
                provenance,
                Some(*edit_seed),
                params.sheet,
                &params.instance_path,
                &params.design_edit,
            )
        }
        DesignEditIntent::TransformExisting { .. } => design_clipboard_edit::build(
            context,
            provenance,
            params.sheet,
            &params.instance_path,
            &params.design_edit,
        ),
    }
}
