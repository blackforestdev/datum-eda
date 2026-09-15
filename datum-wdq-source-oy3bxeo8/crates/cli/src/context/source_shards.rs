use eda_engine::substrate::{DesignModel, SourceShardDirtyState};
use serde_json::{Map, Value};

pub(super) fn empty_source_shard_status() -> Value {
    serde_json::json!({
        "total": 0,
        "clean": 0,
        "dirty": 0,
        "missing": 0,
        "unknown": 0,
        "attention": []
    })
}

pub(super) fn update_source_shard_status(object: &mut Map<String, Value>, model: &DesignModel) {
    let mut clean = 0;
    let mut dirty = 0;
    let mut missing = 0;
    let mut unknown = 0;
    for shard in &model.source_shards {
        match shard.dirty_state {
            SourceShardDirtyState::Clean => clean += 1,
            SourceShardDirtyState::Dirty => dirty += 1,
            SourceShardDirtyState::Missing => missing += 1,
            SourceShardDirtyState::Unknown => unknown += 1,
        }
    }
    let attention = model
        .source_shards
        .iter()
        .filter(|shard| shard.dirty_state != SourceShardDirtyState::Clean)
        .map(|shard| {
            serde_json::json!({
                "relative_path": shard.relative_path,
                "kind": shard.kind,
                "authority": shard.authority,
                "taxon": shard.taxon,
                "dirty_state": shard.dirty_state
            })
        })
        .collect::<Vec<_>>();
    object.insert(
        "source_shard_status".to_string(),
        serde_json::json!({
            "total": model.source_shards.len(),
            "clean": clean,
            "dirty": dirty,
            "missing": missing,
            "unknown": unknown,
            "attention": attention
        }),
    );
}
