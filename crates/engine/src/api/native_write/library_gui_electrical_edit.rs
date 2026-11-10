//! Saved electrical edits retain raw siblings while composing supported pin,
//! anchor, metadata and rectangular-body intent through the canonical writer.
use super::*;
#[path = "library_gui_symbol_relink_anchor.rs"]
mod anchor_relinks;
#[path = "library_gui_symbol_graphic_edit.rs"]
mod graphics;
#[path = "library_gui_unit_pin_usage.rs"]
mod unit_pin_usage;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let body_edit = request.kind == "symbols" && request.fields.contains_key("drawings");
    let graphic_edit = request.kind == "symbols"
        && [
            "line_edits",
            "line_removals",
            "line_additions",
            "rect_edits",
            "rect_removals",
            "graphic_additions",
            "text_edits",
            "text_removals",
        ]
        .iter()
        .any(|key| request.fields.contains_key(*key));
    let adding = request.kind == "units" && request.fields.contains_key("pin_additions");
    let pin_remove = request.kind == "units" && request.fields.contains_key("pin_removals");
    let anchor_add = request.kind == "symbols" && request.fields.contains_key("anchor_additions");
    let anchor_remove = request.kind == "symbols" && request.fields.contains_key("anchor_removals");
    let anchor_relink = request.kind == "symbols" && request.fields.contains_key("anchor_relinks");
    let anchor_edit = anchor_add || anchor_remove || anchor_relink;
    let (target, mut object) = if body_edit || graphic_edit || adding || pin_remove || anchor_edit {
        super::super::saved_source::load(context.model, request.object_id, &request.kind)?
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
    let pin_removals = if pin_remove {
        unit_pin_usage::removals(context.model, &request, &previous)?
    } else {
        HashSet::new()
    };
    let removals = if anchor_remove {
        anchor_removals(&request, &previous)?
    } else {
        HashSet::new()
    };
    let anchor_basis = if anchor_edit {
        let unit: Uuid = serde_json::from_value(previous["unit"].clone())?;
        Some((unit, anchor_relinks::retained_unit(context.model, unit)?))
    } else {
        None
    };
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
                let mut edited = value.clone();
                if let (Some(before), Some(after)) =
                    (previous["pins"].as_object(), edited.as_object_mut())
                {
                    for (key, raw) in before.iter().filter(|(key, _)| {
                        key.parse::<Uuid>()
                            .ok()
                            .is_some_and(|id| pin_removals.contains(&id))
                    }) {
                        if let Some(pending) = after.get(key) {
                            if without_fields(pending, &["name", "direction", "electrical_type"])
                                != without_fields(raw, &["name", "direction", "electrical_type"])
                            {
                                return Err(refusal(
                                    "Removed pin still requires its original identity and raw metadata",
                                ));
                            }
                            after.insert(key.clone(), raw.clone());
                        }
                    }
                }
                validate_pins(&object["pins"], &edited)?;
                object["pins"] = edited;
            }
            ("units", "pin_additions" | "pin_removals") => {} // Compose after metadata.
            (
                "symbols",
                "line_edits" | "line_removals" | "line_additions" | "rect_edits" | "rect_removals"
                | "graphic_additions" | "text_edits" | "text_removals",
            ) => {}
            ("symbols", "drawings") => {
                validate_rectangle(&object["drawings"], value)?;
                object["drawings"] = value.clone();
            }
            ("symbols", "anchor_additions" | "anchor_removals" | "anchor_relinks") => {} // Compose after poses.
            ("symbols", "pin_anchors") => {
                let mut posed = value.clone();
                if let (Some(before), Some(after)) =
                    (previous["pin_anchors"].as_array(), posed.as_array_mut())
                {
                    if before.len() == after.len() {
                        for (old, new) in before.iter().zip(after) {
                            let removed = old["pin"]
                                .as_str()
                                .and_then(|s| s.parse::<Uuid>().ok())
                                .is_some_and(|id| removals.contains(&id));
                            // Explicit removal discards only this row's pending pose.
                            if removed && old.get("pin") == new.get("pin") {
                                *new = old.clone();
                            }
                        }
                    }
                }
                validate_anchors(&object["pin_anchors"], &posed)?;
                object["pin_anchors"] = posed;
            }
            _ => {
                return Err(refusal(
                    "Saved electrical edits support names, explicit pin, anchor, Line, Rect and Text edits only",
                ));
            }
        }
    }
    if graphic_edit {
        graphics::apply(&request, &previous, &mut object)?;
    }
    let mut references = HashSet::new();
    if let Some((unit, raw_unit)) = &anchor_basis {
        if anchor_relink {
            references.extend(anchor_relinks::apply(
                &request,
                &previous,
                &mut object,
                &removals,
                raw_unit,
            )?);
        }
        references.extend(edit_anchors(
            &request,
            &mut object,
            &removals,
            *unit,
            raw_unit,
        )?);
    }
    if pin_remove {
        references.extend(unit_pin_usage::require_unused(
            context.model,
            request.object_id,
            &pin_removals,
        )?);
        object["pins"].as_object_mut().unwrap().retain(|key, _| {
            key.parse::<Uuid>()
                .ok()
                .is_none_or(|id| !pin_removals.contains(&id))
        });
    }
    if adding {
        add_pins(context.model, &request, &mut object)?;
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
    let mut references: Vec<_> = references.into_iter().collect();
    references.sort();
    for id in references.into_iter().rev() {
        prepared.batch.operations.insert(
            0,
            crate::substrate::Operation::GuardObjectRevision {
                object_id: id,
                expected_object_revision: context.model.objects[&id].object_revision,
            },
        );
    }
    if body_edit || graphic_edit || adding || pin_remove || anchor_edit {
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

// Resolve identities against the original ordered source, before disposable poses.
fn anchor_removals(
    request: &LibraryDefinitionRequest,
    object: &Value,
) -> Result<HashSet<Uuid>, EngineError> {
    let rows = request.fields["anchor_removals"]
        .as_array()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| refusal("Select an existing anchor to remove"))?;
    let mut original = HashSet::new();
    if let Some(raw) = object.get("pin_anchors") {
        for row in raw
            .as_array()
            .ok_or_else(|| refusal("Symbol anchors must remain an ordered array"))?
        {
            let anchor: crate::pool::SymbolPinAnchor = serde_json::from_value(row.clone())?;
            if !original.insert(anchor.pin) {
                return Err(refusal("Existing Symbol anchor identity is ambiguous"));
            }
        }
    }
    let mut removed = HashSet::new();
    for value in rows {
        let id: Uuid = serde_json::from_value(value.clone())?;
        if !removed.insert(id) || !original.contains(&id) {
            return Err(refusal(
                "Anchor removal requires one unique existing source anchor",
            ));
        }
    }
    Ok(removed)
}

// Preserve old ordered raw rows; only genuinely new anchors are serialized.
fn edit_anchors(
    request: &LibraryDefinitionRequest,
    object: &mut Value,
    removals: &HashSet<Uuid>,
    unit: Uuid,
    raw_unit: &Value,
) -> Result<HashSet<Uuid>, EngineError> {
    let pins = raw_unit["pins"].as_object().unwrap();
    let members: HashSet<Uuid> = pins.keys().map(|key| key.parse().unwrap()).collect();
    let mut anchored = HashSet::new();
    if let Some(value) = object.get("pin_anchors") {
        for raw in value
            .as_array()
            .ok_or_else(|| refusal("Symbol anchors must remain an ordered array"))?
        {
            let anchor: crate::pool::SymbolPinAnchor = serde_json::from_value(raw.clone())?;
            if !anchored.insert(anchor.pin) {
                return Err(refusal("Existing Symbol anchor identity is ambiguous"));
            }
        }
    }
    if !removals.is_empty() && anchored.iter().any(|id| !members.contains(id)) {
        return Err(refusal(
            "Existing anchor links are not members of the retained Unit",
        ));
    }
    let mut references = HashSet::from([unit]);
    for id in removals {
        crate::api::native_write::schematic_library::unit_pin(pins, *id)?;
        if !anchored.remove(id) {
            return Err(refusal("Original anchor removal membership changed"));
        }
        references.insert(*id);
    }
    let additions = match request.fields.get("anchor_additions") {
        Some(rows) => rows
            .as_array()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| refusal("Add an explicit named pin anchor"))?
            .as_slice(),
        None => &[],
    };
    let mut appended = Vec::new();
    for raw in additions {
        let anchor: crate::pool::SymbolPinAnchor = serde_json::from_value(raw.clone())?;
        if removals.contains(&anchor.pin) {
            return Err(refusal(
                "An anchor cannot be removed and added for the same Unit pin in one change",
            ));
        }
        let pin = crate::api::native_write::schematic_library::unit_pin(pins, anchor.pin)?;
        if pin["name"].as_str().is_none_or(|s| s.trim().is_empty()) {
            return Err(refusal("Choose an existing named Unit pin"));
        }
        super::admit_anchor(&mut anchored, &anchor)?;
        references.insert(anchor.pin);
        // Only NEW anchors are serialized: unknown UUID-bearing extension data
        // cannot manufacture another object class through this append intent.
        appended.push(serde_json::to_value(anchor)?);
    }
    let anchors = object
        .as_object_mut()
        .unwrap()
        .entry("pin_anchors")
        .or_insert_with(|| json!([]));
    let anchors = anchors
        .as_array_mut()
        .ok_or_else(|| refusal("Symbol anchors must remain an ordered array"))?;
    anchors.retain(|raw| {
        raw["pin"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            .is_none_or(|id| !removals.contains(&id))
    });
    anchors.extend(appended);
    Ok(references)
}

// New rows use the existing creation builder; retained raw pins are never serialized.
fn add_pins(
    model: &crate::substrate::DesignModel,
    request: &LibraryDefinitionRequest,
    object: &mut Value,
) -> Result<(), EngineError> {
    let original = object["pins"]
        .as_object()
        .ok_or_else(|| refusal("Unit pins are unavailable"))?;
    let mut identities = HashSet::new();
    for (key, value) in original {
        let id: Uuid = key
            .parse()
            .map_err(|_| refusal("Saved logical pin identity is invalid"))?;
        let pin: crate::pool::Pin = serde_json::from_value(value.clone())?;
        if id.is_nil()
            || pin.uuid != id
            || !identities.insert(id)
            || model.objects.get(&id).is_none_or(|p| {
                p.source_shard_id != model.objects[&request.object_id].source_shard_id
            })
        {
            return Err(refusal(
                "Saved logical pin membership is missing or ambiguous",
            ));
        }
    }
    let rows = request.fields["pin_additions"]
        .as_object()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| refusal("Add an explicit named logical pin"))?;
    for key in rows.keys() {
        let id: Uuid = key
            .parse()
            .map_err(|_| refusal("New pin identity is invalid"))?;
        if id.is_nil() || !identities.insert(id) || model.objects.contains_key(&id) {
            return Err(refusal("New pin requires a fresh unowned identity"));
        }
    }
    let mut creation = request.clone();
    creation.fields = json!({"pins":rows}).as_object().unwrap().clone();
    let payload = super::unit_payload(&creation, object["name"].as_str().unwrap_or(""))?;
    object["pins"]
        .as_object_mut()
        .unwrap()
        .extend(payload["pins"].as_object().unwrap().clone());
    Ok(())
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

#[cfg(test)]
#[path = "library_gui_unit_add_pin_tests.rs"]
mod add_pin_tests;

#[cfg(test)]
#[path = "library_gui_symbol_add_anchor_tests.rs"]
mod add_anchor_tests;

#[cfg(test)]
#[path = "library_gui_symbol_remove_anchor_tests.rs"]
mod remove_anchor_tests;

#[cfg(test)]
#[path = "library_gui_unit_remove_pin_tests.rs"]
mod remove_pin_tests;

#[cfg(test)]
#[path = "library_gui_symbol_relink_anchor_tests.rs"]
mod relink_anchor_tests;

#[cfg(test)]
#[path = "library_gui_symbol_line_tests.rs"]
mod line_tests;

#[cfg(test)]
#[path = "library_gui_symbol_rect_tests.rs"]
mod rect_tests;

#[cfg(test)]
#[path = "library_gui_symbol_text_tests.rs"]
mod text_tests;
