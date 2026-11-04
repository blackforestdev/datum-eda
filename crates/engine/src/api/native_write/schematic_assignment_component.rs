//! Existing schematic-only ComponentInstance identity is retained atomically.
use super::*;

use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
pub(in crate::api::native_write) fn current(
    model: &DesignModel,
    symbol: Uuid,
) -> Result<Option<(Uuid, Value)>, EngineError> {
    let value = current_set(model, &BTreeSet::from([symbol]))?.remove(&symbol);
    if value.as_ref().is_some_and(|(_, raw)| {
        raw["placed_symbol_refs"]
            .as_array()
            .is_some_and(|r| r.len() != 1)
    }) {
        return Err(invalid("Part assignment requires one Symbol per component"));
    }
    Ok(value)
}
#[path = "schematic_component_members.rs"]
mod members;
pub(in crate::api::native_write) use members::{current_set, gate_set};
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
/// An edit preserves every semantic/raw binding field and advances only
/// its owner epoch and the exact changed member pins. Capture ownership once for the complete set.
pub(in crate::api::native_write) fn repin_symbols(
    model: &DesignModel,
    symbols: &[PlacedSymbol],
) -> Result<Vec<Operation>, EngineError> {
    let ids: BTreeSet<_> = symbols.iter().map(|s| s.uuid).collect();
    let bindings = current_set(model, &ids)?;
    let unique: BTreeMap<_, _> = bindings.into_values().collect();
    unique
        .into_iter()
        .map(|(id, previous)| {
            let mut next = previous.clone();
            next["object_revision"] = json!(
                model.objects[&id]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("Component revision overflow"))?
            );
            for member in next["placed_symbol_refs"]
                .as_array_mut()
                .ok_or_else(|| invalid("Component members unavailable"))?
            {
                let symbol: Uuid = serde_json::from_value(member["object_id"].clone())?;
                if ids.contains(&symbol) {
                    member["object_revision"] = json!(
                        model.objects[&symbol]
                            .object_revision
                            .0
                            .checked_add(1)
                            .ok_or_else(|| invalid("Symbol revision overflow"))?
                    );
                }
            }
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
    if let Some((id, previous)) = current_set(model, &BTreeSet::from([symbol]))?.remove(&symbol) {
        let mut next = previous.clone();
        next["object_revision"] = json!(
            model.objects[&id]
                .object_revision
                .0
                .checked_add(1)
                .ok_or_else(|| invalid("Component revision overflow"))?
        );
        next["part_ref"] = component_instance["part_ref"].clone();
        let changed: BTreeSet<_> = options.members.iter().map(|member| member.uuid).collect();
        for member in next["placed_symbol_refs"].as_array_mut().unwrap() {
            let id: Uuid = serde_json::from_value(member["object_id"].clone())?;
            if !changed.contains(&id) {
                return Err(invalid(
                    "Part assignment must include every component member",
                ));
            }
            member["object_revision"] = json!(
                model.objects[&id]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("Symbol revision overflow"))?
            );
        }
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
