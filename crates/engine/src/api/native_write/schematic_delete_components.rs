//! Exact member removal preserves surviving gates and raw component complements.
use super::*;
use serde_json::{Value, json};
pub(super) fn detached_component(
    raw: &Value,
    removed: &BTreeSet<Uuid>,
) -> Result<Value, EngineError> {
    let mut next = raw.clone();
    next["object_revision"] = json!(
        raw["object_revision"]
            .as_u64()
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| invalid("Component revision overflow"))?
    );
    let refs = next["placed_symbol_refs"]
        .as_array_mut()
        .ok_or_else(|| invalid("Component members unavailable"))?;
    let mut ids = BTreeSet::new();
    for r in refs.iter() {
        ids.insert(serde_json::from_value::<Uuid>(r["object_id"].clone())?);
    }
    refs.retain(|r| !removed.iter().any(|id| r["object_id"] == json!(id)));
    if let Some(roles) = next.get_mut("placed_symbol_roles") {
        let roles = roles
            .as_object_mut()
            .ok_or_else(|| invalid("Component roles malformed"))?;
        for id in ids.intersection(removed) {
            roles.remove(&id.to_string());
        }
    }
    Ok(next)
}
pub(in crate::api::native_write) fn validate_owned_components(
    ops: &[Operation],
) -> Result<(), EngineError> {
    let symbols: BTreeSet<_> = ops
        .iter()
        .filter_map(|op| match op {
            Operation::DeleteSchematicSymbol { symbol_id, .. } => Some(*symbol_id),
            _ => None,
        })
        .collect();
    let mut owners = BTreeSet::new();
    let mut components = BTreeSet::new();
    for op in ops {
        let (id, raw, next) = match op {
            Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => (*component_instance_id, component_instance, None),
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => (
                *component_instance_id,
                previous_component_instance,
                Some(component_instance),
            ),
            _ => continue,
        };
        let parsed: crate::substrate::ComponentInstanceShard =
            serde_json::from_value(json!({"schema_version":1,"component_instance":raw}))?;
        let ci = parsed.component_instance;
        let selected: Vec<_> = ci
            .placed_symbol_refs
            .iter()
            .filter(|r| symbols.contains(&r.object_id))
            .collect();
        if ci.uuid != id
            || !components.insert(id)
            || selected.is_empty()
            || !ci.placed_package_refs.is_empty()
            || selected.iter().any(|r| !owners.insert(r.object_id))
            || ci.library_bindings.values().any(|b| {
                !matches!(
                    b.binding_role,
                    crate::substrate::LibraryBindingRole::Part
                        | crate::substrate::LibraryBindingRole::Symbol
                ) || !b.local_override_refs.is_empty()
            })
        {
            return Err(invalid(
                "Deletion requires exact owned schematic-only members",
            ));
        }
        let expected = detached_component(raw, &symbols)?;
        if let Some(next) = next {
            if expected != *next
                || expected["placed_symbol_refs"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            {
                return Err(invalid("Deletion changes unrelated component payload"));
            }
        } else if !expected["placed_symbol_refs"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            return Err(invalid("Deletion would discard surviving component gates"));
        }
    }
    Ok(())
}
