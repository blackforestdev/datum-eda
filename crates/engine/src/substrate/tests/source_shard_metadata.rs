use super::super::import_map::write_legacy_import_map_sidecar;
use super::*;
use std::collections::BTreeMap;

#[test]
fn resolver_marks_native_source_shards_as_authored_clean_authority() {
    let root = temp_project_root("source_shard_authority");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    for kind in [
        SourceShardKind::ProjectManifest,
        SourceShardKind::SchematicRoot,
        SourceShardKind::SchematicSheet,
        SourceShardKind::BoardRoot,
        SourceShardKind::RulesRoot,
    ] {
        let shard = model
            .source_shards
            .iter()
            .find(|shard| shard.kind == kind)
            .unwrap_or_else(|| panic!("missing source shard kind {kind:?}"));
        assert_eq!(shard.authority, SourceShardAuthority::AuthoredDesign);
        assert_eq!(shard.dirty_state, SourceShardDirtyState::Clean);
    }
}

#[test]
fn resolver_discovers_import_map_sidecar_as_identity_metadata() {
    let root = temp_project_root("import_map_sidecar");
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
        &root.join(".datum/import_map/kicad.json"),
        serde_json::json!({
            "schema_version": 1,
            "entries": [{
                "import_key": "kicad:board:main",
                "object_id": board_id,
                "source_shard_id": board_shard.shard_id,
                "source_hash": board_shard.content_hash
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with import map");
    let entry = after
        .import_map
        .get("kicad:board:main")
        .expect("import map entry should resolve");
    assert_eq!(entry.object_id, board_id);
    assert_eq!(entry.source_shard_id, board_shard.shard_id);
    assert_eq!(entry.source_tool, "");
    assert_eq!(entry.source_path, "");
    assert_eq!(entry.source_object_ref, "");
    assert_eq!(after.model_revision, before.model_revision);
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.authority == SourceShardAuthority::SidecarMetadata
            && shard.relative_path == ".datum/import_map/kicad.json"
    }));
}

#[test]
fn import_identity_allocator_reuses_resolved_import_map_identity() {
    let root = temp_project_root("import_identity_reuse");
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
        &root.join(".datum/import_map/kicad.json"),
        serde_json::json!({
            "schema_version": 1,
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
        .expect("project resolves with import map");
    let allocation = allocate_import_identity(&model.import_map, "kicad:board:main");

    assert_eq!(allocation.object_id, board_id);
    assert!(allocation.reused_existing);
}

#[test]
fn import_identity_allocator_allocates_deterministic_new_identity_for_new_key() {
    let import_map = BTreeMap::new();

    let first = allocate_import_identity(&import_map, "kicad:footprint:R1");
    let second = allocate_import_identity(&import_map, "kicad:footprint:R1");
    let other = allocate_import_identity(&import_map, "kicad:footprint:R2");

    assert_eq!(first.object_id, second.object_id);
    assert_ne!(first.object_id, other.object_id);
    assert!(!first.reused_existing);
}

#[test]
fn legacy_import_map_sidecar_round_trips_through_resolver() {
    let root = temp_project_root("persist_import_map_sidecar");
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
    let path = write_legacy_import_map_sidecar(
        &root,
        "kicad.json",
        vec![ImportMapEntry {
            import_key: "kicad:board:main".to_string(),
            object_id: board_id,
            source_shard_id: board_shard.shard_id,
            source_tool: "kicad".to_string(),
            source_path: "fixtures/board.kicad_pcb".to_string(),
            source_object_ref: "board-root".to_string(),
            source_hash: board_shard.content_hash.clone(),
        }],
    )
    .expect("import map shard persists");

    assert_eq!(path, root.join(".datum/import_map/kicad.json"));
    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with persisted import map");
    let entry = after
        .import_map
        .get("kicad:board:main")
        .expect("import map entry should resolve");
    assert_eq!(entry.object_id, board_id);
    assert_eq!(entry.source_shard_id, board_shard.shard_id);
    assert_eq!(entry.source_tool, "kicad");
    assert_eq!(entry.source_path, "fixtures/board.kicad_pcb");
    assert_eq!(entry.source_object_ref, "board-root");
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.authority == SourceShardAuthority::SidecarMetadata
            && shard.relative_path == ".datum/import_map/kicad.json"
    }));
}

#[test]
fn legacy_import_map_sidecar_rejects_unsafe_shard_names() {
    let root = temp_project_root("persist_import_map_rejects_unsafe_names");
    let error = write_legacy_import_map_sidecar(&root, "../escape.json", Vec::new())
        .expect_err("unsafe shard name should fail");

    assert!(error.to_string().contains("invalid import map shard name"));
    assert!(!root.join("escape.json").exists());
}

#[test]
fn resolver_reports_import_map_entries_that_reference_missing_objects() {
    let root = temp_project_root("import_map_missing_object");
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
    let missing_object = Uuid::new_v4();
    write_json(
        &root.join(".datum/import_map/kicad.json"),
        serde_json::json!({
            "schema_version": 1,
            "entries": [{
                "import_key": "kicad:missing:object",
                "object_id": missing_object,
                "source_shard_id": board_shard.shard_id,
                "source_hash": board_shard.content_hash
            }]
        }),
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid import map sidecar");
    assert!(after.import_map.is_empty());
    assert!(after.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "import_map_missing_object"
            && diagnostic.message.contains(&missing_object.to_string())
    }));
}

#[test]
fn resolver_rejects_future_native_source_shard_schema_version() {
    let root = temp_project_root("future_source_shard_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let board_path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_path).expect("read board root"))
            .expect("board root JSON should parse");
    board["schema_version"] = serde_json::json!(2);
    write_json(&board_path, board);

    let error = ProjectResolver::new(&root)
        .resolve()
        .expect_err("future board schema version should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported BoardRoot schema_version 2")
    );
    assert!(error.to_string().contains("board/board.json"));
}
