//! Source-byte admission, canonical replay and reconstruction share one materializer.
#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
fn disk_project() -> (PathBuf, DesignModel) {
    let root =
        development_workspace::path("materialization-tests").join(Uuid::new_v4().to_string());
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model)
}
fn board_shard(model: &DesignModel) -> SourceShardRef {
    model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .unwrap()
        .clone()
}
fn provenance() -> CommitProvenance {
    CommitProvenance {
        actor: "unit-test".into(),
        source: CommitSource::Test,
        reason: "materialization source parity".into(),
    }
}
#[test]
fn journal_materialization_preserves_raw_canonical_and_replayed_basis() {
    let (root, mut model) = disk_project();
    let mut shard = board_shard(&model);
    let original = read_json_value(&shard.path).unwrap();
    let mut raw = b" \n".to_vec();
    raw.extend(serde_json::to_vec_pretty(&original).unwrap());
    // Actual journal would rename an unmatched source. Both admission branches
    // must retain the descriptor-matched preimage instead of replaying it.
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: provenance(),
                operations: vec![Operation::SetBoardName {
                    board_id: original["uuid"].as_str().unwrap().parse().unwrap(),
                    name: "Changed".into(),
                }],
            },
        )
        .unwrap();
    std::fs::write(&shard.path, &raw).unwrap();
    shard.content_hash = sha256_hex(&raw);
    assert_eq!(materialized_shard_value(&model, &shard).unwrap(), original);
    shard.content_hash = canonical_json_hash(&original).unwrap();
    assert_eq!(materialized_shard_value(&model, &shard).unwrap(), original);
    shard.content_hash = "unmatched".into();
    let replayed = materialized_shard_value(&model, &shard).unwrap();
    assert_eq!(replayed["name"], "Changed");
    assert!(
        matches!(super::super::journal::capture_shard_value(&model,&shard),Err(EngineError::Validation(message)) if message == "stale captured source basis: board/board.json")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn journal_materialization_preserves_malformed_and_unreadable_nonreconstructable_errors() {
    let (root, model) = disk_project();
    let shard = board_shard(&model);
    for bytes in [&b"{"[..], &b"\xff"[..]] {
        std::fs::write(&shard.path, bytes).unwrap();
        assert_eq!(
            materialized_shard_value(&model, &shard)
                .unwrap_err()
                .to_string(),
            read_json_value(&shard.path).unwrap_err().to_string()
        );
    }
    std::fs::remove_file(&shard.path).unwrap();
    assert_eq!(
        materialized_shard_value(&model, &shard)
            .unwrap_err()
            .to_string(),
        read_json_value(&shard.path).unwrap_err().to_string()
    );
    std::fs::create_dir(&shard.path).unwrap();
    assert_eq!(
        materialized_shard_value(&model, &shard)
            .unwrap_err()
            .to_string(),
        read_json_value(&shard.path).unwrap_err().to_string()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn journal_materialization_reconstructs_missing_unreadable_and_malformed_sheet() {
    let (root, mut model) = disk_project();
    let sheet_id = Uuid::new_v4();
    let relative = format!("sheets/{sheet_id}.json");
    let schematic_value = read_json_value(&root.join("schematic/schematic.json")).unwrap();
    let schematic_id = schematic_value["uuid"].as_str().unwrap().parse().unwrap();
    let sheet = serde_json::json!({"schema_version":1,"uuid":sheet_id,"name":"Child","symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}});
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: provenance(),
                operations: vec![Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id,
                    relative_path: relative.clone(),
                    sheet: sheet.clone(),
                }],
            },
        )
        .unwrap();
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.relative_path == format!("schematic/{relative}"))
        .unwrap()
        .clone();
    std::fs::remove_file(&shard.path).unwrap();
    assert_eq!(materialized_shard_value(&model, &shard).unwrap(), sheet);
    std::fs::create_dir(&shard.path).unwrap();
    assert_eq!(materialized_shard_value(&model, &shard).unwrap(), sheet);
    std::fs::remove_dir(&shard.path).unwrap();
    std::fs::write(&shard.path, b"{").unwrap();
    assert_eq!(materialized_shard_value(&model, &shard).unwrap(), sheet);
    std::fs::remove_dir_all(root).unwrap();
}
