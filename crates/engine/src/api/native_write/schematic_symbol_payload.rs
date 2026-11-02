//! Resolver-owned raw symbols for edits that must retain imported extensions.
use crate::{
    error::EngineError, ir::geometry::Point, schematic::PlacedSymbol, substrate::DesignModel,
};
use serde_json::{Value, json};
use uuid::Uuid;

pub(super) fn source_symbol_payload(
    model: &DesignModel,
    sheet: Uuid,
    original: &PlacedSymbol,
) -> Result<Value, EngineError> {
    let invalid = |message: &str| EngineError::Validation(message.into());
    let object = model
        .objects
        .get(&original.uuid)
        .ok_or_else(|| invalid("Selected symbol source object is missing"))?;
    let shards: Vec<_> = model
        .source_shards
        .iter()
        .filter(|shard| shard.shard_id == object.source_shard_id)
        .collect();
    if shards.len() != 1 {
        return Err(invalid("Selected symbol source shard is ambiguous"));
    }
    let payload =
        model.materialized_source_shard_value_by_relative_path(&shards[0].relative_path)?;
    if payload.get("uuid") != Some(&json!(sheet)) {
        return Err(invalid("Selected symbol source sheet does not match"));
    }
    let symbol = payload
        .get("symbols")
        .and_then(|symbols| symbols.get(original.uuid.to_string()))
        .cloned()
        .ok_or_else(|| invalid("Selected symbol payload is missing"))?;
    if serde_json::from_value::<PlacedSymbol>(symbol.clone())? != *original {
        return Err(invalid("Selected symbol payload does not match its source"));
    }
    Ok(symbol)
}

/// Change authored coordinates without discarding extensions on an existing point.
pub(super) fn set_point(payload: &mut Value, point: Point) {
    if !payload.is_object() {
        *payload = json!({});
    }
    payload["x"] = json!(point.x);
    payload["y"] = json!(point.y);
}
