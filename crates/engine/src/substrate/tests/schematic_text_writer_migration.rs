use super::*;

#[test]
fn journaled_schematic_text_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_text_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let text_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let text = serde_json::json!({
        "uuid": text_id,
        "text": "VIN",
        "position": { "x": 150, "y": 160 },
        "rotation": 90
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
                    reason: "place schematic text".to_string(),
                },
                operations: vec![Operation::CreateSchematicText {
                    sheet_id,
                    text_id,
                    text: text.clone(),
                }],
            },
        )
        .expect("schematic text create should commit");

    let mut edited = text.clone();
    edited["text"] = serde_json::json!("VOUT");
    edited["rotation"] = serde_json::json!(180);
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "edit schematic text".to_string(),
                },
                operations: vec![Operation::SetSchematicText {
                    sheet_id,
                    text_id,
                    text: edited.clone(),
                }],
            },
        )
        .expect("schematic text set should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(sheet["texts"][text_id.to_string()]["text"], "VOUT");

    let mut stale_sheet = sheet.clone();
    stale_sheet["texts"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["texts"][text_id.to_string()]["rotation"],
        180
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic text edit".to_string(),
            },
        )
        .expect("schematic text set undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert_eq!(undone["texts"][text_id.to_string()]["text"], "VIN");

    reopened
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(reopened.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic text".to_string(),
                },
                operations: vec![Operation::DeleteSchematicText {
                    sheet_id,
                    text_id,
                    text,
                }],
            },
        )
        .expect("schematic text delete should commit");
    let deleted = read_json_value(&sheet_path).expect("sheet should read after delete");
    assert!(deleted["texts"].as_object().unwrap().is_empty());
}

#[test]
fn journaled_schematic_drawing_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_drawing_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let drawing_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let drawing = serde_json::json!({
        "Line": {
            "uuid": drawing_id,
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
                    reason: "place schematic drawing".to_string(),
                },
                operations: vec![Operation::CreateSchematicDrawing {
                    sheet_id,
                    drawing_id,
                    drawing: drawing.clone(),
                }],
            },
        )
        .expect("schematic drawing create should commit");

    let mut edited = drawing.clone();
    edited["Line"]["to"] = serde_json::json!({ "x": 50, "y": 60 });
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "edit schematic drawing".to_string(),
                },
                operations: vec![Operation::SetSchematicDrawing {
                    sheet_id,
                    drawing_id,
                    drawing: edited.clone(),
                }],
            },
        )
        .expect("schematic drawing set should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(
        sheet["drawings"][drawing_id.to_string()]["Line"]["to"]["x"],
        50
    );

    let mut stale_sheet = sheet.clone();
    stale_sheet["drawings"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["drawings"][drawing_id.to_string()]["Line"]["to"]["y"],
        60
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic drawing edit".to_string(),
            },
        )
        .expect("schematic drawing set undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert_eq!(
        undone["drawings"][drawing_id.to_string()]["Line"]["to"]["x"],
        30
    );

    reopened
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(reopened.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic drawing".to_string(),
                },
                operations: vec![Operation::DeleteSchematicDrawing {
                    sheet_id,
                    drawing_id,
                    drawing,
                }],
            },
        )
        .expect("schematic drawing delete should commit");
    let deleted = read_json_value(&sheet_path).expect("sheet should read after delete");
    assert!(deleted["drawings"].as_object().unwrap().is_empty());
}

#[test]
fn journaled_schematic_symbol_create_set_delete_and_undoes() {
    let root = temp_project_root("schematic_symbol_journal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let symbol_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let symbol = serde_json::json!({
        "uuid": symbol_id,
        "part": null,
        "entity": null,
        "gate": null,
        "lib_id": "Device:R",
        "reference": "R1",
        "value": "10k",
        "fields": [],
        "pins": [],
        "position": { "x": 10, "y": 20 },
        "rotation": 0,
        "mirrored": false,
        "unit_selection": null,
        "display_mode": "LibraryDefault",
        "pin_overrides": [],
        "hidden_power_behavior": "SourceDefinedImplicit"
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
                    reason: "place schematic symbol".to_string(),
                },
                operations: vec![Operation::CreateSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol: symbol.clone(),
                }],
            },
        )
        .expect("schematic symbol create should commit");

    let mut edited = symbol.clone();
    edited["reference"] = serde_json::json!("R2");
    edited["position"] = serde_json::json!({ "x": 30, "y": 40 });
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "edit schematic symbol".to_string(),
                },
                operations: vec![Operation::SetSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol: edited.clone(),
                }],
            },
        )
        .expect("schematic symbol set should commit");

    let sheet_path = root.join("schematic/sheets/main.json");
    let sheet = read_json_value(&sheet_path).expect("sheet should read");
    assert_eq!(sheet["symbols"][symbol_id.to_string()]["reference"], "R2");

    let mut stale_sheet = sheet.clone();
    stale_sheet["symbols"] = serde_json::json!({});
    write_json(&sheet_path, stale_sheet);
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("reopen should replay journal");
    let replayed_sheet = replayed
        .materialized_source_shard_value(SourceShardKind::SchematicSheet)
        .expect("materialized schematic sheet should read");
    assert_eq!(
        replayed_sheet["symbols"][symbol_id.to_string()]["position"]["x"],
        30
    );

    let mut reopened = replayed;
    reopened
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".to_string(),
                source: CommitSource::Test,
                reason: "undo schematic symbol edit".to_string(),
            },
        )
        .expect("schematic symbol set undo should commit");
    let undone = read_json_value(&sheet_path).expect("sheet should read after undo");
    assert_eq!(undone["symbols"][symbol_id.to_string()]["reference"], "R1");

    reopened
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(reopened.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete schematic symbol".to_string(),
                },
                operations: vec![Operation::DeleteSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol,
                }],
            },
        )
        .expect("schematic symbol delete should commit");
    let deleted = read_json_value(&sheet_path).expect("sheet should read after delete");
    assert!(deleted["symbols"].as_object().unwrap().is_empty());
}
