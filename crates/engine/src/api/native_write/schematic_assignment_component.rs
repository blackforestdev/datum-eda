//! Existing schematic-only ComponentInstance identity is retained atomically.
use super::*;
use crate::substrate::{
    ComponentInstanceAuthority, ComponentInstanceShard, LibraryBindingRole, SourceShardKind,
};

use std::collections::{BTreeMap, BTreeSet};

pub(super) fn current(
    model: &DesignModel,
    symbol: Uuid,
) -> Result<Option<(Uuid, Value)>, EngineError> {
    Ok(current_set(model, &BTreeSet::from([symbol]))?.remove(&symbol))
}
fn current_set(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    let mut found = BTreeMap::new();
    // Include unresolved source records: a dropped stale projection is never "unbound".
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ComponentInstance)
    {
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let parsed: ComponentInstanceShard = serde_json::from_value(value.clone())?;
        let ci = &parsed.component_instance;
        let Some(symbol) = ci
            .placed_symbol_refs
            .iter()
            .find_map(|r| symbols.contains(&r.object_id).then_some(r.object_id))
        else {
            continue;
        };
        let projected = model
            .component_instances
            .get(&ci.uuid)
            .ok_or_else(|| invalid("ComponentInstance source is stale or unresolved"))?;
        if found.contains_key(&symbol)
            || ci.placed_symbol_refs.len() != 1
            || !ci.placed_package_refs.is_empty()
            || projected.authority != ComponentInstanceAuthority::Authored
        {
            return Err(invalid(
                "This edit supports one schematic-only Symbol per authored ComponentInstance",
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
        found.insert(symbol, (ci.uuid, value["component_instance"].clone()));
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
/// Capture the same strict raw ownership once for the complete selected set.
/// A surviving external reference cannot be silently discarded with its owner.
pub(in crate::api::native_write) fn delete_bindings(
    model: &DesignModel,
    symbols: &BTreeSet<Uuid>,
    children: &BTreeSet<Uuid>,
) -> Result<BTreeMap<Uuid, (Uuid, Value)>, EngineError> {
    let bindings = current_set(model, symbols)?;
    let mut removed = symbols.clone();
    removed.extend(children);
    removed.extend(bindings.values().map(|(id, _)| *id));
    if model.relationships.values().any(|r| {
        r.from
            .iter()
            .chain(&r.to)
            .any(|r| removed.contains(&r.object_id))
    }) || model.variants.values().any(|v| {
        v.fitted
            .keys()
            .chain(v.property_overrides.keys())
            .any(|id| removed.contains(id))
    }) || model.electrical_identities.values().any(|r| {
        matches!(&r.identity,
            crate::substrate::ElectricalIdentity::NetRelationship { evidence, .. }
            if evidence.iter().any(|e| removed.contains(&e.component_instance.object_id)))
    }) {
        return Err(invalid(
            "Component has surviving relationship or variant dependencies",
        ));
    }
    Ok(bindings)
}
/// A transform preserves every semantic/raw binding field and only advances
/// the two exact revision pins. Capture ownership once for the complete set.
pub(in crate::api::native_write) fn repin_symbols(
    model: &DesignModel,
    symbols: &[PlacedSymbol],
) -> Result<Vec<Operation>, EngineError> {
    let ids: BTreeSet<_> = symbols.iter().map(|s| s.uuid).collect();
    let bindings = current_set(model, &ids)?;
    bindings
        .into_iter()
        .map(|(symbol, (id, previous))| {
            let mut next = previous.clone();
            next["object_revision"] = json!(
                model.objects[&id]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("Component revision overflow"))?
            );
            next["placed_symbol_refs"][0]["object_revision"] = json!(
                model.objects[&symbol]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("Symbol revision overflow"))?
            );
            Ok(Operation::SetComponentInstance {
                component_instance_id: id,
                previous_component_instance: previous,
                component_instance: next,
            })
        })
        .collect()
}
pub(super) fn operation(
    model: &DesignModel,
    options: &AssignmentOptions,
    choice: &AssignmentChoice,
) -> Result<Operation, EngineError> {
    let symbol = options.original.uuid;
    let library = options
        .original
        .lib_id
        .as_ref()
        .unwrap()
        .parse()
        .map_err(|_| invalid("Native Symbol identity unavailable"))?;
    let Operation::CreateComponentInstance {
        component_instance_id,
        mut component_instance,
    } = super::super::component_instances::build_placed_symbol_component_instance_op(
        model,
        symbol,
        library,
        choice.part,
    )?
    else {
        unreachable!()
    };
    let revision = model
        .objects
        .get(&symbol)
        .and_then(|o| o.object_revision.0.checked_add(1))
        .ok_or_else(|| invalid("Symbol revision overflow"))?;
    component_instance["placed_symbol_refs"][0]["object_revision"] = json!(revision);
    if let Some((id, previous)) = current(model, symbol)? {
        let mut next = previous.clone();
        next["object_revision"] = json!(
            model.objects[&id]
                .object_revision
                .0
                .checked_add(1)
                .ok_or_else(|| invalid("Component revision overflow"))?
        );
        next["part_ref"] = component_instance["part_ref"].clone();
        next["placed_symbol_refs"][0]["object_revision"] = json!(revision);
        if next.get("library_bindings").is_none() {
            next["library_bindings"] = json!({});
        }
        let bindings = next["library_bindings"]
            .as_object_mut()
            .ok_or_else(|| invalid("Invalid component library bindings"))?;
        let old = bindings
            .iter()
            .find(|(_, b)| b["binding_role"] == "part")
            .map(|(id, _)| id.clone());
        if let Some(key) = old {
            let binding = bindings.get_mut(&key).unwrap();
            binding["target_object_id"] = json!(choice.part);
            binding["pinned_object_revision"] =
                json!(model.objects[&choice.part].object_revision.0);
        } else {
            let binding = component_instance["library_bindings"]
                .as_object()
                .unwrap()
                .values()
                .next()
                .unwrap()
                .clone();
            let key = super::super::ids::derive_object_id(
                &model.project.project_id,
                "library-binding",
                &[id.to_string(), "part".into(), choice.part.to_string()],
            );
            bindings.insert(key.to_string(), binding);
        }
        Ok(Operation::SetComponentInstance {
            component_instance_id: id,
            previous_component_instance: previous,
            component_instance: next,
        })
    } else {
        if model.objects.contains_key(&component_instance_id) {
            return Err(invalid("Derived component identity is already occupied"));
        }
        Ok(Operation::CreateComponentInstance {
            component_instance_id,
            component_instance,
        })
    }
}
