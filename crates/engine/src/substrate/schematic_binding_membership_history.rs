//! Exact recorded source birth/removal paired with schematic-only membership.
//! Only epochs are rebased; every raw semantic complement remains immutable.
use super::*;
pub(super) fn refs(raw: &Value) -> Option<BTreeMap<Uuid, u64>> {
    if !raw["placed_package_refs"].as_array()?.is_empty() {
        return None;
    }
    let mut refs = BTreeMap::new();
    for r in raw["placed_symbol_refs"].as_array()? {
        let id = serde_json::from_value(r["object_id"].clone()).ok()?;
        if refs.insert(id, r["object_revision"].as_u64()?).is_some() {
            return None;
        }
    }
    (!refs.is_empty()).then_some(refs)
}
fn component_allowed(id: Uuid, raw: &Value) -> Option<()> {
    let parsed: crate::substrate::ComponentInstanceShard =
        serde_json::from_value(json!({"schema_version":1,"component_instance":raw})).ok()?;
    if parsed.component_instance.uuid != id
        || parsed
            .component_instance
            .library_bindings
            .values()
            .any(|b| {
                !matches!(
                    b.binding_role,
                    crate::substrate::LibraryBindingRole::Part
                        | crate::substrate::LibraryBindingRole::Symbol
                ) || !b.local_override_refs.is_empty()
            })
    {
        return None;
    }
    refs(raw)?;
    Some(())
}
fn changed_members(
    old: &Value,
    next: &Value,
    symbols: &BTreeMap<Uuid, bool>,
) -> Option<BTreeSet<Uuid>> {
    let before = refs(old)?;
    let after = refs(next)?;
    let removed: BTreeSet<_> = before
        .keys()
        .filter(|id| !after.contains_key(id))
        .copied()
        .collect();
    let added: BTreeSet<_> = after
        .keys()
        .filter(|id| !before.contains_key(id))
        .copied()
        .collect();
    if removed.is_empty() == added.is_empty() {
        return None;
    }
    if removed.iter().any(|id| symbols.get(id) != Some(&false))
        || added.iter().any(|id| symbols.get(id) != Some(&true))
    {
        return None;
    }
    let mut expected = old.clone();
    expected["object_revision"] = next["object_revision"].clone();
    expected["placed_symbol_refs"]
        .as_array_mut()?
        .retain(|r| !removed.iter().any(|id| r["object_id"] == json!(id)));
    // Appends have one exact new reference; restoration of removed members can
    // occur in their original positions. Surviving raw members are never rewritten.
    let surviving: Vec<_> = next["placed_symbol_refs"]
        .as_array()?
        .iter()
        .filter(|r| !added.iter().any(|id| r["object_id"] == json!(id)))
        .cloned()
        .collect();
    if expected["placed_symbol_refs"] != json!(surviving) {
        return None;
    }
    expected["placed_symbol_refs"] = next["placed_symbol_refs"].clone();
    for id in &removed {
        if let Some(roles) = expected.get_mut("placed_symbol_roles") {
            roles.as_object_mut()?.remove(&id.to_string());
        }
    }
    for id in &added {
        let role = next
            .get("placed_symbol_roles")
            .and_then(|r| r.get(id.to_string()));
        if let Some(role) = role {
            if expected.get("placed_symbol_roles").is_none() {
                expected["placed_symbol_roles"] = json!({});
            }
            expected["placed_symbol_roles"][id.to_string()] = role.clone();
        }
    }
    // Optional empty role map may have been introduced by the ordinary append.
    if expected
        .get("placed_symbol_roles")
        .is_some_and(|v| v.as_object().is_some_and(|v| v.is_empty()))
        && next.get("placed_symbol_roles").is_none()
    {
        expected.as_object_mut()?.remove("placed_symbol_roles");
    }
    (expected == *next).then(|| removed.union(&added).copied().collect())
}
pub(super) fn capture(target: &TransactionRecord) -> Option<BTreeSet<Uuid>> {
    if inverse(&target.operations).as_ref() != Some(&target.inverse_operations) {
        return None;
    }
    let mut symbols = BTreeMap::new();
    let mut sheet = None;
    let mut direction = None;
    let mut ids = BTreeSet::new();
    for op in &target.operations {
        if let Some((owner, id, class, _, create)) = source(op) {
            if sheet.replace(owner).is_some_and(|old| old != owner)
                || !ids.insert(id)
                || direction.replace(create).is_some_and(|old| old != create)
            {
                return None;
            }
            if class == "symbols" {
                symbols.insert(id, create);
            }
        }
    }
    let mut members = BTreeSet::new();
    let mut owned = BTreeSet::new();
    for op in &target.operations {
        let (id, raw, affected) = match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            }
            | Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => {
                let create = matches!(op, Operation::CreateComponentInstance { .. });
                let refs = refs(component_instance)?;
                if refs.keys().any(|s| symbols.get(s) != Some(&create))
                    || (create
                        && target.transaction_kind == crate::substrate::TransactionKind::Normal)
                {
                    return None;
                }
                (
                    *component_instance_id,
                    component_instance,
                    refs.keys().copied().collect(),
                )
            }
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => {
                component_allowed(*component_instance_id, previous_component_instance)?;
                let changed =
                    changed_members(previous_component_instance, component_instance, &symbols)?;
                (*component_instance_id, component_instance, changed)
            }
            _ => continue,
        };
        component_allowed(id, raw)?;
        if !members.insert(id) || affected.into_iter().any(|s| !owned.insert(s)) {
            return None;
        }
    }
    if direction == Some(true)
        && target.transaction_kind == crate::substrate::TransactionKind::Normal
        && (symbols.len() != 1 || owned.len() != 1 || ids.len() != 1)
    {
        return None;
    }
    (!members.is_empty()).then_some(members)
}
pub(super) fn state(prior: &[TransactionRecord], id: Uuid) -> Option<Value> {
    let mut current = None;
    for tx in prior {
        for op in &tx.operations {
            match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                }
                | Operation::SetComponentInstance {
                    component_instance_id,
                    component_instance,
                    ..
                } if *component_instance_id == id => current = Some(component_instance.clone()),
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } if *component_instance_id == id => current = None,
                _ => {}
            }
        }
    }
    current
}
fn semantics(raw: &Value) -> Option<Value> {
    refs(raw)?;
    let mut out = raw.clone();
    out["object_revision"] = Value::Null;
    for r in out["placed_symbol_refs"].as_array_mut()? {
        r["object_revision"] = Value::Null;
    }
    Some(out)
}
pub(super) fn normalized(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Option<Vec<Operation>> {
    capture(target)?;
    let index = prior
        .iter()
        .position(|t| t.transaction_id == target.transaction_id)?;
    let mut ops = target.inverse_operations.clone();
    let births: BTreeMap<_, _> = ops
        .iter()
        .filter_map(|op| match op {
            Operation::CreateSchematicSymbol {
                symbol_id, symbol, ..
            } => Some((
                *symbol_id,
                symbol
                    .get("object_revision")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            )),
            _ => None,
        })
        .collect();
    for op in &mut ops {
        let Some(id) = super::super::family::component_id(op) else {
            continue;
        };
        let historical = state(&prior[..=index], id);
        let current = state(prior, id);
        match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            } => {
                if current.is_some() || historical.is_some() {
                    return None;
                }
                component_instance["object_revision"] = json!(0);
                for r in component_instance["placed_symbol_refs"].as_array_mut()? {
                    let symbol: Uuid = serde_json::from_value(r["object_id"].clone()).ok()?;
                    r["object_revision"] = json!(births.get(&symbol)?);
                }
            }
            Operation::SetComponentInstance {
                previous_component_instance,
                component_instance,
                ..
            } => {
                let current = current.as_ref()?;
                if historical.as_ref() != Some(&*previous_component_instance)
                    || semantics(previous_component_instance)? != semantics(current)?
                {
                    return None;
                }
                *previous_component_instance = current.clone();
                component_instance["object_revision"] =
                    json!(current["object_revision"].as_u64()?.checked_add(1)?);
                let pins = refs(current)?;
                for r in component_instance["placed_symbol_refs"].as_array_mut()? {
                    let symbol: Uuid = serde_json::from_value(r["object_id"].clone()).ok()?;
                    r["object_revision"] =
                        json!(births.get(&symbol).or_else(|| pins.get(&symbol))?);
                }
            }
            Operation::DeleteComponentInstance {
                component_instance, ..
            } => {
                let current = current.as_ref()?;
                if historical.as_ref() != Some(&*component_instance)
                    || semantics(component_instance)? != semantics(current)?
                {
                    return None;
                }
                *component_instance = current.clone();
            }
            _ => {}
        }
    }
    Some(ops)
}
