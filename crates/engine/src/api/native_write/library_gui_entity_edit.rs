//! Saved Entity corrections preserve raw gates; only their Symbol link may change.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    if !request.fields.contains_key("gate_symbols") {
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
    for (key, value) in request.fields {
        match key.as_str() {
            "gate_symbols" => {
                let changes: BTreeMap<String, Uuid> = serde_json::from_value(value)?;
                if changes.is_empty() {
                    return Err(refusal("Choose an existing gate Symbol to correct"));
                }
                let gates = object["gates"]
                    .as_object_mut()
                    .ok_or_else(|| refusal("Saved Entity gates are unavailable"))?;
                let mut seen = BTreeSet::new();
                for (key, replacement) in changes {
                    let gate: Uuid = key
                        .parse()
                        .map_err(|_| refusal("Gate identity is malformed"))?;
                    if !seen.insert(gate) {
                        return Err(refusal("Gate correction identity is ambiguous"));
                    }
                    let matches: Vec<_> = gates
                        .keys()
                        .filter(|k| k.parse::<Uuid>().ok() == Some(gate))
                        .cloned()
                        .collect();
                    if matches.len() != 1 {
                        return Err(refusal("Choose one existing unambiguous gate"));
                    }
                    let raw = gates.get_mut(&matches[0]).unwrap();
                    let parsed: crate::pool::Gate = serde_json::from_value(raw.clone())?;
                    if parsed.uuid != gate {
                        return Err(refusal("Saved gate membership and identity differ"));
                    }
                    reference(context.model, parsed.unit, "units")?;
                    for symbol in [parsed.symbol, replacement] {
                        let body = reference(context.model, symbol, "symbols")?;
                        if body["unit"].as_str().and_then(|s| s.parse::<Uuid>().ok())
                            != Some(parsed.unit)
                        {
                            return Err(refusal("Choose a Symbol for this gate's unchanged Unit"));
                        }
                        references.insert(symbol);
                    }
                    references.insert(parsed.unit);
                    if replacement != parsed.symbol {
                        preserve_users(context.model, request.object_id, gate, parsed.unit)?;
                    }
                    raw.as_object_mut()
                        .unwrap()
                        .insert("symbol".into(), json!(replacement));
                }
            }
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
    let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
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
    unit: Uuid,
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
                    if captured_gate.uuid != gate
                        || captured_gate.symbol != symbol
                        || captured_gate.unit != unit
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
