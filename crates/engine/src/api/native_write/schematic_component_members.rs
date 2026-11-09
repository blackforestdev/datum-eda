//! One raw authored owner census, including every sibling of a selected gate.
use super::*;
use crate::api::native_write::schematic_selected_source::SelectedSchematicSource;
use crate::substrate::{
    ComponentInstanceAuthority, ComponentInstanceShard, ElectricalSelectionSnapshot,
    LibraryBindingRole, SourceShardKind,
};

pub(in crate::api::native_write) fn current_set(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    capture(model, symbols, false)
}
/// Adding a member requires complete Gate proof even for the original singleton.
pub(in crate::api::native_write) fn gate_set(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    capture(model, symbols, true)
}
fn capture(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
    require_gates: bool,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    let mut found = BTreeMap::new();
    let mut source = None;
    let mut ownership: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    let mut qualified_members = BTreeMap::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ComponentInstance)
    {
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let parsed: ComponentInstanceShard = serde_json::from_value(value.clone())?;
        let ci = &parsed.component_instance;
        for member in &ci.placed_symbol_refs {
            ownership.entry(member.object_id).or_default().push(ci.uuid);
        }
        if !ci
            .placed_symbol_refs
            .iter()
            .any(|r| symbols.contains(&r.object_id))
        {
            continue;
        }
        let projected = model
            .component_instances
            .get(&ci.uuid)
            .ok_or_else(|| invalid("ComponentInstance source is stale or unresolved"))?;
        if ci.placed_symbol_refs.is_empty()
            || !ci.placed_package_refs.is_empty()
            || projected.authority != ComponentInstanceAuthority::Authored
        {
            return Err(invalid(
                "This edit requires authored schematic-only component membership",
            ));
        }
        let current = |r: &crate::substrate::RevisionedRef| {
            model
                .objects
                .get(&r.object_id)
                .is_some_and(|o| o.object_revision == r.object_revision)
        };
        if !ci.placed_symbol_refs.iter().all(current)
            || !ci.part_ref.as_ref().is_none_or(current)
            || model
                .objects
                .get(&ci.uuid)
                .is_none_or(|o| o.object_revision != ci.object_revision)
            || ci.library_bindings.values().any(|b| {
                !matches!(
                    b.binding_role,
                    LibraryBindingRole::Part | LibraryBindingRole::Symbol
                ) || !b.local_override_refs.is_empty()
            })
        {
            return Err(invalid(
                "This edit requires current schematic-only library bindings without overrides",
            ));
        }
        // Preserve the established single-Symbol lifecycle, including partially
        // assigned native/imported symbols. Its caller already qualifies source
        // occurrence; ordinary property edits must not demand new Gate binding.
        if !require_gates && ci.placed_symbol_refs.len() == 1 {
            let symbol = ci.placed_symbol_refs[0].object_id;
            qualified_members.insert(symbol, ci.uuid);
            if found
                .insert(symbol, (ci.uuid, value["component_instance"].clone()))
                .is_some()
            {
                return Err(invalid("Symbol has ambiguous component ownership"));
            }
            continue;
        }
        if source.is_none() {
            let snapshot = ElectricalSelectionSnapshot::capture(model)
                .map_err(|e| invalid(format!("Component source unavailable: {e:?}")))?;
            let paths: BTreeMap<_, _> = snapshot
                .authored_sources()
                .map_err(|e| invalid(format!("Component occurrences unavailable: {e:?}")))?
                .into_iter()
                .filter(|s| s.occurrence.class == "symbols")
                .map(|s| (s.occurrence.source_id, s.occurrence.instance_path))
                .collect();
            source = Some((SelectedSchematicSource::from_snapshot(&snapshot)?, paths));
        }
        let (source, paths) = source.as_ref().unwrap();
        let part = ci
            .part_ref
            .as_ref()
            .ok_or_else(|| invalid("Multiple Gates require a complete current Part binding"))?
            .object_id;
        let part_raw = materialized_pool_object(model, part, "parts")?;
        let entity = id(&part_raw, "entity")?;
        let entity_raw = materialized_pool_object(model, entity, "entities")?;
        let mut sheet = None;
        let mut common_path = None;
        let mut members = BTreeSet::new();
        let mut gates = BTreeSet::new();
        let mut libraries = BTreeSet::new();
        for reference in &ci.placed_symbol_refs {
            let symbol = reference.object_id;
            qualified_members.insert(symbol, ci.uuid);
            if !members.insert(symbol) {
                return Err(invalid("Component Symbol membership is duplicated"));
            }
            // All siblings must remain unique source objects in this same source-sheet view.
            let path = paths
                .get(&symbol)
                .ok_or_else(|| invalid("Component occurrence is unavailable"))?;
            let owner = source.sheet("symbols", symbol, path)?;
            if common_path.replace(path).is_some_and(|old| old != path) {
                return Err(invalid(
                    "Component gates must share one qualified occurrence",
                ));
            }
            if sheet.replace(owner).is_some_and(|old| old != owner) {
                return Err(invalid(
                    "Component gates must share one unique source sheet",
                ));
            }
            let original = &source.source.sheets[&owner].symbols[&symbol];
            let raw = super::super::super::schematic_symbol_payload::source_symbol_payload(
                model, owner, original,
            )?;
            if raw["locked"].as_bool() == Some(true) {
                return Err(invalid("A component gate is locked"));
            }
            if !original.pin_overrides.is_empty() {
                return Err(invalid("Native gate pin overrides are unsupported"));
            }
            let (library, unit) = super::super::options_impl::validate_symbol(model, original)?;
            let gate = original
                .gate
                .ok_or_else(|| invalid("Component Gate identity is unavailable"))?;
            // Current exact chains retain their established admission. A changed
            // gate link requires the member's authenticated original Entity;
            // never compare an old placed body against an unrelated latest link.
            let captured;
            let declared = if entity_raw["gates"][gate.to_string()]["symbol"] == json!(library) {
                &entity_raw["gates"][gate.to_string()]
            } else {
                let definition =
                    model.materialized_placed_entity_definition(symbol, entity, gate, library)?;
                let raw = definition["gates"]
                    .as_object()
                    .and_then(|gates| {
                        gates
                            .iter()
                            .find(|(key, _)| key.parse::<Uuid>().ok() == Some(gate))
                    })
                    .map(|(_, raw)| raw)
                    .ok_or_else(|| invalid("Captured Gate is unavailable"))?;
                let logical: crate::pool::Gate = serde_json::from_value(raw.clone())?;
                captured = serde_json::to_value(logical)?;
                &captured
            };
            if original.part != Some(part)
                || original.entity != Some(entity)
                || !gates.insert(gate)
                || declared["uuid"] != json!(gate)
                || declared["symbol"] != json!(library)
                || declared["unit"] != json!(unit)
            {
                return Err(invalid(
                    "Component members must use distinct compatible declared Gates",
                ));
            }
            super::super::options_impl::validate_maps(model, part, &part_raw, gate)?;
            libraries.insert(library);
            if symbols.contains(&symbol)
                && found
                    .insert(symbol, (ci.uuid, value["component_instance"].clone()))
                    .is_some()
            {
                return Err(invalid("Symbol has ambiguous component ownership"));
            }
        }
        if ci.library_bindings.values().any(|b| {
            b.binding_role == LibraryBindingRole::Symbol
                && !libraries.contains(&b.target_object_id)
                && !entity_raw["gates"].as_object().is_some_and(|gates| {
                    gates
                        .values()
                        .any(|g| g["symbol"] == json!(b.target_object_id))
                })
        }) {
            return Err(invalid(
                "Component library Symbol binding does not match any gate",
            ));
        }
    }
    if qualified_members.iter().any(|(symbol, owner)| {
        ownership
            .get(symbol)
            .is_none_or(|owners| owners.as_slice() != [*owner])
    }) || model.component_instances.values().any(|c| {
        c.placed_symbol_refs.iter().any(|symbol| {
            qualified_members
                .get(symbol)
                .is_some_and(|owner| *owner != c.id)
        })
    }) {
        return Err(invalid(
            "A component gate has unsupported or ambiguous ownership",
        ));
    }
    if model.component_instances.values().any(|c| {
        c.placed_symbol_refs.iter().any(|symbol| {
            symbols.contains(symbol) && found.get(symbol).is_none_or(|(id, _)| *id != c.id)
        })
    }) {
        return Err(invalid(
            "Symbol has unsupported or ambiguous component ownership",
        ));
    }
    Ok(found)
}
