use std::path::Path;

use uuid::Uuid;

use super::{
    CommitProvenance, CommitReport, DesignModel, EngineError, OperationBatch, TransactionKind,
    TransactionRecord,
};

impl DesignModel {
    pub fn commit_journal_undo(
        &mut self,
        project_root: &Path,
        provenance: CommitProvenance,
    ) -> Result<CommitReport, EngineError> {
        let target = self
            .journal
            .last()
            .cloned()
            .ok_or_else(|| EngineError::Operation("cannot undo empty journal".to_string()))?;
        if target.transaction_kind == TransactionKind::Undo {
            return Err(EngineError::Operation(
                "last journal transaction is already an undo; use redo".to_string(),
            ));
        }
        self.commit_compensating_transaction(
            project_root,
            &target,
            TransactionKind::Undo,
            provenance,
        )
    }

    pub fn commit_journal_redo(
        &mut self,
        project_root: &Path,
        provenance: CommitProvenance,
    ) -> Result<CommitReport, EngineError> {
        let target = self
            .journal
            .last()
            .cloned()
            .ok_or_else(|| EngineError::Operation("cannot redo empty journal".to_string()))?;
        if target.transaction_kind != TransactionKind::Undo {
            if target.transaction_kind == TransactionKind::Normal
                && self
                    .journal
                    .iter()
                    .rev()
                    .skip(1)
                    .any(|transaction| transaction.transaction_kind == TransactionKind::Undo)
            {
                return Err(EngineError::Operation(
                    "redo stack was cleared by a newer normal transaction".to_string(),
                ));
            }
            return Err(EngineError::Operation(
                "last journal transaction is not an undo; nothing to redo".to_string(),
            ));
        }
        self.commit_compensating_transaction(
            project_root,
            &target,
            TransactionKind::Redo,
            provenance,
        )
    }

    fn commit_compensating_transaction(
        &mut self,
        project_root: &Path,
        target: &TransactionRecord,
        transaction_kind: TransactionKind,
        provenance: CommitProvenance,
    ) -> Result<CommitReport, EngineError> {
        if target.inverse_operations.is_empty() {
            return Err(EngineError::Operation(format!(
                "transaction {} has no inverse operations",
                target.transaction_id
            )));
        }
        let batch = OperationBatch {
            batch_id: Uuid::new_v5(
                &self.project.project_id,
                format!(
                    "datum-eda:{}:{}:{}",
                    transaction_kind.as_batch_label(),
                    target.transaction_id,
                    self.model_revision.0
                )
                .as_bytes(),
            ),
            expected_model_revision: Some(self.model_revision.clone()),
            provenance,
            operations: target.inverse_operations.clone(),
        };
        let (undo_of, redo_of) = match transaction_kind {
            TransactionKind::Undo => (Some(target.transaction_id), None),
            TransactionKind::Redo => (None, Some(target.transaction_id)),
            TransactionKind::Normal => (None, None),
        };
        self.commit_journaled_with_links(project_root, batch, transaction_kind, undo_of, redo_of)
    }
}

impl TransactionKind {
    fn as_batch_label(self) -> &'static str {
        match self {
            TransactionKind::Normal => "normal",
            TransactionKind::Undo => "undo",
            TransactionKind::Redo => "redo",
        }
    }
}
