//! Qualify unused saved logical pins against existing library and placed authority.
use super::super::super::{part, saved_source};
use super::*;
use crate::pool::LibraryGraph;
use crate::substrate::{DesignModel, ElectricalSelectionSnapshot, LibraryBindingRole};
use std::collections::BTreeMap;
#[path = "library_gui_unit_pin_retained.rs"]
mod retained;

fn uuid(value: &Value) -> Option<Uuid> {
    value.as_str().and_then(|raw| raw.parse().ok())
}

pub(super) fn removals(
    model: &DesignModel,
    request: &LibraryDefinitionRequest,
    original: &Value,
) -> Result<HashSet<Uuid>, EngineError> {
    let rows = original["pins"]
        .as_object()
        .ok_or_else(|| refusal("Saved Unit pins must be an original identity map"))?;
    let owner = &model.objects[&request.object_id];
    let mut identities = HashSet::new();
    for (key, raw) in rows {
        let id: Uuid = key
            .parse()
            .map_err(|_| refusal("Saved logical pin identity is invalid"))?;
        if id.is_nil()
            || uuid(&raw["uuid"]) != Some(id)
            || !identities.insert(id)
            || model.objects.get(&id).is_none_or(|pin| {
                pin.domain != "pool"
                    || pin.source_shard_id != owner.source_shard_id
                    || !(pin.kind == "pins" || pin.kind.parse::<Uuid>().ok() == Some(id))
            })
        {
            return Err(refusal(
                "Saved logical pin membership is missing or ambiguous",
            ));
        }
    }
    let selected = request.fields["pin_removals"]
        .as_array()
        .filter(|rows| !rows.is_empty())
        .ok_or_else(|| refusal("Select an existing logical pin to remove"))?;
    let mut removed = HashSet::new();
    for raw in selected {
        let id: Uuid = serde_json::from_value(raw.clone())?;
        if !identities.contains(&id) || !removed.insert(id) {
            return Err(refusal(
                "Pin removal requires one unique original logical pin",
            ));
        }
    }
    if request
        .fields
        .get("pin_additions")
        .and_then(Value::as_object)
        .is_some_and(|rows| {
            rows.keys().any(|key| {
                key.parse::<Uuid>()
                    .ok()
                    .is_some_and(|id| removed.contains(&id))
            })
        })
    {
        return Err(refusal(
            "A removed original pin cannot be added again in the same draft",
        ));
    }
    Ok(removed)
}

fn current_source(
    model: &DesignModel,
    id: Uuid,
    kind: &str,
    references: &mut HashSet<Uuid>,
) -> Result<Value, EngineError> {
    let (_, raw) = saved_source::reference(model, id, kind)?;
    if raw["schema_version"].as_u64() != Some(1) {
        return Err(refusal("Pin usage source schema is unavailable"));
    }
    references.insert(id);
    Ok(raw)
}

fn entity_uses_unit(graph: &LibraryGraph, entity: Uuid, unit: Uuid) -> bool {
    graph
        .entities
        .get(&entity)
        .and_then(|raw| raw["gates"].as_object())
        .is_some_and(|gates| gates.values().any(|gate| uuid(&gate["unit"]) == Some(unit)))
}

fn candidate_row(key: &str, row: &Value, removed: &HashSet<Uuid>) -> bool {
    if row.get("gate").is_some() || row.get("pin").is_some() {
        uuid(&row["pin"]).is_some_and(|id| removed.contains(&id))
    } else {
        key.parse::<Uuid>()
            .ok()
            .is_some_and(|id| removed.contains(&id))
    }
}

fn anchor_reference(raw: &Value, removed: &HashSet<Uuid>) -> Result<bool, EngineError> {
    let Some(anchors) = raw.get("pin_anchors") else {
        return Ok(false);
    };
    let anchors = anchors
        .as_array()
        .ok_or_else(|| refusal("Relevant Symbol anchor references are unavailable"))?;
    for anchor in anchors {
        let pin = uuid(&anchor["pin"])
            .ok_or_else(|| refusal("Relevant Symbol logical pin identity is unavailable"))?;
        if removed.contains(&pin) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn duplicate_reference_candidates(
    model: &DesignModel,
    counts: &BTreeMap<Uuid, usize>,
    removed: &HashSet<Uuid>,
) -> Result<(), EngineError> {
    if !counts.values().any(|count| *count > 1) {
        return Ok(());
    }
    // The existing graph exposes the registered owner. Inspect only duplicate
    // roots for exact pin references before that owner can hide another source.
    for source in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        let Ok(raw) = model.materialized_source_shard_value_by_relative_path(&source.relative_path)
        else {
            continue;
        };
        if uuid(&raw["uuid"])
            .and_then(|id| counts.get(&id))
            .is_none_or(|count| *count <= 1)
        {
            continue;
        }
        let candidate = match source.relative_path.rsplit('/').nth(1) {
            Some("symbols") => raw["pin_anchors"].as_array().is_some_and(|rows| {
                rows.iter()
                    .any(|row| uuid(&row["pin"]).is_some_and(|id| removed.contains(&id)))
            }),
            Some("parts" | "pin_pad_maps") => {
                let rows = if source.relative_path.rsplit('/').nth(1) == Some("parts") {
                    &raw["pad_map"]
                } else {
                    &raw["mappings"]
                };
                rows.as_object().is_some_and(|rows| {
                    rows.iter()
                        .any(|(key, row)| candidate_row(key, row, removed))
                })
            }
            _ => false,
        };
        if candidate {
            return Err(refusal(
                "Logical pin reference has ambiguous duplicate library sources",
            ));
        }
    }
    Ok(())
}

fn mappings(
    model: &DesignModel,
    graph: &LibraryGraph,
    part_id: Uuid,
    rows: &Value,
    unit: Uuid,
    removed: &HashSet<Uuid>,
    references: &mut HashSet<Uuid>,
) -> Result<(), EngineError> {
    let Some(rows) = rows.as_object() else {
        return Ok(());
    };
    if !rows
        .iter()
        .any(|(key, value)| candidate_row(key, value, removed))
    {
        return Ok(());
    }
    let part = graph
        .parts
        .get(&part_id)
        .ok_or_else(|| refusal("Referenced pin mapping has no qualified Part"))?;
    let entity = uuid(&part["entity"])
        .ok_or_else(|| refusal("Referenced pin mapping has no qualified Entity"))?;
    current_source(model, part_id, "parts", references)
        .map_err(|error| refusal(format!("Pin mapping Part qualification failed: {error}")))?;
    current_source(model, entity, "entities", references)
        .map_err(|error| refusal(format!("Pin mapping Entity qualification failed: {error}")))?;
    let gates = graph
        .entities
        .get(&entity)
        .ok_or_else(|| refusal("Pin mapping Entity source is unavailable"))?["gates"]
        .as_object()
        .ok_or_else(|| refusal("Referenced pin mapping gates are unavailable"))?;
    for (key, row) in rows
        .iter()
        .filter(|(key, row)| candidate_row(key, row, removed))
    {
        let mut diagnostics = Vec::new();
        let resolved = graph.resolve_pin_pad_map_mapping(
            key,
            row,
            Some(entity),
            "Unit pin removal",
            &mut diagnostics,
        );
        let Some((_, gate, Some(pin))) = resolved.filter(|_| diagnostics.is_empty()) else {
            return Err(refusal(
                "A retained pin mapping cannot be qualified; logical pin removal refused",
            ));
        };
        let matched: Vec<_> = gates
            .iter()
            .filter(|(key, _)| key.parse::<Uuid>().ok() == Some(gate))
            .map(|(_, gate)| gate)
            .collect();
        let [gate_source] = matched.as_slice() else {
            return Err(refusal(
                "A retained pin mapping gate is unavailable or ambiguous",
            ));
        };
        if uuid(&gate_source["uuid"]) != Some(gate)
            || model.objects.get(&gate).is_none_or(|owner| {
                owner.domain != "pool"
                    || owner.source_shard_id != model.objects[&entity].source_shard_id
                    || !(owner.kind == "gates" || owner.kind.parse::<Uuid>().ok() == Some(gate))
            })
        {
            return Err(refusal(
                "A retained pin mapping gate identity is unavailable or ambiguous",
            ));
        }
        let gate_unit = uuid(&gate_source["unit"])
            .ok_or_else(|| refusal("A retained pin mapping Unit is unavailable"))?;
        let logical = current_source(model, gate_unit, "units", references)?;
        let pin_rows: Vec<_> = logical["pins"]
            .as_object()
            .ok_or_else(|| refusal("Pin mapping logical Unit membership is unavailable"))?
            .iter()
            .filter(|(key, _)| key.parse::<Uuid>().ok() == Some(pin))
            .collect();
        if pin_rows.len() != 1
            || uuid(&pin_rows[0].1["uuid"]) != Some(pin)
            || model.objects.get(&pin).is_none_or(|owner| {
                owner.domain != "pool"
                    || owner.source_shard_id != model.objects[&gate_unit].source_shard_id
                    || !(owner.kind == "pins" || owner.kind.parse::<Uuid>().ok() == Some(pin))
            })
        {
            return Err(refusal(
                "Pin mapping logical pin membership is unavailable or ambiguous",
            ));
        }
        if gate_unit == unit && removed.contains(&pin) {
            return Err(refusal(
                "Logical pin is still referenced by a retained pin/pad mapping",
            ));
        }
    }
    Ok(())
}

pub(super) fn require_unused(
    model: &DesignModel,
    unit: Uuid,
    removed: &HashSet<Uuid>,
) -> Result<HashSet<Uuid>, EngineError> {
    let (graph, counts) = part::graph(model)?;
    part::qualify(model, &graph, &counts, unit, "units")?;
    duplicate_reference_candidates(model, &counts, removed)?;
    let mut references = removed.clone();
    references.insert(unit);
    for (id, raw) in &graph.symbols {
        let exact_pin = raw["pin_anchors"].as_array().is_some_and(|rows| {
            rows.iter()
                .any(|anchor| uuid(&anchor["pin"]).is_some_and(|id| removed.contains(&id)))
        });
        if uuid(&raw["unit"]) != Some(unit) && !exact_pin {
            continue;
        }
        let source = current_source(model, *id, "symbols", &mut references)?;
        if anchor_reference(&source, removed)? {
            return Err(refusal(
                "Logical pin is still referenced by a Symbol anchor",
            ));
        }
    }
    for (id, part) in &graph.parts {
        mappings(
            model,
            &graph,
            *id,
            &part["pad_map"],
            unit,
            removed,
            &mut references,
        )?;
    }
    for (id, map) in &graph.pin_pad_maps {
        let candidates = map["mappings"].as_object().is_some_and(|rows| {
            rows.iter()
                .any(|(key, row)| candidate_row(key, row, removed))
        });
        if candidates {
            current_source(model, *id, "pin_pad_maps", &mut references)?;
            let part = uuid(&map["part"])
                .ok_or_else(|| refusal("Referenced pin/pad mapping Part is unavailable"))?;
            mappings(
                model,
                &graph,
                part,
                &map["mappings"],
                unit,
                removed,
                &mut references,
            )?;
        }
    }
    placed_and_retained(model, &graph, unit, removed, &mut references)?;
    Ok(references)
}

fn placed_and_retained(
    model: &DesignModel,
    graph: &LibraryGraph,
    unit: Uuid,
    removed: &HashSet<Uuid>,
    references: &mut HashSet<Uuid>,
) -> Result<(), EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|error| refusal(format!("Pin usage source basis is unavailable: {error:?}")))?;
    let source = snapshot.schematic_source();
    let mut placed = BTreeMap::new();
    for sheet in source.into_iter().flat_map(|source| source.sheets.values()) {
        for (id, symbol) in &sheet.symbols {
            if placed.insert(*id, symbol).is_some() {
                return Err(refusal("Placed pin usage membership is ambiguous"));
            }
        }
    }
    for (id, symbol) in &placed {
        let direct = symbol
            .pins
            .iter()
            .any(|pin| pin.library_pin.is_some_and(|pin| removed.contains(&pin)));
        let library = symbol
            .lib_id
            .as_ref()
            .and_then(|raw| raw.parse::<Uuid>().ok());
        let current_hint = library
            .and_then(|id| graph.symbols.get(&id))
            .is_some_and(|raw| uuid(&raw["unit"]) == Some(unit));
        let Some(library) = library else {
            if direct {
                return Err(refusal(
                    "A placed pin reference has no qualified Symbol source",
                ));
            }
            continue;
        };
        let frozen = model.materialized_placed_symbol_definition(*id, library);
        match frozen {
            Ok(Some(raw)) => {
                if direct {
                    return Err(refusal(
                        "Logical pin is still referenced by an actual placed pin",
                    ));
                }
                if uuid(&raw["unit"]) == Some(unit) {
                    if anchor_reference(&raw, removed)? {
                        return Err(refusal(
                            "Logical pin is still referenced by a frozen placed Symbol",
                        ));
                    }
                    references.insert(*id);
                }
            }
            _ if direct || current_hint => {
                return Err(refusal(
                    "Relevant placed Symbol source is unavailable or ambiguous",
                ));
            }
            _ => {} // Unrelated unavailable captures do not qualify this Unit.
        }
    }
    retained::qualify(model, graph, &placed, unit, removed, references)?;
    Ok(())
}
