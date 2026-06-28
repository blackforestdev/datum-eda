use std::path::Path;

use uuid::Uuid;

use super::{
    CommitDiff, CommitReport, DesignModel, EngineError, JournalCursor, ModelRevision, Operation,
    OperationBatch, TransactionKind, TransactionRecord, compute_model_revision,
    journal::{
        append_transaction_journal, inverse_operations_for_batch, promote_staged_shard_writes,
        sort_source_shards, stage_operation_shard_writes, update_staged_source_hashes,
        write_journal_cursor,
    },
    operation_application_batch::apply_operations,
    proposal_policy::{direct_commit_proposal_policy_blockers, format_proposal_policy_blockers},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CommitPolicyContext {
    Direct,
    AcceptedProposalApply,
}

impl DesignModel {
    pub fn commit(&mut self, batch: OperationBatch) -> Result<CommitReport, EngineError> {
        validate_direct_commit_proposal_policy(
            &batch,
            TransactionKind::Normal,
            CommitPolicyContext::Direct,
        )?;
        self.commit_without_direct_policy(batch)
    }

    pub(super) fn commit_without_direct_policy(
        &mut self,
        batch: OperationBatch,
    ) -> Result<CommitReport, EngineError> {
        validate_non_empty_operation_batch(&batch)?;
        validate_object_revision_guards(self, &batch.operations)?;
        let batch = batch_without_object_revision_guards(batch)?;
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
        self.commit_journaled_with_links(
            project_root,
            batch,
            TransactionKind::Normal,
            None,
            None,
            CommitPolicyContext::Direct,
        )
    }

    pub(super) fn commit_journaled_accepted_proposal_apply(
        &mut self,
        project_root: &Path,
        batch: OperationBatch,
    ) -> Result<CommitReport, EngineError> {
        self.commit_journaled_with_links(
            project_root,
            batch,
            TransactionKind::Normal,
            None,
            None,
            CommitPolicyContext::AcceptedProposalApply,
        )
    }

    pub(super) fn commit_journaled_with_links(
        &mut self,
        project_root: &Path,
        batch: OperationBatch,
        transaction_kind: TransactionKind,
        undo_of: Option<Uuid>,
        redo_of: Option<Uuid>,
        policy_context: CommitPolicyContext,
    ) -> Result<CommitReport, EngineError> {
        self.commit_journaled_with_links_and_inverse(
            project_root,
            batch,
            transaction_kind,
            undo_of,
            redo_of,
            None,
            policy_context,
            None,
        )
    }

    pub(super) fn commit_journaled_with_links_and_inverse(
        &mut self,
        project_root: &Path,
        batch: OperationBatch,
        transaction_kind: TransactionKind,
        undo_of: Option<Uuid>,
        redo_of: Option<Uuid>,
        inverse_operations_override: Option<Vec<super::Operation>>,
        policy_context: CommitPolicyContext,
        after_model_revision_override: Option<ModelRevision>,
    ) -> Result<CommitReport, EngineError> {
        validate_direct_commit_proposal_policy(&batch, transaction_kind, policy_context)?;
        validate_non_empty_operation_batch(&batch)?;
        validate_object_revision_guards(self, &batch.operations)?;
        let batch = batch_without_object_revision_guards(batch)?;
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

        let inverse_operations = match inverse_operations_override {
            Some(operations) => operations,
            None => inverse_operations_for_batch(self, &batch)?,
        };
        let staged_writes = stage_operation_shard_writes(project_root, self, &batch)?;
        let mut committed = self.clone();
        update_staged_source_hashes(&mut committed.source_shards, &staged_writes)?;
        sort_source_shards(&mut committed.source_shards);
        let mut report = committed.commit_without_direct_policy(batch)?;
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
        refresh_journaled_report_revision(
            &mut committed,
            &mut report,
            after_model_revision_override,
        )?;
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

fn refresh_journaled_report_revision(
    committed: &mut DesignModel,
    report: &mut CommitReport,
    after_model_revision_override: Option<ModelRevision>,
) -> Result<(), EngineError> {
    sort_source_shards(&mut committed.source_shards);
    committed.model_revision = after_model_revision_override.unwrap_or_else(|| {
        compute_model_revision(
            &committed.project.project_id,
            &committed.source_shards,
            &committed.objects,
        )
    });
    let transaction_id = Uuid::new_v5(
        &committed.project.project_id,
        format!(
            "datum-eda:transaction:{}:{}",
            report.transaction.batch_id, committed.model_revision.0
        )
        .as_bytes(),
    );
    report.transaction.transaction_id = transaction_id;
    report.transaction.after_model_revision = committed.model_revision.clone();
    if let Some(transaction) = committed.journal.last_mut() {
        transaction.transaction_id = report.transaction.transaction_id;
        transaction.after_model_revision = report.transaction.after_model_revision.clone();
    }
    Ok(())
}

fn validate_direct_commit_proposal_policy(
    batch: &OperationBatch,
    transaction_kind: TransactionKind,
    policy_context: CommitPolicyContext,
) -> Result<(), EngineError> {
    if transaction_kind != TransactionKind::Normal
        || policy_context == CommitPolicyContext::AcceptedProposalApply
    {
        return Ok(());
    }

    let blockers = direct_commit_proposal_policy_blockers(batch);
    if blockers.is_empty() {
        return Ok(());
    }

    Err(EngineError::Validation(format!(
        "commit violates proposal policy: {}",
        format_proposal_policy_blockers(&blockers)
    )))
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

fn validate_object_revision_guards(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<(), EngineError> {
    for operation in operations {
        if let Operation::GuardObjectRevision {
            object_id,
            expected_object_revision,
        } = operation
        {
            let object = model.objects.get(object_id).ok_or(EngineError::NotFound {
                object_type: "domain_object",
                uuid: *object_id,
            })?;
            if object.object_revision != *expected_object_revision {
                return Err(EngineError::Operation(format!(
                    "object revision mismatch for {}: expected {}, current {}",
                    object_id, expected_object_revision.0, object.object_revision.0
                )));
            }
        }
    }
    Ok(())
}

fn batch_without_object_revision_guards(
    mut batch: OperationBatch,
) -> Result<OperationBatch, EngineError> {
    batch
        .operations
        .retain(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }));
    validate_non_empty_operation_batch(&batch)?;
    Ok(batch)
}
