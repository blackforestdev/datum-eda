use super::*;
use crate::api::native_write::{
    schematic_annotation::tests::setup,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, ProjectResolver, electrical_test_partitions};
use std::path::Path;

fn execute(
    model: &mut DesignModel,
    root: &Path,
    action: Action,
) -> Result<proposal::NativeProposalResponse, EngineError> {
    proposal::execute(
        model,
        &NativeProposalRequest {
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        },
    )
}
fn rows(model: &DesignModel, sheet: Uuid) -> BTreeMap<Uuid, FieldRowEdit> {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&sheet]
        .symbols
        .values()
        .map(|s| {
            (
                s.uuid,
                FieldRowEdit {
                    symbol: s.uuid,
                    reference: s.reference.clone(),
                    value: s.value.clone(),
                },
            )
        })
        .collect()
}
fn history_provenance() -> crate::substrate::CommitProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "field batch history").into()
}

fn assert_binding_epochs(model: &DesignModel, symbols: &[Uuid], owners: &[Uuid]) {
    for (symbol, owner) in symbols.iter().zip(owners) {
        assert!(
            model.component_instances[owner]
                .placed_symbol_refs
                .contains(symbol)
        );
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == model.objects[owner].source_shard_id)
            .unwrap();
        let raw = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap();
        for member in raw["component_instance"]["placed_symbol_refs"]
            .as_array()
            .unwrap()
        {
            let id: Uuid = serde_json::from_value(member["object_id"].clone()).unwrap();
            assert_eq!(
                member["object_revision"],
                json!(model.objects[&id].object_revision.0)
            );
        }
    }
}

#[test]
fn field_batch_swaps_values_raw_payload_reject_history_and_cold_reopen() {
    let (root, mut model, sheet, ids, owners) = setup("field-batch-swaps");
    let originals = rows(&model, sheet);
    let raw = source_sheet_payload(&model, sheet, ids[0]).unwrap();
    let identities = model.electrical_identities.clone();
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let mut edits = vec![originals[&ids[0]].clone(), originals[&ids[1]].clone()];
    edits[0].reference = originals[&ids[1]].reference.clone();
    edits[1].reference = originals[&ids[0]].reference.clone();
    edits[0].value = "4k7 literal".into();
    edits[1].value.clear();
    let expected = preview(&model, sheet, &[], &edits).unwrap();
    assert_eq!(expected.changes.len(), 2);
    let id = Uuid::new_v4();
    let draft = execute(
        &mut model,
        &root,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: edits.clone(),
        },
    )
    .unwrap();
    assert_eq!(draft.fields, Some(expected.clone()));
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    execute(&mut model, &root, Action::Reject { proposal_id: id }).unwrap();
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    let id = Uuid::new_v4();
    execute(
        &mut model,
        &root,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: edits.clone(),
        },
    )
    .unwrap();
    let count = model.journal.len();
    let applied = execute(&mut model, &root, Action::AcceptApply { proposal_id: id }).unwrap();
    assert!(applied.transaction_id.is_some());
    assert_eq!(applied.fields, Some(expected));
    assert_eq!(model.journal.len(), count + 2);
    let design_records: Vec<_> = model
        .journal
        .iter()
        .skip(count)
        .filter(|tx| {
            tx.operations
                .iter()
                .any(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
        })
        .collect();
    assert_eq!(
        design_records.len(),
        1,
        "one atomic design transaction after Accepted metadata"
    );
    assert_eq!(
        Some(design_records[0].transaction_id),
        applied.transaction_id
    );
    let mut wanted = raw.clone();
    for edit in &edits {
        wanted["symbols"][edit.symbol.to_string()]["reference"] = json!(edit.reference);
        wanted["symbols"][edit.symbol.to_string()]["value"] = json!(edit.value);
    }
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), wanted);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding_epochs(&model, &ids, &owners);
    model
        .commit_journal_undo(&root, history_provenance())
        .unwrap();
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding_epochs(&model, &ids, &owners);
    model
        .commit_journal_redo(&root, history_provenance())
        .unwrap();
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), wanted);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source_sheet_payload(&cold, sheet, ids[0]).unwrap(), wanted);
    assert_eq!(cold.electrical_identities, identities);
    assert_binding_epochs(&cold, &ids, &owners);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn field_batch_collisions_noop_and_stale_proposals_do_not_partially_apply() {
    let (root, mut model, sheet, ids, _) = setup("field-batch-refusals");
    let original = rows(&model, sheet);
    let raw = source_sheet_payload(&model, sheet, ids[0]).unwrap();
    let mut edit = original[&ids[0]].clone();
    edit.value = "new value".into();
    let valid = edit.clone();
    let mut invalid_edits = Vec::new();
    edit.reference = original[&ids[1]].reference.clone();
    invalid_edits.push(vec![edit.clone()]);
    edit.reference.clear();
    invalid_edits.push(vec![edit.clone()]);
    edit = valid.clone();
    edit.value = "two\nlines".into();
    invalid_edits.push(vec![edit]);
    invalid_edits.push(vec![valid.clone(), valid.clone()]);
    invalid_edits.push(vec![FieldRowEdit {
        symbol: Uuid::new_v4(),
        ..valid.clone()
    }]);
    invalid_edits.push(vec![original[&ids[0]].clone()]);
    for edits in invalid_edits {
        let before = model.model_revision.clone();
        let count = model.journal.len();
        assert!(
            execute(
                &mut model,
                &root,
                Action::CreateFieldBatch {
                    proposal_id: Uuid::new_v4(),
                    sheet,
                    instance_path: vec![],
                    edits,
                }
            )
            .is_err()
        );
        assert_eq!(model.model_revision, before);
        assert_eq!(model.journal.len(), count);
        assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    }
    let id = Uuid::new_v4();
    execute(
        &mut model,
        &root,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: vec![valid.clone()],
        },
    )
    .unwrap();
    let mut other = original[&ids[2]].clone();
    other.value = "intervening edit".into();
    let prepared = build(&model, sheet, &[], &[other]).unwrap();
    crate::api::native_write::commit_prepared(&mut model, &root, prepared).unwrap();
    let changed = source_sheet_payload(&model, sheet, ids[0]).unwrap();
    assert!(execute(&mut model, &root, Action::AcceptApply { proposal_id: id }).is_err());
    assert_eq!(
        source_sheet_payload(&model, sheet, ids[0]).unwrap(),
        changed
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn field_batch_multigate_owner_is_repinned_once_with_unchanged_sibling() {
    use crate::api::native_write::schematic_gate_placement::{self as gates, tests as fixture};
    let (root, mut model, first, req) = fixture::setup("field-batch-multigate");
    let second = gates::preview(&model, &req).unwrap().symbol.uuid;
    fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let original = rows(&model, req.sheet);
    let sibling = model.objects[&second].object_revision;
    let owner = model.objects[&req.component_instance].object_revision;
    let mut edit = original[&first].clone();
    edit.value = "shared owner, one changed gate".into();
    let draft = execute(
        &mut model,
        &root,
        Action::CreateFieldBatch {
            proposal_id: Uuid::new_v4(),
            sheet: req.sheet,
            instance_path: vec![],
            edits: vec![edit],
        },
    )
    .unwrap();
    let batch = &model.proposals[&draft.proposal_id].batch;
    assert_eq!(
        batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::SetComponentInstance { .. }))
            .count(),
        1
    );
    execute(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    )
    .unwrap();
    assert_eq!(
        model.objects[&req.component_instance].object_revision.0,
        owner.0 + 1
    );
    assert_eq!(model.objects[&second].object_revision, sibling);
    assert_eq!(rows(&model, req.sheet)[&second], original[&second]);
    let applied = rows(&model, req.sheet);
    let sibling_raw = source_sheet_payload(&model, req.sheet, second).unwrap()["symbols"]
        [second.to_string()]
    .clone();
    let owners = [req.component_instance, req.component_instance];
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding_epochs(&model, &[first, second], &owners);
    model
        .commit_journal_undo(&root, history_provenance())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(rows(&model, req.sheet), original);
    assert_binding_epochs(&model, &[first, second], &owners);
    assert_eq!(
        source_sheet_payload(&model, req.sheet, second).unwrap()["symbols"][second.to_string()],
        sibling_raw
    );
    model
        .commit_journal_redo(&root, history_provenance())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(rows(&model, req.sheet), applied);
    assert_binding_epochs(&model, &[first, second], &owners);
    assert_eq!(
        model.component_instances[&req.component_instance]
            .placed_symbol_refs
            .len(),
        2
    );
    assert_eq!(model.objects[&second].object_revision, sibling);
    assert_eq!(
        source_sheet_payload(&model, req.sheet, second).unwrap()["symbols"][second.to_string()],
        sibling_raw
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn field_batch_locked_row_refuses_all_changes_and_tampered_receipt_cannot_apply() {
    let (root, mut model, sheet, ids, _) = setup("field-batch-locked-tamper");
    let mut locked =
        source_sheet_payload(&model, sheet, ids[2]).unwrap()["symbols"][ids[2].to_string()].clone();
    locked["locked"] = json!(true);
    let write = crate::api::native_write::BatchComposer::compose(
        &model,
        WriteProvenance::new("test", CommitSource::Manual, "lock fixture row"),
    )
    .push_op(Operation::SetSchematicSymbol {
        sheet_id: sheet,
        symbol_id: ids[2],
        symbol: locked,
    })
    .finish()
    .unwrap();
    crate::api::native_write::commit_prepared(&mut model, &root, write).unwrap();
    let original = rows(&model, sheet);
    let mut valid = original[&ids[0]].clone();
    valid.value = "valid requested edit".into();
    let mut locked = original[&ids[2]].clone();
    locked.value = "locked requested edit".into();
    let raw = source_sheet_payload(&model, sheet, ids[0]).unwrap();
    let revision = model.model_revision.clone();
    let count = model.journal.len();
    assert!(
        execute(
            &mut model,
            &root,
            Action::CreateFieldBatch {
                proposal_id: Uuid::new_v4(),
                sheet,
                instance_path: vec![],
                edits: vec![valid.clone(), locked]
            }
        )
        .is_err()
    );
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.journal.len(), count);
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    let id = Uuid::new_v4();
    execute(
        &mut model,
        &root,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: vec![valid],
        },
    )
    .unwrap();
    // Deliberately corrupt the loaded receipt; acceptance must revalidate its
    // exact operation family before it can mutate any durable source.
    let batch = &mut model.proposals.get_mut(&id).unwrap().batch;
    let mut tampered = false;
    for op in &mut batch.operations {
        if let Operation::SetSchematicSymbol { symbol, .. } = op {
            symbol["unexpected_field"] = json!("not a Reference/Value edit");
            tampered = true;
        }
    }
    assert!(tampered);
    let count = model.journal.len();
    assert!(execute(&mut model, &root, Action::AcceptApply { proposal_id: id }).is_err());
    assert_eq!(model.journal.len(), count);
    assert_eq!(source_sheet_payload(&model, sheet, ids[0]).unwrap(), raw);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source_sheet_payload(&cold, sheet, ids[0]).unwrap(), raw);
    std::fs::remove_dir_all(root).unwrap();
}
