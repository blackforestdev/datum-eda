use super::super::source_shard::{
    source_shard_taxon_for_path, validate_source_shard_ownership_path,
};
use super::super::source_shard_ref_builders::source_shard_ref_for_bytes;
use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

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
fn source_shard_ownership_allows_canonical_authority_paths() {
    for (kind, relative_path) in [
        (SourceShardKind::ProjectManifest, "project.json"),
        (SourceShardKind::SchematicRoot, "schematic/schematic.json"),
        (SourceShardKind::SchematicSheet, "schematic/main.json"),
        (SourceShardKind::SchematicDefinition, "schematic/defs.json"),
        (SourceShardKind::BoardRoot, "board/board.json"),
        (SourceShardKind::RulesRoot, "rules/rules.json"),
        (SourceShardKind::Pool, "pool/parts/example.json"),
        (
            SourceShardKind::ManufacturingPlan,
            ".datum/manufacturing_plans/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::PanelProjection,
            ".datum/panel_projections/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::OutputJob,
            ".datum/output_jobs/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ComponentInstance,
            ".datum/component_instances/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::Relationship,
            ".datum/relationships/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::VariantOverlay,
            ".datum/variants/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ArtifactMetadata,
            ".datum/artifacts/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ArtifactRun,
            ".datum/artifact_runs/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::OutputJobRun,
            ".datum/output_job_runs/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::CheckRun,
            ".datum/check_runs/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ZoneFill,
            ".datum/zone_fills/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ProposalMetadata,
            ".datum/proposals/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa.json",
        ),
        (
            SourceShardKind::ForwardAnnotationReview,
            ".datum/forward_annotation_review/review.json",
        ),
    ] {
        validate_source_shard_ownership_path(&kind, relative_path)
            .unwrap_or_else(|error| panic!("{kind:?} should own {relative_path}: {error}"));
        if relative_path.starts_with(".datum/") || relative_path.starts_with("pool/") {
            assert!(
                source_shard_taxon_for_path(&kind, relative_path).is_some(),
                "{kind:?} {relative_path} should have concrete source-shard taxon"
            );
        }
    }
}

#[test]
fn source_shard_ownership_rejects_cross_authority_or_unsafe_paths() {
    for (kind, relative_path) in [
        (SourceShardKind::BoardRoot, ".datum/check_runs/run.json"),
        (SourceShardKind::CheckRun, "board/board.json"),
        (
            SourceShardKind::ArtifactMetadata,
            ".datum/output_jobs/job.json",
        ),
        (SourceShardKind::ImportMap, ".datum/artifacts/artifact.json"),
        (
            SourceShardKind::ForwardAnnotationReview,
            ".datum/forward_annotation_review/other.json",
        ),
        (SourceShardKind::SchematicSheet, "../escape.json"),
        (SourceShardKind::Pool, "/tmp/pool/object.json"),
        (SourceShardKind::Pool, "pool/unknown/example.json"),
        (SourceShardKind::Pool, "pool/example.json"),
    ] {
        let error = validate_source_shard_ownership_path(&kind, relative_path)
            .expect_err("invalid source shard owner/path pair should fail");
        assert!(
            error
                .to_string()
                .contains("source shard ownership mismatch"),
            "unexpected error for {kind:?} {relative_path}: {error}"
        );
    }
}

#[test]
fn pool_source_shard_paths_require_concrete_taxonomy() {
    for (relative_path, expected_taxon) in [
        ("pool/units/example.json", SourceShardTaxon::PoolUnit),
        ("pool/symbols/example.json", SourceShardTaxon::PoolSymbol),
        ("pool/entities/example.json", SourceShardTaxon::PoolEntity),
        ("pool/parts/example.json", SourceShardTaxon::PoolPart),
        ("pool/packages/example.json", SourceShardTaxon::PoolPackage),
        (
            "pool/footprints/example.json",
            SourceShardTaxon::PoolFootprint,
        ),
        (
            "pool/padstacks/example.json",
            SourceShardTaxon::PoolPadstack,
        ),
        (
            "pool/pin_pad_maps/example.json",
            SourceShardTaxon::PoolPinPadMap,
        ),
    ] {
        validate_source_shard_ownership_path(&SourceShardKind::Pool, relative_path)
            .unwrap_or_else(|error| panic!("pool should own {relative_path}: {error}"));
        assert_eq!(
            source_shard_taxon_for_path(&SourceShardKind::Pool, relative_path),
            Some(expected_taxon)
        );
    }
}

#[test]
fn byte_backed_source_shard_refs_enforce_ownership_paths() {
    let bytes = br#"{"schema_version":1}"#;
    let error = source_shard_ref_for_bytes(
        SourceShardKind::CheckRun,
        PathBuf::from("board/board.json"),
        "board/board.json".to_string(),
        Some(1),
        bytes,
        "invalid_check_run",
    )
    .expect_err("byte-backed ref should reject cross-authority path");
    assert_eq!(error.code, "invalid_check_run");
    assert!(
        error.message.contains("source shard ownership mismatch"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn byte_backed_source_shard_refs_derive_pool_taxonomy() {
    let bytes = br#"{"schema_version":1}"#;
    let shard = source_shard_ref_for_bytes(
        SourceShardKind::Pool,
        PathBuf::from("pool/symbols/example.json"),
        "pool/symbols/example.json".to_string(),
        Some(1),
        bytes,
        "invalid_pool",
    )
    .expect("byte-backed pool ref should build");
    assert_eq!(shard.taxon, Some(SourceShardTaxon::PoolSymbol));
    assert_eq!(shard.authority, SourceShardAuthority::AuthoredDesign);
    assert_eq!(shard.dirty_state, SourceShardDirtyState::Clean);
}

#[test]
fn resolved_source_shards_use_only_concrete_authorities() {
    let root = temp_project_root("source_shard_concrete_authority");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");

    assert!(!model.source_shards.is_empty());
    for shard in &model.source_shards {
        assert!(
            matches!(
                shard.authority,
                SourceShardAuthority::AuthoredDesign
                    | SourceShardAuthority::ImportedDesign
                    | SourceShardAuthority::SidecarMetadata
                    | SourceShardAuthority::GeneratedEvidence
            ),
            "source shard {} should use a concrete authority",
            shard.relative_path
        );
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
            && shard.taxon == Some(SourceShardTaxon::ImportMap)
            && shard.authority == SourceShardAuthority::SidecarMetadata
            && shard.relative_path == ".datum/import_map/kicad.json"
    }));
}

#[test]
fn resolver_rejects_unsupported_import_map_schema_version() {
    let root = temp_project_root("import_map_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    write_json(
        &root.join(".datum/import_map/future.json"),
        serde_json::json!({
            "schema_version": 2,
            "entries": []
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with import map diagnostic");

    assert!(model.import_map.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_import_map"
            && diagnostic
                .message
                .contains("unsupported ImportMap schema_version 2")
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
fn journaled_import_map_shard_round_trips_through_resolver() {
    let root = temp_project_root("journaled_import_map_sidecar");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = before
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .expect("board shard should exist");
    let board_shard_id = board_shard.shard_id;
    let board_content_hash = board_shard.content_hash.clone();
    let shard = serde_json::json!({
        "schema_version": 1,
        "entries": [{
            "import_key": "kicad:board:main",
            "object_id": board_id,
            "source_shard_id": board_shard_id,
            "source_tool": "kicad",
            "source_path": "fixtures/board.kicad_pcb",
            "source_object_ref": "board-root",
            "source_hash": board_content_hash
        }]
    });

    before
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(before.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create import map shard".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: ".datum/import_map/kicad.json".to_string(),
                    shard,
                }],
            },
        )
        .expect("import map shard should commit");

    assert!(
        root.join(".datum/import_map/kicad.json").exists(),
        "journal promotion should create the import-map shard"
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with persisted import map");
    let entry = after
        .import_map
        .get("kicad:board:main")
        .expect("import map entry should resolve");
    assert_eq!(entry.object_id, board_id);
    assert_eq!(entry.source_shard_id, board_shard_id);
    assert_eq!(entry.source_tool, "kicad");
    assert_eq!(entry.source_path, "fixtures/board.kicad_pcb");
    assert_eq!(entry.source_object_ref, "board-root");
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap
            && shard.taxon == Some(SourceShardTaxon::ImportMap)
            && shard.authority == SourceShardAuthority::SidecarMetadata
            && shard.relative_path == ".datum/import_map/kicad.json"
    }));
}

#[test]
fn journaled_import_map_shard_rejects_unsafe_relative_path() {
    let root = temp_project_root("journaled_import_map_rejects_unsafe_path");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before import map");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject unsafe import map shard".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: "../escape.json".to_string(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": []
                    }),
                }],
            },
        )
        .expect_err("unsafe import-map shard path should fail");

    assert!(
        error
            .to_string()
            .contains("source shard ownership mismatch"),
        "unexpected import-map path validation error: {error}"
    );
    assert!(
        !root.join("escape.json").exists(),
        "unsafe import-map operation must not write outside the project"
    );
}

#[test]
fn journaled_import_map_shard_undo_removes_promoted_sidecar() {
    let root = temp_project_root("journaled_import_map_undo");
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
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create import map shard for undo".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: ".datum/import_map/kicad.json".to_string(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:main",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash.clone()
                        }]
                    }),
                }],
            },
        )
        .expect("import-map create should commit");

    let mut resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after import map create");
    assert!(resolved.import_map.contains_key("kicad:board:main"));

    resolved
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo import map shard".to_string(),
            },
        )
        .expect("undo should remove import-map shard");

    let after_undo = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves after import map undo");
    assert!(after_undo.import_map.is_empty());
    assert!(
        !root.join(".datum/import_map/kicad.json").exists(),
        "undo should remove promoted import-map sidecar"
    );
}

#[test]
fn journal_replay_deleted_import_map_suppresses_stale_promoted_sidecar() {
    let root = temp_project_root("journaled_import_map_deleted_stale_sidecar");
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
    let shard = serde_json::json!({
        "schema_version": 1,
        "entries": [{
            "import_key": "kicad:board:main",
            "object_id": board_id,
            "source_shard_id": board_shard.shard_id,
            "source_tool": "kicad",
            "source_path": "fixtures/board.kicad_pcb",
            "source_object_ref": "board-root",
            "source_hash": board_shard.content_hash.clone()
        }]
    });
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
                    reason: "create import map shard before delete".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: shard.clone(),
                }],
            },
        )
        .expect("import-map create should commit");

    let promoted_path = root.join(&relative_path);
    let stale_promoted_bytes =
        std::fs::read(&promoted_path).expect("promoted import-map sidecar should exist");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete import map shard".to_string(),
                },
                operations: vec![Operation::DeleteImportMapShard {
                    relative_path: relative_path.clone(),
                    shard,
                }],
            },
        )
        .expect("import-map delete should commit");
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted import-map sidecar"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted import-map sidecar should be restored");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with stale promoted import-map sidecar");
    assert!(
        replayed.import_map.is_empty(),
        "journaled delete must suppress stale import-map entries"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ImportMap && shard.relative_path == relative_path
    }));
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
