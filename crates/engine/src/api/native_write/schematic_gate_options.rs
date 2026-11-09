//! Invocation-only fixed-owner Gate choices from the canonical library and occurrence.
use super::*;
pub(super) fn source(model: &DesignModel) -> Result<crate::schematic::Schematic, EngineError> {
    crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Gate source unavailable: {e:?}")))?
        .schematic_source()
        .cloned()
        .ok_or_else(|| invalid("Native schematic source unavailable"))
}
pub(super) fn occurrence(
    model: &DesignModel,
    symbol: Uuid,
) -> Result<(Uuid, Vec<Uuid>), EngineError> {
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Gate source unavailable: {e:?}")))?;
    let paths: Vec<_> = snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Gate occurrence unavailable: {e:?}")))?
        .into_iter()
        .filter(|s| s.occurrence.class == "symbols" && s.occurrence.source_id == symbol)
        .map(|s| s.occurrence.instance_path)
        .collect();
    let [path] = paths.as_slice() else {
        return Err(invalid("Gate requires exactly one source occurrence"));
    };
    let source =
        crate::api::native_write::schematic_selected_source::SelectedSchematicSource::from_snapshot(&snapshot)?;
    Ok((source.sheet("symbols", symbol, path)?, path.clone()))
}
pub fn options(
    model: &DesignModel,
    selected_symbol: Uuid,
    path: &[Uuid],
) -> Result<GatePlacementOptions, EngineError> {
    let (qualified_sheet, qualified_path) = occurrence(model, selected_symbol)?;
    if qualified_path != path {
        return Err(invalid("Selected Gate occurrence changed"));
    }
    let (_, raw) = component::gate_set(model, &BTreeSet::from([selected_symbol]))?
        .remove(&selected_symbol)
        .ok_or_else(|| invalid("Select an assigned schematic-only Symbol"))?;
    let ci = id(&raw, "uuid")?;
    // gate_set has authenticated the actual owner and agreement of its Part
    // declarations. Additional members extend that chain, not today's Part.
    let reference: crate::substrate::RevisionedRef =
        serde_json::from_value(raw["part_ref"].clone())?;
    let part = reference.object_id;
    let part_raw =
        model.materialized_pool_object_revision(part, "parts", reference.object_revision)?;
    let entity = id(&part_raw, "entity")?;
    let entity_raw = materialized_pool_object(model, entity, "entities")?;
    let source = source(model)?;
    let sheet = source
        .sheets
        .values()
        .find(|s| s.symbols.contains_key(&selected_symbol))
        .ok_or_else(|| invalid("Selected Symbol source is unavailable"))?;
    let used: BTreeSet<_> = raw["placed_symbol_refs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let member = id(r, "object_id")?;
            sheet.symbols[&member]
                .gate
                .ok_or_else(|| invalid("Gate identity is missing"))
        })
        .collect::<Result<_, EngineError>>()?;
    let mut choices = Vec::new();
    for (key, g) in entity_raw["gates"]
        .as_object()
        .ok_or_else(|| invalid("Entity gates unavailable"))?
    {
        let gate: Uuid = key
            .parse()
            .map_err(|_| invalid("Gate identity malformed"))?;
        if used.contains(&gate) {
            continue;
        }
        let library = id(g, "symbol")?;
        let unit = id(g, "unit")?;
        let body = materialized_pool_object(model, library, "symbols")?;
        if id(g, "uuid")? != gate || id(&body, "unit")? != unit {
            return Err(invalid("Declared Gate body and Unit disagree"));
        }
        options_impl::validate_maps(model, part, &part_raw, gate)?;
        materialize_pool_symbol_pins(model, Some(&library.to_string()))?;
        choices.push(GatePlacementChoice {
            gate,
            gate_name: g["name"].as_str().unwrap_or("").into(),
            unit,
            library_symbol: library,
            symbol_name: body["name"].as_str().unwrap_or("").into(),
        });
    }
    choices.sort_by(|a, b| (&a.gate_name, a.gate).cmp(&(&b.gate_name, b.gate)));
    Ok(GatePlacementOptions {
        component_instance: ci,
        sheet: qualified_sheet,
        instance_path: path.to_vec(),
        part,
        part_name: part_raw["mpn"].as_str().unwrap_or("").into(),
        entity,
        entity_name: entity_raw["name"].as_str().unwrap_or("").into(),
        choices,
    })
}
pub(super) fn owner_symbol(model: &DesignModel, ci: Uuid) -> Result<Uuid, EngineError> {
    model
        .component_instances
        .get(&ci)
        .and_then(|c| c.placed_symbol_refs.first())
        .copied()
        .ok_or_else(|| invalid("Component owner is unavailable"))
}
