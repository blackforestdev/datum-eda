use super::*;
use crate::{
    api::native_write::{
        commit_prepared, electrical::topology_tests::fixture,
        schematic_connectivity::build_create_schematic_label, schematic_edit::prepare_native_birth,
        schematic_sheets::build_create_schematic_sheet,
    },
    ir::geometry::Point,
    schematic::{NetLabel, Schematic},
    substrate::{
        CommitSource, DesignModel, ElectricalSelectionSnapshot, Operation, ProjectResolver,
        electrical_test_partitions,
    },
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "edit selected label")
}
fn source(model: &DesignModel) -> Schematic {
    ElectricalSelectionSnapshot::capture(model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone()
}
fn build(
    model: &DesignModel,
    root: &Path,
    label: Uuid,
    name: &str,
    kind: Option<&str>,
) -> Result<PreparedWrite, EngineError> {
    rename_label(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        serde_json::json!({"label":label,"instance_path":[],"name":name,"kind":kind}),
    )
}
fn add_label(model: &mut DesignModel, root: &Path, sheet: Uuid, name: &str) -> NetLabel {
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: name.into(),
        kind: LabelKind::Global,
        position: Point::new(100, 200),
    };
    let write = build_create_schematic_label(model, provenance(), sheet, &label).unwrap();
    let write = prepare_native_birth(model, write).unwrap();
    commit_prepared(model, root, write).unwrap();
    label
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
fn native_label_edit_rename_and_kind_change_repartition_with_history_and_reopen() {
    let (root, mut model, schematic, first_sheet) = fixture("label_edit_topology");
    let second_sheet = Uuid::new_v4();
    let write = build_create_schematic_sheet(&model, provenance(), schematic, second_sheet, "sheets/second.json", serde_json::json!({"schema_version":1,"uuid":second_sheet,"name":"Second","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let first = add_label(&mut model, &root, first_sheet, "A");
    let second = add_label(&mut model, &root, second_sheet, "B");
    let before = source(&model);
    let initial_partitions = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(initial_partitions.len(), 2);
    let initial_identities = model.electrical_identities.clone();
    let write = build(&model, &root, second.uuid, "A", None).unwrap();
    assert_eq!(write.primary_object_id, Some(second.uuid));
    assert_eq!(
        write.batch.expected_model_revision,
        Some(model.model_revision.clone())
    );
    assert_eq!(
        write
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::SetSchematicLabel { .. }))
            .count(),
        1
    );
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before,
        "prepare/cancel does not write"
    );
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let merged = source(&model);
    let mut expected = second.clone();
    expected.name = "A".into();
    assert_eq!(merged.sheets[&second_sheet].labels[&second.uuid], expected);
    assert_eq!(merged.sheets[&first_sheet].labels[&first.uuid], first);
    let merged_partitions = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(merged_partitions.len(), 1);
    assert_eq!(merged_partitions[0].len(), 2);
    let merged_identities = model.electrical_identities.clone();
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        initial_partitions
    );
    assert_eq!(model.electrical_identities, initial_identities);
    history(&mut model, &root, true);
    assert_eq!(source(&model), merged);
    assert_eq!(model.electrical_identities, merged_identities);
    let write = build(&model, &root, second.uuid, "A", Some("local")).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let split = source(&model);
    expected.kind = LabelKind::Local;
    assert_eq!(split.sheets[&second_sheet].labels[&second.uuid], expected);
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        initial_partitions
    );
    let split_identities = model.electrical_identities.clone();
    history(&mut model, &root, false);
    assert_eq!(source(&model), merged);
    assert_eq!(
        electrical_test_partitions(&model, &[]).unwrap(),
        merged_partitions
    );
    assert_eq!(model.electrical_identities, merged_identities);
    history(&mut model, &root, true);
    assert_eq!(source(&model), split);
    assert_eq!(model.electrical_identities, split_identities);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_label_edit_rejects_invalid_occurrence_name_and_stale_commit() {
    let (root, mut model, _, sheet) = fixture("label_edit_guards");
    let label = add_label(&mut model, &root, sheet, "A");
    for name in ["", "   ", "A\nB", "A\tB", "A"] {
        assert!(build(&model, &root, label.uuid, name, None).is_err());
    }
    assert!(build(&model, &root, Uuid::new_v4(), "B", None).is_err());
    assert!(build(&model, &root, label.uuid, "B", Some("unknown")).is_err());
    assert!(
        rename_label(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            provenance(),
            serde_json::json!({"label":label.uuid,"instance_path":[Uuid::new_v4()],"name":"B"})
        )
        .is_err()
    );
    for (kind, expected) in [
        ("local", LabelKind::Local),
        ("global", LabelKind::Global),
        ("hierarchical", LabelKind::Hierarchical),
        ("power", LabelKind::Power),
    ] {
        let write = build(&model, &root, label.uuid, "B", Some(kind)).unwrap();
        let edited: NetLabel = write
            .batch
            .operations
            .iter()
            .find_map(|op| match op {
                Operation::SetSchematicLabel { label, .. } => {
                    Some(serde_json::from_value(label.clone()).unwrap())
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(edited.uuid, label.uuid);
        assert_eq!(edited.position, label.position);
        assert_eq!(edited.kind, expected);
    }
    let stale = build(&model, &root, label.uuid, "STALE", None).unwrap();
    let current = build(&model, &root, label.uuid, "CURRENT", Some("power")).unwrap();
    commit_prepared(&mut model, &root, current).unwrap();
    let before = source(&model);
    let identities = model.electrical_identities.clone();
    let revision = model.model_revision.clone();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(source(&model), before);
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}
