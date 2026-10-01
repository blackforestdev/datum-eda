//! Journaled proposal prediction uses the same identity basis and postimages as apply.
use super::{CommitReport, DesignModel, EngineError, OperationBatch, TransactionKind};
use std::path::Path;

pub(super) fn report(
    model: &DesignModel,
    project_root: &Path,
    batch: &OperationBatch,
) -> Result<CommitReport, EngineError> {
    let batch =
        super::electrical_transaction::compose(model, batch.clone(), TransactionKind::Normal)?;
    let staged = super::journal::stage_operation_shard_writes(project_root, model, &batch)?;
    let mut preview = model.clone();
    super::journal::update_staged_source_hashes(&mut preview.source_shards, &staged)?;
    super::electrical_transaction::retain_preimage_hashes(model, &mut preview);
    super::journal::sort_source_shards(&mut preview.source_shards);
    let result = preview
        .commit_without_direct_policy(batch.clone())
        .and_then(|mut report| {
            super::journal::update_staged_source_hashes(&mut preview.source_shards, &staged)?;
            super::commit::refresh_journaled_report_revision(&mut preview, &mut report, None)?;
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
