//! Exact metadata inverse rebasing; history links retain real canonical origins.
use super::*;

/// Normal pre-anchor metadata is authenticated without recursively revalidating
/// every older metadata transaction. Its guard must equal the actual folded epoch.
pub(in crate::substrate) fn validate_package_field_transaction(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
    package: Uuid,
    epoch: ObjectRevision,
) -> Result<(), EngineError> {
    let changes = if tx.transaction_kind == TransactionKind::Normal {
        let proposal = receipt(
            prior,
            &tx.operations,
            Some(tx.transaction_id),
            Some(tx.batch_id),
        )?;
        DesignModel::validate_component_metadata_proposal(&proposal)?;
        if tx.provenance != proposal.batch.provenance
            || tx.provenance.source != CommitSource::Manual
            || !proposal.batch.operations.iter().any(|op| {
                matches!(op,
                Operation::GuardObjectRevision { object_id, expected_object_revision }
                    if *object_id == package && *expected_object_revision == epoch)
            })
        {
            return Err(invalid(
                "historical package metadata differs from its guarded source epoch",
            ));
        }
        Changes::capture(&tx.operations)?
    } else {
        transaction_transition(tx, prior)?.changes
    };
    if !changes.references.contains_key(&package) && !changes.values.contains_key(&package) {
        return Err(invalid(
            "historical package metadata has no qualified receipt",
        ));
    }
    Ok(())
}

fn target<'a>(
    tx: &TransactionRecord,
    prior: &'a [TransactionRecord],
) -> Result<&'a TransactionRecord, EngineError> {
    let id = match tx.transaction_kind {
        TransactionKind::Undo if tx.redo_of.is_none() => tx.undo_of,
        TransactionKind::Redo if tx.undo_of.is_none() => tx.redo_of,
        _ => None,
    }
    .ok_or_else(|| invalid("compensation link is unavailable"))?;
    prior
        .iter()
        .find(|t| t.transaction_id == id)
        .ok_or_else(|| invalid("compensation target is outside accepted history"))
}

pub(super) fn origin_project(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<Uuid, EngineError> {
    if tx.transaction_kind == TransactionKind::Normal {
        return Ok(receipt(
            prior,
            &tx.operations,
            Some(tx.transaction_id),
            Some(tx.batch_id),
        )?
        .project_id);
    }
    let selected = target(tx, prior)?;
    let index = prior
        .iter()
        .position(|t| t.transaction_id == selected.transaction_id)
        .unwrap();
    origin_project(selected, &prior[..index])
}

pub(super) fn transaction_transition(
    tx: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<Transition, EngineError> {
    if tx.transaction_kind == TransactionKind::Normal {
        return normal_transition(tx, prior);
    }
    let selected = target(tx, prior)?;
    if !crate::substrate::journal_history::selected_target(prior, selected, tx.transaction_kind) {
        return Err(invalid(
            "metadata compensation is not the canonical selected target",
        ));
    }
    let expected = normalized(selected, prior)?;
    if tx.operations != expected || inverse(&expected, selected)? != tx.inverse_operations {
        return Err(invalid(
            "metadata compensation differs from its exact canonical inverse",
        ));
    }
    transition(&tx.operations, &tx.inverse_operations, prior)
}

fn latest_net(
    prior: &[TransactionRecord],
    id: Uuid,
) -> Result<ElectricalIdentityRecord, EngineError> {
    let mut current = None;
    for op in prior.iter().flat_map(|t| &t.operations) {
        match op {
            Operation::CreateElectricalIdentity { record }
            | Operation::SetElectricalIdentity { record, .. }
                if record.id == id =>
            {
                current = Some(record.clone())
            }
            Operation::DeleteElectricalIdentity { record } if record.id == id => current = None,
            _ => {}
        }
    }
    current.ok_or_else(|| invalid("current Net evidence has no canonical origin"))
}

fn same_members(before: &Value, after: &Value) -> bool {
    let normalize = |raw: &Value| {
        let mut raw = raw.clone();
        raw["object_revision"] = Value::Null;
        for key in ["placed_symbol_refs", "placed_package_refs"] {
            if let Some(refs) = raw[key].as_array_mut() {
                for reference in refs {
                    reference["object_revision"] = Value::Null;
                }
            }
        }
        raw
    };
    normalize(before) == normalize(after)
}

fn normalized(
    selected: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<Vec<Operation>, EngineError> {
    let index = prior
        .iter()
        .position(|t| t == selected)
        .ok_or_else(|| invalid("metadata target differs from the accepted transaction"))?;
    let original = transaction_transition(selected, &prior[..index])?;
    let project = origin_project(selected, &prior[..index])?;
    let mut operations = selected.inverse_operations.clone();
    let intended = Changes::capture(&operations)?;
    let mut current_components = BTreeMap::new();
    let mut after = BTreeMap::new();
    for (id, reference) in &original.after {
        resolve_journal(prior, reference, project)?;
        let current = latest_component(prior, *id)?;
        if !same_members(&current, &original.changes.components[id].1) {
            return Err(invalid(
                "metadata inverse would change physical component semantics",
            ));
        }
        let source: PersistedComponentInstance = serde_json::from_value(current.clone())?;
        let ([symbol], [package]) = (
            source.placed_symbol_refs.as_slice(),
            source.placed_package_refs.as_slice(),
        ) else {
            return Err(invalid(
                "metadata inverse has unsupported component membership",
            ));
        };
        let mut updated = current.clone();
        let epoch = next(source.object_revision, 1)?;
        updated["object_revision"] = serde_json::to_value(epoch)?;
        if intended.symbols.contains_key(&symbol.object_id) {
            updated["placed_symbol_refs"][0]["object_revision"] =
                serde_json::to_value(next(symbol.object_revision, 1)?)?;
        } else {
            let count = u64::from(intended.references.contains_key(&package.object_id))
                + u64::from(intended.values.contains_key(&package.object_id));
            updated["placed_package_refs"][0]["object_revision"] = serde_json::to_value(next(
                super::super::placed_package_history::recorded_package(prior, package)?.1,
                count,
            )?)?;
        }
        current_components.insert(*id, (current, updated));
        after.insert(
            *id,
            RevisionedRef {
                object_id: *id,
                object_revision: epoch,
            },
        );
    }
    for op in &mut operations {
        match op {
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => {
                let (before, updated) = current_components
                    .get(component_instance_id)
                    .ok_or_else(|| invalid("metadata inverse contains another component"))?;
                *previous_component_instance = before.clone();
                *component_instance = updated.clone();
            }
            Operation::SetElectricalIdentity { previous, record } => {
                let current = latest_net(prior, record.id)?;
                let mut updated = current.clone();
                updated.object_revision = next(current.object_revision, 1)?;
                let ElectricalIdentity::NetRelationship { evidence, .. } = &mut updated.identity
                else {
                    return Err(invalid(
                        "metadata inverse targets another electrical family",
                    ));
                };
                for pin in evidence {
                    if let Some(reference) = after.get(&pin.component_instance.object_id) {
                        pin.component_instance = reference.clone();
                    }
                }
                *previous = Box::new(current);
                *record = updated;
            }
            _ => {}
        }
    }
    transition(&operations, &inverse(&operations, selected)?, prior)?;
    Ok(operations)
}

/// Reproduce the journal owner's inverse grouping: board source, ordered sheet
/// source, then non-core metadata; reverse once at the end.
fn inverse(
    operations: &[Operation],
    target: &TransactionRecord,
) -> Result<Vec<Operation>, EngineError> {
    let mut board = Vec::new();
    let mut symbols = Vec::new();
    let mut other = Vec::new();
    for op in operations {
        match op {
            Operation::SetBoardPackageReference { package_id, .. } => board.push(target.operations.iter()
                .find(|o| matches!(o, Operation::SetBoardPackageReference { package_id: id, .. } if id == package_id))
                .ok_or_else(|| invalid("recorded PCB reference inverse source is unavailable"))?.clone()),
            Operation::SetBoardPackageValue { package_id, .. } => board.push(target.operations.iter()
                .find(|o| matches!(o, Operation::SetBoardPackageValue { package_id: id, .. } if id == package_id))
                .ok_or_else(|| invalid("recorded PCB value inverse source is unavailable"))?.clone()),
            Operation::SetSchematicSymbol { sheet_id, symbol_id, .. } => symbols.push(target.operations.iter()
                .find(|o| matches!(o, Operation::SetSchematicSymbol { sheet_id: sheet, symbol_id: id, .. }
                    if sheet == sheet_id && id == symbol_id))
                .ok_or_else(|| invalid("recorded Symbol inverse source is unavailable"))?.clone()),
            Operation::SetComponentInstance { component_instance_id, previous_component_instance, component_instance } =>
                other.push(Operation::SetComponentInstance { component_instance_id: *component_instance_id,
                    previous_component_instance: component_instance.clone(), component_instance: previous_component_instance.clone() }),
            Operation::SetElectricalIdentity { previous, record } => other.push(Operation::SetElectricalIdentity {
                previous: Box::new(record.clone()), record: (**previous).clone(),
            }),
            Operation::SetProposalMetadata { proposal_id, relative_path, previous_proposal, proposal } =>
                other.push(Operation::SetProposalMetadata { proposal_id: *proposal_id, relative_path: relative_path.clone(),
                    previous_proposal: proposal.clone(), proposal: previous_proposal.clone() }),
            _ => return Err(invalid("unsupported metadata inverse operation")),
        }
    }
    board.extend(symbols);
    board.extend(other);
    board.reverse();
    Ok(board)
}

/// A restored Applied status may be a real Redo, not a new proposal apply.
/// Only the actual current Redo target can authorize its rebased whole batch.
pub(super) fn pending_redo_transition(
    model: &DesignModel,
    proposal_id: Uuid,
    operations: &[Operation],
    inverse: &[Operation],
) -> Result<Option<Transition>, EngineError> {
    let targets = crate::substrate::journal_history::targets(&model.journal)
        .map_err(EngineError::Operation)?;
    let Some(selected) = targets.redo else {
        return Ok(None);
    };
    if !super::frame::announces_metadata(&selected.operations)
        || !selected.operations.iter().any(|operation| {
            matches!(operation,
            Operation::SetProposalMetadata { proposal_id: id, .. } if *id == proposal_id)
        })
    {
        return Ok(None);
    }
    let expected = normalized(selected, &model.journal)?;
    if expected != operations {
        return Err(invalid(
            "metadata Redo differs from its exact current history target",
        ));
    }
    Ok(Some(transition(operations, inverse, &model.journal)?))
}

pub(super) fn pending_transition(
    model: &DesignModel,
    operations: &[Operation],
    inverse: &[Operation],
) -> Result<Transition, EngineError> {
    let targets = crate::substrate::journal_history::targets(&model.journal)
        .map_err(EngineError::Operation)?;
    for selected in [targets.undo, targets.redo].into_iter().flatten() {
        if normalized(selected, &model.journal).is_ok_and(|expected| expected == operations) {
            return transition(operations, inverse, &model.journal);
        }
    }
    Err(invalid(
        "prospective inverse is not an exact current metadata history target",
    ))
}

pub(in crate::substrate) fn prepare(
    model: &DesignModel,
    selected: &TransactionRecord,
    batch: &mut crate::substrate::OperationBatch,
) -> Result<bool, EngineError> {
    // Unrelated existing families retain their original owners and strict gates.
    let index = model
        .journal
        .iter()
        .position(|t| t == selected)
        .ok_or_else(|| invalid("history target differs from accepted source"))?;
    if !super::frame::announces_metadata(&selected.operations) {
        return Ok(false);
    }
    transaction_transition(selected, &model.journal[..index])?;
    if batch.operations != selected.inverse_operations
        || !crate::substrate::journal_history::current_target(&model.journal, selected)
    {
        return Err(invalid("metadata compensation target or preimage changed"));
    }
    batch.operations = normalized(selected, &model.journal)?;
    Ok(true)
}

pub(in crate::substrate) fn matches(
    tx: &TransactionRecord,
    selected: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    crate::substrate::journal_history::selected_target(prior, selected, tx.transaction_kind)
        && normalized(selected, prior).is_ok_and(|ops| {
            ops == tx.operations
                && inverse(&ops, selected).is_ok_and(|expected| expected == tx.inverse_operations)
        })
}

#[cfg(test)]
impl DesignModel {
    /// Exercise the real prospective kernel on the actual post-Undo model.
    /// This bridge is test-only; no production visibility or write path changes.
    pub(crate) fn assert_current_metadata_redo_admission(&self) {
        let original = serde_json::to_vec(self).unwrap();
        let targets = crate::substrate::journal_history::targets(&self.journal).unwrap();
        let selected = targets.redo.expect("real Undo must expose a Redo target");
        let canonical = normalized(selected, &self.journal).unwrap();
        for case in 0..4 {
            let mut operations = canonical.clone();
            match case {
                0 => {}
                1 => {
                    let op = operations
                        .iter_mut()
                        .find(|op| matches!(op, Operation::SetBoardPackageValue { .. }))
                        .unwrap();
                    let Operation::SetBoardPackageValue { value, .. } = op else {
                        unreachable!()
                    };
                    value.push_str(" unreviewed");
                }
                2 => {
                    operations.retain(|op| !matches!(op, Operation::SetElectricalIdentity { .. }));
                    assert!(
                        operations.len() < canonical.len(),
                        "negative must actually drop Net evidence"
                    );
                }
                _ => {
                    let op = operations
                        .iter_mut()
                        .find(|op| matches!(op, Operation::SetProposalMetadata { .. }))
                        .unwrap();
                    let Operation::SetProposalMetadata { proposal_id, .. } = op else {
                        unreachable!()
                    };
                    *proposal_id = Uuid::new_v4();
                }
            }
            let mut candidate = self.clone();
            let result = candidate.commit_without_direct_policy(crate::substrate::OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(self.model_revision.clone()),
                provenance: selected.provenance.clone(),
                operations,
            });
            if case == 0 {
                result.expect("exact current Redo must pass prospective validation");
            } else {
                let error = result
                    .expect_err("changed current Redo must refuse")
                    .to_string();
                if case < 3 {
                    assert!(
                        error.contains(
                            "metadata Redo differs from its exact current history target"
                        ),
                        "{error}"
                    );
                }
            }
        }
        assert_eq!(
            serde_json::to_vec(self).unwrap(),
            original,
            "prospective admission cannot alter the real accepted model"
        );
    }
}
