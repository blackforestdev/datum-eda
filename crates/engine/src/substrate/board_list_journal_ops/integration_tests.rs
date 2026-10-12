use crate::substrate::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;
fn write_json(path: &Path, value: serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent dir should create");
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&value).expect("json should serialize"),
    )
    .expect("json should write");
}
fn temp_project_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("datum_project_resolver_{name}_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).expect("temp project root should create");
    root
}
fn write_minimal_project(root: &Path, project_id: Uuid, board_id: Uuid) {
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": project_id,
            "name": "resolver-test",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json"
        }),
    );
    write_json(
        &root.join("schematic/schematic.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"schematic"),
            "sheets": {
                Uuid::new_v5(&project_id, b"sheet").to_string(): "sheets/main.json"
            },
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": []
        }),
    );
    write_json(
        &root.join("schematic/sheets/main.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"sheet"),
            "name": "Main",
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
        }),
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {}
        }),
    );
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "rules": []
        }),
    );
}

#[test]
fn board_list_pass_canonical_commit_reopen_replay_and_undo() {
    let root = temp_project_root("board_list_pass");
    let project = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project, board_id);
    let path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    board["texts"] = serde_json::json!([]);
    write_json(&path, board);
    let original = std::fs::read(&path).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let count = 1024;
    let operations=(0..count).map(|n|{let id=Uuid::new_v5(&project,format!("text-{n}").as_bytes());Operation::CreateBoardText{text_id:id,text:serde_json::json!({"uuid":id,"text":"I","position":{"x":n*2000000,"y":0},"rotation":0,"height_nm":1000000,"stroke_width_nm":100000,"layer":1})}}).collect();
    let provenance = CommitProvenance {
        actor: "unit-test".into(),
        source: CommitSource::Test,
        reason: "canonical indexed board list pass".into(),
    };
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: provenance.clone(),
                operations,
            },
        )
        .unwrap();
    assert_eq!(report.transaction.diff.created.len(), count as usize);
    assert_eq!(report.transaction.inverse_operations.len(), count as usize);
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, model.model_revision);
    let materialized = reopened
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        materialized["texts"].as_array().unwrap().len(),
        count as usize
    );
    let undo = reopened.commit_journal_undo(&root, provenance).unwrap();
    assert_eq!(
        undo.transaction.undo_of,
        Some(report.transaction.transaction_id)
    );
    let final_model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        final_model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["texts"],
        serde_json::json!([])
    );
    // Reconstructing from the original shard and complete journal applies both
    // transactions through the same pass and preserves exact array order.
    let (journal, diagnostics) = crate::substrate::journal::read_transaction_journal(&root);
    assert!(diagnostics.is_empty());
    let mut replay: serde_json::Value = serde_json::from_slice(&original).unwrap();
    crate::substrate::journal::replay_journal_shard_value(
        &SourceShardKind::BoardRoot,
        &mut replay,
        &journal,
    )
    .unwrap();
    assert_eq!(replay["texts"], serde_json::json!([]));
    assert_eq!(journal.len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}
