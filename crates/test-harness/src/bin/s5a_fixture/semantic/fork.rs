//! A second authored Sheet retains the root for ordinary native acquisition.
use super::*;
use eda_engine::substrate::{DesignModel, SourceShardKind};
fn remap(value: &mut serde_json::Value, ids: &std::collections::BTreeMap<String, String>) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(replacement) = ids.get(s) {
                *s = replacement.clone();
            }
        }
        serde_json::Value::Array(a) => {
            for child in a {
                remap(child, ids);
            }
        }
        serde_json::Value::Object(map) => {
            let old = std::mem::take(map);
            for (key, mut child) in old {
                remap(&mut child, ids);
                map.insert(ids.get(&key).cloned().unwrap_or(key), child);
            }
        }
        _ => {}
    }
}
pub(super) fn create(
    model: &mut DesignModel,
    path: &Path,
    schematic: Uuid,
    sheet: Uuid,
) -> Result<Uuid> {
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.kind == SourceShardKind::SchematicSheet
                && s.shard_id == model.objects[&sheet].source_shard_id
        })
        .context("actual root Sheet")?;
    let mut source =
        model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    let mut ids = std::collections::BTreeMap::from([
        (sheet.to_string(), id(50).to_string()),
        (
            "50000000-0000-4000-8000-000000001000".into(),
            id(151).to_string(),
        ),
    ]);
    for n in [1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14] {
        ids.insert(id(n).to_string(), id(n + 100).to_string());
    }
    remap(&mut source, &ids);
    source["name"] = "Repeated semantic definition".into();
    source["schema_version"] = serde_json::json!(1);
    let write = schematic_sheets::build_create_schematic_sheet(
        model,
        provenance(),
        schematic,
        id(50),
        "semantic-child.json",
        source,
    )?;
    native_write::commit_prepared(model, path, write)?;
    Ok(id(50))
}
