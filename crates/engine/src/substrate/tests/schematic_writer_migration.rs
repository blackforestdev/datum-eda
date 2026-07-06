use super::*;

#[test]
fn journaled_schematic_wire_create_delete_and_undoes() {
    let root = temp_project_root("schematic_wire_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let wire_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let wire = serde_json::json!({
        "uuid": wire_id,
        "from": { "x": 10, "y": 20 },
        "to": { "x": 30, "y": 40 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "draw schematic wire".to_string(),
                },
                operations: vec![Operation::CreateSchematicWire {
                    sheet_id,
                    wire_id,
                    wire: wire.clone(),
                }],
            },
        )
        .expect("schematic wire create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(sheet["wires"][wire_id.to_string()]["from"]["x"], 10);
    assert!(model.objects.contains_key(&wire_id));

    let mut stale_sheet = sheet.clone();
    stale_sheet["wires"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(replayed_sheet["wires"][wire_id.to_string()]["to"]["y"], 40);

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic wire".to_string(),
            },
        )
        .expect("schematic wire undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert!(undone["wires"].as_object().unwrap().is_empty());
}

#[test]
fn journaled_schematic_junction_create_delete_and_undoes() {
    let root = temp_project_root("schematic_junction_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let junction_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let junction = serde_json::json!({
        "uuid": junction_id,
        "position": { "x": 50, "y": 60 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic junction".to_string(),
                },
                operations: vec![Operation::CreateSchematicJunction {
                    sheet_id,
                    junction_id,
                    junction: junction.clone(),
                }],
            },
        )
        .expect("schematic junction create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["junctions"][junction_id.to_string()]["position"]["x"],
        50
    );
    assert!(model.objects.contains_key(&junction_id));

    let mut stale_sheet = sheet.clone();
    stale_sheet["junctions"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["junctions"][junction_id.to_string()]["position"]["y"],
        60
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic junction".to_string(),
            },
        )
        .expect("schematic junction undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert!(undone["junctions"].as_object().unwrap().is_empty());
}

#[test]
fn journaled_schematic_noconnect_create_delete_and_undoes() {
    let root = temp_project_root("schematic_noconnect_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let noconnect_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let noconnect = serde_json::json!({
        "uuid": noconnect_id,
        "symbol": Uuid::new_v4(),
        "pin": Uuid::new_v4(),
        "position": { "x": 70, "y": 80 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic noconnect".to_string(),
                },
                operations: vec![Operation::CreateSchematicNoConnect {
                    sheet_id,
                    noconnect_id,
                    noconnect: noconnect.clone(),
                }],
            },
        )
        .expect("schematic noconnect create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["noconnects"][noconnect_id.to_string()]["position"]["x"],
        70
    );
    assert!(model.objects.contains_key(&noconnect_id));

    let mut stale_sheet = sheet.clone();
    stale_sheet["noconnects"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["noconnects"][noconnect_id.to_string()]["position"]["y"],
        80
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic noconnect".to_string(),
            },
        )
        .expect("schematic noconnect undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert!(undone["noconnects"].as_object().unwrap().is_empty());
}

#[test]
fn normalized_schematic_marker_junction_commits_replays_and_undoes() {
    let root = temp_project_root("schematic_marker_junction_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let marker_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let marker = serde_json::json!({
        "uuid": marker_id,
        "position": { "x": 90, "y": 100 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic marker".to_string(),
                },
                operations: vec![Operation::PlaceSchematicMarker {
                    sheet_id,
                    marker_id,
                    marker_kind: SchematicMarkerKind::Junction,
                    marker: marker.clone(),
                }],
            },
        )
        .expect("schematic marker create should commit");
    assert!(matches!(
        report.transaction.operations[0],
        Operation::PlaceSchematicMarker {
            marker_kind: SchematicMarkerKind::Junction,
            ..
        }
    ));

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["junctions"][marker_id.to_string()]["position"]["x"],
        90
    );
    assert!(model.objects.contains_key(&marker_id));

    let mut stale_sheet = sheet.clone();
    stale_sheet["junctions"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["junctions"][marker_id.to_string()]["position"]["y"],
        100
    );

    let mut reopened = replayed;
    let undo_report = reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic marker".to_string(),
            },
        )
        .expect("schematic marker undo should commit");
    assert!(matches!(
        undo_report.transaction.operations[0],
        Operation::DeleteSchematicJunction { junction_id, .. } if junction_id == marker_id
    ));
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert!(undone["junctions"].as_object().unwrap().is_empty());
}

#[test]
fn normalized_schematic_marker_noconnect_commits_to_noconnect_map() {
    let root = temp_project_root("schematic_marker_noconnect_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let marker_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let marker = serde_json::json!({
        "uuid": marker_id,
        "symbol": symbol_id,
        "pin": pin_id,
        "position": { "x": 110, "y": 120 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic marker".to_string(),
                },
                operations: vec![Operation::PlaceSchematicMarker {
                    sheet_id,
                    marker_id,
                    marker_kind: SchematicMarkerKind::NoConnect,
                    marker: marker.clone(),
                }],
            },
        )
        .expect("schematic marker create should commit");
    assert!(matches!(
        report.transaction.operations[0],
        Operation::PlaceSchematicMarker {
            marker_kind: SchematicMarkerKind::NoConnect,
            ..
        }
    ));

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["noconnects"][marker_id.to_string()]["position"]["x"],
        110
    );
    assert!(model.objects.contains_key(&marker_id));
}

#[test]
fn journaled_schematic_label_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_label_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let label_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let label = serde_json::json!({
        "uuid": label_id,
        "kind": "Global",
        "name": "VIN",
        "position": { "x": 90, "y": 100 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic label".to_string(),
                },
                operations: vec![Operation::CreateSchematicLabel {
                    sheet_id,
                    label_id,
                    label: label.clone(),
                }],
            },
        )
        .expect("schematic label create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(sheet["labels"][label_id.to_string()]["name"], "VIN");
    assert!(model.objects.contains_key(&label_id));

    let mut renamed = label.clone();
    renamed["name"] = serde_json::json!("VOUT");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "rename schematic label".to_string(),
                },
                operations: vec![Operation::SetSchematicLabel {
                    sheet_id,
                    label_id,
                    label: renamed.clone(),
                }],
            },
        )
        .expect("schematic label set should commit");

    let sheet = read_json_value(&sheet_path).expect("sheet should read after rename");
    assert_eq!(sheet["labels"][label_id.to_string()]["name"], "VOUT");

    let mut stale_sheet = sheet.clone();
    stale_sheet["labels"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["labels"][label_id.to_string()]["name"],
        "VOUT"
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic label rename".to_string(),
            },
        )
        .expect("schematic label set undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert_eq!(undone["labels"][label_id.to_string()]["name"], "VIN");

    reopened
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(reopened.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic label".to_string(),
                },
                operations: vec![Operation::DeleteSchematicLabel {
                    sheet_id,
                    label_id,
                    label,
                }],
            },
        )
        .expect("schematic label delete should commit");
    let deleted = read_json_value(&sheet_path).expect("sheet should read after delete");
    assert!(deleted["labels"].as_object().unwrap().is_empty());

    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic label delete".to_string(),
            },
        )
        .expect("schematic label delete undo should commit");
    let restored = read_json_value(&sheet_path).expect("sheet should read after delete undo");
    assert_eq!(restored["labels"][label_id.to_string()]["name"], "VIN");
}

#[test]
fn journaled_schematic_port_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_port_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let port_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let port = serde_json::json!({
        "uuid": port_id,
        "name": "SUB_IN",
        "direction": "Input",
        "position": { "x": 110, "y": 120 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic port".to_string(),
                },
                operations: vec![Operation::CreateSchematicPort {
                    sheet_id,
                    port_id,
                    port: port.clone(),
                }],
            },
        )
        .expect("schematic port create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(sheet["ports"][port_id.to_string()]["name"], "SUB_IN");
    assert!(model.objects.contains_key(&port_id));

    let mut edited = port.clone();
    edited["name"] = serde_json::json!("SUB_IO");
    edited["direction"] = serde_json::json!("Bidirectional");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "edit schematic port".to_string(),
                },
                operations: vec![Operation::SetSchematicPort {
                    sheet_id,
                    port_id,
                    port: edited.clone(),
                }],
            },
        )
        .expect("schematic port set should commit");

    let sheet = read_json_value(&sheet_path).expect("sheet should read after edit");
    assert_eq!(sheet["ports"][port_id.to_string()]["name"], "SUB_IO");

    let mut stale_sheet = sheet.clone();
    stale_sheet["ports"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["ports"][port_id.to_string()]["name"],
        "SUB_IO"
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic port edit".to_string(),
            },
        )
        .expect("schematic port set undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert_eq!(undone["ports"][port_id.to_string()]["name"], "SUB_IN");

    reopened
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(reopened.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic port".to_string(),
                },
                operations: vec![Operation::DeleteSchematicPort {
                    sheet_id,
                    port_id,
                    port,
                }],
            },
        )
        .expect("schematic port delete should commit");
    let deleted = read_json_value(&sheet_path).expect("sheet should read after delete");
    assert!(deleted["ports"].as_object().unwrap().is_empty());

    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic port delete".to_string(),
            },
        )
        .expect("schematic port delete undo should commit");
    let restored = read_json_value(&sheet_path).expect("sheet should read after delete undo");
    assert_eq!(restored["ports"][port_id.to_string()]["name"], "SUB_IN");
}

#[test]
fn journaled_schematic_bus_and_entry_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_bus_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let bus_id = Uuid::new_v4();
    let entry_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let bus = serde_json::json!({
        "uuid": bus_id,
        "name": "DATA",
        "members": ["D0", "D1"]
    });
    let entry = serde_json::json!({
        "uuid": entry_id,
        "bus": bus_id,
        "wire": null,
        "position": { "x": 130, "y": 140 }
    });

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "create schematic bus".to_string(),
                },
                operations: vec![Operation::CreateSchematicBus {
                    sheet_id,
                    bus_id,
                    bus: bus.clone(),
                }],
            },
        )
        .expect("schematic bus create should commit");

    let mut edited_bus = bus.clone();
    edited_bus["members"] = serde_json::json!(["D0", "D1", "D2"]);
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "edit schematic bus members".to_string(),
                },
                operations: vec![Operation::SetSchematicBus {
                    sheet_id,
                    bus_id,
                    bus: edited_bus.clone(),
                }],
            },
        )
        .expect("schematic bus set should commit");

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic bus entry".to_string(),
                },
                operations: vec![Operation::CreateSchematicBusEntry {
                    sheet_id,
                    bus_entry_id: entry_id,
                    bus_entry: entry.clone(),
                }],
            },
        )
        .expect("schematic bus entry create should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["buses"][bus_id.to_string()]["members"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        sheet["bus_entries"][entry_id.to_string()]["bus"],
        bus_id.to_string()
    );

    let mut stale_sheet = sheet.clone();
    stale_sheet["buses"] = serde_json::json!({});
    stale_sheet["bus_entries"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["bus_entries"][entry_id.to_string()]["bus"],
        bus_id.to_string()
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic bus entry".to_string(),
            },
        )
        .expect("schematic bus entry undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert!(undone["bus_entries"].as_object().unwrap().is_empty());
}

#[test]
fn journaled_schematic_bus_entry_rejects_missing_bus() {
    let root = temp_project_root("schematic_bus_entry_missing_bus");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let entry_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let err = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "place schematic bus entry".to_string(),
                },
                operations: vec![Operation::CreateSchematicBusEntry {
                    sheet_id,
                    bus_entry_id: entry_id,
                    bus_entry: serde_json::json!({
                        "uuid": entry_id,
                        "bus": Uuid::new_v4(),
                        "wire": null,
                        "position": { "x": 1, "y": 2 }
                    }),
                }],
            },
        )
        .expect_err("missing bus should reject");
    assert!(format!("{err}").contains("schematic_sheet_object"));
}
