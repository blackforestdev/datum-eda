//! Saved Entity corrections preserve raw gates; only requested names and Unit/Symbol links change.
use super::*;
use std::collections::BTreeSet;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    if ![
        "gate_symbols",
        "gate_names",
        "gate_units",
        "gate_additions",
        "gate_removals",
    ]
    .iter()
    .any(|key| request.fields.contains_key(*key))
    {
        return super::super::metadata_edit::build(
            context,
            provenance,
            request,
            "entities",
            &["name", "prefix", "manufacturer", "tags"],
            &["name"],
        );
    }
    if request.kind != "entities" {
        return Err(refusal("Expected a saved Entity"));
    }
    let (target, mut object) = saved_source::load(context.model, request.object_id, "entities")?;
    let mut references = BTreeSet::new();
    let mut original = object.clone();
    let removing = request.fields.contains_key("gate_removals");
    if let Some(value) = request.fields.get("gate_removals") {
        let ids: Vec<Uuid> = serde_json::from_value(value.clone())?;
        let mut seen = BTreeSet::new();
        if ids.is_empty() {
            return Err(refusal("Choose an existing gate to remove"));
        }
        for id in ids {
            if !seen.insert(id) {
                return Err(refusal("Gate removal identity is ambiguous"));
            }
            let (_, gate) = gate_mut(&mut object, id)?;
            reference(context.model, gate.unit, "units")?;
            reference(context.model, gate.symbol, "symbols")?;
            references.extend([gate.unit, gate.symbol]);
            preserve_users(context.model, request.object_id, id, true)?;
            let gates = object["gates"].as_object_mut().unwrap();
            let key = gates
                .keys()
                .find(|k| k.parse::<Uuid>().ok() == Some(id))
                .unwrap()
                .clone();
            gates.remove(&key);
        }
    }
    let adding = request.fields.contains_key("gate_additions");
    if let Some(value) = request.fields.get("gate_additions") {
        let rows = value
            .as_object()
            .filter(|m| !m.is_empty())
            .ok_or_else(|| refusal("Add an explicit named gate"))?;
        let gates = object["gates"]
            .as_object_mut()
            .ok_or_else(|| refusal("Saved gates are unavailable"))?;
        let mut fresh = BTreeSet::new();
        for (key, value) in rows {
            let id: Uuid = key
                .parse()
                .map_err(|_| refusal("New gate identity is malformed"))?;
            let gate: GateIntent = serde_json::from_value(value.clone())?;
            if id.is_nil()
                || gate.uuid != id
                || !fresh.insert(id)
                || context.model.objects.contains_key(&id)
                || gates.keys().any(|key| key.parse::<Uuid>().ok() == Some(id))
            {
                return Err(refusal("New gate requires a fresh unowned identity"));
            }
            if gate.name.trim().is_empty() {
                return Err(refusal("Enter a gate name"));
            }
            let unit = reference(context.model, gate.unit, "units")?;
            let symbol = reference(context.model, gate.symbol, "symbols")?;
            if symbol["unit"].as_str().and_then(|s| s.parse::<Uuid>().ok()) != Some(gate.unit)
                || [unit, symbol]
                    .iter()
                    .any(|raw| raw["name"].as_str().is_none_or(|s| s.trim().is_empty()))
            {
                return Err(refusal(
                    "Choose an existing named Unit and compatible named Symbol",
                ));
            }
            references.extend([gate.unit, gate.symbol]);
            gates.insert(key.clone(), value.clone());
        }
    }
    let mut touched = BTreeSet::new();
    // Stage all requested leaves first: compatibility belongs to the candidate
    // pair, independent of request field ordering.
    for (field, property) in [
        ("gate_names", "name"),
        ("gate_units", "unit"),
        ("gate_symbols", "symbol"),
    ] {
        if let Some(value) = request.fields.get(field) {
            let changes = value
                .as_object()
                .filter(|m| !m.is_empty())
                .ok_or_else(|| refusal("Choose an existing gate to correct"))?;
            let mut seen = BTreeSet::new();
            for (key, value) in changes {
                let id: Uuid = key
                    .parse()
                    .map_err(|_| refusal("Gate identity is malformed"))?;
                if !seen.insert(id) {
                    return Err(refusal("Gate correction identity is ambiguous"));
                }
                let (raw, _) = gate_mut(&mut object, id)?;
                let replacement = if property == "name" {
                    let text = value
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .ok_or_else(|| refusal("Enter a gate name"))?;
                    json!(text)
                } else {
                    json!(serde_json::from_value::<Uuid>(value.clone())?)
                };
                raw.as_object_mut()
                    .unwrap()
                    .insert(property.into(), replacement);
                touched.insert(id);
            }
        }
    }
    for id in touched {
        let (_, old) = gate_mut(&mut original, id)?;
        reference(context.model, old.unit, "units")?;
        let old_symbol = reference(context.model, old.symbol, "symbols")?;
        if old_symbol["unit"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            != Some(old.unit)
        {
            return Err(refusal("Saved gate pair is incompatible"));
        }
        references.extend([old.unit, old.symbol]);
        let (_, gate) = gate_mut(&mut object, id)?;
        let unit = reference(context.model, gate.unit, "units")?;
        let symbol = reference(context.model, gate.symbol, "symbols")?;
        if symbol["unit"].as_str().and_then(|s| s.parse::<Uuid>().ok()) != Some(gate.unit) {
            return Err(refusal(
                "Choose a Symbol belonging to the selected gate Unit",
            ));
        }
        if request.fields.contains_key("gate_units")
            && [unit, symbol]
                .iter()
                .any(|raw| raw["name"].as_str().is_none_or(|s| s.trim().is_empty()))
        {
            return Err(refusal(
                "Choose an existing named Unit and compatible named Symbol",
            ));
        }
        references.extend([gate.unit, gate.symbol]);
        if old.unit != gate.unit || old.symbol != gate.symbol {
            preserve_users(context.model, request.object_id, id, false)?;
        }
    }
    let unit_edit = request.fields.contains_key("gate_units");
    for (key, value) in request.fields {
        match key.as_str() {
            "gate_names" | "gate_units" | "gate_symbols" | "gate_additions" | "gate_removals" => {}
            "name" | "prefix" | "manufacturer" => {
                let text = value
                    .as_str()
                    .ok_or_else(|| refusal(format!("{key} must be text")))?;
                if key == "name" && text.trim().is_empty() {
                    return Err(refusal("Enter name"));
                }
                object.as_object_mut().unwrap().insert(key, value);
            }
            "tags" => {
                let _: Vec<String> = serde_json::from_value(value.clone())?;
                object.as_object_mut().unwrap().insert(key, value);
            }
            _ => {
                return Err(refusal(
                    "Saved Entity editing refuses topology, identity and policy fields",
                ));
            }
        }
    }
    if removing && object["gates"].as_object().is_some_and(|g| g.is_empty()) {
        return Err(refusal("Add one or more named gates"));
    }
    let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
    if unit_edit || adding || removing {
        validate_parts(context.model, request.object_id, &object, &mut references)?;
    }
    let mut prepared = library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )?;
    for id in references.into_iter().rev() {
        prepared.batch.operations.insert(
            0,
            Operation::GuardObjectRevision {
                object_id: id,
                expected_object_revision: context.model.objects[&id].object_revision,
            },
        );
    }
    Ok(prepared)
}

fn gate_mut(object: &mut Value, id: Uuid) -> Result<(&mut Value, crate::pool::Gate), EngineError> {
    let gates = object["gates"]
        .as_object_mut()
        .ok_or_else(|| refusal("Saved Entity gates are unavailable"))?;
    let matches: Vec<_> = gates
        .keys()
        .filter(|k| k.parse::<Uuid>().ok() == Some(id))
        .cloned()
        .collect();
    if matches.len() != 1 {
        return Err(refusal("Choose one existing unambiguous gate"));
    }
    let raw = gates.get_mut(&matches[0]).unwrap();
    let gate: crate::pool::Gate = serde_json::from_value(raw.clone())?;
    if gate.uuid != id {
        return Err(refusal("Saved gate membership and identity differ"));
    }
    Ok((raw, gate))
}

// Reference reads do not require ownership of the referenced pool's write path.
fn reference(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    kind: &str,
) -> Result<Value, EngineError> {
    library_definition::validate_pool_definition_reference(model, id, kind, kind)?;
    let mut candidates = Vec::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if raw["uuid"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(id) {
            candidates.push((shard, raw));
        }
    }
    if candidates.len() != 1 {
        return Err(refusal("Library reference is missing or ambiguous"));
    }
    let (shard, raw) = candidates.pop().unwrap();
    if model.objects[&id].source_shard_id != shard.shard_id {
        return Err(refusal("Library reference source ownership differs"));
    }
    if raw["schema_version"].as_u64() != Some(1) {
        return Err(refusal("Library reference schema is unsupported"));
    }
    match kind {
        "units" => {
            let _: crate::pool::Unit = serde_json::from_value(raw.clone())?;
        }
        "symbols" => {
            let _: crate::pool::Symbol = serde_json::from_value(raw.clone())?;
        }
        _ => return Err(refusal("Unsupported gate reference kind")),
    }
    Ok(raw)
}

fn preserve_users(
    model: &crate::substrate::DesignModel,
    entity: Uuid,
    gate: Uuid,
    removing: bool,
) -> Result<(), EngineError> {
    for shard in model.source_shards.iter().filter(|s| {
        matches!(
            s.kind,
            SourceShardKind::SchematicSheet
                | SourceShardKind::SchematicRoot
                | SourceShardKind::SchematicDefinition
        )
    }) {
        let source =
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let sheets: Vec<_> = if let Some(sheets) = source.get("sheets").and_then(Value::as_object) {
            sheets.values().collect()
        } else {
            vec![&source]
        };
        for sheet in sheets {
            for (key, placed) in sheet
                .get("symbols")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                let id = |field: &str| placed[field].as_str().and_then(|s| s.parse::<Uuid>().ok());
                if id("entity") == Some(entity) && id("gate") == Some(gate) {
                    if removing {
                        return Err(refusal("Gate is used by an existing schematic placement"));
                    }
                    let placed_id: Uuid = key
                        .parse()
                        .map_err(|_| refusal("Placed identity is malformed"))?;
                    if id("uuid") != Some(placed_id) {
                        return Err(refusal("Placed membership identity differs"));
                    }
                    let symbol = id("lib_id")
                        .ok_or_else(|| refusal("Placed native Symbol is unavailable"))?;
                    let captured = model
                        .materialized_placed_entity_definition(placed_id, entity, gate, symbol)?;
                    let captured_raw = captured["gates"]
                        .as_object()
                        .and_then(|gates| {
                            gates
                                .iter()
                                .find(|(key, _)| key.parse::<Uuid>().ok() == Some(gate))
                        })
                        .map(|(_, raw)| raw)
                        .ok_or_else(|| refusal("Captured gate is unavailable"))?;
                    let captured_gate: crate::pool::Gate =
                        serde_json::from_value(captured_raw.clone())?;
                    let body = model
                        .materialized_placed_symbol_definition(placed_id, symbol)?
                        .ok_or_else(|| refusal("Captured placed Symbol is unavailable"))?;
                    if captured_gate.uuid != gate
                        || captured_gate.symbol != symbol
                        || body["unit"].as_str().and_then(|s| s.parse::<Uuid>().ok())
                            != Some(captured_gate.unit)
                    {
                        return Err(refusal(
                            "Existing placed gate has no compatible captured chain",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

// Reuse the Part adapter's materialized graph; all validation is prospective.
fn validate_parts(
    model: &crate::substrate::DesignModel,
    entity: Uuid,
    candidate: &Value,
    references: &mut BTreeSet<Uuid>,
) -> Result<(), EngineError> {
    let (mut graph, counts) = super::super::part::graph(model)?;
    graph.entities.insert(entity, candidate.clone());
    let mut subjects = BTreeSet::from([graph.subjects[&entity].clone()]);
    let mut qualify = |id, kind| -> Result<(), EngineError> {
        super::super::part::qualify(model, &graph, &counts, id, kind)?;
        references.insert(id);
        Ok(())
    };
    for (id, raw) in &graph.parts {
        if raw["entity"].as_str().and_then(|s| s.parse::<Uuid>().ok()) != Some(entity) {
            continue;
        }
        qualify(*id, "parts")?;
        let part: crate::pool::Part = serde_json::from_value(raw.clone())?;
        qualify(part.package, "packages")?;
        subjects.insert(graph.subjects[id].clone());
        let mut maps: BTreeSet<_> = part.default_pin_pad_map.into_iter().collect();
        maps.extend(
            graph
                .pin_pad_maps
                .iter()
                .filter(|(_, raw)| {
                    raw["part"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(*id)
                })
                .map(|(id, _)| *id),
        );
        let mut rows: Vec<&Value> = raw
            .get("pad_map")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|m| m.values())
            .collect();
        if let Some(footprint) = part.default_footprint {
            qualify(footprint, "footprints")?;
            subjects.insert(graph.subjects[&footprint].clone());
        }
        for map_id in maps {
            qualify(map_id, "pin_pad_maps")?;
            subjects.insert(graph.subjects[&map_id].clone());
            let map_raw = &graph.pin_pad_maps[&map_id];
            let map: crate::pool::PinPadMap = serde_json::from_value(map_raw.clone())?;
            if map.part != *id {
                return Err(refusal("Retained Map belongs to another Part"));
            }
            if let Some(footprint) = map.footprint {
                qualify(footprint, "footprints")?;
                subjects.insert(graph.subjects[&footprint].clone());
            }
            rows.extend(
                map_raw
                    .get("mappings")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|m| m.values()),
            );
        }
        for row in rows {
            let mapped: crate::pool::PadMapEntry = serde_json::from_value(row.clone())?;
            let gates = candidate["gates"]
                .as_object()
                .ok_or_else(|| refusal("Entity gates unavailable"))?;
            let matches: Vec<_> = gates
                .iter()
                .filter(|(k, _)| k.parse::<Uuid>().ok() == Some(mapped.gate))
                .collect();
            if matches.len() != 1 {
                return Err(refusal("Mapped Gate is missing or ambiguous"));
            }
            let gate: crate::pool::Gate = serde_json::from_value(matches[0].1.clone())?;
            if gate.uuid != mapped.gate {
                return Err(refusal("Mapped Gate identity differs"));
            }
            qualify(gate.unit, "units")?;
        }
    }
    for diagnostic in graph.dependency_diagnostics() {
        if diagnostic.severity == "error"
            && subjects.iter().any(|s| {
                diagnostic.subject == *s || diagnostic.subject.starts_with(&format!("{s}#"))
            })
        {
            return Err(refusal(diagnostic.message));
        }
    }
    Ok(())
}
