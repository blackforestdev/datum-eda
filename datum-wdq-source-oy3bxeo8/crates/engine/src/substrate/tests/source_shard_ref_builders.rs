use super::super::source_shard_ref_builders::source_shard_ref_for_staged_write;
use super::*;
use std::path::PathBuf;

#[test]
fn staged_source_shard_refs_enforce_ownership_paths() {
    let error = source_shard_ref_for_staged_write(
        SourceShardKind::CheckRun,
        PathBuf::from("board/board.json"),
        "board/board.json".to_string(),
        Some(1),
        "sha256:test".to_string(),
    )
    .expect_err("staged ref should reject cross-authority path");
    assert!(
        error
            .to_string()
            .contains("source shard ownership mismatch"),
        "unexpected error: {error}"
    );
}

#[test]
fn staged_source_shard_refs_derive_pool_taxonomy() {
    let shard = source_shard_ref_for_staged_write(
        SourceShardKind::Pool,
        PathBuf::from("pool/symbols/example.json"),
        "pool/symbols/example.json".to_string(),
        Some(1),
        "sha256:test".to_string(),
    )
    .expect("staged pool ref should build");
    assert_eq!(shard.taxon, Some(SourceShardTaxon::PoolSymbol));
    assert_eq!(shard.authority, SourceShardAuthority::AuthoredDesign);
    assert_eq!(shard.dirty_state, SourceShardDirtyState::Clean);
    assert_eq!(shard.schema_version, Some(1));
    assert_eq!(shard.content_hash, "sha256:test");
}
