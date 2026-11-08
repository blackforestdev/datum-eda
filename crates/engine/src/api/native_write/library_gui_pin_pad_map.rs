//! New gate-qualified maps and explicit, proven-unplaced default assignment.
use super::super::library_pin_pad_map::{self as maps, PinPadMapMapping};
use super::*;
use crate::substrate::{
    ComponentInstanceAuthority, ComponentInstanceShard, ElectricalSelectionSnapshot,
    LibraryBindingRole, Operation,
};
use std::collections::HashSet;

fn object(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    kind: &str,
) -> Result<Value, EngineError> {
    super::super::schematic_library::materialized_pool_object(model, id, kind)
}
fn reference(raw: &Value, key: &str) -> Result<Uuid, EngineError> {
    raw.get(key)
        .and_then(Value::as_str)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| refusal(format!("Choose an existing named {key}")))
}
/// Source row identity must be unique before a typed UUID map could coalesce it.
fn row<'a>(raw: &'a Value, collection: &str, id: Uuid) -> Result<&'a Value, EngineError> {
    let rows = raw
        .get(collection)
        .and_then(Value::as_object)
        .ok_or_else(|| refusal(format!("{collection} are unavailable")))?;
    let candidates: Vec<_> = rows
        .iter()
        .filter(|(key, _)| Uuid::parse_str(key).ok() == Some(id))
        .collect();
    if candidates.len() != 1 || reference(candidates[0].1, "uuid")? != id {
        return Err(refusal(format!(
            "{collection} identity {id} is unavailable or ambiguous"
        )));
    }
    Ok(candidates[0].1)
}
pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if !create {
        return Err(refusal(
            "Saved PinPadMap editing is unavailable in this increment",
        ));
    }
    if request
        .fields
        .keys()
        .any(|k| !["part", "footprint", "mappings", "assign_default"].contains(&k.as_str()))
    {
        return Err(refusal("PinPadMap creation contains an unsupported field"));
    }
    let fields = Value::Object(request.fields);
    let part_id = reference(&fields, "part")?;
    let footprint_id = reference(&fields, "footprint")?;
    let assign = match fields.get("assign_default") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(refusal("Default assignment must be an explicit boolean")),
    };
    let part = object(context.model, part_id, "parts")?;
    let entity_id = reference(&part, "entity")?;
    let package_id = reference(&part, "package")?;
    let _ = object(context.model, package_id, "packages")?;
    let entity = object(context.model, entity_id, "entities")?;
    let footprint = object(context.model, footprint_id, "footprints")?;
    if reference(&footprint, "package")? != package_id {
        return Err(refusal("Choose a Footprint belonging to the Part Package"));
    }
    let submitted = fields
        .get("mappings")
        .and_then(Value::as_object)
        .ok_or_else(|| refusal("Choose named pad, gate and logical pin mappings"))?;
    let mut mappings = Vec::new();
    let mut references = HashSet::from([part_id, entity_id, package_id, footprint_id]);
    for (pad, entry) in submitted {
        let pad = Uuid::parse_str(pad).map_err(|_| refusal("Pad identity is invalid"))?;
        if entry
            .as_object()
            .is_none_or(|r| r.keys().any(|k| !["gate", "pin"].contains(&k.as_str())))
        {
            return Err(refusal("Mapping rows contain only a gate and logical pin"));
        }
        let gate = reference(entry, "gate")?;
        let pin = reference(entry, "pin")?;
        row(&footprint, "pads", pad)?;
        let gate_raw = row(&entity, "gates", gate)?;
        let unit_id = reference(gate_raw, "unit")?;
        let unit = object(context.model, unit_id, "units")?;
        row(&unit, "pins", pin)?;
        references.insert(unit_id);
        mappings.push(PinPadMapMapping { pad, gate, pin });
    }
    // Validate the original list before JSON-object construction could merge aliases.
    maps::validate_pin_pad_map_payload(context.model, part_id, Some(footprint_id), &mappings)?;
    let mut specs = vec![PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new("pool", "pin_pad_maps", request.object_id),
        object: maps::pin_pad_map_payload(
            request.object_id,
            part_id,
            Some(footprint_id),
            &mappings,
        ),
    }];
    if assign {
        if part
            .get("default_footprint")
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<Uuid>().ok())
            != Some(footprint_id)
        {
            return Err(refusal(
                "Default assignment requires the Part's matching explicit default Footprint; create the map without assignment",
            ));
        }
        prove_unplaced(context.model, part_id)?;
        specs.push(maps::set_part_default_pin_pad_map_spec(
            context.model,
            "pool",
            part_id,
            request.object_id,
        )?);
    }
    let mut prepared =
        library::build_pool_library_write(context.model, provenance, Some("pool"), specs)?;
    let mut refs: Vec<_> = references.into_iter().collect();
    refs.sort();
    for id in refs.into_iter().rev() {
        if !prepared
            .batch
            .operations
            .iter()
            .any(|op| matches!(op,Operation::GuardObjectRevision {object_id,..} if *object_id==id))
        {
            prepared.batch.operations.insert(
                0,
                Operation::GuardObjectRevision {
                    object_id: id,
                    expected_object_revision: context.model.objects[&id].object_revision,
                },
            );
        }
    }
    Ok(prepared)
}
fn unavailable() -> EngineError {
    refusal("Cannot establish that the Part is unplaced; create the map without default assignment")
}
fn placed() -> EngineError {
    refusal(
        "This Part has authored placed users or declarations; create the map without default assignment",
    )
}
fn unique_keys(raw: &Value) -> Result<(), EngineError> {
    let rows = raw.as_object().ok_or_else(unavailable)?;
    let mut ids = HashSet::new();
    for key in rows.keys() {
        let id = Uuid::parse_str(key).map_err(|_| unavailable())?;
        if !ids.insert(id) {
            return Err(unavailable());
        }
    }
    Ok(())
}
/// Negative proof uses the existing canonical captured source basis. Resolved
/// component instances alone omit invalid persisted declarations and are unsafe.
fn prove_unplaced(model: &crate::substrate::DesignModel, part: Uuid) -> Result<(), EngineError> {
    if model.diagnostics.iter().any(|d| {
        matches!(
            d.code.as_str(),
            "invalid_component_instance_shard"
                | "missing_component_instance_shard"
                | "component_instance_inventory_unavailable"
        )
    }) {
        return Err(unavailable());
    }
    for kind in [SourceShardKind::BoardRoot, SourceShardKind::SchematicRoot] {
        if model
            .source_shards
            .iter()
            .filter(|s| s.kind == kind)
            .count()
            != 1
        {
            return Err(unavailable());
        }
    }
    if model.component_instances.values().any(|ci| {
        ci.part_ref == Some(part)
            || ci
                .library_bindings
                .values()
                .any(|b| b.binding_role == LibraryBindingRole::Part && b.target_object_id == part)
    }) {
        return Err(placed());
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model).map_err(|_| unavailable())?;
    let mut basis_ids = HashSet::new();
    if snapshot
        .source_basis()
        .iter()
        .any(|s| !basis_ids.insert(s.shard_id))
    {
        return Err(unavailable());
    }
    let mut components = HashSet::new();
    let mut sheets = HashSet::new();
    let mut definitions = HashSet::new();
    let mut definition_roots = Vec::new();
    let mut board_roots = 0;
    if let Some(schematic) = snapshot.schematic_source() {
        for sheet in schematic.sheets.values() {
            if sheet.symbols.values().any(|s| s.part == Some(part)) {
                return Err(placed());
            }
        }
    }
    for shard in &model.source_shards {
        if !matches!(
            shard.kind,
            SourceShardKind::ComponentInstance
                | SourceShardKind::BoardRoot
                | SourceShardKind::SchematicRoot
                | SourceShardKind::SchematicSheet
                | SourceShardKind::SchematicDefinition
        ) {
            continue;
        }
        if shard.schema_version.is_some_and(|v| v != 1) {
            return Err(unavailable());
        }
        if !snapshot
            .source_basis()
            .iter()
            .any(|s| s.shard_id == shard.shard_id && s.content_hash == shard.content_hash)
        {
            return Err(unavailable());
        }
        let raw = snapshot
            .source_shard_value(snapshot.revision(), shard.shard_id)
            .map_err(|_| unavailable())?;
        if raw
            .get("schema_version")
            .is_some_and(|v| v.as_u64() != Some(1))
        {
            return Err(unavailable());
        }
        match shard.kind {
            SourceShardKind::ComponentInstance => {
                if let Some(bindings) = raw.pointer("/component_instance/library_bindings") {
                    unique_keys(bindings)?;
                }
                let declared: ComponentInstanceShard =
                    serde_json::from_value(raw.clone()).map_err(|_| unavailable())?;
                let ci = declared.component_instance;
                if declared.schema_version != 1 || !components.insert(ci.uuid) {
                    return Err(unavailable());
                }
                if ci.part_ref.as_ref().is_some_and(|r| r.object_id == part)
                    || ci.library_bindings.values().any(|b| {
                        b.binding_role == LibraryBindingRole::Part && b.target_object_id == part
                    })
                {
                    return Err(placed());
                }
                if model.component_instances.get(&ci.uuid).is_none_or(|owner| {
                    owner.authority != ComponentInstanceAuthority::Authored
                        || owner.object_revision != ci.object_revision
                        || owner.library_bindings != ci.library_bindings
                }) {
                    return Err(unavailable());
                }
            }
            SourceShardKind::BoardRoot => {
                board_roots += 1;
                if board_roots > 1 {
                    return Err(unavailable());
                }
                let packages = raw.get("packages").ok_or_else(unavailable)?;
                unique_keys(packages)?;
                for (key, value) in packages.as_object().unwrap() {
                    let package: crate::board::PlacedPackage =
                        serde_json::from_value(value.clone()).map_err(|_| unavailable())?;
                    if Uuid::parse_str(key).ok() != Some(package.uuid) {
                        return Err(unavailable());
                    }
                    if package.part == part {
                        return Err(placed());
                    }
                }
            }
            SourceShardKind::SchematicSheet => {
                let symbols = raw.get("symbols").ok_or_else(unavailable)?;
                unique_keys(symbols)?;
                let sheet: crate::schematic::Sheet =
                    serde_json::from_value(raw.clone()).map_err(|_| unavailable())?;
                if !sheets.insert(sheet.uuid) {
                    return Err(unavailable());
                }
                for (key, value) in symbols.as_object().unwrap() {
                    let symbol: crate::schematic::PlacedSymbol =
                        serde_json::from_value(value.clone()).map_err(|_| unavailable())?;
                    if Uuid::parse_str(key).ok() != Some(symbol.uuid) {
                        return Err(unavailable());
                    }
                    if symbol.part == Some(part) {
                        return Err(placed());
                    }
                }
            }
            SourceShardKind::SchematicDefinition => {
                let definition: crate::schematic::SheetDefinition =
                    serde_json::from_value(raw.clone()).map_err(|_| unavailable())?;
                if !definitions.insert(definition.uuid) {
                    return Err(unavailable());
                }
                definition_roots.push(definition.root_sheet);
            }
            // The snapshot's canonical schematic loader validates root indexes
            // and supplies the complete typed source membership checked above.
            SourceShardKind::SchematicRoot => {}
            _ => unreachable!(),
        }
    }
    if definition_roots.iter().any(|root| !sheets.contains(root)) {
        return Err(unavailable());
    }
    Ok(())
}
#[cfg(test)]
#[path = "library_gui_pin_pad_map_tests.rs"]
mod tests;
