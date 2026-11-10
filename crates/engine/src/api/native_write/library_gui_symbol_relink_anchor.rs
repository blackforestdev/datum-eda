//! Correct a saved anchor reference without reconstructing its ordered raw row.
use super::*;
use std::collections::BTreeMap;

// Existing Add/Remove membership admission is shared by reference correction.
pub(super) fn retained_unit(
    model: &crate::substrate::DesignModel,
    unit: Uuid,
) -> Result<Value, EngineError> {
    let (_, raw) = super::super::super::saved_source::reference(model, unit, "units")?;
    if raw["schema_version"].as_u64() != Some(1) {
        return Err(refusal("Retained Unit requires supported schema version 1"));
    }
    let _: crate::pool::Unit = serde_json::from_value(raw.clone())?;
    let pins = raw["pins"]
        .as_object()
        .ok_or_else(|| refusal("Retained Unit pins unavailable"))?;
    let mut members = HashSet::new();
    for (key, value) in pins {
        let id: Uuid = key
            .parse()
            .map_err(|_| refusal("Unit pin identity is malformed"))?;
        let pin: crate::pool::Pin = serde_json::from_value(value.clone())?;
        if !members.insert(id)
            || pin.uuid != id
            || model.objects.get(&id).is_none_or(|owner| {
                owner.source_shard_id != model.objects[&unit].source_shard_id
                    || owner.domain != "pool"
                    || (owner.kind != "pins" && owner.kind.parse::<Uuid>().ok() != Some(id))
            })
        {
            return Err(refusal(
                "Retained Unit pin membership is missing or ambiguous",
            ));
        }
    }
    Ok(raw)
}

pub(super) fn apply(
    request: &LibraryDefinitionRequest,
    original: &Value,
    object: &mut Value,
    removals: &HashSet<Uuid>,
    unit: &Value,
) -> Result<HashSet<Uuid>, EngineError> {
    let changes = request.fields["anchor_relinks"]
        .as_object()
        .filter(|rows| !rows.is_empty())
        .ok_or_else(|| refusal("Choose an existing anchor and a named Unit pin"))?;
    let before = original["pin_anchors"]
        .as_array()
        .ok_or_else(|| refusal("Existing anchor correction requires an ordered source array"))?;
    let pins = unit["pins"].as_object().unwrap();
    let mut positions = BTreeMap::new();
    for (index, raw) in before.iter().enumerate() {
        let anchor: crate::pool::SymbolPinAnchor = serde_json::from_value(raw.clone())?;
        if anchor.pin.is_nil() || positions.insert(anchor.pin, index).is_some() {
            return Err(refusal("Existing Symbol anchor identity is ambiguous"));
        }
        crate::api::native_write::schematic_library::unit_pin(pins, anchor.pin)?;
    }
    let mut corrections = BTreeMap::new();
    let mut targets = HashSet::new();
    for (key, value) in changes {
        let old: Uuid = key
            .parse()
            .map_err(|_| refusal("Original anchor identity is invalid"))?;
        let next: Uuid = serde_json::from_value(value.clone())?;
        if !positions.contains_key(&old) || corrections.insert(old, next).is_some() {
            return Err(refusal(
                "Anchor correction requires one unique original row",
            ));
        }
        if removals.contains(&old)
            || request
                .fields
                .get("anchor_additions")
                .and_then(Value::as_array)
                .is_some_and(|rows| {
                    rows.iter().any(|row| {
                        row["pin"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(old)
                    })
                })
        {
            return Err(refusal(
                "The same anchor cannot be corrected and removed or added in one draft",
            ));
        }
        if next.is_nil() || !targets.insert(next) || (next != old && positions.contains_key(&next))
        {
            return Err(refusal(
                "Choose an unanchored named pin; occupied targets and swaps are unavailable",
            ));
        }
        let target = crate::api::native_write::schematic_library::unit_pin(pins, next)?;
        if next != old
            && target["name"]
                .as_str()
                .is_none_or(|name| name.trim().is_empty())
        {
            return Err(refusal("Choose an existing named Unit pin"));
        }
    }
    let after = object["pin_anchors"]
        .as_array_mut()
        .ok_or_else(|| refusal("Existing anchor correction requires the original ordered rows"))?;
    if after.len() != before.len() {
        return Err(refusal(
            "Anchor correction preserves the original row positions",
        ));
    }
    let mut references = HashSet::new();
    for (old, next) in corrections {
        let index = positions[&old];
        if after[index]["pin"] != before[index]["pin"] {
            return Err(refusal(
                "Anchor correction requires the captured original row identity",
            ));
        }
        // Same-target selection retains even noncanonical UUID spelling.
        if old != next {
            after[index]["pin"] = json!(next);
        }
        references.extend([old, next]);
    }
    Ok(references)
}
