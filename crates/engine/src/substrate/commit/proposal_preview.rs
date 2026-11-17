//! One in-memory batch/report owner; a canonical preview proof never reaches disk commit.
use super::*;
use crate::substrate::{
    component_instance::metadata_history::CanonicalMetadataPreview, operation_application_batch,
};

impl DesignModel {
    pub(in crate::substrate) fn commit_without_direct_policy(
        &mut self,
        batch: OperationBatch,
    ) -> Result<CommitReport, EngineError> {
        self.apply_in_memory_batch(batch, None, None)
    }

    pub(in crate::substrate) fn preview_canonical_metadata_batch(
        &mut self,
        batch: OperationBatch,
        proof: &CanonicalMetadataPreview<'_>,
    ) -> Result<CommitReport, EngineError> {
        self.apply_in_memory_batch(batch, Some(proof), None)
    }

    /// This provisional model must remain local to the journal commit until
    /// finish_evaluated certifies all final sources before publication.
    pub(in crate::substrate) fn apply_library_candidate(
        &mut self,
        batch: OperationBatch,
        admission: &crate::substrate::library_asset_journal_ops::admission::PreparedLibraryAdmission,
    ) -> Result<CommitReport, EngineError> {
        let permit = admission.evaluation_permit(self, &batch)?;
        self.apply_in_memory_batch(batch, None, Some(&permit))
    }

    fn apply_in_memory_batch(
        &mut self,
        batch: OperationBatch,
        preview: Option<&CanonicalMetadataPreview<'_>>,
        library: Option<
            &crate::substrate::library_asset_journal_ops::admission::LibraryEvaluationPermit<'_>,
        >,
    ) -> Result<CommitReport, EngineError> {
        if let Some(proof) = preview {
            proof.validate_input(self, &batch)?;
        }
        validate_non_empty_operation_batch(&batch)?;
        validate_object_revision_guards(self, &batch.operations)?;
        let batch = batch_without_object_revision_guards(batch)?;
        if let Some(expected) = &batch.expected_model_revision
            && expected != &self.model_revision
        {
            return Err(EngineError::Operation(format!(
                "model revision mismatch: expected {}, current {}",
                expected.0, self.model_revision.0
            )));
        }

        let before_model_revision = self.model_revision.clone();
        let mut diff = CommitDiff::default();
        if let Some(permit) = library {
            operation_application_batch::apply_library_candidate(
                self,
                &batch.operations,
                &mut diff,
                permit,
            )?;
        } else if let Some(proof) = preview {
            operation_application_batch::apply_metadata_preview(
                self,
                &batch.operations,
                &mut diff,
                proof,
            )?;
        } else {
            operation_application_batch::apply_operations(self, &batch.operations, &mut diff)?;
        }
        diff.modified.sort();
        diff.modified.dedup();
        self.model_revision =
            crate::substrate::board_batch_basis::authored_revision_after_operations(
                self,
                &before_model_revision,
                &batch.operations,
            );
        let transaction = TransactionRecord {
            transaction_id: Uuid::new_v5(
                &self.project.project_id,
                format!(
                    "datum-eda:transaction:{}:{}",
                    batch.batch_id, self.model_revision.0
                )
                .as_bytes(),
            ),
            batch_id: batch.batch_id,
            transaction_kind: TransactionKind::Normal,
            undo_of: None,
            redo_of: None,
            before_model_revision,
            after_model_revision: self.model_revision.clone(),
            provenance: batch.provenance,
            agent_provenance: agent_commit_provenance_from_environment()?,
            diff,
            operations: batch.operations,
            inverse_operations: Vec::new(),
        };
        self.journal.push(transaction.clone());
        Ok(CommitReport {
            transaction,
            journal_len: self.journal.len(),
        })
    }
}
