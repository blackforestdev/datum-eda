use std::path::Path;

use uuid::Uuid;

use super::{
    CommitDiff, CommitReport, DesignModel, EngineError, JournalCursor, OperationBatch,
    TransactionKind, TransactionRecord, compute_model_revision,
    journal::{
        append_transaction_journal, inverse_operations_for_batch, promote_staged_shard_writes,
        sort_source_shards, stage_operation_shard_writes, update_staged_source_hashes,
        write_journal_cursor,
    },
    operation_application_batch::apply_operations,
};

impl DesignModel {
    pub fn commit(&mut self, batch: OperationBatch) -> Result<CommitReport, EngineError> {
        validate_non_empty_operation_batch(&batch)?;
        if let Some(expected) = &batch.expected_model_revision {
            if expected != &self.model_revision {
                return Err(EngineError::Operation(format!(
                    "model revision mismatch: expected {}, current {}",
                    expected.0, self.model_revision.0
                )));
            }
        }

        let before_model_revision = self.model_revision.clone();
        let mut diff = CommitDiff::default();
        apply_operations(self, &batch.operations, &mut diff)?;
        diff.modified.sort();
        diff.modified.dedup();
        self.model_revision =
            compute_model_revision(&self.project.project_id, &self.source_shards, &self.objects);
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

    pub fn commit_journaled(
        &mut self,
        project_root: &Path,
        batch: OperationBatch,
    ) -> Result<CommitReport, EngineError> {
        self.commit_journaled_with_links(project_root, batch, TransactionKind::Normal, None, None)
    }

    pub(super) fn commit_journaled_with_links(
        &mut self,
        project_root: &Path,
        batch: OperationBatch,
        transaction_kind: TransactionKind,
        undo_of: Option<Uuid>,
        redo_of: Option<Uuid>,
    ) -> Result<CommitReport, EngineError> {
        validate_non_empty_operation_batch(&batch)?;
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(EngineError::Operation(format!(
                "journal cursor mismatch: applied {} transaction(s), journal has {}",
                self.journal_cursor.applied_transaction_count,
                self.journal.len()
            )));
        }
        let Some(expected) = &batch.expected_model_revision else {
            return Err(EngineError::Operation(
                "journaled commit requires expected_model_revision".to_string(),
            ));
        };
        if expected != &self.model_revision {
            return Err(EngineError::Operation(format!(
                "model revision mismatch: expected {}, current {}",
                expected.0, self.model_revision.0
            )));
        }

        let inverse_operations = inverse_operations_for_batch(self, &batch)?;
        let staged_writes = stage_operation_shard_writes(project_root, self, &batch)?;
        let mut committed = self.clone();
        update_staged_source_hashes(&mut committed.source_shards, &staged_writes);
        sort_source_shards(&mut committed.source_shards);
        let mut report = committed.commit(batch)?;
        report.transaction.transaction_kind = transaction_kind;
        report.transaction.undo_of = undo_of;
        report.transaction.redo_of = redo_of;
        report.transaction.inverse_operations = inverse_operations;
        if let Some(transaction) = committed.journal.last_mut() {
            transaction.transaction_kind = report.transaction.transaction_kind;
            transaction.undo_of = report.transaction.undo_of;
            transaction.redo_of = report.transaction.redo_of;
            transaction.inverse_operations = report.transaction.inverse_operations.clone();
        }
        append_transaction_journal(project_root, &report.transaction)?;
        promote_staged_shard_writes(staged_writes)?;
        committed.journal_cursor = JournalCursor {
            applied_transaction_count: report.journal_len,
        };
        write_journal_cursor(project_root, &committed.journal_cursor)?;
        *self = committed;
        Ok(report)
    }
}

fn validate_non_empty_operation_batch(batch: &OperationBatch) -> Result<(), EngineError> {
    if batch.operations.is_empty() {
        return Err(EngineError::Operation(format!(
            "operation batch {} has no operations",
            batch.batch_id
        )));
    }
    Ok(())
}
