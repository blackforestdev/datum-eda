//! Bounded native source capture and exact pending pool materializations.
use super::*;

pub(super) fn project_header(
    root: &Path,
    model: &DesignModel,
    operations: &[Operation],
    limits: LibraryAssetLimits,
) -> Result<String, EngineError> {
    let heads: Vec<_> = model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ProjectManifest)
        .collect();
    let [head] = heads.as_slice() else {
        return Err(refusal("project manifest source is absent or duplicated"));
    };
    let raw =
        crate::substrate::library_asset_resolver::manifest_preimage(root, limits.metadata_bytes)?;
    let original_hash = crate::substrate::sha256_hex(raw.as_bytes());
    if head.relative_path != "project.json" || original_hash != head.content_hash {
        return Err(refusal("stale project header preimage"));
    }
    let mut value: Value = serde_json::from_str(&raw)?;
    let original = value.clone();
    if value
        .get("uuid")
        .and_then(Value::as_str)
        .and_then(|v| v.parse::<Uuid>().ok())
        != Some(model.project.project_id)
    {
        return Err(refusal("project header owner differs"));
    }
    let mut changed = false;
    for operation in operations {
        changed |=
            crate::substrate::project_manifest_journal_ops::apply_project_manifest_operation(
                &mut value, operation,
            )?
            .unwrap_or(false);
    }
    crate::substrate::library_asset_resolver::validate_profile_header(&value)?;
    if value.get("schema_version").and_then(Value::as_u64) != Some(2) {
        return Err(refusal("library format baseline is absent or downgraded"));
    }
    if !changed {
        return Ok(original_hash);
    }
    crate::library_assets::verify_native_snapshot(raw.as_bytes(), &original)?;
    Ok(crate::substrate::sha256_hex(
        format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(&value)?
        )
        .as_bytes(),
    ))
}

pub(super) fn pool_index(
    values: &BTreeMap<String, Value>,
) -> Result<BTreeMap<Uuid, String>, EngineError> {
    let mut index = BTreeMap::new();
    for (path, value) in values {
        let id = value
            .get("uuid")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or_else(|| refusal("pool source lacks an unambiguous root identity"))?;
        if index.insert(id, path.clone()).is_some() {
            return Err(refusal("multiple native sources share one asset identity"));
        }
    }
    Ok(index)
}

pub(super) fn pending_pool(
    root: &Path,
    model: &DesignModel,
    operations: &[Operation],
    limits: LibraryAssetLimits,
    managed_paths: &BTreeSet<String>,
) -> Result<(BTreeMap<String, Value>, BTreeMap<String, Value>), EngineError> {
    let mut values = BTreeMap::new();
    let mut used = 0usize;
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        if values.len() >= limits.files {
            return Err(refusal("pool inventory exceeds admission policy"));
        }
        let remaining = limits
            .total_bytes
            .checked_sub(used)
            .ok_or_else(|| refusal("pool raw read budget exhausted"))?;
        // Proposed parent visibility seam: reuse the resolver's bounded reader.
        let bytes = crate::substrate::library_asset_resolver::read_content(
            root,
            &shard.relative_path,
            &BTreeMap::new(),
            limits.blob_bytes.min(remaining),
        )?;
        used = used
            .checked_add(bytes.capacity())
            .ok_or_else(|| refusal("pool read budget overflow"))?;
        if crate::substrate::sha256_hex(&bytes) != shard.content_hash {
            return Err(refusal("stale native source preimage"));
        }
        let value: Value = serde_json::from_slice(&bytes)?;
        if managed_paths.contains(&shard.relative_path) {
            crate::library_assets::verify_native_snapshot(&bytes, &value)?;
        }
        if values.insert(shard.relative_path.clone(), value).is_some() {
            return Err(refusal("duplicate native pool source path"));
        }
    }
    let before = values.clone();
    for operation in operations {
        let Some((path, created, deleted)) = pool_target(operation) else {
            continue;
        };
        validate_source_shard_ownership_path(&SourceShardKind::Pool, path)?;
        if created && values.contains_key(path) {
            return Err(refusal("pool create destination already exists"));
        }
        if !created && !values.contains_key(path) {
            return Err(refusal("pool mutation source is absent"));
        }
        if deleted {
            let previous = match operation {
                Operation::DeletePoolPackage { package, .. } => package,
                Operation::DeletePoolPadstack { padstack, .. } => padstack,
                Operation::DeletePoolLibraryObject { object, .. } => object,
                _ => unreachable!("classified pool deletion"),
            };
            if values.get(path) != Some(previous) {
                return Err(refusal("pending pool Delete preimage differs"));
            }
            if values.remove(path).is_none() {
                return Err(refusal("pool deletion source is absent"));
            }
            continue;
        }
        let value = values.entry(path.into()).or_insert(Value::Null);
        if let Operation::SetPoolLibraryObject {
            previous_object, ..
        } = operation
            && &*value != previous_object
        {
            return Err(refusal("pending pool Set preimage differs"));
        }
        if !apply_pool_shard_operation(&SourceShardKind::Pool, value, operation)? {
            return Err(refusal("unsupported pool projection operation"));
        }
        if values.len() > limits.files {
            return Err(refusal("pending pool inventory exceeds policy"));
        }
    }
    Ok((values, before))
}

fn pool_target(operation: &Operation) -> Option<(&str, bool, bool)> {
    match operation {
        Operation::CreatePoolPackage { relative_path, .. }
        | Operation::CreatePoolPadstack { relative_path, .. }
        | Operation::CreatePoolLibraryObject { relative_path, .. } => {
            Some((relative_path, true, false))
        }
        Operation::DeletePoolPackage { relative_path, .. }
        | Operation::DeletePoolPadstack { relative_path, .. }
        | Operation::DeletePoolLibraryObject { relative_path, .. } => {
            Some((relative_path, false, true))
        }
        Operation::SetPoolLibraryObject { relative_path, .. }
        | Operation::AttachPoolPartModel { relative_path, .. }
        | Operation::DetachPoolPartModel { relative_path, .. } => {
            Some((relative_path, false, false))
        }
        _ => None,
    }
}
