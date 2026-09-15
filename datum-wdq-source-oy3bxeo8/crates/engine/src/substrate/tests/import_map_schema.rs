use super::*;

#[test]
fn resolver_defaults_legacy_import_map_shard_schema_version() {
    let root = temp_project_root("import_map_legacy_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = before
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    write_json(
        &root.join(".datum/import_map/legacy.json"),
        serde_json::json!({
            "entries": [{
                "import_key": "kicad:board:main",
                "object_id": board_id,
                "source_shard_id": board_shard.shard_id,
                "source_hash": board_shard.content_hash
            }]
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with legacy import map");
    assert!(model.import_map.contains_key("kicad:board:main"));
    assert_eq!(
        model.import_map["kicad:board:main"].status,
        ImportMapEntryStatus::Active
    );
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_import_map")
    );
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap && shard.schema_version.is_none()
    }));
}

#[test]
fn import_map_entry_status_serializes_as_lifecycle_provenance() {
    let entry = ImportMapEntry {
        import_key: "kicad:board:main".to_string(),
        object_id: Uuid::new_v4(),
        source_shard_id: Uuid::new_v4(),
        status: ImportMapEntryStatus::MissingInSource,
        source_tool: "kicad".to_string(),
        source_path: "fixtures/board.kicad_pcb".to_string(),
        source_object_ref: "board-root".to_string(),
        source_hash: "sha256:test".to_string(),
    };

    let value = serde_json::to_value(&entry).expect("entry serializes");
    assert_eq!(value["status"], "missing_in_source");
    let round_trip: ImportMapEntry = serde_json::from_value(value).expect("entry deserializes");
    assert_eq!(round_trip.status, ImportMapEntryStatus::MissingInSource);
}
