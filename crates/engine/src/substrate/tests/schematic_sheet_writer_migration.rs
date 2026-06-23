use super::*;

fn empty_sheet(sheet_id: Uuid, name: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": sheet_id,
        "name": name,
        "symbols": {},
        "wires": {},
        "junctions": {},
        "labels": {},
        "buses": {},
        "bus_entries": {},
        "ports": {},
        "noconnects": {},
        "texts": {},
        "drawings": {}
    })
}

#[test]
fn journaled_schematic_sheet_create_delete_and_undoes() {
    let root = temp_project_root("schematic_sheet_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let schematic_id = Uuid::new_v5(&project_id, b"schematic");
    let sheet_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let relative_path = format!("sheets/{sheet_id}.json");
    let sheet = empty_sheet(sheet_id, "Child");

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
                    reason: "create schematic sheet".to_string(),
                },
                operations: vec![Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative_path.clone(),
                    sheet: sheet.clone(),
                }],
            },
        )
        .expect("schematic sheet create should commit");

    let schematic_path = root.join("schematic/schematic.json");
    let sheet_path = root.join("schematic").join(&relative_path);
    let schematic = read_json_value(&schematic_path).expect("schematic root should read");
    assert_eq!(schematic["sheets"][sheet_id.to_string()], relative_path);
    assert_eq!(
        read_json_value(&sheet_path).expect("created sheet should read")["name"],
        "Child"
    );
    assert!(model.objects.contains_key(&sheet_id));

    let mut stale_root = schematic.clone();
    stale_root["sheets"]
        .as_object_mut()
        .expect("sheets should be an object")
        .remove(&sheet_id.to_string());
    write_json(&schematic_path, stale_root);
    std::fs::remove_file(&sheet_path).expect("promoted sheet should remove");
    let stale_root = read_json_value(&schematic_path).expect("stale root should read");
    assert!(
        !stale_root["sheets"]
            .as_object()
            .unwrap()
            .contains_key(&sheet_id.to_string())
    );
    assert!(!sheet_path.exists());

    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
        .expect("materialized created sheet should read");
    assert_eq!(replayed_sheet["name"], "Child");
    assert!(replayed.objects.contains_key(&sheet_id));

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic sheet".to_string(),
            },
        )
        .expect("schematic sheet undo should commit");
    let undone_root = read_json_value(&schematic_path).expect("schematic root should read");
    assert!(
        !undone_root["sheets"]
            .as_object()
            .unwrap()
            .contains_key(&sheet_id.to_string())
    );
    assert!(!sheet_path.exists());
}

#[test]
fn journaled_schematic_sheet_recovers_promoted_root_missing_sheet_file() {
    let root = temp_project_root("schematic_sheet_promoted_root_missing_sheet");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let schematic_id = Uuid::new_v5(&project_id, b"schematic");
    let sheet_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let relative_path = format!("sheets/{sheet_id}.json");
    let sheet = empty_sheet(sheet_id, "CrashRecovered");

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
                    reason: "create schematic sheet".to_string(),
                },
                operations: vec![Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative_path.clone(),
                    sheet: sheet.clone(),
                }],
            },
        )
        .expect("schematic sheet create should commit");

    let sheet_path = root.join("schematic").join(&relative_path);
    std::fs::remove_file(&sheet_path).expect("promoted sheet should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should tolerate root-referenced missing journal sheet");
    assert!(replayed.objects.contains_key(&sheet_id));
    assert!(replayed.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "missing_referenced_schematic_sheet"
            && diagnostic.path.as_ref() == Some(&sheet_path)
    }));
    let replayed_sheet = replayed
        .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
        .expect("materialized created sheet should read");
    assert_eq!(replayed_sheet["name"], "CrashRecovered");
}

#[test]
fn journaled_schematic_sheet_delete_cascades_payload_objects_and_undoes() {
    let root = temp_project_root("schematic_sheet_payload_cascade");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let schematic_id = Uuid::new_v5(&project_id, b"schematic");
    let sheet_id = Uuid::new_v4();
    let label_id = Uuid::new_v4();
    let wire_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let relative_path = format!("sheets/{sheet_id}.json");
    let mut sheet = empty_sheet(sheet_id, "Populated");
    sheet["labels"] = serde_json::json!({
        label_id.to_string(): {
            "uuid": label_id,
            "kind": "global",
            "name": "VIN",
            "position": { "x": 10, "y": 20 }
        }
    });
    sheet["wires"] = serde_json::json!({
        wire_id.to_string(): {
            "uuid": wire_id,
            "from": { "x": 10, "y": 20 },
            "to": { "x": 30, "y": 40 }
        }
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
                    reason: "create populated schematic sheet".to_string(),
                },
                operations: vec![Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative_path.clone(),
                    sheet: sheet.clone(),
                }],
            },
        )
        .expect("populated sheet create should commit");
    assert!(model.objects.contains_key(&sheet_id));
    assert!(model.objects.contains_key(&label_id));
    assert!(model.objects.contains_key(&wire_id));

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete populated schematic sheet".to_string(),
                },
                operations: vec![Operation::DeleteSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative_path.clone(),
                    sheet: sheet.clone(),
                }],
            },
        )
        .expect("populated sheet delete should commit");
    assert!(!model.objects.contains_key(&sheet_id));
    assert!(!model.objects.contains_key(&label_id));
    assert!(!model.objects.contains_key(&wire_id));

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo populated sheet delete".to_string(),
            },
        )
        .expect("populated sheet undo should commit");
    assert!(model.objects.contains_key(&sheet_id));
    assert!(model.objects.contains_key(&label_id));
    assert!(model.objects.contains_key(&wire_id));
}
