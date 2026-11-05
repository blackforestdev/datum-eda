//! Real native multi-sheet census and one Proposal/history transaction family.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_annotation::tests::setup,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, ProjectResolver, electrical_test_partitions};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "project annotation fixture")
}
fn verb(model: &mut DesignModel, root: &Path, name: &str, params: Value) -> Uuid {
    let write = (find_native_write_verb(name).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}
fn act(
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
fn sheets(model: &DesignModel) -> BTreeMap<Uuid, Value> {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .filter_map(|s| {
            s.symbols.keys().next().map(|id| {
                (
                    s.uuid,
                    crate::api::native_write::schematic_symbol_payload::source_sheet_payload(
                        model, s.uuid, *id,
                    )
                    .unwrap(),
                )
            })
        })
        .collect()
}
fn check_owner_pins(model: &DesignModel) {
    for (id, owner) in &model.component_instances {
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == model.objects[id].source_shard_id)
            .unwrap();
        let raw = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap();
        assert!(!owner.placed_symbol_refs.is_empty());
        for member in raw["component_instance"]["placed_symbol_refs"]
            .as_array()
            .unwrap()
        {
            let symbol: Uuid = serde_json::from_value(member["object_id"].clone()).unwrap();
            assert!(owner.placed_symbol_refs.contains(&symbol));
            assert_eq!(
                member["object_revision"],
                json!(model.objects[&symbol].object_revision.0)
            );
        }
    }
}
#[test]
fn project_annotation_multisheet_review_atomic_history_and_cold_reopen() {
    let (root, mut model, original_sheet, ids, _) = setup("project-annotation-roots");
    let second = Uuid::from_u128(1);
    assert!(second < original_sheet);
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":second,"name":"Offscreen amplifier"}),
    );
    let offscreen = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":second,
        "library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U90","value":"OFFSCREEN","x_nm":0,"y_nm":0}),
    );
    for (index, symbol) in ids.iter().enumerate() {
        verb(
            &mut model,
            &root,
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbol,"instance_path":[],"reference":format!("U{}",80+index)}),
        );
    }
    let original = sheets(&model);
    let identities = model.electrical_identities.clone();
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let plan = preview(&model).unwrap();
    assert_eq!(plan.sheets.len(), 2);
    assert_eq!(plan.sheets[0].sheet, second);
    assert_eq!(plan.sheets[0].name, "Offscreen amplifier");
    assert_eq!(plan.sheets[0].changes[0].symbol, offscreen);
    assert_eq!(plan.sheets[0].changes[0].reference, "U1");
    assert_eq!(plan.change_count(), 4);
    let id = Uuid::new_v4();
    let draft = act(
        &mut model,
        &root,
        Action::CreateProjectAnnotation { proposal_id: id },
    )
    .unwrap();
    assert_eq!(draft.project_annotation, Some(plan.clone()));
    assert!(draft.annotation.is_none() && draft.fields.is_none() && draft.assignment.is_none());
    assert_eq!(sheets(&model), original);
    act(&mut model, &root, Action::Reject { proposal_id: id }).unwrap();
    assert_eq!(sheets(&model), original);
    let id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateProjectAnnotation { proposal_id: id },
    )
    .unwrap();
    let count = model.journal.len();
    let result = act(&mut model, &root, Action::AcceptApply { proposal_id: id }).unwrap();
    assert_eq!(model.journal.len(), count + 2);
    assert_eq!(result.project_annotation, Some(plan.clone()));
    let tx = model.journal.last().unwrap();
    assert_eq!(Some(tx.transaction_id), result.transaction_id);
    let touched: BTreeSet<_> = tx
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetSchematicSymbol { sheet_id, .. } => Some(*sheet_id),
            _ => None,
        })
        .collect();
    assert_eq!(touched, BTreeSet::from([original_sheet, second]));
    let mut expected = original.clone();
    for sheet in &plan.sheets {
        for c in &sheet.changes {
            expected.get_mut(&sheet.sheet).unwrap()["symbols"][c.symbol.to_string()]["reference"] =
                json!(c.reference);
        }
    }
    assert_eq!(sheets(&model), expected);
    assert!(preview(&model).unwrap().change_count() == 0);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    model = ProjectResolver::new(&root).resolve().unwrap();
    check_owner_pins(&model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(sheets(&model), original);
    check_owner_pins(&model);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(sheets(&model), expected);
    check_owner_pins(&model);
    assert_eq!(model.electrical_identities, identities);
    assert!(ids.iter().all(|id| model.objects.contains_key(id)));
    let revision = model.model_revision.clone();
    let count = model.journal.len();
    assert!(
        act(
            &mut model,
            &root,
            Action::CreateProjectAnnotation {
                proposal_id: Uuid::new_v4()
            }
        )
        .is_err()
    );
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_annotation_offscreen_locked_stale_and_hierarchy_refuse_without_partial_writes() {
    let (root, mut model, _, _, _) = setup("project-annotation-offscreen-refusal");
    let second = Uuid::new_v4();
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":second,"name":"Offscreen"}),
    );
    let symbol = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":second,
        "library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U90","value":"OFFSCREEN","x_nm":0,"y_nm":0}),
    );
    for locked in [true, false] {
        let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
        let original =
            snapshot.schematic_source().unwrap().sheets[&second].symbols[&symbol].clone();
        let mut payload = sheets(&model)[&second]["symbols"][symbol.to_string()].clone();
        payload["locked"] = json!(locked);
        let mut operations = vec![Operation::SetSchematicSymbol {
            sheet_id: second,
            symbol_id: symbol,
            symbol: payload,
        }];
        operations.extend(
            crate::api::native_write::schematic_assignment::repin_symbols(&model, &[original])
                .unwrap(),
        );
        let write = crate::api::native_write::BatchComposer::compose(&model, provenance())
            .push_ops(operations)
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        if locked {
            let raw = sheets(&model);
            let revision = model.model_revision.clone();
            let count = model.journal.len();
            assert!(
                act(
                    &mut model,
                    &root,
                    Action::CreateProjectAnnotation {
                        proposal_id: Uuid::new_v4()
                    }
                )
                .is_err()
            );
            assert_eq!(sheets(&model), raw);
            assert_eq!(model.model_revision, revision);
            assert_eq!(model.journal.len(), count);
        }
    }
    let id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateProjectAnnotation { proposal_id: id },
    )
    .unwrap();
    verb(
        &mut model,
        &root,
        "datum.schematic.set_symbol_reference",
        json!({"symbol":symbol,"instance_path":[],"reference":"U99"}),
    );
    let changed = sheets(&model);
    let count = model.journal.len();
    assert!(act(&mut model, &root, Action::AcceptApply { proposal_id: id }).is_err());
    assert_eq!(sheets(&model), changed);
    assert_eq!(model.journal.len(), count);
    let source_id = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .uuid;
    let definition = crate::schematic::SheetDefinition {
        uuid: Uuid::new_v4(),
        name: "Hierarchy".into(),
        root_sheet: second,
    };
    let write = crate::api::native_write::schematic_sheets::build_create_schematic_definition(
        &model,
        provenance(),
        source_id,
        definition.uuid,
        "definitions/project-annotation-refusal.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let raw = sheets(&model);
    let revision = model.model_revision.clone();
    let count = model.journal.len();
    assert!(
        act(
            &mut model,
            &root,
            Action::CreateProjectAnnotation {
                proposal_id: Uuid::new_v4()
            }
        )
        .is_err()
    );
    assert_eq!(sheets(&model), raw);
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_annotation_multigate_and_empty_sheet_remain_complete_through_cold_history() {
    use crate::api::native_write::schematic_gate_placement::{self as gates, tests as fixture};
    let (root, mut model, first, request) = fixture::setup("project-annotation-multigate");
    let second_gate = gates::preview(&model, &request).unwrap().symbol.uuid;
    fixture::action(
        &mut model,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    for (symbol, reference) in [(first, "U80"), (second_gate, "U81")] {
        verb(
            &mut model,
            &root,
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbol,"instance_path":[],"reference":reference}),
        );
    }
    let offscreen = Uuid::from_u128(1);
    let empty = Uuid::from_u128(2);
    for (sheet, name) in [(offscreen, "Offscreen"), (empty, "Empty root")] {
        verb(
            &mut model,
            &root,
            "datum.schematic.create_sheet",
            json!({"sheet":sheet,"name":name}),
        );
    }
    verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":offscreen,
        "library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U90","value":"OFFSCREEN","x_nm":0,"y_nm":0}),
    );
    let before = sheets(&model);
    let members = model.component_instances[&request.component_instance]
        .placed_symbol_refs
        .clone();
    let summary = preview(&model).unwrap();
    assert_eq!(summary.sheets.len(), 3);
    let blank = summary.sheets.iter().find(|s| s.sheet == empty).unwrap();
    assert_eq!(blank.symbol_count, 0);
    assert!(blank.changes.is_empty());
    let id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateProjectAnnotation { proposal_id: id },
    )
    .unwrap();
    act(&mut model, &root, Action::AcceptApply { proposal_id: id }).unwrap();
    assert_eq!(model.journal.last().unwrap().operations.iter().filter(|op|matches!(op,
        Operation::SetComponentInstance {component_instance_id,..} if *component_instance_id==request.component_instance)).count(),1);
    let applied = sheets(&model);
    assert_ne!(applied, before);
    for undo in [true, false, true, false] {
        model = ProjectResolver::new(&root).resolve().unwrap();
        check_owner_pins(&model);
        if undo {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        check_owner_pins(&model);
        assert_eq!(
            sheets(&model),
            if undo {
                before.clone()
            } else {
                applied.clone()
            }
        );
        assert_eq!(
            model.component_instances[&request.component_instance].placed_symbol_refs,
            members
        );
        assert!(
            ElectricalSelectionSnapshot::capture(&model)
                .unwrap()
                .schematic_source()
                .unwrap()
                .sheets
                .contains_key(&empty)
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_annotation_refuses_tampered_sheet_association_and_value_payload() {
    let (root, mut model, _, ids, _) = setup("project-annotation-tamper");
    let other = Uuid::new_v4();
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":other,"name":"Other root"}),
    );
    verb(
        &mut model,
        &root,
        "datum.schematic.set_symbol_reference",
        json!({"symbol":ids[0],"instance_path":[],"reference":"U90"}),
    );
    let id = Uuid::new_v4();
    act(
        &mut model,
        &root,
        Action::CreateProjectAnnotation { proposal_id: id },
    )
    .unwrap();
    let original = model.proposals[&id].clone();
    let before = sheets(&model);
    let revision = model.model_revision.clone();
    let count = model.journal.len();
    for change_sheet in [true, false] {
        let mut corrupted = original.clone();
        let op = corrupted
            .batch
            .operations
            .iter_mut()
            .find(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
            .unwrap();
        if let Operation::SetSchematicSymbol {
            sheet_id, symbol, ..
        } = op
        {
            if change_sheet {
                *sheet_id = other;
            } else {
                symbol["value"] = json!("unreviewed Value mutation");
            }
        }
        model.proposals.insert(id, corrupted);
        assert!(act(&mut model, &root, Action::AcceptApply { proposal_id: id }).is_err());
        assert_eq!(model.model_revision, revision);
        assert_eq!(model.journal.len(), count);
        assert_eq!(sheets(&model), before);
    }
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(sheets(&cold), before);
    std::fs::remove_dir_all(root).unwrap();
}
