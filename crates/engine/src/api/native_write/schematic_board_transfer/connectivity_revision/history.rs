//! Full immutable receipts authenticated against real canonical journal history.
use super::{invalid, validate_recorded};
use crate::{
    error::EngineError,
    substrate::{
        CommitSource, DesignModel, Operation, Proposal, ProposalStatus, TransactionKind,
        TransactionRecord,
    },
};
use std::collections::BTreeSet;

fn draft_form(proposal: &Proposal) -> Proposal {
    let mut draft = proposal.clone();
    draft.status = ProposalStatus::Draft;
    draft.applied_transaction_id = None;
    draft
}

fn applied(
    transaction: &TransactionRecord,
    before: &Proposal,
    after: &Proposal,
    metadata: &Operation,
) -> Result<(), EngineError> {
    let mut expected = before.clone();
    expected.status = ProposalStatus::Applied;
    expected.applied_transaction_id = Some(transaction.transaction_id);
    let mut operations: Vec<_> = before
        .batch
        .operations
        .iter()
        .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
        .cloned()
        .collect();
    operations.push(metadata.clone());
    let mut inverse = Vec::new();
    for operation in operations.iter().rev() {
        inverse.push(match operation {
            Operation::SetElectricalIdentity { previous, record } => {
                Operation::SetElectricalIdentity {
                    previous: Box::new(record.clone()),
                    record: previous.as_ref().clone(),
                }
            }
            Operation::SetProposalMetadata {
                proposal_id,
                relative_path,
                previous_proposal,
                proposal,
            } => Operation::SetProposalMetadata {
                proposal_id: *proposal_id,
                relative_path: relative_path.clone(),
                previous_proposal: proposal.clone(),
                proposal: previous_proposal.clone(),
            },
            _ => return Err(invalid("Applied receipt includes an unrelated operation")),
        });
    }
    if before.status != ProposalStatus::Accepted
        || before.applied_transaction_id.is_some()
        || expected != *after
        || transaction.batch_id != before.batch.batch_id
        || transaction.provenance != before.batch.provenance
        || transaction.before_model_revision != before.prepared_against
        || transaction.operations != operations
        || transaction.inverse_operations != inverse
    {
        return Err(invalid(
            "Applied receipt, design operations or inverse differ from its Accepted source",
        ));
    }
    Ok(())
}

fn compensation(
    prior: &[TransactionRecord],
    transaction: &TransactionRecord,
) -> Result<(), EngineError> {
    let target = match transaction.transaction_kind {
        TransactionKind::Undo if transaction.redo_of.is_none() => transaction.undo_of,
        TransactionKind::Redo if transaction.undo_of.is_none() => transaction.redo_of,
        _ => None,
    }
    .and_then(|id| prior.iter().find(|tx| tx.transaction_id == id))
    .ok_or_else(|| invalid("Receipt compensation lacks its actual earlier transaction"))?;
    // This family changes only relationship records and its own receipt. It has
    // no component/library epoch rebasing and cannot use that history grammar.
    if transaction.operations != target.inverse_operations
        || transaction.inverse_operations != target.operations
    {
        return Err(invalid(
            "Receipt compensation differs from its exact canonical inverse",
        ));
    }
    Ok(())
}

/// Also admits a genuinely new Draft, before its metadata has been committed.
/// Existing IDs always require their complete recorded immutable origin.
pub(super) fn authenticate(model: &DesignModel, proposal: &Proposal) -> Result<(), EngineError> {
    let id = proposal.proposal_id;
    let path = format!(".datum/proposals/{id}.json");
    let mut origin: Option<Proposal> = None;
    let mut current: Option<Proposal> = None;
    let mut applied_ids = BTreeSet::new();
    for (index, tx) in model.journal.iter().enumerate() {
        let metadata: Vec<_> = tx
            .operations
            .iter()
            .filter(|op| match op {
                Operation::CreateProposalMetadata { proposal_id, .. }
                | Operation::SetProposalMetadata { proposal_id, .. }
                | Operation::DeleteProposalMetadata { proposal_id, .. } => *proposal_id == id,
                _ => false,
            })
            .collect();
        if metadata.is_empty() {
            continue;
        }
        let [operation] = metadata.as_slice() else {
            return Err(invalid(
                "Receipt has multiple metadata writes in one transaction",
            ));
        };
        if tx.transaction_kind != TransactionKind::Normal {
            compensation(&model.journal[..index], tx)?;
        } else if tx.undo_of.is_some()
            || tx.redo_of.is_some()
            || tx.provenance.source != CommitSource::Manual
        {
            return Err(invalid(
                "Receipt origin or review has invalid provenance/history links",
            ));
        }
        match operation {
            Operation::CreateProposalMetadata {
                relative_path,
                proposal: value,
                ..
            } => {
                let draft: Proposal = serde_json::from_value(value.clone())?;
                validate_recorded(&draft)?;
                if relative_path != &path
                    || current.is_some()
                    || draft.proposal_id != id
                    || draft.status != ProposalStatus::Draft
                    || draft.applied_transaction_id.is_some()
                    || tx.operations.len() != 1
                {
                    return Err(invalid("Receipt did not originate as one complete Draft"));
                }
                if tx.transaction_kind == TransactionKind::Normal {
                    if origin.replace(draft.clone()).is_some() {
                        return Err(invalid("Receipt ID has more than one normal Draft origin"));
                    }
                } else if origin.as_ref() != Some(&draft) {
                    return Err(invalid("Restored Draft differs from its actual origin"));
                }
                current = Some(draft);
            }
            Operation::SetProposalMetadata {
                relative_path,
                previous_proposal,
                proposal: value,
                ..
            } => {
                let before: Proposal = serde_json::from_value(previous_proposal.clone())?;
                let after: Proposal = serde_json::from_value(value.clone())?;
                if relative_path != &path
                    || current.as_ref() != Some(&before)
                    || origin.as_ref() != Some(&draft_form(&after))
                    || (after.status == ProposalStatus::Applied)
                        != after.applied_transaction_id.is_some()
                {
                    return Err(invalid(
                        "Receipt changed its immutable Draft payload or metadata preimage",
                    ));
                }
                if tx.transaction_kind == TransactionKind::Normal {
                    if after.status == ProposalStatus::Applied {
                        applied(tx, &before, &after, operation)?;
                        applied_ids.insert(tx.transaction_id);
                    } else if before.status != ProposalStatus::Draft
                        || !matches!(
                            after.status,
                            ProposalStatus::Accepted
                                | ProposalStatus::Deferred
                                | ProposalStatus::Rejected
                        )
                        || tx.operations.len() != 1
                    {
                        return Err(invalid(
                            "Receipt review is not an ordinary Draft status transition",
                        ));
                    }
                }
                current = Some(after);
            }
            Operation::DeleteProposalMetadata {
                relative_path,
                proposal: value,
                ..
            } => {
                let previous: Proposal = serde_json::from_value(value.clone())?;
                if relative_path != &path
                    || current.as_ref() != Some(&previous)
                    || tx.transaction_kind == TransactionKind::Normal
                {
                    return Err(invalid(
                        "Receipt deletion is not exact history compensation",
                    ));
                }
                current = None;
            }
            _ => unreachable!(),
        }
    }
    if let Some(recorded) = current {
        if recorded != *proposal
            || model.proposals.get(&id) != Some(proposal)
            || (proposal.status == ProposalStatus::Applied
                && !proposal
                    .applied_transaction_id
                    .is_some_and(|id| applied_ids.contains(&id)))
        {
            return Err(invalid(
                "Current receipt differs from its complete canonical history",
            ));
        }
    } else if origin.is_some()
        || model.proposals.contains_key(&id)
        || proposal.status != ProposalStatus::Draft
        || proposal.applied_transaction_id.is_some()
    {
        return Err(invalid(
            "Receipt is missing its required Draft or Applied history",
        ));
    }
    Ok(())
}
