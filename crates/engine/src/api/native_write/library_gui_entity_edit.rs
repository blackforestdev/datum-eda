//! Metadata-only patches retain the exact saved gate and optional raw data.
use super::*;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let target = PoolLibraryObjectTarget::new("pool", "entities", request.object_id);
    let candidates = context
        .model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
        .filter(|s| {
            context
                .model
                .materialized_source_shard_value_by_relative_path(&s.relative_path)
                .ok()
                .and_then(|v| {
                    v.get("uuid")
                        .and_then(Value::as_str)
                        .and_then(|s| Uuid::parse_str(s).ok())
                })
                == Some(request.object_id)
        })
        .count();
    if candidates != 1 {
        return Err(refusal(
            "Saved Entity identity has missing or ambiguous source candidates",
        ));
    }
    let source = context
        .model
        .objects
        .get(&request.object_id)
        .filter(|o| o.domain == "pool" && o.kind == "entities")
        .ok_or_else(|| refusal("Saved Entity has no matching canonical source"))?;
    if !context.model.source_shards.iter().any(|s| {
        s.shard_id == source.source_shard_id
            && s.kind == SourceShardKind::Pool
            && s.relative_path == target.relative_path
    }) {
        return Err(refusal(
            "Saved Entity is not owned by the canonical project pool path",
        ));
    }
    let mut object = library::pool_library_object_payload(
        context.model,
        &target.relative_path,
        request.object_id,
    )?;
    for (key, value) in request.fields {
        match key.as_str() {
            "name" | "prefix" | "manufacturer" => {
                let text = value
                    .as_str()
                    .ok_or_else(|| refusal(format!("Entity {key} must be text")))?;
                if key == "name" && text.trim().is_empty() {
                    return Err(refusal("Enter an Entity name"));
                }
            }
            "tags" => {
                let _: Vec<String> = serde_json::from_value(value.clone())?;
            }
            _ => {
                return Err(refusal(
                    "Saved Entity editing supports name, prefix, manufacturer and tags only",
                ));
            }
        }
        object
            .as_object_mut()
            .ok_or_else(|| refusal("Entity payload is not an object"))?
            .insert(key, value);
    }
    let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
    library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )
}
