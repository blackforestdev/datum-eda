//! Logical transaction stacks derived from the append-only compensation journal.
//! The durable JournalCursor remains an applied-record integrity watermark.
use super::{DesignModel, EngineError, TransactionKind, TransactionRecord};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub(super) struct JournalHistoryTargets<'a> {
    pub(super) undo: Option<&'a TransactionRecord>,
    pub(super) redo: Option<&'a TransactionRecord>,
}
impl JournalHistoryTargets<'_> {
    fn availability(&self) -> (bool, bool) {
        (
            self.undo.is_some_and(|t| !t.inverse_operations.is_empty()),
            self.redo.is_some_and(|t| !t.inverse_operations.is_empty()),
        )
    }
    pub(super) fn target(&self, kind: TransactionKind) -> Option<&TransactionRecord> {
        match kind {
            TransactionKind::Undo => self.undo,
            TransactionKind::Redo => self.redo,
            TransactionKind::Normal => None,
        }
    }
}
impl DesignModel {
    pub fn journal_history_availability(&self) -> Result<(bool, bool), EngineError> {
        self.journal_history_targets()
            .map(|history| history.availability())
    }
    pub(super) fn journal_history_targets(&self) -> Result<JournalHistoryTargets<'_>, EngineError> {
        targets(&self.journal).map_err(EngineError::Operation)
    }
}
/// Validate stack identity/order without interpreting payloads. The existing link
/// validator separately proves each exact semantic inverse and its epoch rebasing.
pub(super) fn targets(journal: &[TransactionRecord]) -> Result<JournalHistoryTargets<'_>, String> {
    let mut undo: Vec<&TransactionRecord> = Vec::new();
    let mut redo: Vec<&TransactionRecord> = Vec::new();
    let mut ids = BTreeSet::new();
    for tx in journal {
        if !ids.insert(tx.transaction_id) {
            return Err("duplicate journal transaction identity".into());
        }
        match tx.transaction_kind {
            TransactionKind::Normal => {
                if tx.undo_of.is_some() || tx.redo_of.is_some() {
                    return Err("normal transaction carries a compensation link".into());
                }
                undo.push(tx);
                redo.clear();
            }
            TransactionKind::Undo => {
                let expected = undo.last().map(|t| t.transaction_id);
                if tx.undo_of.is_none() || tx.undo_of != expected || tx.redo_of.is_some() {
                    return Err("undo link is not the most recent applied transaction".into());
                }
                undo.pop();
                redo.push(tx);
            }
            TransactionKind::Redo => {
                let expected = redo.last().map(|t| t.transaction_id);
                if tx.redo_of.is_none() || tx.redo_of != expected || tx.undo_of.is_some() {
                    return Err("redo link is not the most recent pending undo (or was cleared by a newer normal transaction)".into());
                }
                redo.pop();
                undo.push(tx);
            }
        }
    }
    Ok(JournalHistoryTargets {
        undo: undo.last().copied(),
        redo: redo.last().copied(),
    })
}
pub(super) fn selected_target(
    prior: &[TransactionRecord],
    target: &TransactionRecord,
    kind: TransactionKind,
) -> bool {
    targets(prior).is_ok_and(|history| history.target(kind) == Some(target))
}
pub(super) fn current_target(prior: &[TransactionRecord], target: &TransactionRecord) -> bool {
    selected_target(prior, target, TransactionKind::Undo)
        || selected_target(prior, target, TransactionKind::Redo)
}
