//! Pure PM057 D2-H1 link proof; no filesystem reads or revision equivalence guess.
use super::*;
use crate::substrate::{ElectricalIdentity, ElectricalIdentityRecord, ObjectRevision, Operation};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) fn matches(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    validate(tx, target, prior).is_some()
}
fn pads(value: &Value) -> Option<BTreeSet<Uuid>> {
    let mut result = BTreeSet::new();
    if let Some(values) = value.get("component_pads") {
        for value in values.as_array()? {
            if !result.insert(serde_json::from_value(value["uuid"].clone()).ok()?) {
                return None;
            }
        }
    }
    Some(result)
}
fn plus(value: u64) -> Option<u64> {
    value.checked_add(1)
}
fn latest_ci(prior: &[TransactionRecord], id: Uuid) -> Option<Value> {
    let mut result = None;
    for op in prior.iter().flat_map(|t| &t.operations) {
        match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                component_instance,
                ..
            } if *component_instance_id == id => result = Some(component_instance.clone()),
            Operation::DeleteComponentInstance {
                component_instance_id,
                ..
            } if *component_instance_id == id => result = None,
            _ => {}
        }
    }
    result
}
fn source_payload(value: &Value) -> Option<Value> {
    let mut value = value.clone();
    value.as_object_mut()?.remove("pad_source_transition");
    Some(value)
}
fn latest_package(prior: &[TransactionRecord], id: Uuid) -> Option<(Uuid, Value)> {
    let mut result = None;
    for op in prior.iter().flat_map(|t| &t.operations) {
        match op {
            Operation::CreateBoardPackage {
                package_id,
                package,
                materialized,
            } if *package_id == id => {
                result = Some((
                    serde_json::from_value(package["package"].clone()).ok()?,
                    source_payload(materialized)?,
                ))
            }
            Operation::SetBoardPackagePackage {
                package_id,
                package_ref_id,
                materialized,
                ..
            } if *package_id == id => {
                result = Some((*package_ref_id, source_payload(materialized)?))
            }
            Operation::DeleteBoardPackage { package_id, .. } if *package_id == id => result = None,
            _ => {}
        }
    }
    result
}
fn ci_semantics(value: &Value) -> Value {
    let mut value = value.clone();
    value["object_revision"] = Value::Null;
    if let Some(refs) = value["placed_package_refs"].as_array_mut() {
        for reference in refs {
            reference["object_revision"] = Value::Null;
        }
    }
    value
}
fn net_semantics(record: &ElectricalIdentityRecord, ci: Uuid) -> Option<Value> {
    let mut value = serde_json::to_value(record).ok()?;
    value["object_revision"] = Value::Null;
    for e in value["identity"]["evidence"].as_array_mut()? {
        if e["component_instance"]["object_id"] == json!(ci) {
            e["component_instance"]["object_revision"] = Value::Null;
            e["board_pad"]["object_revision"] = Value::Null;
        }
    }
    Some(value)
}
fn package_epoch(prior: &[TransactionRecord], id: Uuid) -> Option<u64> {
    let mut epoch = None;
    let mut born = false;
    for tx in prior {
        let mut actions = Vec::new();
        for op in &tx.operations {
            let action = match op {
                Operation::CreateBoardPackage { package_id, .. } if *package_id == id => Some(0),
                Operation::DeleteBoardPackage { package_id, .. } if *package_id == id => Some(2),
                Operation::SetBoardPackagePart { package_id, .. }
                | Operation::SetBoardPackagePackage { package_id, .. }
                | Operation::SetBoardPackageValue { package_id, .. }
                | Operation::SetBoardPackageReference { package_id, .. }
                | Operation::SetBoardPackagePosition { package_id, .. }
                | Operation::SetBoardPackageLayer { package_id, .. }
                | Operation::SetComponentSide { package_id, .. }
                | Operation::SetBoardPackageRotation { package_id, .. }
                | Operation::SetBoardPackageLocked { package_id, .. }
                    if *package_id == id =>
                {
                    Some(1)
                }
                Operation::BumpObjectRevision { object_id } if *object_id == id => Some(1),
                _ => None,
            };
            if let Some(action) = action {
                actions.push(action);
            }
        }
        if actions.len() > 1 {
            return None;
        }
        let action = actions.first().copied();
        for (kind, ids) in [
            (0, &tx.diff.created),
            (1, &tx.diff.modified),
            (2, &tx.diff.deleted),
        ] {
            if ids.iter().filter(|x| **x == id).count() != usize::from(action == Some(kind)) {
                return None;
            }
        }
        match action {
            Some(0) => {
                if epoch.is_some() || (born && tx.transaction_kind == TransactionKind::Normal) {
                    return None;
                }
                epoch = Some(0);
                born = true;
            }
            Some(1) => epoch = Some(plus(epoch?)?),
            Some(2) => {
                epoch?;
                epoch = None;
            }
            None => {}
            _ => return None,
        }
    }
    epoch
}
// Independently derive Pad registration epochs from recorded typed producers.
// The candidate cannot invent a birth or count by changing its diff metadata.
fn pad_epochs(prior: &[TransactionRecord], needed: &BTreeSet<Uuid>) -> Option<BTreeMap<Uuid, u64>> {
    let mut epochs = BTreeMap::new();
    let mut born = BTreeSet::new();
    for tx in prior {
        let mut changes = BTreeMap::new();
        for op in &tx.operations {
            let (old, new) = match op {
                Operation::CreateBoardPackage { materialized, .. } => {
                    (BTreeSet::new(), pads(materialized)?)
                }
                Operation::DeleteBoardPackage { materialized, .. } => {
                    (pads(materialized)?, BTreeSet::new())
                }
                Operation::SetBoardPackagePackage {
                    previous_materialized,
                    materialized,
                    ..
                } => (pads(previous_materialized)?, pads(materialized)?),
                Operation::BumpObjectRevision { object_id } if needed.contains(object_id) => {
                    if changes.insert(*object_id, 1).is_some() {
                        return None;
                    }
                    let revision = plus(*epochs.get(object_id)?)?;
                    epochs.insert(*object_id, revision);
                    continue;
                }
                _ => continue,
            };
            for id in old.union(&new).filter(|id| needed.contains(id)) {
                let change = match (old.contains(id), new.contains(id)) {
                    (false, true) => {
                        if epochs.contains_key(id)
                            || (born.contains(id) && tx.transaction_kind == TransactionKind::Normal)
                        {
                            return None;
                        }
                        epochs.insert(*id, 0);
                        born.insert(*id);
                        0
                    }
                    (true, true) => {
                        let revision = plus(*epochs.get(id)?)?;
                        epochs.insert(*id, revision);
                        1
                    }
                    (true, false) => {
                        epochs.remove(id)?;
                        2
                    }
                    _ => return None,
                };
                if changes.insert(*id, change).is_some() {
                    return None;
                }
            }
        }
        for (kind, ids) in [
            (0, &tx.diff.created),
            (1, &tx.diff.modified),
            (2, &tx.diff.deleted),
        ] {
            let recorded: Vec<_> = ids
                .iter()
                .filter(|id| needed.contains(id))
                .copied()
                .collect();
            let actual: BTreeSet<_> = changes
                .iter()
                .filter(|(_, v)| **v == kind)
                .map(|(id, _)| *id)
                .collect();
            if recorded.len() != actual.len()
                || recorded.into_iter().collect::<BTreeSet<_>>() != actual
            {
                return None;
            }
        }
    }
    Some(epochs)
}
fn validate(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Option<()> {
    if prior.last()?.transaction_id != target.transaction_id {
        return None;
    }
    let mut operations = target.inverse_operations.clone();
    let mut component = None;
    let mut placement = None;
    for op in &operations {
        match op {
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => {
                if component
                    .replace((
                        *component_instance_id,
                        previous_component_instance.clone(),
                        component_instance.clone(),
                    ))
                    .is_some()
                {
                    return None;
                }
            }
            Operation::SetBoardPackagePackage {
                package_id,
                package_ref_id,
                previous_materialized,
                materialized,
                ..
            } => {
                if placement
                    .replace((
                        *package_id,
                        *package_ref_id,
                        previous_materialized.clone(),
                        materialized.clone(),
                    ))
                    .is_some()
                {
                    return None;
                }
            }
            Operation::SetElectricalIdentity { .. } => {}
            _ => return None,
        }
    }
    let (ci, current_ci, desired_ci) = component?;
    let (package, package_ref, old_pads, new_pads) = placement?;
    let before_target = &prior[..prior.len().checked_sub(1)?];
    if latest_ci(prior, ci)? != current_ci
        || latest_ci(before_target, ci)? != desired_ci
        || latest_package(prior, package)? != (package_ref, source_payload(&old_pads)?)
        || latest_package(before_target, package)? != (package_ref, source_payload(&new_pads)?)
    {
        return None;
    }
    if ci_semantics(&current_ci) != ci_semantics(&desired_ci)
        || current_ci["uuid"] != json!(ci)
        || current_ci["placed_package_refs"].as_array()?.len() != 1
        || current_ci["placed_package_refs"][0]["object_id"] != json!(package)
    {
        return None;
    }
    let ci_revision = plus(current_ci["object_revision"].as_u64()?)?;
    if current_ci["placed_package_refs"][0]["object_revision"].as_u64()?
        != package_epoch(prior, package)?
        || desired_ci["placed_package_refs"][0]["object_revision"].as_u64()?
            != package_epoch(before_target, package)?
    {
        return None;
    }
    let package_revision = plus(current_ci["placed_package_refs"][0]["object_revision"].as_u64()?)?;
    let original_desired_revision = desired_ci["object_revision"].as_u64()?;
    let old_ids = pads(&old_pads)?;
    let new_ids = pads(&new_pads)?;
    let needed = old_ids.union(&new_ids).copied().collect();
    let epochs = pad_epochs(prior, &needed)?;
    let earlier_epochs = pad_epochs(before_target, &needed)?;
    if old_ids.iter().any(|id| !epochs.contains_key(id)) {
        return None;
    }
    let final_epochs: BTreeMap<_, _> = new_ids
        .iter()
        .map(|id| {
            let revision = if old_ids.contains(id) {
                plus(*epochs.get(id)?)?
            } else {
                if epochs.contains_key(id) {
                    return None;
                }
                0
            };
            Some((*id, revision))
        })
        .collect::<Option<_>>()?;
    let mut latest_ci = None;
    let mut records = BTreeMap::new();
    let mut advances: BTreeMap<Uuid, u64> = BTreeMap::new();
    for prior_tx in prior {
        let mut writes = BTreeSet::new();
        for op in &prior_tx.operations {
            match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                }
                | Operation::SetComponentInstance {
                    component_instance_id,
                    component_instance,
                    ..
                } if *component_instance_id == ci => latest_ci = Some(component_instance),
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } if *component_instance_id == ci => latest_ci = None,
                _ => {}
            }
            if let Some((id, record, delete)) =
                crate::substrate::electrical_identity_store::write(op)
            {
                if !writes.insert(id) {
                    return None;
                }
                let existed = records.contains_key(&id);
                if delete {
                    records.remove(&id)?;
                } else {
                    if existed {
                        if !prior_tx.diff.modified.contains(&id) {
                            return None;
                        }
                        let count = advances.entry(id).or_default();
                        *count = plus(*count)?;
                    } else if !prior_tx.diff.created.contains(&id) {
                        return None;
                    }
                    records.insert(id, record.clone());
                }
            }
        }
    }
    if latest_ci? != &current_ci {
        return None;
    }
    let required: BTreeSet<_> = records
        .values()
        .filter_map(|r| {
            if let ElectricalIdentity::NetRelationship { evidence, .. } = &r.identity {
                evidence
                    .iter()
                    .any(|e| e.component_instance.object_id == ci)
                    .then_some(r.id)
            } else {
                None
            }
        })
        .collect();
    for id in &required {
        for prior_tx in prior {
            let set_count = prior_tx
                .operations
                .iter()
                .filter(
                    |op| matches!(op,Operation::SetElectricalIdentity{record,..} if record.id==*id),
                )
                .count();
            if set_count > 1
                || prior_tx.diff.modified.iter().filter(|x| **x == *id).count() != set_count
            {
                return None;
            }
        }
    }
    let mut supplied = BTreeSet::new();
    for op in &mut operations {
        match op {
            Operation::SetComponentInstance {
                component_instance, ..
            } => {
                component_instance["object_revision"] = json!(ci_revision);
                component_instance["placed_package_refs"][0]["object_revision"] =
                    json!(package_revision);
            }
            Operation::SetBoardPackagePackage { materialized, .. } => {
                if let Some(values) = materialized.get_mut("component_pads") {
                    for pad in values.as_array_mut()? {
                        if pad["source_binding"]["component_instance"]["object_id"] != json!(ci)
                            || pad["source_binding"]["component_instance"]["object_revision"]
                                != json!(original_desired_revision)
                        {
                            return None;
                        }
                        pad["source_binding"]["component_instance"]["object_revision"] =
                            json!(ci_revision);
                    }
                }
            }
            Operation::SetElectricalIdentity { previous, record } => {
                if !required.contains(&record.id)
                    || !supplied.insert(record.id)
                    || records.get(&record.id) != Some(previous.as_ref())
                    || net_semantics(previous, ci)? != net_semantics(record, ci)?
                {
                    return None;
                }
                let before_record = before_target
                    .iter()
                    .flat_map(|t| &t.operations)
                    .filter_map(crate::substrate::electrical_identity_store::write)
                    .filter(|(id, _, _)| *id == record.id)
                    .last()?;
                if before_record.2 || before_record.1 != record {
                    return None;
                }
                let ElectricalIdentity::NetRelationship {
                    evidence: current_evidence,
                    ..
                } = &previous.identity
                else {
                    return None;
                };
                for e in current_evidence
                    .iter()
                    .filter(|e| e.component_instance.object_id == ci)
                {
                    if e.component_instance.object_revision.0
                        != current_ci["object_revision"].as_u64()?
                        || e.board_pad.object_revision.0 != *epochs.get(&e.board_pad.object_id)?
                    {
                        return None;
                    }
                }
                let durable_revision = previous
                    .object_revision
                    .0
                    .checked_add(*advances.get(&record.id).unwrap_or(&0))?;
                record.object_revision = ObjectRevision(plus(durable_revision)?);
                let ElectricalIdentity::NetRelationship { evidence, .. } = &mut record.identity
                else {
                    return None;
                };
                for e in evidence
                    .iter_mut()
                    .filter(|e| e.component_instance.object_id == ci)
                {
                    if e.component_instance.object_revision.0 != original_desired_revision {
                        return None;
                    }
                    if e.board_pad.object_revision.0
                        != *earlier_epochs.get(&e.board_pad.object_id)?
                    {
                        return None;
                    }
                    e.component_instance.object_revision = ObjectRevision(ci_revision);
                    e.board_pad.object_revision =
                        ObjectRevision(*final_epochs.get(&e.board_pad.object_id)?);
                }
            }
            _ => return None,
        }
    }
    if supplied != required || tx.operations != operations {
        return None;
    }
    let created: BTreeSet<_> = new_ids.difference(&old_ids).copied().collect();
    let deleted: BTreeSet<_> = old_ids.difference(&new_ids).copied().collect();
    let modified: BTreeSet<_> = old_ids
        .intersection(&new_ids)
        .copied()
        .chain([ci, package])
        .chain(required)
        .collect();
    for (actual, expected) in [
        (&tx.diff.created, created),
        (&tx.diff.deleted, deleted),
        (&tx.diff.modified, modified),
    ] {
        if actual.len() != expected.len()
            || actual.iter().copied().collect::<BTreeSet<_>>() != expected
        {
            return None;
        }
    }
    // Match the existing journal hooks: board inverse first, non-core inverses
    // in operation order, then reverse. Only its ephemeral transition is omitted.
    let mut inverse = Vec::new();
    for op in &operations {
        if let Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized,
            materialized,
        } = op
        {
            let mut restored = previous_materialized.clone();
            restored.as_object_mut()?.remove("pad_source_transition");
            inverse.push(Operation::SetBoardPackagePackage {
                package_id: *package_id,
                package_ref_id: *package_ref_id,
                previous_materialized: materialized.clone(),
                materialized: restored,
            });
        }
    }
    for op in &operations {
        match op {
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => inverse.push(Operation::SetComponentInstance {
                component_instance_id: *component_instance_id,
                previous_component_instance: component_instance.clone(),
                component_instance: previous_component_instance.clone(),
            }),
            Operation::SetElectricalIdentity { previous, record } => {
                inverse.push(Operation::SetElectricalIdentity {
                    previous: Box::new(record.clone()),
                    record: previous.as_ref().clone(),
                })
            }
            _ => {}
        }
    }
    inverse.reverse();
    (tx.inverse_operations == inverse).then_some(())
}
