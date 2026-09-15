use super::*;

#[test]
fn journal_replay_recovers_missing_import_map_sidecar_into_import_map() {
    let root = temp_project_root("journal_replay_recovers_import_map_sidecar");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    let relative_path = ".datum/import_map/kicad.json".to_string();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "recover import map shard from journal".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:main",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash
                        }]
                    }),
                }],
            },
        )
        .expect("import-map create should commit");
    std::fs::remove_file(root.join(&relative_path))
        .expect("promoted import-map sidecar should remove");

    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after import-map sidecar removal");
    assert!(replayed.import_map.contains_key("kicad:board:main"));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Missing
    }));
}

#[test]
fn journal_replay_marks_stale_promoted_import_map_sidecar_dirty() {
    let root = temp_project_root("journal_replay_dirty_import_map_sidecar");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    let relative_path = ".datum/import_map/kicad.json".to_string();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "recover stale import map shard from journal".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:main",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash
                        }]
                    }),
                }],
            },
        )
        .expect("import-map create should commit");
    write_json(
        &root.join(&relative_path),
        serde_json::json!({
            "schema_version": 1,
            "entries": []
        }),
    );

    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after stale import-map sidecar");
    assert!(replayed.import_map.contains_key("kicad:board:main"));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Dirty
    }));
}

#[test]
fn journal_replay_recovers_unreadable_import_map_sidecar_as_unknown() {
    let root = temp_project_root("journal_replay_unknown_import_map_sidecar");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    let relative_path = ".datum/import_map/kicad.json".to_string();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "recover unreadable import map shard from journal".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:main",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash
                        }]
                    }),
                }],
            },
        )
        .expect("import-map create should commit");
    let path = root.join(&relative_path);
    std::fs::remove_file(&path).expect("promoted import-map sidecar should remove");
    std::fs::create_dir(&path).expect("unreadable replacement directory should create");

    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after unreadable import-map sidecar");
    assert!(replayed.import_map.contains_key("kicad:board:main"));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Unknown
    }));
}

#[test]
fn journal_replay_validates_import_map_undo_after_missing_sidecar() {
    let root = temp_project_root("journal_replay_validates_import_map_undo");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    let relative_path = ".datum/import_map/kicad.json".to_string();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create import map shard before undo".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:main",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash
                        }]
                    }),
                }],
            },
        )
        .expect("import-map create should commit");
    std::fs::remove_file(root.join(&relative_path))
        .expect("promoted import-map sidecar should remove");

    let mut replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after missing import-map sidecar");
    replayed
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo import map shard after missing sidecar".to_string(),
            },
        )
        .expect("undo should remove recovered import-map shard");

    let after_undo = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after import-map undo");
    assert!(after_undo.import_map.is_empty());
    assert_eq!(after_undo.journal.len(), 2);
    assert!(!after_undo.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "journal_after_revision_mismatch"
            || diagnostic.code == "journal_cursor_out_of_range"
    }));
}
