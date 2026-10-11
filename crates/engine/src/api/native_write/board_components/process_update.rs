//! Explicit process refresh intent over PM057 complete correspondence and recorded allocation.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

use super::super::{PreparedWrite, WriteProvenance};
use super::pad_materialization::{
    ComponentPadMaterializationRequest, ComponentPadTransition,
    build_materialize_component_pad_sources,
};
use crate::error::EngineError;
use crate::substrate::{DesignModel, Operation, SourceShardKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessUpdateRequest {
    pub materialization: ComponentPadMaterializationRequest,
    pub retirements: BTreeSet<Uuid>,
    pub allocations: BTreeSet<Uuid>,
    pub legacy_references: BTreeSet<Uuid>,
}

pub fn build_process_update(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &ProcessUpdateRequest,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    if crate::substrate::process_operations::capability_from_board(&board)?.is_none() {
        return Err(EngineError::Validation(
            "process update requires separate explicit capability adoption".into(),
        ));
    }
    let prepared =
        build_materialize_component_pad_sources(model, provenance, &request.materialization)?;
    let value = prepared
        .batch
        .operations
        .iter()
        .find_map(|op| match op {
            Operation::SetBoardPackagePackage { materialized, .. } => {
                materialized.get("pad_source_transition")
            }
            _ => None,
        })
        .ok_or_else(|| {
            EngineError::Validation("process update lacks complete transition proof".into())
        })?;
    let proof: ComponentPadTransition = serde_json::from_value(value.clone())?;
    let allocated: BTreeSet<_> = proof.allocations.keys().copied().collect();
    if proof.retirements != request.retirements
        || allocated != request.allocations
        || proof.legacy_references != request.legacy_references
    {
        return Err(EngineError::Validation("process update correspondence is incomplete: every allocation, retirement and legacy reference requires explicit intent".into()));
    }
    Ok(prepared)
}
