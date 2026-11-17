//! Journaled proposal prediction uses the same identity basis and postimages as apply.
use super::component_instance::metadata_history::CanonicalMetadataPreview;
use super::{CommitReport, DesignModel, EngineError, OperationBatch, TransactionKind};
use std::path::Path;

/// Creation has no published Proposal receipt. Its private proof is minted by
/// exact canonical recomputation; the clone's report is computed only afterward.
pub(super) fn nonwriting_metadata_report(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<Option<CommitReport>, EngineError> {
    let Some(proof) = CanonicalMetadataPreview::capture(model, batch)? else {
        return Ok(None);
    };
    let mut preview = model.clone();
    preview
        .preview_canonical_metadata_batch(batch.clone(), &proof)
        .map(Some)
}

pub(super) fn report(
    model: &DesignModel,
    project_root: &Path,
    batch: &OperationBatch,
) -> Result<CommitReport, EngineError> {
    // Managed previews use the same private lease-qualified composition as
    // commit. The lease ends with this report, before the actual apply lease.
    let managed = model.project.schema_version == Some(2)
        || batch
            .operations
            .iter()
            .any(|op| matches!(op, super::Operation::LibraryAssets { .. }));
    let lease = managed
        .then(|| crate::revision::ProjectWriteLease::acquire(project_root))
        .transpose()?;
    let input = if managed {
        super::commit::validate_object_revision_guards(model, &batch.operations)?;
        super::commit::batch_without_object_revision_guards(batch.clone())?
    } else {
        batch.clone()
    };
    let batch = super::electrical_transaction::compose(model, input, TransactionKind::Normal)?;
    let batch = if let Some(lease) = lease.as_ref() {
        super::library_asset_journal_ops::composition::complete(
            project_root,
            lease,
            model,
            batch,
            TransactionKind::Normal,
        )?
    } else {
        batch
    };
    let admission = if let Some(lease) = lease.as_ref() {
        super::library_asset_journal_ops::admission::prepare_under_lease(
            project_root,
            lease,
            model,
            &batch,
            super::library_asset_resolver::server_policy()?,
        )?
    } else {
        None
    };
    let proof = CanonicalMetadataPreview::capture(model, &batch)?;
    let staged = super::journal::stage_operation_shard_writes(project_root, model, &batch)?;
    let mut preview = model.clone();
    super::journal::update_staged_source_hashes(&mut preview.source_shards, &staged)?;
    super::electrical_transaction::retain_preimage_hashes(model, &mut preview);
    super::journal::sort_source_shards(&mut preview.source_shards);
    let result = if let Some(proof) = proof.as_ref() {
        preview.preview_canonical_metadata_batch(batch.clone(), proof)
    } else if let Some(admission) = admission.as_ref() {
        preview.apply_library_candidate(batch.clone(), admission)
    } else {
        preview.commit_without_direct_policy(batch.clone())
    }
    .and_then(|mut report| {
        super::journal::update_staged_source_hashes(&mut preview.source_shards, &staged)?;
        if admission.is_some() {
            let inverse = super::journal::inverse_operations_for_batch(model, &batch)?;
            report.transaction.inverse_operations = inverse.clone();
            preview.journal.last_mut().unwrap().inverse_operations = inverse;
        }
        super::commit::refresh_journaled_report_revision(&mut preview, &mut report, None)?;
        if let (Some(admission), Some(lease)) = (admission, lease.as_ref()) {
            admission.finish_evaluated(project_root, lease, model, &preview, &staged)?;
        }
        Ok(report)
    });
    // Existing private staged preview ownership, never published source.
    let stage_dir = project_root
        .join(".datum/stage")
        .join(batch.batch_id.to_string());
    match std::fs::remove_dir_all(stage_dir) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {}
    }
    result
}

/// Shared unjournaled composition/application. Callers perform board admission
/// and, for real writes, the direct Proposal policy check before entering.
pub(super) fn apply_unjournaled(
    model: &mut DesignModel,
    batch: OperationBatch,
) -> Result<CommitReport, EngineError> {
    let batch = super::electrical_transaction::compose(model, batch, TransactionKind::Normal)?;
    super::electrical_history::validate(model, &batch.operations, TransactionKind::Normal)?;
    model.commit_without_direct_policy(batch)
}

pub(super) fn unjournaled_report(
    model: &DesignModel,
    batch: OperationBatch,
) -> Result<CommitReport, EngineError> {
    let mut preview = model.clone();
    super::board_source_profile::refuse_unjournaled(&preview, &batch)?;
    apply_unjournaled(&mut preview, batch)
}
