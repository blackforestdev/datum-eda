//! Read-only package-reference continuity across recorded position/rotation edits.
//! This does not repin component sources or manufacture a history epoch.
use super::pool_history::PoolHistory;
use super::{DomainObject, EngineError, ObjectRevision, RevisionedRef, SourceShardKind};
use crate::substrate::library_asset_journal_ops::history::{
    Proof, qualified_package_transaction_with_proof,
};
use crate::substrate::{Operation, TransactionKind, TransactionRecord};
use serde_json::Value;
use uuid::Uuid;

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Placed package history: {message}"))
}

/// Some(true) is a pose edit; Some(false) is another mutation of this package.
fn target(operation: &Operation, id: Uuid) -> Option<bool> {
    let (target, pose) = match operation {
        Operation::SetBoardPackagePosition { package_id, .. }
        | Operation::SetBoardPackageRotation { package_id, .. } => (*package_id, true),
        Operation::CreateBoardPackage { package_id, .. }
        | Operation::DeleteBoardPackage { package_id, .. }
        | Operation::SetBoardPackagePart { package_id, .. }
        | Operation::SetBoardPackagePackage { package_id, .. }
        | Operation::SetBoardPackageValue { package_id, .. }
        | Operation::SetBoardPackageReference { package_id, .. }
        | Operation::SetBoardPackageLayer { package_id, .. }
        | Operation::SetComponentSide { package_id, .. }
        | Operation::SetBoardPackageLocked { package_id, .. } => (*package_id, false),
        Operation::BumpObjectRevision { object_id } => (*object_id, false),
        _ => return None,
    };
    (target == id).then_some(pose)
}

fn compensation(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
    pose_only: bool,
    package: Uuid,
    proof: &mut Proof<'_>,
) -> bool {
    if tx.transaction_kind == TransactionKind::Normal {
        return tx.undo_of.is_none() && tx.redo_of.is_none();
    }
    let id = match tx.transaction_kind {
        TransactionKind::Undo if tx.redo_of.is_none() => tx.undo_of,
        TransactionKind::Redo if tx.undo_of.is_none() => tx.redo_of,
        _ => None,
    };
    id.and_then(|id| prior.iter().find(|record| record.transaction_id == id))
        .is_some_and(|selected| {
            crate::substrate::journal_history::selected_target(prior, selected, tx.transaction_kind)
                && ((tx.operations == selected.inverse_operations
                    && tx.inverse_operations == selected.operations)
                    || (!pose_only
                        && crate::substrate::schematic_assignment_history::matches(
                            tx, selected, prior,
                        ))
                    || (!pose_only
                        && qualified_package_transaction_with_proof(tx, prior, package, proof)
                            .unwrap_or(false)))
        })
}

/// Fold only this package's real source operations using the canonical board
/// owner. The declared epoch is an anchor: earlier authored metadata is retained,
/// but every subsequent package mutation must be a proved pose edit.
fn qualifies(
    current: &Value,
    journal: &[TransactionRecord],
    reference: &RevisionedRef,
    revision: ObjectRevision,
) -> Result<bool, EngineError> {
    let id = reference.object_id;
    if revision.0 <= reference.object_revision.0 {
        return Ok(false);
    }
    let raw = current["packages"][id.to_string()].clone();
    if raw["uuid"].as_str() != Some(id.to_string().as_str()) {
        return Err(invalid("current source package identity differs"));
    }
    serde_json::from_value::<crate::board::PlacedPackage>(raw.clone())?;
    let (recorded, epoch) = recorded_package(journal, reference)?;
    Ok(epoch == revision && recorded == raw)
}

/// Package-only projection of real creation, edits and inverse preimages in an
/// accepted prefix. No DesignModel, source baseline or journal entry is invented.
pub(in crate::substrate) fn recorded_package(
    journal: &[TransactionRecord],
    reference: &RevisionedRef,
) -> Result<(Value, ObjectRevision), EngineError> {
    Proof::new(journal).package(journal, reference)
}

pub(in crate::substrate) fn recorded_package_with_proof(
    journal: &[TransactionRecord],
    reference: &RevisionedRef,
    proof: &mut Proof<'_>,
) -> Result<(Value, ObjectRevision), EngineError> {
    crate::substrate::journal_history::targets(journal).map_err(EngineError::Operation)?;
    let id = reference.object_id;
    let births: Vec<_> = journal
        .iter()
        .filter_map(|tx| {
            tx.operations.iter().find_map(|op| match op {
                Operation::CreateBoardPackage {
                    package_id,
                    package,
                    materialized,
                } if *package_id == id => Some((tx, package, materialized)),
                _ => None,
            })
        })
        .collect();
    let [(birth, package, materialized)] = births.as_slice() else {
        return Err(invalid("package has no unique recorded creation"));
    };
    birth
        .inverse_operations
        .iter()
        .find(|op| {
            matches!(op,
        Operation::DeleteBoardPackage { package_id, package: before, materialized: physical }
            if *package_id == id && before == *package && physical == *materialized)
        })
        .ok_or_else(|| invalid("package creation inverse is unavailable"))?;
    // The canonical board owner folds the recorded package/materialization
    // subtree. Other board objects are neither inferred nor consulted here.
    let mut board = serde_json::json!({"packages":{},"pads":{}});
    for name in crate::substrate::board_journal_ops::component_materialization_map_names() {
        board[name] = serde_json::json!({});
    }
    let mut epoch = None::<u64>;
    for (index, tx) in journal.iter().enumerate() {
        let relevant: Vec<_> = tx
            .operations
            .iter()
            .filter(|op| target(op, id).is_some())
            .collect();
        if relevant.is_empty() {
            if tx.diff.modified.contains(&id)
                || tx.diff.created.contains(&id)
                || tx.diff.deleted.contains(&id)
            {
                return Err(invalid(
                    "package revision has an unrecognized source mutation",
                ));
            }
            continue;
        }
        let mut expected = Vec::new();
        let mut bridged = false;
        let mut metadata_checked = false;
        for op in relevant {
            match op {
                Operation::CreateBoardPackage { .. }
                    if epoch.is_none() && tx.transaction_kind == TransactionKind::Normal =>
                {
                    epoch = Some(0);
                    crate::substrate::board_journal_ops::apply_board_operation(&mut board, op)?;
                }
                Operation::CreateBoardPackage { .. }
                | Operation::DeleteBoardPackage { .. }
                | Operation::BumpObjectRevision { .. } => {
                    return Err(invalid(
                        "package birth/deletion/revision epoch is ambiguous",
                    ));
                }
                _ => {
                    let before =
                        epoch.ok_or_else(|| invalid("package has no recorded creation"))?;
                    if before >= reference.object_revision.0 {
                        if target(op, id) != Some(true) {
                            return Err(invalid("non-pose mutation follows the declared anchor"));
                        }
                        bridged = true;
                        crate::substrate::board_journal_ops::inverse_board_operation(
                            &mut board,
                            op,
                            &mut expected,
                        )?;
                    } else {
                        if matches!(
                            op,
                            Operation::SetBoardPackageReference { .. }
                                | Operation::SetBoardPackageValue { .. }
                        ) {
                            if !metadata_checked {
                                super::metadata_history::validate_package_field_transaction(
                                    tx,
                                    &journal[..index],
                                    id,
                                    ObjectRevision(before),
                                )?;
                                metadata_checked = true;
                            }
                        } else if target(op, id) != Some(true)
                            && !(matches!(op, Operation::SetBoardPackagePackage { .. })
                                && qualified_package_transaction_with_proof(
                                    tx,
                                    &journal[..index],
                                    id,
                                    proof,
                                )?)
                        {
                            return Err(invalid(
                                "non-metadata mutation precedes the declared anchor",
                            ));
                        }
                        crate::substrate::board_journal_ops::inverse_board_operation(
                            &mut board,
                            op,
                            &mut expected,
                        )?;
                    }
                    epoch = Some(
                        before
                            .checked_add(1)
                            .ok_or_else(|| invalid("package epoch overflow"))?,
                    );
                }
            }
        }
        if !compensation(tx, &journal[..index], bridged, id, proof) {
            return Err(invalid(
                "package compensation has no exact canonical origin",
            ));
        }
        if !expected.is_empty() {
            expected.reverse();
            let actual: Vec<_> = tx
                .inverse_operations
                .iter()
                .filter(|op| {
                    target(op, id).is_some()
                        && !matches!(
                            op,
                            Operation::CreateBoardPackage { .. }
                                | Operation::DeleteBoardPackage { .. }
                        )
                })
                .cloned()
                .collect();
            if actual != expected {
                return Err(invalid(
                    "recorded pose inverse differs from the source preimage",
                ));
            }
        }
    }
    let epoch = epoch.ok_or_else(|| invalid("package creation is unavailable"))?;
    if epoch < reference.object_revision.0 {
        return Err(invalid(
            "declared package anchor is outside the accepted prefix",
        ));
    }
    Ok((
        board["packages"][id.to_string()].clone(),
        ObjectRevision(epoch),
    ))
}

pub(in crate::substrate) fn matches(
    history: &PoolHistory<'_>,
    object: &DomainObject,
    reference: &RevisionedRef,
) -> Result<bool, EngineError> {
    // Map entries are registered with pointer-derived kinds (their UUID key).
    // Class authority comes from the actual BoardRoot packages map below.
    if object.object_id != reference.object_id || object.domain != "board" {
        return Ok(false);
    }
    let mut owners = history
        .shards
        .iter()
        .filter(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::BoardRoot);
    let owner = owners
        .next()
        .ok_or_else(|| invalid("board source owner is missing"))?;
    if owners.next().is_some() {
        return Err(invalid("board source owner is ambiguous"));
    }
    let bytes = std::fs::read(&owner.path)?;
    if crate::substrate::sha256_hex(&bytes) != owner.content_hash {
        return Err(invalid("board source changed during resolution"));
    }
    qualifies(
        &serde_json::from_slice(&bytes)?,
        history.journal,
        reference,
        object.object_revision,
    )
}

#[cfg(test)]
#[path = "placed_package_history_tests.rs"]
mod tests;
