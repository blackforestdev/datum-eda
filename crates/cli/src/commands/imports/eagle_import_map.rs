use std::path::Path;

use eda_engine::pool::Pool;
use eda_engine::substrate::{ImportMapEntry, ImportMapEntryStatus};

use super::imports::{
    eagle_pool_import_key, eagle_pool_object_refs, eagle_pool_relative_path,
    source_shard_id_for_relative_path,
};

pub(super) fn eagle_pool_import_map_entries(
    existing_import_map: &std::collections::BTreeMap<String, ImportMapEntry>,
    pool_path: &str,
    pool: &Pool,
    source: &Path,
    source_hash: &str,
) -> Vec<ImportMapEntry> {
    let source_path = source.display().to_string();
    let mut desired = std::collections::BTreeMap::new();
    for (object_kind, object_id) in eagle_pool_object_refs(pool) {
        let import_key = eagle_pool_import_key(source, object_kind, object_id);
        let relative_path = eagle_pool_relative_path(pool_path, object_kind, object_id);
        let source_object_ref = eagle_pool_source_object_ref(pool, object_kind, object_id)
            .unwrap_or_else(|| format!("{object_kind}:{object_id}"));
        desired.insert(
            import_key.clone(),
            ImportMapEntry {
                import_key: import_key.clone(),
                object_id,
                source_shard_id: source_shard_id_for_relative_path(&relative_path),
                status: ImportMapEntryStatus::Active,
                source_tool: "eagle".to_string(),
                source_path: source_path.clone(),
                source_object_ref,
                source_hash: source_hash.to_string(),
            },
        );
    }
    for (import_key, entry) in existing_import_map {
        if desired.contains_key(import_key)
            || !is_same_eagle_library_source_entry(entry, &source_path)
        {
            continue;
        }
        let mut entry = entry.clone();
        entry.status = ImportMapEntryStatus::MissingInSource;
        entry.source_hash = source_hash.to_string();
        desired.insert(import_key.clone(), entry);
    }
    if desired.iter().all(|(import_key, entry)| {
        existing_import_map
            .get(import_key)
            .is_some_and(|existing| existing == entry)
    }) {
        return Vec::new();
    }
    desired.into_values().collect()
}

fn is_same_eagle_library_source_entry(entry: &ImportMapEntry, source_path: &str) -> bool {
    entry.source_tool == "eagle" && entry.source_path == source_path
}

fn eagle_pool_source_object_ref(
    pool: &Pool,
    object_kind: &str,
    object_id: uuid::Uuid,
) -> Option<String> {
    match object_kind {
        "units" => pool
            .units
            .get(&object_id)
            .map(|unit| format!("symbol-unit:{}", unit.name)),
        "symbols" => pool
            .symbols
            .get(&object_id)
            .map(|symbol| format!("symbol:{}", symbol.name)),
        "entities" => pool
            .entities
            .get(&object_id)
            .map(|entity| format!("deviceset:{}", entity.name)),
        "packages" => pool
            .packages
            .get(&object_id)
            .map(|package| format!("package:{}", package.name)),
        "parts" => pool.parts.get(&object_id).map(|part| {
            let package = pool
                .packages
                .get(&part.package)
                .map(|package| package.name.as_str())
                .unwrap_or("unknown-package");
            format!("device:{}:package:{package}", part.value)
        }),
        "padstacks" => pool
            .padstacks
            .get(&object_id)
            .map(|padstack| format!("padstack:{}", padstack.name)),
        _ => None,
    }
}
