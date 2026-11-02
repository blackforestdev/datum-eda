//! Resolver-owned raw source members for edits that retain imported extensions.
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
    source_member_payload(model, sheet, "symbols", original.uuid, original)
}

fn source_member_payload<T: serde::de::DeserializeOwned + PartialEq>(
    model: &DesignModel,
    sheet: Uuid,
    class: &str,
    id: Uuid,
    original: &T,
) -> Result<Value, EngineError> {
    let payload = source_sheet_payload(model, sheet, id)?;
    member_payload(&payload, class, id, original)
}

pub(super) fn source_sheet_payload(
    model: &DesignModel,
    sheet: Uuid,
    id: Uuid,
) -> Result<Value, EngineError> {
    let invalid = |message: &str| EngineError::Validation(message.into());
    let object = model
        .objects
        .get(&id)
        .ok_or_else(|| invalid("Selected schematic member source object is missing"))?;
    let shards: Vec<_> = model
        .source_shards
        .iter()
        .filter(|shard| shard.shard_id == object.source_shard_id)
        .collect();
    if shards.len() != 1 {
        return Err(invalid(
            "Selected schematic member source shard is ambiguous",
        ));
    }
    let payload =
        model.materialized_source_shard_value_by_relative_path(&shards[0].relative_path)?;
    if payload.get("uuid") != Some(&json!(sheet)) {
        return Err(invalid(
            "Selected schematic member source sheet does not match",
        ));
    }
    Ok(payload)
}

pub(super) fn member_payload<T: serde::de::DeserializeOwned + PartialEq>(
    payload: &Value,
    class: &str,
    id: Uuid,
    original: &T,
) -> Result<Value, EngineError> {
    let invalid = |message: &str| EngineError::Validation(message.into());
    let member = payload
        .get(class)
        .and_then(|symbols| symbols.get(id.to_string()))
        .cloned()
        .ok_or_else(|| invalid("Selected schematic member payload is missing"))?;
    if serde_json::from_value::<T>(member.clone())? != *original {
        return Err(invalid(
            "Selected schematic member payload does not match its source",
        ));
    }
    Ok(member)
}

/// Change authored coordinates without discarding extensions on an existing point.
pub(super) fn set_point(payload: &mut Value, point: Point) {
    if !payload.is_object() {
        *payload = json!({});
    }
    payload["x"] = json!(point.x);
    payload["y"] = json!(point.y);
}
