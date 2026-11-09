//! Saved Part corrections retain raw relationships and frozen placed users.
use super::*;
use crate::substrate::{DesignModel, Operation};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    const METADATA: &[&str] = &[
        "manufacturer",
        "mpn",
        "value",
        "description",
        "datasheet",
        "tags",
    ];
    let entity_edit = request.fields.contains_key("entity");
    let package_edit = request.fields.contains_key("package");
    if !entity_edit
        && !package_edit
        && !request.fields.contains_key("default_footprint")
        && !request.fields.contains_key("default_pin_pad_map")
    {
        return super::super::metadata_edit::build(
            context,
            provenance,
            request,
            "parts",
            METADATA,
            &["manufacturer", "mpn"],
        );
    }
    if request.kind != "parts" {
        return Err(refusal("Expected a saved Part"));
    }
    let (target, mut object) = saved_source::load(context.model, request.object_id, "parts")?;
    for (key, value) in request.fields {
        if matches!(key.as_str(), "entity" | "package") {
            let label = if key == "entity" { "Entity" } else { "Package" };
            let reference: Uuid = value
                .as_str()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| refusal(format!("Choose an existing named {label}")))?;
            object[&key] = json!(reference);
        } else if matches!(key.as_str(), "default_footprint" | "default_pin_pad_map") {
            let label = if key == "default_footprint" {
                "Footprint"
            } else {
                "PinPadMap"
            };
            match value {
                Value::Null => {
                    object.as_object_mut().unwrap().remove(&key);
                }
                Value::String(text) => {
                    let id: Uuid = text.parse().map_err(|_| {
                        refusal(format!(
                            "Choose a named {label} or explicitly clear the default"
                        ))
                    })?;
                    object.as_object_mut().unwrap().insert(key, json!(id));
                }
                _ => {
                    return Err(refusal(format!(
                        "Default {label} must be an existing identity or an explicit empty choice",
                    )));
                }
            }
        } else {
            if !METADATA.contains(&key.as_str()) {
                return Err(refusal(
                    "Saved Part editing refuses Map payload, identity and policy changes",
                ));
            }
            if key == "tags" {
                let _: Vec<String> = serde_json::from_value(value.clone())?;
            } else {
                let text = value
                    .as_str()
                    .ok_or_else(|| refusal(format!("{key} must be text")))?;
                if matches!(key.as_str(), "manufacturer" | "mpn") && text.trim().is_empty() {
                    return Err(refusal(format!("Enter {key}")));
                }
            }
            object.as_object_mut().unwrap().insert(key, value);
        }
    }
    let part: crate::pool::Part = serde_json::from_value(object.clone())?;
    let mut references = BTreeSet::from([(part.entity, "entities"), (part.package, "packages")]);
    references.extend(part.default_footprint.map(|id| (id, "footprints")));
    references.extend(part.default_pin_pad_map.map(|id| (id, "pin_pad_maps")));
    let (mut graph, counts) = graph(context.model)?;
    for &(id, kind) in &references {
        qualify(context.model, &graph, &counts, id, kind)?;
    }
    if package_edit {
        let package = &graph.packages[&part.package];
        if package["name"]
            .as_str()
            .is_none_or(|name| name.trim().is_empty())
        {
            return Err(refusal("Choose an existing named Package"));
        }
        let mut seen = BTreeSet::new();
        for collection in ["terminals", "pads"] {
            for (key, row) in package
                .get(collection)
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                let id: Uuid = serde_json::from_value(row["uuid"].clone())?;
                if key.parse::<Uuid>().ok() != Some(id)
                    || !seen.insert(id)
                    || context.model.objects.get(&id).is_none_or(|o| {
                        o.source_shard_id != context.model.objects[&part.package].source_shard_id
                    })
                {
                    return Err(refusal(
                        "Package member identity is missing, foreign or ambiguous",
                    ));
                }
            }
        }
        let package: crate::pool::Package = serde_json::from_value(package.clone())?;
        // Missing optional pads is empty membership, never a legacy-map exemption.
        if part
            .pad_map
            .keys()
            .any(|pad| !package.pads.contains_key(pad))
        {
            return Err(refusal(
                "Retained legacy mapping references a pad outside the selected Package",
            ));
        }
    }
    if entity_edit {
        let entity = &graph.entities[&part.entity];
        if entity["name"]
            .as_str()
            .is_none_or(|name| name.trim().is_empty())
        {
            return Err(refusal("Choose an existing named Entity"));
        }
        let gates = entity["gates"]
            .as_object()
            .ok_or_else(|| refusal("Entity gates are unavailable"))?;
        let mut seen = BTreeSet::new();
        for (key, raw) in gates {
            let gate: crate::pool::Gate = serde_json::from_value(raw.clone())?;
            if key.parse::<Uuid>().ok() != Some(gate.uuid)
                || !seen.insert(gate.uuid)
                || context.model.objects.get(&gate.uuid).is_none_or(|owner| {
                    owner.source_shard_id != context.model.objects[&part.entity].source_shard_id
                })
            {
                return Err(refusal(
                    "Entity Gate source identity is missing, foreign or ambiguous",
                ));
            }
            for (id, kind) in [(gate.unit, "units"), (gate.symbol, "symbols")] {
                qualify(context.model, &graph, &counts, id, kind)?;
                references.insert((id, kind));
            }
            let symbol: crate::pool::Symbol =
                serde_json::from_value(graph.symbols[&gate.symbol].clone())?;
            if symbol.unit != gate.unit {
                return Err(refusal(
                    "Entity Gate Symbol must belong to its declared Unit",
                ));
            }
        }
    }
    let mut maps: BTreeSet<_> = part.default_pin_pad_map.into_iter().collect();
    if entity_edit || package_edit {
        maps.extend(
            graph
                .pin_pad_maps
                .iter()
                .filter(|(_, raw)| {
                    raw["part"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(part.uuid)
                })
                .map(|(id, _)| *id),
        );
    }
    // Clearing a default never erases retained nondefault Map authority.
    for map_id in maps {
        qualify(context.model, &graph, &counts, map_id, "pin_pad_maps")?;
        references.insert((map_id, "pin_pad_maps"));
        let map: crate::pool::PinPadMap =
            serde_json::from_value(graph.pin_pad_maps[&map_id].clone())?;
        if let Some(footprint) = map.footprint {
            qualify(context.model, &graph, &counts, footprint, "footprints")?;
            references.insert((footprint, "footprints"));
            let source: crate::pool::Footprint =
                serde_json::from_value(graph.footprints[&footprint].clone())?;
            if package_edit && source.package != part.package {
                return Err(refusal(
                    "Retained Map Footprint must belong to the selected Part Package",
                ));
            }
        }
        let gates = graph.entities[&part.entity]["gates"]
            .as_object()
            .ok_or_else(|| refusal("Mapped Entity gates are unavailable"))?;
        for row in graph.pin_pad_maps[&map_id]
            .get("mappings")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|m| m.values())
        {
            // Qualify every raw row before UUID-key deserialization can coalesce
            // aliases and conceal a consumed Gate/Unit dependency.
            let mapped: crate::pool::PadMapEntry = serde_json::from_value(row.clone())?;
            let matches: Vec<_> = gates
                .iter()
                .filter(|(key, _)| key.parse::<Uuid>().ok() == Some(mapped.gate))
                .collect();
            if matches.len() != 1 {
                return Err(refusal("Mapped Entity Gate is missing or ambiguous"));
            }
            let gate: crate::pool::Gate = serde_json::from_value(matches[0].1.clone())?;
            if gate.uuid != mapped.gate {
                return Err(refusal(
                    "Mapped Entity Gate identity differs from its source key",
                ));
            }
            qualify(context.model, &graph, &counts, gate.unit, "units")?;
            references.insert((gate.unit, "units"));
        }
    }
    // Validate the actual candidate graph through existing effective default-map
    // rules. Entity corrections also qualify retained legacy and nondefault
    // mappings; unrelated objects remain outside this correction's authority.
    graph.parts.insert(part.uuid, object.clone());
    let part_subject = &graph.subjects[&part.uuid];
    let map_subject = part
        .default_pin_pad_map
        .and_then(|id| graph.subjects.get(&id));
    let affected_maps: Vec<_> = graph
        .pin_pad_maps
        .iter()
        .filter(|(_, raw)| {
            (entity_edit || package_edit)
                && raw["part"].as_str().and_then(|s| s.parse::<Uuid>().ok()) == Some(part.uuid)
        })
        .filter_map(|(id, _)| graph.subjects.get(id))
        .collect();
    let footprint_subjects: Vec<_> = references
        .iter()
        .filter(|(_, kind)| *kind == "footprints")
        .filter_map(|(id, _)| graph.subjects.get(id))
        .collect();
    for diagnostic in graph.dependency_diagnostics() {
        let related = diagnostic.subject == *part_subject
            || ((entity_edit || package_edit)
                && diagnostic
                    .subject
                    .starts_with(&format!("{part_subject}#pad_map/")))
            || (package_edit
                && graph.subjects.get(&part.package).is_some_and(|s| {
                    diagnostic.subject == *s
                        || diagnostic.subject.starts_with(&format!("{s}#pads/"))
                }))
            || affected_maps.iter().any(|s| {
                diagnostic.subject == **s
                    || diagnostic.subject.starts_with(&format!("{s}#mappings/"))
            })
            || map_subject.is_some_and(|s| {
                diagnostic.subject == *s
                    || diagnostic.subject.starts_with(&format!("{s}#mappings/"))
            })
            || footprint_subjects.iter().any(|s| {
                diagnostic.subject.as_str() == s.as_str()
                    || diagnostic.subject.starts_with(&format!("{s}#pads/"))
            });
        if related && diagnostic.severity == "error" {
            return Err(refusal(diagnostic.message));
        }
    }
    preserve_users(context.model, part.uuid)?;
    saved_source::preserve_pinned_revision(context.model, part.uuid, "parts", &object)?;
    let mut prepared = library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )?;
    for (id, _) in references.into_iter().rev() {
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

fn graph(
    model: &DesignModel,
) -> Result<(crate::pool::LibraryGraph, BTreeMap<Uuid, usize>), EngineError> {
    let mut graph = crate::pool::LibraryGraph::default();
    let mut counts = BTreeMap::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        let Ok(raw) = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)
        else {
            continue;
        };
        let Some(id) = raw["uuid"].as_str().and_then(|v| v.parse::<Uuid>().ok()) else {
            continue;
        };
        *counts.entry(id).or_default() += 1;
        if let Some(owner) = model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.source_shard_id == shard.shard_id)
        {
            graph.insert_pool_object(&owner.kind, id, &shard.relative_path, raw);
        }
    }
    Ok((graph, counts))
}
fn qualify(
    model: &DesignModel,
    graph: &crate::pool::LibraryGraph,
    counts: &BTreeMap<Uuid, usize>,
    id: Uuid,
    kind: &str,
) -> Result<(), EngineError> {
    library_definition::validate_pool_definition_reference(model, id, kind, kind)?;
    if counts.get(&id) != Some(&1) {
        return Err(refusal(
            "Part relationship reference is missing or ambiguous",
        ));
    }
    let raw = match kind {
        "entities" => graph.entities.get(&id),
        "packages" => graph.packages.get(&id),
        "footprints" => graph.footprints.get(&id),
        "pin_pad_maps" => graph.pin_pad_maps.get(&id),
        "units" => graph.units.get(&id),
        "symbols" => graph.symbols.get(&id),
        _ => None,
    }
    .ok_or_else(|| refusal("Part relationship source ownership is unavailable"))?;
    if raw["schema_version"].as_u64() != Some(1) {
        return Err(refusal("Part relationship source schema is unsupported"));
    }
    match kind {
        "entities" => {
            let _: crate::pool::Entity = serde_json::from_value(raw.clone())?;
        }
        "packages" => {
            let _: crate::pool::Package = serde_json::from_value(raw.clone())?;
        }
        "footprints" => {
            let _: crate::pool::Footprint = serde_json::from_value(raw.clone())?;
        }
        "pin_pad_maps" => {
            let _: crate::pool::PinPadMap = serde_json::from_value(raw.clone())?;
        }
        "units" => {
            let _: crate::pool::Unit = serde_json::from_value(raw.clone())?;
        }
        "symbols" => {
            let _: crate::pool::Symbol = serde_json::from_value(raw.clone())?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn preserve_users(model: &DesignModel, part: Uuid) -> Result<(), EngineError> {
    use crate::substrate::{
        ComponentInstanceAuthority, ComponentInstanceShard, LibraryBindingRole,
    };
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ComponentInstance)
    {
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let ci = &raw["component_instance"];
        let refers_to_part = ci["part_ref"]["object_id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
            == Some(part)
            || ci["library_bindings"].as_object().is_some_and(|bindings| {
                bindings.values().any(|b| {
                    b["binding_role"] == "part"
                        && b["target_object_id"]
                            .as_str()
                            .and_then(|s| s.parse::<Uuid>().ok())
                            == Some(part)
                })
            });
        if !refers_to_part {
            continue;
        }
        let source: ComponentInstanceShard = serde_json::from_value(raw)?;
        let ci = &source.component_instance;
        let mut revisions = BTreeSet::new();
        if let Some(pin) = ci.part_ref.as_ref().filter(|r| r.object_id == part) {
            revisions.insert(pin.object_revision);
        }
        revisions.extend(
            ci.library_bindings
                .values()
                .filter(|b| {
                    b.binding_role == LibraryBindingRole::Part && b.target_object_id == part
                })
                .map(|b| b.pinned_object_revision),
        );
        if revisions.is_empty() {
            continue;
        }
        if model
            .component_instances
            .get(&ci.uuid)
            .is_none_or(|c| c.authority != ComponentInstanceAuthority::Authored)
            || model.objects.get(&ci.uuid).is_none_or(|o| {
                o.source_shard_id != shard.shard_id || o.object_revision != ci.object_revision
            })
        {
            return Err(refusal(
                "Existing Part user has no authenticated current source ownership",
            ));
        }
        for revision in revisions {
            model.materialized_pool_object_revision(part, "parts", revision)?;
        }
    }
    // Never let a source-default correction select new geometry for a legacy
    // physical user whose old binding cannot be authenticated.
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::BoardRoot)
    {
        let board = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        for (key, raw) in board
            .get("packages")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            if raw["part"].as_str().and_then(|s| s.parse::<Uuid>().ok()) != Some(part) {
                continue;
            }
            let id: Uuid = key
                .parse()
                .map_err(|_| refusal("Placed Package identity is malformed"))?;
            let owners = model
                .component_instances
                .values()
                .filter(|ci| {
                    ci.authority == ComponentInstanceAuthority::Authored
                        && ci.placed_package_refs.contains(&id)
                })
                .count();
            if owners > 1 {
                return Err(refusal(
                    "Existing physical Part user has no unique frozen component owner",
                ));
            }
            if owners == 0 {
                use crate::substrate::component_pad_source::{
                    ComponentPadMaterializationStatus, component_pad_projection,
                };
                let projection = component_pad_projection(model, &board)?;
                let pads = board["component_pads"][key]
                    .as_array()
                    .filter(|p| !p.is_empty())
                    .ok_or_else(|| {
                        refusal("Existing physical Part user has no frozen Pad proof")
                    })?;
                for pad in pads {
                    let pad: crate::substrate::component_pad_source::ComponentPadSource =
                        serde_json::from_value(pad.clone())?;
                    pad.validate_frozen()?;
                    let pad_id = pad.uuid;
                    if projection.statuses.get(&pad_id)
                        != Some(&ComponentPadMaterializationStatus::Current)
                        || projection
                            .query_pads
                            .get(&pad_id)
                            .is_none_or(|p| p.terminal().package != id)
                        || pad
                            .source_binding
                            .as_ref()
                            .is_none_or(|b| b.package_id != id)
                        || projection.unavailable.contains_key(&pad_id)
                    {
                        return Err(refusal(
                            "Existing physical Part user has unverified frozen Pad sources",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
