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

/// Display-field authoring has a separate, fully qualified physical-member
/// path. Move, assignment, membership and deletion retain `current_set`'s
/// schematic-only restriction.
pub(in crate::api::native_write) fn metadata_set(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    let bound = model.component_instances.values().any(|ci| {
        !ci.placed_package_refs.is_empty()
            && ci.placed_symbol_refs.iter().any(|id| symbols.contains(id))
    });
    if !bound {
        return current_set(model, symbols);
    }
    let mut result = BTreeMap::new();
    let mut qualified_sheets = BTreeSet::new();
    for symbol in symbols {
        let owners: Vec<_> = model
            .component_instances
            .values()
            .filter(|ci| ci.placed_symbol_refs.contains(symbol))
            .collect();
        let [owner] = owners.as_slice() else {
            return Err(invalid(
                "Metadata fields require one authored component owner",
            ));
        };
        if owner.authority != ComponentInstanceAuthority::Authored
            || owner.placed_symbol_refs != vec![*symbol]
            || owner.placed_package_refs.len() != 1
        {
            return Err(invalid(
                "This metadata review requires one-to-one placed component counterparts",
            ));
        }
        let object = model
            .objects
            .get(&owner.id)
            .ok_or_else(|| invalid("Component metadata source is unavailable"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.shard_id == object.source_shard_id && s.kind == SourceShardKind::ComponentInstance
            })
            .ok_or_else(|| invalid("Component metadata owner is unavailable"))?;
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let typed: ComponentInstanceShard = serde_json::from_value(raw.clone())?;
        if typed.component_instance.uuid != owner.id
            || typed.component_instance.object_revision != object.object_revision
            || typed.component_instance.placed_symbol_refs.len() != 1
            || typed.component_instance.placed_package_refs.len() != 1
            || typed.component_instance.placed_symbol_refs[0].object_id != *symbol
            || typed.component_instance.placed_symbol_refs[0].object_revision
                != model.objects[symbol].object_revision
            || typed.component_instance.placed_package_refs[0].object_id
                != owner.placed_package_refs[0]
        {
            return Err(invalid(
                "Component metadata source or member revision is stale",
            ));
        }
        let member = model
            .objects
            .get(symbol)
            .ok_or_else(|| invalid("Symbol source is unavailable"))?;
        let sheet_owner = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == member.source_shard_id)
            .ok_or_else(|| invalid("Symbol source owner is unavailable"))?;
        let sheet =
            model.materialized_source_shard_value_by_relative_path(&sheet_owner.relative_path)?;
        let sheet_id: Uuid = serde_json::from_value(sheet["uuid"].clone())?;
        if qualified_sheets.insert(sheet_id) {
            crate::api::native_write::schematic_board_transfer::field_revision::preview(
                model,
                sheet_id,
                &[],
            )?;
        }
        result.insert(*symbol, (owner.id, raw["component_instance"].clone()));
    }
    Ok(result)
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
        let declared_part = ci
            .part_ref
            .as_ref()
            .map(|reference| {
                if ci.library_bindings.values().any(|binding| {
                    binding.binding_role == LibraryBindingRole::Part
                        && (binding.target_object_id != reference.object_id
                            || binding.pinned_object_revision != reference.object_revision)
                }) {
                    return Err(invalid("Component Part declarations disagree"));
                }
                model.materialized_pool_object_revision(
                    reference.object_id,
                    "parts",
                    reference.object_revision,
                )
            })
            .transpose()?;
        if !ci.placed_symbol_refs.iter().all(current)
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
            .ok_or_else(|| invalid("Multiple Gates require a complete declared Part binding"))?
            .object_id;
        let part_raw =
            declared_part.ok_or_else(|| invalid("Declared Part source is unavailable"))?;
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
