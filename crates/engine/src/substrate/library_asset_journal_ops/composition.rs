//! Derive portable occurrence records from one real, unpublished canonical
//! evaluation. Final commit independently evaluates and admits the complete batch.
use crate::{
    error::EngineError,
    library_assets::encode_metadata,
    revision::ProjectWriteLease,
    substrate::{
        DesignModel, Operation, OperationBatch, TransactionKind,
        commit::refresh_journaled_report_revision,
        journal::{
            inverse_operations_for_batch, stage_operation_shard_writes, update_staged_source_hashes,
        },
        library_asset_operations::{LibraryAssetOperation, LibraryStateValues},
        library_asset_resolver::{read_state, server_policy},
    },
};
use std::path::Path;

pub(in crate::substrate) fn complete(
    root: &Path,
    lease: &ProjectWriteLease,
    before: &DesignModel,
    mut batch: OperationBatch,
    kind: TransactionKind,
) -> Result<OperationBatch, EngineError> {
    // Explicit adoption and compensation already carry their exact reviewed
    // state. Never repair a caller-supplied transition or a historical inverse.
    if before.project.schema_version != Some(2)
        || kind != TransactionKind::Normal
        || batch
            .operations
            .iter()
            .any(|op| matches!(op, Operation::LibraryAssets { .. }))
        || !batch.operations.iter().any(|op| {
            matches!(
                op,
                Operation::CreateComponentInstance { .. }
                    | Operation::SetComponentInstance { .. }
                    | Operation::DeleteComponentInstance { .. }
            )
        })
    {
        return Ok(batch);
    }
    lease.validate(root)?;
    let limits = server_policy()?;
    let previous = read_state(root, limits)?;
    let (mut lock, _) = previous.decode(limits.metadata_bytes)?;
    if lock.assets.is_empty() {
        return Ok(batch);
    }
    let admission = super::admission::prepare_under_lease(root, lease, before, &batch, limits)?
        .ok_or_else(|| super::refusal("managed binding composition requires admission"))?;
    let writes = stage_operation_shard_writes(root, before, &batch)?;
    let inverse = inverse_operations_for_batch(before, &batch)?;
    let mut candidate = before.clone();
    update_staged_source_hashes(&mut candidate.source_shards, &writes)?;
    crate::substrate::electrical_transaction::retain_preimage_hashes(before, &mut candidate);
    let mut report = candidate.apply_library_candidate(batch.clone(), &admission)?;
    update_staged_source_hashes(&mut candidate.source_shards, &writes)?;
    report.transaction.inverse_operations = inverse.clone();
    candidate.journal.last_mut().unwrap().inverse_operations = inverse;
    refresh_journaled_report_revision(&mut candidate, &mut report, None)?;
    let bindings = admission.derive_component_bindings(root, lease, before, &candidate, &writes)?;
    if bindings == lock.bindings {
        return Ok(batch);
    }
    lock.bindings = bindings;
    let state = LibraryStateValues {
        lock: encode_metadata(&lock)?,
        catalog: previous.catalog.clone(),
    };
    batch.operations.push(Operation::LibraryAssets {
        operation: LibraryAssetOperation::SetState {
            project_id: before.project.project_id,
            previous,
            state,
            content: Vec::new(),
        },
    });
    Ok(batch)
}
