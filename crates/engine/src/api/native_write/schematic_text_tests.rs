//! Persisted literal text, raw extensions, qualified admission and atomic history.
use super::*;
use crate::{
    api::native_write::{
        commit_prepared,
        electrical::topology_tests::fixture,
        registry::find_native_write_verb,
        schematic_delete_proposal::{
            self, DeleteClass, DeleteTarget, NativeProposalAction, NativeProposalRequest,
        },
        schematic_sheets::{
            build_create_schematic_definition, build_create_schematic_sheet_instance,
        },
    },
    substrate::{
        CommitSource, DesignModel, ProjectResolver, ProposalStatus, electrical_test_partitions,
    },
};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "standalone text")
}
fn build(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
}
fn patch(sheet: Uuid, text: Uuid, changes: Value) -> Value {
    let mut params = json!({"sheet":sheet,"text":text,"instance_path":[]});
    params
        .as_object_mut()
        .unwrap()
        .extend(changes.as_object().unwrap().clone());
    params
}
fn raw_sheet(model: &DesignModel, sheet: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            model
                .materialized_source_shard_value_by_relative_path(&s.relative_path)
                .is_ok_and(|v| v.get("uuid") == Some(&json!(sheet)))
        })
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let (root, mut model, schematic, sheet) = fixture(name);
    let wire = build(
        &model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"points":[{"x":0,"y":0},{"x":100,"y":0}]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, wire).unwrap();
    let create = build(
        &model,
        &root,
        "datum.schematic.place_text",
        json!({"sheet":sheet,"value":"  note ${literal} Ω  ","x_nm":12,"y_nm":34}),
    )
    .unwrap();
    let id = create.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, create).unwrap();
    let mut raw = raw_sheet(&model, sheet)["texts"][id.to_string()].clone();
    raw["future"] = json!({"retained":[1,"opaque"]});
    raw["position"]["future"] = json!("absolute");
    let install = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicText {
            sheet_id: sheet,
            text_id: id,
            text: raw,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, install).unwrap();
    (root, model, schematic, sheet, id)
}
fn history(model: &mut DesignModel, root: &Path, redo: bool) {
    if redo {
        model
            .commit_journal_redo(root, provenance().into())
            .unwrap();
    } else {
        model
            .commit_journal_undo(root, provenance().into())
            .unwrap();
    }
    *model = ProjectResolver::new(root).resolve().unwrap();
}
#[test]
fn native_text_literal_pose_raw_delete_and_history() {
    let (root, mut model, _, sheet, id) = setup("native_text_history");
    let before = raw_sheet(&model, sheet);
    assert_eq!(
        before["texts"][id.to_string()]["text"],
        json!("  note ${literal} Ω  ")
    );
    let identities = model.electrical_identities.clone();
    let nets = electrical_test_partitions(&model, &[]).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let basis = TextMoveBasis::capture(&snapshot, id, &[]).unwrap();
    let preview = basis
        .destination(Point::new(i64::MIN, i64::MAX), -90)
        .unwrap();
    assert_eq!(preview.rotation, 270);
    assert_eq!(preview.position, Point::new(i64::MIN, i64::MAX));
    assert_eq!(raw_sheet(&model, sheet), before, "preview is read-only");
    let edit = build(
        &model,
        &root,
        "datum.schematic.place_text",
        patch(
            sheet,
            id,
            json!({"value":"Changed literally","x_nm":i64::MIN,"y_nm":i64::MAX,"rotation_deg":-90}),
        ),
    )
    .unwrap();
    commit_prepared(&mut model, &root, edit).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let mut expected = before.clone();
    let text = &mut expected["texts"][id.to_string()];
    text["text"] = json!("Changed literally");
    text["position"]["x"] = json!(i64::MIN);
    text["position"]["y"] = json!(i64::MAX);
    text["rotation"] = json!(270);
    assert_eq!(raw_sheet(&model, sheet), expected);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), nets);
    history(&mut model, &root, false);
    assert_eq!(raw_sheet(&model, sheet), before);
    history(&mut model, &root, true);
    assert_eq!(raw_sheet(&model, sheet), expected);
    let delete = build(
        &model,
        &root,
        "datum.schematic.delete_text",
        json!({"text":id,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, delete).unwrap();
    let mut deleted = expected.clone();
    deleted["texts"]
        .as_object_mut()
        .unwrap()
        .remove(&id.to_string());
    assert_eq!(raw_sheet(&model, sheet), deleted);
    history(&mut model, &root, false);
    assert_eq!(raw_sheet(&model, sheet), expected);
    history(&mut model, &root, true);
    assert_eq!(raw_sheet(&model, sheet), deleted);
    assert_eq!(model.electrical_identities, identities);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_text_invalid_stale_wrong_source_and_lock_refuse() {
    let (root, mut model, _, sheet, id) = setup("native_text_guards");
    let before = raw_sheet(&model, sheet);
    let revision = model.model_revision.clone();
    let journal = model.journal.len();
    for change in [
        json!({}),
        json!({"value":""}),
        json!({"value":" \t"}),
        json!({"value":"two\nlines"}),
        json!({"rotation_deg":45}),
        json!({"rotation_deg":2147483648_i64}),
        json!({"x_nm":1}),
        json!({"x_nm":18446744073709551615_u64,"y_nm":0}),
        json!({"sheet":Uuid::new_v4(),"value":"other"}),
        json!({"instance_path":[Uuid::new_v4()],"value":"other"}),
        json!({"text":Uuid::new_v4(),"value":"other"}),
        json!({"instance_path":null,"value":"other"}),
        json!({"value":"  note ${literal} Ω  "}),
    ] {
        assert!(
            build(
                &model,
                &root,
                "datum.schematic.place_text",
                patch(sheet, id, change)
            )
            .is_err()
        );
    }
    for params in [
        json!({"sheet":sheet,"value":"new"}),
        json!({"sheet":sheet,"x_nm":0,"y_nm":0}),
        json!({"sheet":sheet,"value":"new","x_nm":0,"y_nm":0,"instance_path":[Uuid::new_v4()]}),
    ] {
        assert!(build(&model, &root, "datum.schematic.place_text", params).is_err());
    }
    assert_eq!(raw_sheet(&model, sheet), before);
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.journal.len(), journal);
    let stale = build(
        &model,
        &root,
        "datum.schematic.place_text",
        patch(sheet, id, json!({"value":"stale"})),
    )
    .unwrap();
    let current = build(
        &model,
        &root,
        "datum.schematic.place_text",
        patch(sheet, id, json!({"rotation_deg":90})),
    )
    .unwrap();
    commit_prepared(&mut model, &root, current).unwrap();
    let after = raw_sheet(&model, sheet);
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(raw_sheet(&model, sheet), after);
    let mut locked = after["texts"][id.to_string()].clone();
    locked["locked"] = json!(true);
    let install = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicText {
            sheet_id: sheet,
            text_id: id,
            text: locked,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, install).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(TextMoveBasis::capture(&snapshot, id, &[]).is_err());
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_text",
            patch(sheet, id, json!({"value":"locked"}))
        )
        .is_err()
    );
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.delete_text",
            json!({"text":id,"instance_path":[]})
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_text_mixed_proposal_delete_restores_exact_raw_and_nets() {
    let (root, mut model, _, sheet, id) = setup("native_text_batch");
    let before = raw_sheet(&model, sheet);
    let identities = model.electrical_identities.clone();
    let source = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let wire = *source.schematic_source().unwrap().sheets[&sheet]
        .wires
        .keys()
        .next()
        .unwrap();
    let proposal = Uuid::new_v4();
    let execute = |model: &mut DesignModel, action| {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        schematic_delete_proposal::execute(model, &request).unwrap()
    };
    let draft = execute(
        &mut model,
        NativeProposalAction::CreateDelete {
            proposal_id: proposal,
            sheet,
            instance_path: vec![],
            targets: vec![
                DeleteTarget {
                    class: DeleteClass::SchematicText,
                    id,
                },
                DeleteTarget {
                    class: DeleteClass::Wire,
                    id: wire,
                },
            ],
        },
    );
    assert_eq!(draft.status, ProposalStatus::Draft);
    assert_eq!(draft.deleted.len(), 2);
    assert_eq!(raw_sheet(&model, sheet), before);
    execute(
        &mut model,
        NativeProposalAction::AcceptApply {
            proposal_id: proposal,
        },
    );
    let after = raw_sheet(&model, sheet);
    assert!(after["texts"].as_object().unwrap().is_empty());
    assert!(after["wires"].as_object().unwrap().is_empty());
    history(&mut model, &root, false);
    assert_eq!(raw_sheet(&model, sheet), before);
    assert_eq!(model.electrical_identities, identities);
    history(&mut model, &root, true);
    assert_eq!(raw_sheet(&model, sheet), after);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_text_repeated_occurrence_refuses_instead_of_editing_shared_source() {
    let (root, mut model, schematic, sheet, id) = setup("native_text_occurrence");
    let definition = Uuid::new_v4();
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition,
        "definitions/text.json",
        json!({"uuid":definition,"name":"Text source","root_sheet":sheet}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut paths = Vec::new();
    for name in ["First", "Second"] {
        let instance = Uuid::new_v4();
        paths.push(instance);
        let write=build_create_schematic_sheet_instance(&model,provenance(),schematic,instance,json!({"uuid":instance,"definition":definition,"parent_sheet":null,"position":{"x":0,"y":0},"name":name,"ports":[]})).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let before = raw_sheet(&model, sheet);
    let tip = crate::revision::transaction_tip(&model.journal);
    for path in [vec![], vec![paths[0]], vec![paths[1]]] {
        assert!(
            build(
                &model,
                &root,
                "datum.schematic.place_text",
                patch(sheet, id, json!({"instance_path":path,"value":"shared"}))
            )
            .is_err()
        );
        assert!(
            build(
                &model,
                &root,
                "datum.schematic.delete_text",
                json!({"text":id,"instance_path":path})
            )
            .is_err()
        );
    }
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_text",
            json!({"sheet":sheet,"value":"new","x_nm":0,"y_nm":0})
        )
        .is_err()
    );
    assert_eq!(raw_sheet(&model, sheet), before);
    assert_eq!(crate::revision::transaction_tip(&model.journal), tip);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_text_imported_angle_moves_without_normalization_and_rotates_explicitly() {
    let (root, mut model, _, sheet, id) = setup("native_text_imported_angle");
    let mut original = raw_sheet(&model, sheet)["texts"][id.to_string()].clone();
    original["rotation"] = json!(37);
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicText {
            sheet_id: sheet,
            text_id: id,
            text: original.clone(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let basis = TextMoveBasis::capture(&snapshot, id, &[]).unwrap();
    assert_eq!(
        basis.destination(Point::new(50, 60), 37).unwrap().rotation,
        37
    );
    assert!(basis.destination(Point::new(50, 60), 127).is_err());
    for rotation in [json!({}), json!({"rotation_deg":37})] {
        let mut change = json!({"x_nm":50,"y_nm":60});
        change
            .as_object_mut()
            .unwrap()
            .extend(rotation.as_object().unwrap().clone());
        let write = build(
            &model,
            &root,
            "datum.schematic.place_text",
            patch(sheet, id, change),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let mut expected = original.clone();
        expected["position"]["x"] = json!(50);
        expected["position"]["y"] = json!(60);
        assert_eq!(raw_sheet(&model, sheet)["texts"][id.to_string()], expected);
        history(&mut model, &root, false);
        assert_eq!(raw_sheet(&model, sheet)["texts"][id.to_string()], original);
    }
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_text",
            patch(sheet, id, json!({"rotation_deg":127}))
        )
        .is_err()
    );
    let write = build(
        &model,
        &root,
        "datum.schematic.place_text",
        patch(sheet, id, json!({"rotation_deg":90})),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        raw_sheet(&model, sheet)["texts"][id.to_string()]["rotation"],
        json!(90)
    );
    history(&mut model, &root, false);
    assert_eq!(raw_sheet(&model, sheet)["texts"][id.to_string()], original);
    std::fs::remove_dir_all(root).unwrap();
}
