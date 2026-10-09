//! One observed root source backs its descriptor and collected objects.
use super::*;

#[test]
fn loaded_source_snapshot_preserves_descriptor_bytes_and_native_refusals() {
    let root = temp_project_root("loaded_source_snapshot");
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
    let path = root.join("board/board.json");
    let original = std::fs::read(&path).unwrap();
    let mut bytes = b" \n".to_vec();
    bytes.extend_from_slice(&original);
    std::fs::write(&path, &bytes).unwrap();
    let (shard, value) = super::super::source_shard::load_source_shard(
        &root,
        SourceShardKind::BoardRoot,
        "board/board.json",
    )
    .unwrap();
    assert_eq!(shard.content_hash, super::super::sha256_hex(&bytes));
    assert_eq!(
        value,
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
    );
    assert_eq!(
        shard,
        super::super::source_shard::read_source_shard(
            &root,
            SourceShardKind::BoardRoot,
            "board/board.json",
            None
        )
        .unwrap()
    );
    assert!(
        super::super::source_shard::load_source_shard(
            &root,
            SourceShardKind::BoardRoot,
            "project.json"
        )
        .is_err()
    );
    let mut unsupported = value;
    unsupported["schema_version"] = serde_json::json!(999);
    std::fs::write(&path, serde_json::to_vec(&unsupported).unwrap()).unwrap();
    let loaded = super::super::source_shard::load_source_shard(
        &root,
        SourceShardKind::BoardRoot,
        "board/board.json",
    )
    .unwrap_err();
    let existing = super::super::source_shard::read_source_shard(
        &root,
        SourceShardKind::BoardRoot,
        "board/board.json",
        None,
    )
    .unwrap_err();
    assert_eq!(loaded.to_string(), existing.to_string());
    std::fs::write(&path, b"{").unwrap();
    assert!(
        super::super::source_shard::load_source_shard(
            &root,
            SourceShardKind::BoardRoot,
            "board/board.json"
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
