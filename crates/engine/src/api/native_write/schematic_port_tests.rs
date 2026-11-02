use super::*;
use crate::{
    api::native_write::{
        commit_prepared, electrical::topology_tests::fixture, registry::find_native_write_verb,
        schematic_connectivity::build_create_schematic_label,
    },
    connectivity::schematic_net_info,
    erc::run_prechecks,
    schematic::{LabelKind, NetLabel, Schematic},
    substrate::{
        CommitSource, DesignModel, Operation, ProjectResolver, electrical_test_partitions,
    },
};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "native interface port")
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
    verb: &str,
    params: serde_json::Value,
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
fn params(sheet: Uuid, port: Option<Uuid>, name: &str) -> serde_json::Value {
    serde_json::json!({"sheet":sheet,"port":port,"instance_path":[],"name":name,"direction":"input","x_nm":100,"y_nm":200})
}
fn has_code(model: &DesignModel, code: &str) -> bool {
    run_prechecks(&source(model))
        .iter()
        .any(|finding| finding.code == code)
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
fn create(model: &mut DesignModel, root: &Path, sheet: Uuid, name: &str) -> Uuid {
    let write = build(
        model,
        root,
        "datum.schematic.place_port",
        params(sheet, None, name),
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}

#[test]
fn native_port_matching_hierarchical_label_edit_delete_and_history_restore_semantics() {
    let (root, mut model, _, sheet) = fixture("native_port_semantics");
    // Interface comparison is active only on sheets that declare ports. Keep
    // an unrelated, matched interface so SIGNAL is a real missing-port case.
    let auxiliary = NetLabel {
        uuid: Uuid::new_v4(),
        name: "AUX".into(),
        kind: LabelKind::Hierarchical,
        position: Point::new(1000, 2000),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &auxiliary).unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut auxiliary_params = params(sheet, None, "AUX");
    auxiliary_params["x_nm"] = serde_json::json!(1000);
    auxiliary_params["y_nm"] = serde_json::json!(2000);
    let write = build(
        &model,
        &root,
        "datum.schematic.place_port",
        auxiliary_params,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(!has_code(&model, "hierarchical_connectivity_mismatch"));
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "SIGNAL".into(),
        kind: LabelKind::Hierarchical,
        position: Point::new(100, 200),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(has_code(&model, "hierarchical_connectivity_mismatch"));
    let before = source(&model);
    let before_ids = model.electrical_identities.clone();
    let id = create(&mut model, &root, sheet, "SIGNAL");
    model = ProjectResolver::new(&root).resolve().unwrap();
    let placed = source(&model);
    assert!(!has_code(&model, "hierarchical_connectivity_mismatch"));
    assert!(!has_code(&model, "unconnected_interface_port"));
    let groups = electrical_test_partitions(&model, &[]).unwrap();
    assert_eq!(groups.len(), 2);
    let signal = groups
        .iter()
        .find(|group| {
            group
                .iter()
                .any(|member| member.class == "ports" && member.source_id == id)
        })
        .unwrap();
    assert_eq!(signal.len(), 2);
    assert!(
        signal
            .iter()
            .any(|member| member.class == "labels" && member.source_id == label.uuid)
    );
    let nets = schematic_net_info(&placed);
    assert_eq!(nets.len(), 2);
    let signal_net = nets.iter().find(|net| net.name == "SIGNAL").unwrap();
    assert_eq!((signal_net.ports, signal_net.labels), (1, 1));
    let placed_ids = model.electrical_identities.clone();
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert_eq!(model.electrical_identities, before_ids);
    assert!(has_code(&model, "hierarchical_connectivity_mismatch"));
    history(&mut model, &root, true);
    assert_eq!(source(&model), placed);
    assert_eq!(model.electrical_identities, placed_ids);
    let mut changed = params(sheet, Some(id), "RENAMED");
    changed["direction"] = serde_json::json!("output");
    let write = build(&model, &root, "datum.schematic.place_port", changed).unwrap();
    assert_eq!(
        write
            .batch
            .operations
            .iter()
            .filter(|op| matches!(op, Operation::SetSchematicPort { .. }))
            .count(),
        1
    );
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let edited = source(&model);
    let mut expected = placed.sheets[&sheet].ports[&id].clone();
    expected.name = "RENAMED".into();
    expected.direction = PortDirection::Output;
    assert_eq!(edited.sheets[&sheet].ports[&id], expected);
    assert!(has_code(&model, "hierarchical_connectivity_mismatch"));
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), groups);
    history(&mut model, &root, false);
    assert_eq!(source(&model), placed);
    assert!(!has_code(&model, "hierarchical_connectivity_mismatch"));
    history(&mut model, &root, true);
    assert_eq!(source(&model), edited);
    let write = build(
        &model,
        &root,
        "datum.schematic.delete_port",
        serde_json::json!({"port":id,"instance_path":[]}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model), before);
    history(&mut model, &root, false);
    assert_eq!(source(&model), edited);
    history(&mut model, &root, true);
    assert_eq!(source(&model), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_port_standalone_birth_has_named_net_erc_and_all_directions() {
    let (root, mut model, _, sheet) = fixture("native_port_standalone");
    let before = source(&model);
    let id = create(&mut model, &root, sheet, "OPEN");
    assert!(has_code(&model, "unconnected_interface_port"));
    let nets = schematic_net_info(&source(&model));
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "OPEN");
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap().len(), 1);
    for (direction, expected) in [
        ("input", PortDirection::Input),
        ("output", PortDirection::Output),
        ("bidirectional", PortDirection::Bidirectional),
        ("passive", PortDirection::Passive),
    ] {
        let mut p = params(sheet, Some(id), "DIRECTION");
        p["direction"] = serde_json::json!(direction);
        let write = build(&model, &root, "datum.schematic.place_port", p).unwrap();
        let port: HierarchicalPort = write
            .batch
            .operations
            .iter()
            .find_map(|op| match op {
                Operation::SetSchematicPort { port, .. } => {
                    Some(serde_json::from_value(port.clone()).unwrap())
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(port.direction, expected);
        assert_eq!(port.uuid, id);
    }
    history(&mut model, &root, false);
    assert_eq!(source(&model), before);
    assert!(!has_code(&model, "unconnected_interface_port"));
    history(&mut model, &root, true);
    assert!(has_code(&model, "unconnected_interface_port"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_port_rejects_wrong_sheet_occurrence_name_and_stale_write_without_mutation() {
    let (root, mut model, _, sheet) = fixture("native_port_guards");
    let p = params(sheet, None, "PORT");
    let before = source(&model);
    build(&model, &root, "datum.schematic.place_port", p).unwrap();
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before,
        "prepare/cancel has no writes"
    );
    let mut nonroot = params(sheet, None, "NEW_PORT");
    nonroot["instance_path"] = serde_json::json!([Uuid::new_v4()]);
    let error = build(&model, &root, "datum.schematic.place_port", nonroot).unwrap_err();
    assert!(error.to_string().contains("supported source-sheet view"));
    assert_eq!(source(&model), before);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    let id = create(&mut model, &root, sheet, "PORT");
    for (field, value) in [
        ("sheet", serde_json::json!(Uuid::new_v4())),
        ("port", serde_json::json!(Uuid::new_v4())),
        ("instance_path", serde_json::json!([Uuid::new_v4()])),
        ("instance_path", serde_json::Value::Null),
        ("name", serde_json::json!("")),
        ("name", serde_json::json!(" \t")),
        ("name", serde_json::json!("A\nB")),
        ("direction", serde_json::json!("invalid")),
    ] {
        let mut p = params(sheet, Some(id), "CHANGED");
        p[field] = value;
        assert!(
            build(&model, &root, "datum.schematic.place_port", p).is_err(),
            "{field}"
        );
    }
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.place_port",
            params(sheet, Some(id), "PORT")
        )
        .is_err()
    );
    assert!(
        build(
            &model,
            &root,
            "datum.schematic.delete_port",
            serde_json::json!({"port":id,"instance_path":[Uuid::new_v4()]})
        )
        .is_err()
    );
    let stale = build(
        &model,
        &root,
        "datum.schematic.place_port",
        params(sheet, Some(id), "STALE"),
    )
    .unwrap();
    let current = build(
        &model,
        &root,
        "datum.schematic.place_port",
        params(sheet, Some(id), "CURRENT"),
    )
    .unwrap();
    commit_prepared(&mut model, &root, current).unwrap();
    let before = source(&model);
    let revision = model.model_revision.clone();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(source(&model), before);
    assert_eq!(model.model_revision, revision);
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap()),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_port_bound_to_sheet_instance_refuses_delete_without_source_or_history_change() {
    use crate::api::native_write::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet,
        build_create_schematic_sheet_instance,
    };
    use crate::schematic::{SheetDefinition, SheetInstance};
    let (root, mut model, schematic, sheet) = fixture("native_port_bound_delete");
    let id = create(&mut model, &root, sheet, "BOUND");
    let child = Uuid::new_v4();
    let write = build_create_schematic_sheet(
        &model,
        provenance(),
        schematic,
        child,
        "sheets/child.json",
        serde_json::json!({"schema_version":1,"uuid":child,"name":"Child",
            "symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},
            "buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let child_label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "BOUND".into(),
        kind: LabelKind::Hierarchical,
        position: Point::zero(),
    };
    let write = build_create_schematic_label(&model, provenance(), child, &child_label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: child,
        name: "Child".into(),
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/child.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let instance = SheetInstance {
        uuid: Uuid::new_v4(),
        definition: definition.uuid,
        parent_sheet: Some(sheet),
        position: Point::zero(),
        name: "Child".into(),
        ports: vec![id],
    };
    let write = build_create_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        instance.uuid,
        serde_json::to_value(&instance).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    // Ensure this reaches the binding guard, not an unrelated occurrence refusal.
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    assert_eq!(
        selected_source(&context, "ports", id, &[]).unwrap().1,
        sheet
    );
    let before = source(&model);
    let revision = model.model_revision.clone();
    let identities = model.electrical_identities.clone();
    let journal = std::fs::read(root.join(crate::substrate::JOURNAL_RELATIVE_PATH)).unwrap();
    let error = build(
        &model,
        &root,
        "datum.schematic.delete_port",
        serde_json::json!({"port":id,"instance_path":[]}),
    )
    .unwrap_err();
    assert!(error.to_string().contains("bound to a sheet instance"));
    assert_eq!(source(&model), before);
    assert_eq!(model.model_revision, revision);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(
        std::fs::read(root.join(crate::substrate::JOURNAL_RELATIVE_PATH)).unwrap(),
        journal
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&reopened), before);
    assert_eq!(reopened.model_revision, revision);
    assert_eq!(reopened.electrical_identities, identities);
    std::fs::remove_dir_all(root).unwrap();
}
