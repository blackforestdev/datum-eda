use std::collections::BTreeSet;

use super::{TransactionKind, TransactionRecord};

pub(super) fn validate_transaction_links(
    transaction: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<(), String> {
    let prior_ids = prior
        .iter()
        .map(|transaction| transaction.transaction_id)
        .collect::<BTreeSet<_>>();
    match transaction.transaction_kind {
        TransactionKind::Normal => {
            if transaction.undo_of.is_some() || transaction.redo_of.is_some() {
                return Err("normal transaction must not carry undo_of or redo_of".to_string());
            }
        }
        TransactionKind::Undo => {
            let Some(undo_of) = transaction.undo_of else {
                return Err("undo transaction must carry undo_of".to_string());
            };
            if transaction.redo_of.is_some() {
                return Err("undo transaction must not carry redo_of".to_string());
            }
            if !prior_ids.contains(&undo_of) {
                return Err(format!(
                    "undo_of {undo_of} does not reference a prior transaction"
                ));
            }
            let Some(tip) = prior.last() else {
                return Err("undo transaction has no prior journal tip".to_string());
            };
            if tip.transaction_id != undo_of {
                return Err(format!(
                    "undo_of {undo_of} does not reference the current journal tip"
                ));
            }
            let target = prior
                .iter()
                .find(|transaction| transaction.transaction_id == undo_of)
                .expect("prior id set should match prior transaction");
            if target.transaction_kind == TransactionKind::Undo {
                return Err(format!("undo_of {undo_of} references an undo transaction"));
            }
            validate_compensating_payload(transaction, target, "undo")?;
        }
        TransactionKind::Redo => {
            let Some(redo_of) = transaction.redo_of else {
                return Err("redo transaction must carry redo_of".to_string());
            };
            if transaction.undo_of.is_some() {
                return Err("redo transaction must not carry undo_of".to_string());
            }
            let Some(target) = prior
                .iter()
                .find(|transaction| transaction.transaction_id == redo_of)
            else {
                return Err(format!(
                    "redo_of {redo_of} does not reference a prior transaction"
                ));
            };
            if target.transaction_kind != TransactionKind::Undo {
                return Err(format!(
                    "redo_of {redo_of} does not reference an undo transaction"
                ));
            }
            let Some(target_index) = prior
                .iter()
                .position(|transaction| transaction.transaction_id == redo_of)
            else {
                return Err(format!(
                    "redo_of {redo_of} does not reference a prior transaction"
                ));
            };
            if prior
                .iter()
                .skip(target_index + 1)
                .any(|transaction| transaction.transaction_kind == TransactionKind::Normal)
            {
                return Err(format!(
                    "redo_of {redo_of} was cleared by a newer normal transaction"
                ));
            }
            validate_compensating_payload(transaction, target, "redo")?;
        }
    }
    Ok(())
}

fn validate_compensating_payload(
    transaction: &TransactionRecord,
    target: &TransactionRecord,
    label: &str,
) -> Result<(), String> {
    if transaction.operations != target.inverse_operations {
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
