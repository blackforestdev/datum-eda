use std::collections::BTreeSet;

use super::{TransactionKind, TransactionRecord};
mod component_binding;
pub(super) use component_binding::rebase_preimages as rebase_component_preimages;

pub(super) fn validate_transaction_links(
    transaction: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<(), String> {
    let history = super::journal_history::targets(prior)?;
    if prior
        .iter()
        .any(|tx| tx.transaction_id == transaction.transaction_id)
    {
        return Err("duplicate journal transaction identity".into());
    }
    match transaction.transaction_kind {
        TransactionKind::Normal => {
            if transaction.undo_of.is_some() || transaction.redo_of.is_some() {
                return Err("normal transaction must not carry undo_of or redo_of".into());
            }
        }
        kind => {
            let (link, other, label) = match kind {
                TransactionKind::Undo => (transaction.undo_of, transaction.redo_of, "undo"),
                TransactionKind::Redo => (transaction.redo_of, transaction.undo_of, "redo"),
                TransactionKind::Normal => unreachable!(),
            };
            let target = history
                .target(kind)
                .ok_or_else(|| format!("nothing to {label}; history stack is empty"))?;
            if link != Some(target.transaction_id) || other.is_some() {
                return Err(format!(
                    "{label} link does not reference the current {label} target"
                ));
            }
            if target.inverse_operations.is_empty() {
                return Err(format!("{label} target has no inverse operations"));
            }
            validate_compensating_payload(transaction, target, prior, label)?;
        }
    }
    Ok(())
}

fn validate_compensating_payload(
    transaction: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
    label: &str,
) -> Result<(), String> {
    if super::schematic_assignment_history::matches(transaction, target, prior)
        || component_binding::matches(transaction, target, prior)
    {
        return Ok(());
    }
    if transaction.operations != target.inverse_operations
        && !super::board_batch_basis::matches_imported_fill_compensation(transaction, target, prior)
    {
        return Err(format!(
            "{label} operations do not match target inverse operations"
        ));
    }
    if transaction.inverse_operations != target.operations {
        return Err(format!(
            "{label} inverse operations do not match target operations"
        ));
    }
    Ok(())
}
