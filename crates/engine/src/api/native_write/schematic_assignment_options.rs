//! Compatibility and occurrence admission shared by GUI choices and Proposal preparation.
use super::super::{
    library_pin_pad_map::{PinPadMapMapping, validate_pin_pad_map_payload},
    schematic_library::materialize_pool_symbol_pins,
    schematic_selected_source::SelectedSchematicSource,
};
use super::*;
use crate::{ir::geometry::Point, substrate::ElectricalSelectionSnapshot};
use std::collections::BTreeSet;

pub fn options(
    model: &DesignModel,
    symbol: Uuid,
    path: &[Uuid],
) -> Result<AssignmentOptions, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Assignment source unavailable: {e:?}")))?;
    let source = SelectedSchematicSource::from_snapshot(&snapshot)?;
    let sheet = source.sheet("symbols", symbol, path)?;
    let original = source.source.sheets[&sheet].symbols[&symbol].clone();
    let raw =
        super::super::schematic_symbol_payload::source_symbol_payload(model, sheet, &original)?;
    if raw["locked"].as_bool() == Some(true) {
        return Err(invalid("Selected Symbol is locked"));
    }
    let library = original
        .lib_id
        .as_deref()
        .and_then(|s| s.parse::<Uuid>().ok())
        .ok_or_else(|| invalid("Assignment requires a native pool Symbol"))?;
    let library_value = materialized_pool_object(model, library, "symbols")?;
    let unit = id(&library_value, "unit")?;
    let expected = materialize_pool_symbol_pins(model, original.lib_id.as_deref())?;
    let mut seen = BTreeSet::new();
    if expected.len() != original.pins.len() {
        return Err(invalid(
            "Placed pins no longer match the native Symbol Unit",
        ));
    }
    for pin in &original.pins {
        let identity = pin
            .library_pin
            .ok_or_else(|| invalid("Placed pin has no native Unit identity"))?;
        if !seen.insert(identity) {
            return Err(invalid("Placed Unit pin identity is duplicated"));
        }
        let expected = expected
            .iter()
            .find(|p| p.library_pin == Some(identity))
            .ok_or_else(|| invalid("Placed Unit pin is unavailable"))?;
        let at = super::super::schematic_edit::place_local_point(
            expected.position,
            Point::new(original.position.x, original.position.y),
            original.rotation,
            original.mirrored,
        )?;
        if pin.position != at
            || pin.name != expected.name
            || pin.number != expected.number
            || pin.electrical_type != expected.electrical_type
        {
            return Err(invalid(
                "Assignment requires unchanged native Unit pin geometry and semantics",
            ));
        }
    }
    let current = component::current(model, symbol)?;
    if let Some((_, value)) = &current {
        if value["library_bindings"]
            .as_object()
            .is_some_and(|bindings| {
                bindings.values().any(|binding| {
                    binding["binding_role"] == "symbol"
                        && binding["target_object_id"] != json!(library)
                })
            })
        {
            return Err(invalid(
                "ComponentInstance refers to a different native Symbol",
            ));
        }
        if value["part_ref"]["object_id"] != json!(original.part) {
            return Err(invalid(
                "Symbol and ComponentInstance Part bindings disagree",
            ));
        }
    }
    let mut choices = Vec::new();
    for object in model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "parts")
    {
        let part = materialized_pool_object(model, object.object_id, "parts")?;
        let entity_id = id(&part, "entity")?;
        let entity = materialized_pool_object(model, entity_id, "entities")?;
        let Some(gates) = entity["gates"].as_object() else {
            continue;
        };
        for (gate_key, g) in gates {
            let Ok(gate) = gate_key.parse::<Uuid>() else {
                continue;
            };
            if g["unit"] != json!(unit) || g["symbol"] != json!(library) || g["uuid"] != json!(gate)
            {
                continue;
            }
            if validate_maps(model, object.object_id, &part, gate).is_err() {
                continue;
            }
            choices.push(AssignmentChoice {
                part: object.object_id,
                part_name: part["mpn"].as_str().unwrap_or("").into(),
                entity: entity_id,
                entity_name: entity["name"].as_str().unwrap_or("").into(),
                gate,
                gate_name: g["name"].as_str().unwrap_or("").into(),
            });
        }
    }
    choices.sort_by(|a, b| {
        (&a.part_name, &a.gate_name, a.part, a.gate).cmp(&(
            &b.part_name,
            &b.gate_name,
            b.part,
            b.gate,
        ))
    });
    Ok(AssignmentOptions {
        sheet,
        original,
        component_instance: current.map(|(id, _)| id),
        choices,
    })
}
fn validate_maps(
    model: &DesignModel,
    part_id: Uuid,
    part: &Value,
    gate: Uuid,
) -> Result<(), EngineError> {
    let mut declared = None;
    // A Part without a map retains existing schematic-only compatibility semantics.
    materialized_pool_object(model, id(part, "package")?, "packages")?;
    if let Some(map) = part
        .get("pad_map")
        .and_then(Value::as_object)
        .filter(|m| !m.is_empty())
    {
        let entries = mappings(map)?;
        declared = Some(entries.iter().any(|e| e.gate == gate));
        validate_pin_pad_map_payload(model, part_id, None, &entries)?;
    }
    if let Some(map_id) = part.get("default_pin_pad_map").filter(|v| !v.is_null()) {
        let map_id: Uuid = serde_json::from_value(map_id.clone())?;
        let map = materialized_pool_object(model, map_id, "pin_pad_maps")?;
        if map["part"] != json!(part_id) {
            return Err(invalid("PinPadMap belongs to another Part"));
        }
        let footprint = map
            .get("footprint")
            .filter(|v| !v.is_null())
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?;
        let entries = map["mappings"]
            .as_object()
            .ok_or_else(|| invalid("PinPadMap mappings unavailable"))?;
        let entries = mappings(entries)?;
        if declared.is_none() {
            declared = Some(entries.iter().any(|e| e.gate == gate));
        }
        validate_pin_pad_map_payload(model, part_id, footprint, &entries)?;
    }
    if declared == Some(false) {
        return Err(invalid("Part mapping does not include the selected Gate"));
    }
    Ok(())
}
fn mappings(values: &serde_json::Map<String, Value>) -> Result<Vec<PinPadMapMapping>, EngineError> {
    values
        .iter()
        .map(|(pad, entry)| {
            Ok(PinPadMapMapping {
                pad: pad.parse().map_err(|_| invalid("Invalid pad identity"))?,
                gate: id(entry, "gate")?,
                pin: id(entry, "pin")?,
            })
        })
        .collect()
}
