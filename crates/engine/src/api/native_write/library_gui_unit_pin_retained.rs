//! Qualify actual retained CI declarations through existing revision authority.
use super::*;

fn part_hint(graph: &LibraryGraph, part: Uuid, unit: Uuid) -> bool {
    graph
        .parts
        .get(&part)
        .and_then(|raw| uuid(&raw["entity"]))
        .is_some_and(|entity| entity_uses_unit(graph, entity, unit))
}

fn retained_part_scope(
    graph: &LibraryGraph,
    part: &Value,
    entities: &[(Uuid, Value)],
    unit: Uuid,
) -> Option<bool> {
    let entity = uuid(&part["entity"])?;
    let captures: Vec<_> = entities
        .iter()
        .filter(|(id, _)| *id == entity)
        .map(|(_, value)| value)
        .collect();
    let sources = if captures.is_empty() {
        vec![graph.entities.get(&entity)?]
    } else {
        captures
    };
    let mut related = false;
    for source in sources {
        let gates = source["gates"].as_object()?;
        for gate in gates.values() {
            related |= uuid(&gate["unit"])? == unit;
        }
    }
    Some(related)
}

fn check_part_context(
    model: &DesignModel,
    graph: &LibraryGraph,
    part: Uuid,
    payload: &Value,
    rows: &Value,
    entities: &[(Uuid, Value)],
    unit: Uuid,
    removed: &HashSet<Uuid>,
    references: &mut HashSet<Uuid>,
) -> Result<(), EngineError> {
    // Every retained occurrence keeps its own Part payload, even equal UUIDs at
    // different revisions. Entity captures are independent contextual authority.
    let mut view = graph.clone();
    view.parts.insert(part, payload.clone());
    let entity_id = uuid(&payload["entity"]);
    let contexts: Vec<_> = entities
        .iter()
        .filter(|(entity, _)| Some(*entity) == entity_id)
        .collect();
    if contexts.is_empty() {
        mappings(model, &view, part, rows, unit, removed, references)?;
    }
    for (entity, value) in contexts {
        let mut frozen = view.clone();
        frozen.entities.insert(*entity, value.clone());
        mappings(model, &frozen, part, rows, unit, removed, references)?;
    }
    Ok(())
}

pub(super) fn qualify(
    model: &DesignModel,
    graph: &LibraryGraph,
    placed: &BTreeMap<Uuid, &crate::schematic::PlacedSymbol>,
    unit: Uuid,
    removed: &HashSet<Uuid>,
    references: &mut HashSet<Uuid>,
) -> Result<(), EngineError> {
    // Inspect registered declarations too: an unresolved authored CI is not an
    // absence proof merely because binding resolution omitted its projection.
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ComponentInstance)
    {
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let input = &raw["component_instance"];
        let hint = input["part_ref"]["object_id"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .is_some_and(|part| part_hint(graph, part, unit))
            || input["library_bindings"]
                .as_object()
                .is_some_and(|bindings| {
                    bindings.values().any(|binding| {
                        let Some(id) = uuid(&binding["target_object_id"]) else {
                            return false;
                        };
                        match binding["binding_role"].as_str() {
                            Some("part") => part_hint(graph, id, unit),
                            Some("pin_pad_map") => graph
                                .pin_pad_maps
                                .get(&id)
                                .and_then(|map| uuid(&map["part"]))
                                .is_some_and(|part| part_hint(graph, part, unit)),
                            _ => false,
                        }
                    })
                });
        let packet = match serde_json::from_value::<crate::substrate::ComponentInstanceShard>(raw) {
            Ok(packet) => packet,
            Err(_) if hint => {
                return Err(refusal("Relevant retained CI declaration is unavailable"));
            }
            // The loader and canonical CI writer deserialize before source
            // registration. Unrelated malformed packets cannot enter this set.
            Err(_) => continue,
        };
        let supported =
            packet.schema_version == crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION;
        let input = packet.component_instance;
        let mut relevant = hint;
        let mut entities = Vec::new();
        for reference in &input.placed_symbol_refs {
            let Some(symbol) = placed.get(&reference.object_id) else {
                continue;
            };
            if let (Some(entity), Some(gate), Some(library)) = (
                symbol.entity,
                symbol.gate,
                symbol.lib_id.as_ref().and_then(|s| s.parse::<Uuid>().ok()),
            ) {
                match model.materialized_placed_entity_definition(
                    reference.object_id,
                    entity,
                    gate,
                    library,
                ) {
                    Ok(value) => {
                        relevant |= value["gates"].as_object().is_some_and(|gates| {
                            gates.values().any(|gate| uuid(&gate["unit"]) == Some(unit))
                        });
                        entities.push((entity, value));
                    }
                    Err(_)
                        if graph
                            .symbols
                            .get(&library)
                            .is_some_and(|raw| uuid(&raw["unit"]) == Some(unit)) =>
                    {
                        return Err(refusal(
                            "Relevant placed Entity source is unavailable or ambiguous",
                        ));
                    }
                    Err(_) => {}
                }
            }
        }
        let mut parts = Vec::new();
        let mut maps = Vec::new();
        let mut unavailable = false;
        let mut unknown_part_scope = false;
        for role in [LibraryBindingRole::Part, LibraryBindingRole::PinPadMap] {
            for binding in input
                .library_bindings
                .values()
                .filter(|binding| binding.binding_role == role)
            {
                let kind = if role == LibraryBindingRole::Part {
                    "parts"
                } else {
                    "pin_pad_maps"
                };
                for (id, revision) in
                    std::iter::once((binding.target_object_id, binding.pinned_object_revision))
                        .chain(
                            binding
                                .local_override_refs
                                .iter()
                                .map(|r| (r.object_id, r.object_revision)),
                        )
                {
                    match model.materialized_pool_object_revision(id, kind, revision) {
                        Ok(value) => {
                            references.insert(id);
                            if role == LibraryBindingRole::Part {
                                match retained_part_scope(graph, &value, &entities, unit) {
                                    Some(related) => relevant |= related,
                                    None => unknown_part_scope = true,
                                }
                                parts.push((id, value));
                            } else {
                                maps.push(value);
                            }
                        }
                        Err(_) => {
                            unavailable = true;
                            unknown_part_scope |= role == LibraryBindingRole::Part;
                        }
                    }
                }
            }
        }
        if !input
            .library_bindings
            .values()
            .any(|b| b.binding_role == LibraryBindingRole::Part)
            && let Some(reference) = input.part_ref
        {
            relevant |= part_hint(graph, reference.object_id, unit);
            match model.materialized_pool_object_revision(
                reference.object_id,
                "parts",
                reference.object_revision,
            ) {
                Ok(value) => {
                    match retained_part_scope(graph, &value, &entities, unit) {
                        Some(related) => relevant |= related,
                        None => unknown_part_scope = true,
                    }
                    references.insert(reference.object_id);
                    parts.push((reference.object_id, value));
                }
                Err(_) => {
                    unavailable = true;
                    unknown_part_scope = true;
                }
            }
        }
        if unavailable && (relevant || unknown_part_scope) {
            return Err(refusal(
                "Relevant retained mapping source is unavailable or ambiguous",
            ));
        }
        for (id, payload) in &parts {
            check_part_context(
                model,
                graph,
                *id,
                payload,
                &payload["pad_map"],
                &entities,
                unit,
                removed,
                references,
            )?;
        }
        for map in &maps {
            let candidates = map["mappings"].as_object().is_some_and(|rows| {
                rows.iter()
                    .any(|(key, row)| candidate_row(key, row, removed))
            });
            if !candidates {
                continue;
            }
            let part = uuid(&map["part"])
                .ok_or_else(|| refusal("Retained pin mapping Part is unavailable"))?;
            let contexts: Vec<_> = parts
                .iter()
                .filter(|(id, _)| *id == part)
                .map(|(_, value)| value)
                .collect();
            if contexts.is_empty() {
                let payload = graph
                    .parts
                    .get(&part)
                    .ok_or_else(|| refusal("Retained mapping Part source is unavailable"))?;
                check_part_context(
                    model,
                    graph,
                    part,
                    payload,
                    &map["mappings"],
                    &entities,
                    unit,
                    removed,
                    references,
                )?;
            } else {
                for payload in contexts {
                    check_part_context(
                        model,
                        graph,
                        part,
                        payload,
                        &map["mappings"],
                        &entities,
                        unit,
                        removed,
                        references,
                    )?;
                }
            }
        }
        if relevant {
            if !supported {
                return Err(refusal("Relevant retained CI schema is unavailable"));
            }
            model
                .objects
                .get(&input.uuid)
                .filter(|owner| {
                    owner.domain == "component_instance"
                        && owner.source_shard_id == shard.shard_id
                        && owner.kind == "component_instance"
                        && owner.object_revision == input.object_revision
                })
                .ok_or_else(|| refusal("Relevant retained CI source ownership is unavailable"))?;
            references.insert(input.uuid);
        }
    }
    Ok(())
}
