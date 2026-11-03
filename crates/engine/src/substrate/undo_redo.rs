use std::path::Path;

use uuid::Uuid;

use super::{
    CommitProvenance, CommitReport, DesignModel, EngineError, Operation, OperationBatch,
    TransactionKind, TransactionRecord,
};
mod component_binding;

impl DesignModel {
    pub fn commit_journal_undo(
        &mut self,
        project_root: &Path,
        provenance: CommitProvenance,
    ) -> Result<CommitReport, EngineError> {
        let target = self
            .journal_history_targets()?
            .undo
            .cloned()
            .ok_or_else(|| EngineError::Operation("nothing to undo".into()))?;
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
            .journal_history_targets()?
            .redo
            .cloned()
            .ok_or_else(|| {
                let reason = if self
                    .journal
                    .last()
                    .is_some_and(|t| t.transaction_kind == TransactionKind::Normal)
                    && self
                        .journal
                        .iter()
                        .any(|t| t.transaction_kind == TransactionKind::Undo)
                {
                    "redo stack was cleared by a newer normal transaction"
                } else {
                    "nothing to redo"
                };
                EngineError::Operation(reason.into())
            })?;
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
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(EngineError::Operation(
                "journal cursor mismatch before compensation".into(),
            ));
        }
        // A caller cannot substitute an old target's payload while retaining its ID
        // and the current tip. The canonical commit rechecks the live basis under lease.
        let durable = super::ProjectResolver::new(project_root).resolve()?;
        if durable.project.project_id != self.project.project_id
            || durable.model_revision != self.model_revision
            || durable.journal != self.journal
            || durable.journal_cursor != self.journal_cursor
            || !super::journal_history::selected_target(&durable.journal, target, transaction_kind)
        {
            return Err(EngineError::Operation(
                "current durable history or compensation target changed".into(),
            ));
        }
        if target.inverse_operations.is_empty() {
            return Err(EngineError::Operation(format!(
                "transaction {} has no inverse operations",
                target.transaction_id
            )));
        }
        let mut batch = OperationBatch {
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
        super::board_batch_basis::bind_imported_fill_history(self, &mut batch)?;
        let current_component_history =
            super::schematic_assignment_history::prepare(self, target, &mut batch)?
                || component_binding::prepare(self, project_root, target, &mut batch)?;
        let (undo_of, redo_of) = match transaction_kind {
            TransactionKind::Undo => (Some(target.transaction_id), None),
            TransactionKind::Redo => (None, Some(target.transaction_id)),
            TransactionKind::Normal => (None, None),
        };
        let after_model_revision =
            if !current_component_history && compensation_needs_revision_override(target) {
                Some(match transaction_kind {
                    TransactionKind::Undo => target.before_model_revision.clone(),
                    TransactionKind::Redo => target.before_model_revision.clone(),
                    TransactionKind::Normal => self.model_revision.clone(),
                })
            } else {
                None
            };
        self.commit_journaled_with_links_and_inverse(
            project_root,
            batch,
            transaction_kind,
            undo_of,
            redo_of,
            if current_component_history {
                None
            } else {
                Some(target.operations.clone())
            },
            super::commit::CommitPolicyContext::Direct,
            after_model_revision,
            None,
            crate::revision::RevisionDesignCommitInput::default(),
        )
    }
}

fn compensation_needs_revision_override(target: &TransactionRecord) -> bool {
    // PM057 source edits advance actual object revisions even when their
    // import-map sidecar is replaced. Metadata compensation cannot stamp the
    // former model hash over that real post-undo source basis.
    if target.operations.iter().any(|op| {
        matches!(
            op,
            Operation::CreateBoardGraphic { .. }
                | Operation::SetBoardGraphic { .. }
                | Operation::DeleteBoardGraphic { .. }
        )
    }) {
        return false;
    }
    // Imported generated copper accompanies real board-source operations.
    // Compensation must retain their actual final revisions, rather than
    // restoring the historical hash used for metadata-only import-map writes.
    if target.operations.iter().chain(&target.inverse_operations).any(|op|
        matches!(op, Operation::SetZoneFill { zone_fill, .. } if
            zone_fill["provenance"].as_str().is_some_and(|p| p.starts_with("datum:kicad-saved-fill:v1;")))) {
        return false;
    }
    target
        .operations
        .iter()
        .chain(target.inverse_operations.iter())
        .any(|operation| {
            matches!(
                operation,
                Operation::AddProjectPoolRef { .. }
                    | Operation::DeleteProjectPoolRef { .. }
                    | Operation::CreatePoolPackage { .. }
                    | Operation::DeletePoolPackage { .. }
                    | Operation::CreatePoolPadstack { .. }
                    | Operation::DeletePoolPadstack { .. }
                    | Operation::CreatePoolLibraryObject { .. }
                    | Operation::DeletePoolLibraryObject { .. }
                    | Operation::CreateImportMapShard { .. }
                    | Operation::DeleteImportMapShard { .. }
            )
        })
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
