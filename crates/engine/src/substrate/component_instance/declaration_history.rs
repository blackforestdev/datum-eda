//! Shared authentication of effective canonical declaration transactions.
use super::super::{DesignModel, Operation, TransactionKind};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Placed Entity source: {message}"))
}
pub(super) struct EffectiveHistory {
    pub origins: BTreeMap<Uuid, Uuid>,
    pub effective: BTreeSet<Uuid>,
    pub positions: BTreeMap<Uuid, usize>,
}
pub(super) fn effective_history(model: &DesignModel) -> Result<EffectiveHistory, EngineError> {
    effective_journal_history(&model.journal)
}
fn effective_journal_history(
    journal: &[TransactionRecord],
) -> Result<EffectiveHistory, EngineError> {
    super::super::journal_history::targets(journal).map_err(EngineError::Operation)?;
    let mut origins = BTreeMap::new();
    let mut effective = BTreeSet::new();
    let mut positions = BTreeMap::new();
    for (index, tx) in journal.iter().enumerate() {
        let origin = match tx.transaction_kind {
            TransactionKind::Normal => tx.transaction_id,
            TransactionKind::Undo | TransactionKind::Redo => {
                let link = tx
                    .undo_of
                    .or(tx.redo_of)
                    .ok_or_else(|| invalid("compensation origin is unavailable"))?;
                *origins
                    .get(&link)
                    .ok_or_else(|| invalid("compensation origin is missing"))?
            }
        };
        origins.insert(tx.transaction_id, origin);
        positions.insert(tx.transaction_id, index);
        if tx.transaction_kind == TransactionKind::Undo {
            effective.remove(&origin);
        } else {
            effective.insert(origin);
        }
    }
    Ok(EffectiveHistory {
        origins,
        effective,
        positions,
    })
}

pub(in crate::substrate) fn assignment_origin<'a>(
    target: &TransactionRecord,
    prior: &'a [TransactionRecord],
) -> Option<&'a TransactionRecord> {
    if !prior.iter().any(|tx| tx == target) {
        return None;
    }
    let history = effective_journal_history(prior).ok()?;
    let origin = history.origins.get(&target.transaction_id)?;
    prior
        .iter()
        .find(|tx| tx.transaction_id == *origin && tx.transaction_kind == TransactionKind::Normal)
}

// Use the existing durable accepted Proposal receipt, never a new classifier or
// a hand-written identity-changing Set, as evidence of explicit reassignment.
pub(super) fn authenticate_assignment(
    model: &DesignModel,
    tx: &TransactionRecord,
    placed: Uuid,
) -> Result<(), EngineError> {
    let proposal = assignment_receipt(tx, placed)?;
    if proposal.project_id != model.project.project_id {
        return Err(invalid("captured assignment belongs to another Project"));
    }
    Ok(())
}

pub(in crate::substrate) fn assignment_receipt(
    tx: &TransactionRecord,
    placed: Uuid,
) -> Result<super::super::Proposal, EngineError> {
    let receipts: Vec<_> = tx
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetProposalMetadata {
                proposal_id,
                relative_path,
                previous_proposal,
                proposal,
            } => Some((*proposal_id, relative_path, previous_proposal, proposal)),
            _ => None,
        })
        .collect();
    let [(id, path, previous, raw)] = receipts.as_slice() else {
        return Err(invalid(
            "captured assignment has no unique applied Proposal receipt",
        ));
    };
    let proposal: super::super::Proposal = serde_json::from_value((*raw).clone())?;
    let previous: super::super::Proposal = serde_json::from_value((*previous).clone())?;
    let expected_path = format!(".datum/proposals/{id}.json");
    if proposal.schema_version != 1
        || previous.schema_version != 1
        || previous.project_id != proposal.project_id
        || proposal.proposal_id != *id
        || **path != expected_path
        || proposal.status != super::super::ProposalStatus::Applied
        || proposal.applied_transaction_id != Some(tx.transaction_id)
        || previous.status != super::super::ProposalStatus::Accepted
        || previous.proposal_id != *id
        || previous.batch != proposal.batch
    {
        return Err(invalid(
            "captured assignment Proposal identity or applied lineage differs",
        ));
    }
    let design: Vec<_> = tx
        .operations
        .iter()
        .filter(|op| {
            !matches!(
                op,
                Operation::SetProposalMetadata { .. } | Operation::GuardObjectRevision { .. }
            )
        })
        .collect();
    let recorded: Vec<_> = proposal
        .batch
        .operations
        .iter()
        .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
        .collect();
    if design != recorded
        || !recorded.iter().any(|op| match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            }
            | Operation::SetComponentInstance {
                component_instance, ..
            } => component_instance["placed_symbol_refs"]
                .as_array()
                .is_some_and(|members| {
                    members.iter().any(|r| {
                        r["object_id"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(placed)
                    })
                }),
            _ => false,
        })
    {
        return Err(invalid(
            "captured assignment design batch differs from its recorded Proposal",
        ));
    }
    Ok(proposal)
}
