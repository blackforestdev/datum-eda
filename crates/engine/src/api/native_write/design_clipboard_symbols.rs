//! Symbol clipboard admission and fresh births reuse ordinary library authority.
use super::{
    component_instances::build_placed_symbol_component_instance_op,
    design_clipboard::{
        ClipboardClass, ClipboardItem, ClipboardPlacement, DesignClipboardBundle,
        checked_cardinal_point,
    },
    schematic_annotation::prefix,
    schematic_library::{
        materialize_pool_symbol_pins, materialized_pool_object, part_placement::PartGateSelection,
    },
    schematic_symbol_payload::set_point,
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::PlacedSymbol,
    substrate::{
        ComponentInstanceAuthority, DesignModel, ElectricalSelectionSnapshot, LibraryBindingRole,
        ObjectRevision, Operation,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolClipboardSource {
    pub definition: Value,
    pub dependencies: BTreeMap<Uuid, ObjectRevision>,
    pub part: Option<PartGateSelection>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolClipboardSources {
    pub sources: BTreeMap<Uuid, SymbolClipboardSource>,
    pub reserved_references: BTreeSet<String>,
}
fn invalid(message: &str) -> EngineError {
    EngineError::Validation(message.into())
}
fn census(model: &DesignModel) -> Result<BTreeSet<String>, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|_| invalid("Clipboard target source unavailable"))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Clipboard requires native schematic source"))?;
    Ok(source
        .sheets
        .values()
        .flat_map(|s| s.symbols.values().map(|p| p.reference.clone()))
        .collect())
}
fn library(symbol: &PlacedSymbol) -> Result<Uuid, EngineError> {
    symbol
        .lib_id
        .as_deref()
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| invalid("Copy requires an authenticated native library Symbol"))
}
fn source(model: &DesignModel, raw: &Value) -> Result<SymbolClipboardSource, EngineError> {
    let placed: PlacedSymbol = serde_json::from_value(raw.clone())?;
    let lib = library(&placed)?;
    let definition = model
        .materialized_placed_symbol_definition(placed.uuid, lib)?
        .ok_or_else(|| invalid("Copied Symbol has no authenticated frozen source"))?;
    if materialized_pool_object(model, lib, "symbols")? != definition {
        return Err(invalid(
            "Copied Symbol source changed; exact new placement is unavailable",
        ));
    }
    let body: crate::pool::Symbol = serde_json::from_value(definition.clone())?;
    let mut dependencies = BTreeMap::new();
    let mut remember = |id| -> Result<(), EngineError> {
        let object = model
            .objects
            .get(&id)
            .ok_or_else(|| invalid("Copied dependency is unavailable"))?;
        dependencies.insert(id, object.object_revision);
        Ok(())
    };
    remember(placed.uuid)?;
    remember(lib)?;
    remember(body.unit)?;
    let mut expected = materialize_pool_symbol_pins(model, placed.lib_id.as_deref())?;
    for pin in &mut expected {
        pin.position = super::schematic_edit::place_local_point(
            pin.position,
            placed.position,
            placed.rotation,
            placed.mirrored,
        )?;
    }
    if expected.len() != placed.pins.len()
        || expected.iter().zip(&placed.pins).any(|(a, b)| {
            a.library_pin != b.library_pin
                || a.number != b.number
                || a.name != b.name
                || a.electrical_type != b.electrical_type
                || a.position != b.position
        })
    {
        return Err(invalid(
            "Copied pins differ from lawful current library placement",
        ));
    }
    let owners: Vec<_> = model
        .component_instances
        .values()
        .filter(|ci| ci.placed_symbol_refs.contains(&placed.uuid))
        .collect();
    if owners.len() > 1 {
        return Err(invalid("Copied Symbol ownership is ambiguous"));
    }
    let part = if let Some(owner) = owners.first() {
        if owner.authority != ComponentInstanceAuthority::Authored
            || owner.placed_symbol_refs != [placed.uuid]
            || !owner.placed_package_refs.is_empty()
        {
            return Err(invalid(
                "Copy requires a complete schematic-only single-gate component",
            ));
        }
        if owner.placed_symbol_roles.len() != 1
            || owner
                .placed_symbol_roles
                .get(&placed.uuid)
                .is_none_or(|role| role.role != "primary" || role.label.is_some())
            || !owner.placed_package_roles.is_empty()
        {
            return Err(invalid(
                "Copied component has unsupported owner role overrides",
            ));
        }
        let part = placed
            .part
            .ok_or_else(|| invalid("Copied owner has no supported Part placement"))?;
        let gate = placed
            .gate
            .ok_or_else(|| invalid("Copied Part gate unavailable"))?;
        let choice = PartGateSelection::capture(model, part, gate, lib)?;
        if placed.entity != Some(choice.entity) || owner.part_ref != Some(part) {
            return Err(invalid(
                "Copied Part ownership differs from its placed source",
            ));
        }
        let entity = materialized_pool_object(model, choice.entity, "entities")?;
        if entity
            .get("gates")
            .and_then(Value::as_object)
            .map(|g| g.len())
            != Some(1)
        {
            return Err(invalid("Multi-gate component copying is not supported"));
        }
        if owner.library_bindings.len() != 1
            || owner.library_bindings.values().any(|b| {
                b.binding_role != LibraryBindingRole::Part
                    || b.target_object_id != part
                    || b.pinned_object_revision != ObjectRevision(choice.part_revision)
                    || !b.local_override_refs.is_empty()
            })
        {
            return Err(invalid(
                "Copied bindings cannot be recreated by ordinary new placement",
            ));
        }
        remember(owner.id)?;
        remember(part)?;
        remember(choice.entity)?;
        Some(choice)
    } else {
        if placed.part.is_some() {
            return Err(invalid("Copied Part has no authored owner"));
        }
        let resolved = super::schematic_library::resolve_pool_symbol_component_binding(
            model,
            placed.lib_id.as_deref(),
        )?;
        match resolved.binding {
            Some(binding)
                if binding.part.is_none()
                    && placed.entity == Some(binding.entity_id)
                    && placed.gate == Some(binding.gate_id) =>
            {
                let entity = materialized_pool_object(model, binding.entity_id, "entities")?;
                if entity
                    .get("gates")
                    .and_then(Value::as_object)
                    .map(|g| g.len())
                    != Some(1)
                {
                    return Err(invalid("Multi-gate component copying is not supported"));
                }
                remember(binding.entity_id)?;
            }
            None if placed.entity.is_none() && placed.gate.is_none() => {}
            _ => {
                return Err(invalid(
                    "Copied source cannot be recreated without changing component ownership",
                ));
            }
        }
        None
    };
    Ok(SymbolClipboardSource {
        definition,
        dependencies,
        part,
    })
}
pub(super) fn capture(
    model: &DesignModel,
    items: &[ClipboardItem],
) -> Result<Option<SymbolClipboardSources>, EngineError> {
    let mut sources = BTreeMap::new();
    for item in items.iter().filter(|i| i.class == ClipboardClass::Symbol) {
        sources.insert(item.id, source(model, &item.raw)?);
    }
    if sources.is_empty() {
        return Ok(None);
    }
    Ok(Some(SymbolClipboardSources {
        sources,
        reserved_references: census(model)?,
    }))
}
pub(super) fn validate(bundle: &DesignClipboardBundle) -> Result<(), EngineError> {
    let ids: BTreeSet<_> = bundle
        .items
        .iter()
        .filter(|i| i.class == ClipboardClass::Symbol)
        .map(|i| i.id)
        .collect();
    match &bundle.symbols {
        None if ids.is_empty() => Ok(()),
        Some(data)
            if !ids.is_empty() && data.sources.keys().copied().collect::<BTreeSet<_>>() == ids =>
        {
            for item in bundle
                .items
                .iter()
                .filter(|i| i.class == ClipboardClass::Symbol)
            {
                let placed: PlacedSymbol = serde_json::from_value(item.raw.clone())?;
                let body: crate::pool::Symbol =
                    serde_json::from_value(data.sources[&item.id].definition.clone())?;
                if body.uuid != library(&placed)? {
                    return Err(invalid("Clipboard definition identity differs"));
                }
                prefix(&placed.reference)?;
            }
            Ok(())
        }
        _ => Err(invalid("Clipboard Symbol source set is incomplete")),
    }
}
pub(super) fn prepare(
    model: &DesignModel,
    bundle: &mut DesignClipboardBundle,
) -> Result<(), EngineError> {
    bundle.validate()?;
    if bundle.symbols.is_none() {
        return Ok(());
    }
    if bundle.source_project != model.project.project_id {
        return Err(invalid("Symbol block paste requires the same Project"));
    }
    let current =
        capture(model, &bundle.items)?.ok_or_else(|| invalid("Copied Symbol source missing"))?;
    if current.sources != bundle.symbols.as_ref().unwrap().sources {
        return Err(invalid(
            "Copied component sources changed; copy the block again",
        ));
    }
    // Authenticate the captured raw members as well as their library revisions.
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|_| invalid("Copied source unavailable"))?;
    let native = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Copied source unavailable"))?;
    super::schematic_edit::require_unique_occurrence(
        native,
        bundle.source_sheet,
        &bundle.instance_path,
    )?;
    for item in bundle
        .items
        .iter()
        .filter(|i| i.class == ClipboardClass::Symbol)
    {
        let original = native
            .sheets
            .get(&bundle.source_sheet)
            .and_then(|s| s.symbols.get(&item.id))
            .ok_or_else(|| invalid("Copied source Symbol is no longer available"))?;
        let raw = super::schematic_symbol_payload::source_symbol_payload(
            model,
            bundle.source_sheet,
            original,
        )?;
        if raw != item.raw {
            return Err(invalid("Copied source Symbol changed; copy again"));
        }
    }
    bundle.symbols.as_mut().unwrap().reserved_references = current.reserved_references;
    Ok(())
}
pub(super) fn references(
    bundle: &DesignClipboardBundle,
) -> Result<BTreeMap<Uuid, String>, EngineError> {
    let mut result = BTreeMap::new();
    let Some(data) = &bundle.symbols else {
        return Ok(result);
    };
    let mut reserved = data.reserved_references.clone();
    let mut symbols: Vec<_> = bundle
        .items
        .iter()
        .filter(|i| i.class == ClipboardClass::Symbol)
        .map(|i| serde_json::from_value::<PlacedSymbol>(i.raw.clone()))
        .collect::<Result<_, _>>()?;
    symbols.sort_by_key(|s| (s.position.y, s.position.x, s.uuid));
    for symbol in symbols {
        let prefix = prefix(&symbol.reference)?;
        let mut number = 1u64;
        loop {
            let name = format!("{prefix}{number}");
            if reserved.insert(name.clone()) {
                result.insert(symbol.uuid, name);
                break;
            }
            number = number
                .checked_add(1)
                .ok_or_else(|| invalid("Copied reference number overflow"))?;
        }
    }
    Ok(result)
}
pub(super) fn owned_id(seed: Uuid, kind: &str, id: Uuid) -> Uuid {
    Uuid::new_v5(&seed, format!("design-clipboard:{kind}:{id}").as_bytes())
}
pub(super) fn placed(
    raw: &Value,
    id: Uuid,
    seed: Uuid,
    reference: Point,
    pose: ClipboardPlacement,
    name: &str,
) -> Result<Value, EngineError> {
    let original: PlacedSymbol = serde_json::from_value(raw.clone())?;
    let mut raw = raw.clone();
    raw["uuid"] = serde_json::to_value(id)?;
    raw["reference"] = name.into();
    raw["rotation"] = ((if pose.mirrored {
        -original.rotation.rem_euclid(360)
    } else {
        original.rotation
    })
    .rem_euclid(360)
        + pose.rotation_deg.rem_euclid(360))
    .rem_euclid(360)
    .into();
    raw["mirrored"] = (pose.mirrored ^ original.mirrored).into();
    set_point(
        &mut raw["position"],
        checked_cardinal_point(original.position, reference, pose)?,
    );
    for (index, pin) in original.pins.iter().enumerate() {
        raw["pins"][index]["uuid"] = serde_json::to_value(owned_id(seed, "pin", pin.uuid))?;
        set_point(
            &mut raw["pins"][index]["position"],
            checked_cardinal_point(pin.position, reference, pose)?,
        );
    }
    for (index, field) in original.fields.iter().enumerate() {
        raw["fields"][index]["uuid"] = serde_json::to_value(owned_id(seed, "field", field.uuid))?;
        if let Some(point) = field.position {
            set_point(
                &mut raw["fields"][index]["position"],
                checked_cardinal_point(point, reference, pose)?,
            );
        }
    }
    for (index, override_) in original.pin_overrides.iter().enumerate() {
        if !original.pins.iter().any(|p| p.uuid == override_.pin) {
            return Err(invalid("Copied pin override refers outside its Symbol"));
        }
        raw["pin_overrides"][index]["pin"] =
            serde_json::to_value(owned_id(seed, "pin", override_.pin))?;
        if let Some(point) = override_.position {
            set_point(
                &mut raw["pin_overrides"][index]["position"],
                checked_cardinal_point(point, reference, pose)?,
            );
        }
    }
    Ok(raw)
}
/// Re-evaluate ordinary new-placement authority; never replay old CI payloads.
pub(super) fn births(
    model: &DesignModel,
    bundle: &DesignClipboardBundle,
    items: &[ClipboardItem],
) -> Result<Vec<Operation>, EngineError> {
    if bundle.symbols.is_none() {
        return Ok(vec![]);
    }
    let mut checked = bundle.clone();
    prepare(model, &mut checked)?;
    if checked.symbols != bundle.symbols {
        return Err(invalid(
            "Paste target references changed; restart placement",
        ));
    }
    let data = bundle.symbols.as_ref().unwrap();
    let mut result = Vec::new();
    for (old, new) in bundle.items.iter().zip(items) {
        if old.class != ClipboardClass::Symbol {
            continue;
        }
        if let Some(part) = &data.sources[&old.id].part {
            part.resolve(model, part.symbol)?;
            result.push(build_placed_symbol_component_instance_op(
                model,
                new.id,
                part.symbol,
                part.part,
            )?);
        }
    }
    Ok(result)
}
pub(super) fn symbol_ids(raw: &Value) -> Result<BTreeSet<Uuid>, EngineError> {
    let symbol: PlacedSymbol = serde_json::from_value(raw.clone())?;
    let ids: Vec<_> = std::iter::once(symbol.uuid)
        .chain(symbol.pins.iter().map(|p| p.uuid))
        .chain(symbol.fields.iter().map(|f| f.uuid))
        .collect();
    let unique: BTreeSet<_> = ids.iter().copied().collect();
    if unique.len() != ids.len() || unique.contains(&Uuid::nil()) {
        return Err(invalid("Copied Symbol owned identities collide"));
    }
    Ok(unique)
}
/// Pure comparison of declared first births, never admission for a historical revision.
pub(super) fn expected_births(
    project: Uuid,
    bundle: &DesignClipboardBundle,
    items: &[ClipboardItem],
) -> Vec<Operation> {
    let Some(data) = &bundle.symbols else {
        return vec![];
    };
    bundle
        .items
        .iter()
        .zip(items)
        .filter_map(|(old, new)| {
            data.sources.get(&old.id)?.part.as_ref().map(|part| {
                super::component_instances::placed_symbol_component_instance_birth(
                    project,
                    new.id,
                    part.symbol,
                    part.part,
                    ObjectRevision(part.part_revision),
                )
            })
        })
        .collect()
}
pub(super) fn birth_ids(op: &Operation) -> Result<BTreeSet<Uuid>, EngineError> {
    let Operation::CreateComponentInstance {
        component_instance_id,
        component_instance,
    } = op
    else {
        return Err(invalid("Clipboard contains an unsupported component birth"));
    };
    let mut ids = BTreeSet::from([*component_instance_id]);
    let bindings = component_instance
        .get("library_bindings")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("Clipboard component bindings unavailable"))?;
    for key in bindings.keys() {
        if !ids.insert(
            key.parse()
                .map_err(|_| invalid("Clipboard binding identity invalid"))?,
        ) {
            return Err(invalid("Clipboard component birth identities collide"));
        }
    }
    Ok(ids)
}
#[cfg(test)]
#[path = "design_clipboard_symbol_tests.rs"]
mod tests;
