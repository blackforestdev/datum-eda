//! Native journal history, distinct from the imported Engine-session undo stack.
use super::WriteProvenance;
use crate::{
    error::EngineError,
    substrate::{CommitReport, DesignModel},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryGuards {
    pub expected_tip: Uuid,
}

pub fn commit_history(
    model: &mut DesignModel,
    root: &Path,
    guards: HistoryGuards,
    redo: bool,
    provenance: WriteProvenance,
) -> Result<CommitReport, EngineError> {
    if crate::revision::transaction_tip(&model.journal) != Some(guards.expected_tip) {
        return Err(EngineError::Operation(
            "native history refused: stale transaction tip".into(),
        ));
    }
    // The canonical commit reacquires and validates this exact model/tip against
    // a fresh persisted preimage before publishing the compensation.
    if redo {
        model.commit_journal_redo(root, provenance.into())
    } else {
        model.commit_journal_undo(root, provenance.into())
    }
}
