//! Saved electrical definitions retain source data; this adapter edits only
//! names, existing logical-pin metadata, anchor poses and one rectangular body.
use super::*;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let body_edit = request.kind == "symbols" && request.fields.contains_key("drawings");
    let (target, mut object) = if body_edit {
        super::super::saved_source::load(context.model, request.object_id, "symbols")?
    } else {
        let target = PoolLibraryObjectTarget::new("pool", &request.kind, request.object_id);
        let object = library::pool_library_object_payload(
            context.model,
            &target.relative_path,
            request.object_id,
        )?;
        (target, object)
    };
    let previous = object.clone();
    for (key, value) in &request.fields {
        match (request.kind.as_str(), key.as_str()) {
            (_, "name") => {
                let name = value
                    .as_str()
                    .filter(|s| !s.trim().is_empty())
                    .ok_or_else(|| refusal("Library name must be non-empty"))?;
                object["name"] = json!(name);
            }
            ("units", "pins") => {
                validate_pins(&object["pins"], value)?;
                object["pins"] = value.clone();
            }
            ("symbols", "drawings") => {
                validate_rectangle(&object["drawings"], value)?;
                object["drawings"] = value.clone();
            }
            ("symbols", "pin_anchors") => {
                validate_anchors(&object["pin_anchors"], value)?;
                object["pin_anchors"] = value.clone();
            }
            _ => {
                return Err(refusal(
                    "Saved electrical edits support names, existing pin metadata, anchor geometry and one existing rectangular body only",
                ));
            }
        }
    }
    // Use the same typed definition admission and reference gate as generic Set.
    match request.kind.as_str() {
        "units" => {
            let _: crate::pool::Unit = serde_json::from_value(object.clone())?;
        }
        "symbols" => {
            let _: crate::pool::Symbol = serde_json::from_value(object.clone())?;
            if object != previous {
                require_frozen_placed_sources(context.model, request.object_id, &object)?;
            }
            library_definition::validate_symbol_replacement_references(
                context.model,
                &target.relative_path,
                &object,
            )?;
        }
        _ => return Err(refusal("Expected a saved Unit or Symbol")),
    }
    let mut prepared = library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )?;
    if body_edit {
        prepared.batch.operations.insert(
            0,
            crate::substrate::Operation::GuardObjectRevision {
                object_id: request.object_id,
                expected_object_revision: context.model.objects[&request.object_id].object_revision,
            },
        );
    }
    Ok(prepared)
}

/// The array is captured intent, not authority to rewrite other graphics or metadata.
fn validate_rectangle(previous: &Value, next: &Value) -> Result<(), EngineError> {
    let (Some(previous), Some(next)) = (previous.as_array(), next.as_array()) else {
        return Err(refusal("Symbol drawings must remain an ordered array"));
    };
    if previous.len() != next.len() {
        return Err(refusal(
            "Body correction preserves every graphic and its array position",
        ));
    }
    let indices: Vec<_> = previous
        .iter()
        .enumerate()
        .filter(|(_, graphic)| graphic.get("Rect").is_some())
        .map(|(i, _)| i)
        .collect();
    let [index] = indices.as_slice() else {
        return Err(refusal(
            "Body correction requires exactly one existing rectangle",
        ));
    };
    let _: crate::pool::Primitive = serde_json::from_value(previous[*index].clone())?;
    let crate::pool::Primitive::Rect { min, max, width } =
        serde_json::from_value(next[*index].clone())?
    else {
        return Err(refusal(
            "Body correction preserves the existing rectangle kind",
        ));
    };
    if min.x >= max.x || min.y >= max.y || width <= 0 {
        return Err(refusal(
            "Symbol body requires ordered corners and positive line width",
        ));
    }
    // Patch only scalar leaves in the original raw graphic. In particular, Point
    // and Rect payloads may carry unknown data that typed serialization would lose.
    let mut permitted = previous.clone();
    let rectangle = &mut permitted[*index]["Rect"];
    rectangle["min"]["x"] = json!(min.x);
    rectangle["min"]["y"] = json!(min.y);
    rectangle["max"]["x"] = json!(max.x);
    rectangle["max"]["y"] = json!(max.y);
    rectangle["width"] = json!(width);
    if &permitted != next {
        return Err(refusal(
            "Body correction preserves other graphics, order and unrelated raw rectangle data",
        ));
    }
    Ok(())
}

fn require_frozen_placed_sources(
    model: &crate::substrate::DesignModel,
    symbol: Uuid,
    replacement: &Value,
) -> Result<(), EngineError> {
    if !model.objects.values().any(|o| o.domain == "schematic") {
        return Ok(());
    }
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|error| refusal(format!("Placed schematic source is unavailable: {error:?}")))?;
    if let Some(source) = snapshot.schematic_source() {
        for placed in source
            .sheets
            .values()
            .flat_map(|s| s.symbols.values())
            .filter(|p| p.lib_id.as_deref().and_then(|id| Uuid::parse_str(id).ok()) == Some(symbol))
        {
            if model
                .materialized_placed_symbol_definition(placed.uuid, symbol)?
                .is_none()
            {
                return Err(refusal(
                    "Cannot edit source: an existing placed Symbol has no authenticated frozen library basis",
                ));
            }
        }
    }
    let next_revision = model.objects[&symbol]
        .object_revision
        .0
        .checked_add(1)
        .ok_or_else(|| refusal("Symbol revision counter is exhausted"))?;
    for binding in model
        .component_instances
        .values()
        .flat_map(|ci| ci.library_bindings.values())
        .filter(|binding| {
            binding.binding_role == crate::substrate::LibraryBindingRole::Symbol
                && binding.target_object_id == symbol
                && binding.pinned_object_revision.0 == next_revision
        })
    {
        let pinned = model.materialized_pool_object_revision(
            symbol,
            "symbols",
            binding.pinned_object_revision,
        )?;
        if &pinned != replacement {
            return Err(refusal(
                "Edit would make an existing pinned Symbol revision ambiguous after restored history",
            ));
        }
    }
    Ok(())
}

fn validate_pins(previous: &Value, next: &Value) -> Result<(), EngineError> {
    let (Some(previous), Some(next)) = (previous.as_object(), next.as_object()) else {
        return Err(refusal(
            "Unit pins must remain a map of existing logical pins",
        ));
    };
    if previous.len() != next.len() || previous.keys().any(|key| !next.contains_key(key)) {
        return Err(refusal(
            "Saved Unit editing preserves logical pin identities",
        ));
    }
    let mut identities = HashSet::new();
    for (id, pin) in next {
        if !identities.insert(
            Uuid::parse_str(id).map_err(|_| refusal("Saved logical pin identity is invalid"))?,
        ) {
            return Err(refusal("Saved logical pin identity is ambiguous"));
        }
        if without_fields(pin, &["name", "direction", "electrical_type"])
            != without_fields(&previous[id], &["name", "direction", "electrical_type"])
        {
            return Err(refusal(
                "Saved pin editing preserves identity and unrelated pin data",
            ));
        }
        let name = pin
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty());
        if pin.get("name") != previous[id].get("name") && name.is_none() {
            return Err(refusal("Unit pin name must be non-empty"));
        }
        for field in ["direction", "electrical_type"] {
            if pin.get(field) != previous[id].get(field) {
                let value = pin
                    .get(field)
                    .ok_or_else(|| refusal("Choose a pin electrical type"))?;
                let _: LibraryPinElectricalType = serde_json::from_value(value.clone())?;
            }
        }
        let _: crate::pool::Pin = serde_json::from_value(pin.clone())?;
    }
    Ok(())
}

fn validate_anchors(previous: &Value, next: &Value) -> Result<(), EngineError> {
    let (Some(previous), Some(next)) = (previous.as_array(), next.as_array()) else {
        return Err(refusal("Symbol anchors must remain an ordered array"));
    };
    if previous.len() != next.len() {
        return Err(refusal(
            "Saved Symbol editing preserves anchor identities and order",
        ));
    }
    let mut identities = HashSet::new();
    for (before, after) in previous.iter().zip(next) {
        let id = after
            .get("pin")
            .and_then(Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| refusal("Saved anchor pin identity is invalid"))?;
        if !identities.insert(id) {
            return Err(refusal("Saved anchor pin identity is ambiguous"));
        }
        if without_fields(before, &["position", "style", "orientation", "length_nm"])
            != without_fields(after, &["position", "style", "orientation", "length_nm"])
            || without_fields(&before["position"], &["x", "y"])
                != without_fields(&after["position"], &["x", "y"])
            || without_fields(&before["style"], &["orientation", "length_nm"])
                != without_fields(&after["style"], &["orientation", "length_nm"])
        {
            return Err(refusal(
                "Saved anchor editing preserves pin links and unrelated geometry data",
            ));
        }
        let typed: crate::pool::SymbolPinAnchor = serde_json::from_value(after.clone())?;
        if (after.pointer("/style/length_nm") != before.pointer("/style/length_nm")
            || after.get("length_nm") != before.get("length_nm"))
            && typed.style.length_nm.is_some_and(|n| n < 0)
        {
            return Err(refusal("Pin stub length must not be negative"));
        }
    }
    Ok(())
}

fn without_fields(value: &Value, fields: &[&str]) -> Value {
    let Some(mut value) = value.as_object().cloned() else {
        return if value.is_null() {
            json!({})
        } else {
            value.clone()
        };
    };
    value.retain(|key, _| !fields.contains(&key.as_str()));
    Value::Object(value)
}

#[cfg(test)]
#[path = "library_gui_electrical_edit_tests.rs"]
mod tests;
