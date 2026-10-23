//! Prospective CI version pins without weakening the original source preimage.
use super::*;
use crate::substrate::{
    ComponentInstance, ComponentInstanceAuthority, ComponentInstanceShard, ObjectRevision,
};

pub(super) fn final_bindings(
    model: &DesignModel,
    batch: &OperationBatch,
    kind: TransactionKind,
) -> Result<BTreeMap<Uuid, ComponentInstance>, EngineError> {
    let mut relevant = BTreeSet::new();
    for op in &batch.operations {
        let value = match op {
            Operation::SetBoardPackagePackage { materialized, .. }
            | Operation::CreateBoardPackage { materialized, .. } => materialized,
            _ => continue,
        };
        for pad in value
            .get("component_pads")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(id) = pad["source_binding"]["component_instance"]["object_id"]
                .as_str()
                .and_then(|v| v.parse::<Uuid>().ok())
            {
                relevant.insert(id);
            }
        }
    }
    let mut result = BTreeMap::new();
    if kind != TransactionKind::Normal {
        return Ok(result);
    }
    for id in relevant {
        let operations: Vec<_> = batch
            .operations
            .iter()
            .filter(|op| match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    ..
                }
                | Operation::SetComponentInstance {
                    component_instance_id,
                    ..
                }
                | Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } => *component_instance_id == id,
                _ => false,
            })
            .collect();
        if operations.is_empty() {
            continue;
        }
        if operations.len() != 1 {
            return Err(invalid(
                "prospective Pad binding requires one guarded existing CI refresh",
            ));
        }
        let Operation::SetComponentInstance {
            previous_component_instance,
            component_instance,
            ..
        } = operations[0]
        else {
            return Err(invalid(
                "prospective Pad binding requires an existing CI refresh",
            ));
        };
        if batch.operations.iter().any(|op| {
            !matches!(
                op,
                Operation::GuardObjectRevision { .. }
                    | Operation::SetBoardPackagePackage { .. }
                    | Operation::SetComponentInstance { .. }
                    | Operation::SetElectricalIdentity { .. }
            )
        }) {
            return Err(invalid(
                "unsupported operation mixed with a prospective component reference refresh",
            ));
        }
        let current = model
            .component_instances
            .get(&id)
            .filter(|ci| ci.authority == ComponentInstanceAuthority::Authored)
            .ok_or_else(|| invalid("prospective binding is not an authored current CI"))?;
        let object = model
            .objects
            .get(&id)
            .ok_or_else(|| invalid("missing current CI source object"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.shard_id == object.source_shard_id && s.kind == SourceShardKind::ComponentInstance
            })
            .ok_or_else(|| invalid("missing current CI shard"))?;
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let source: ComponentInstanceShard = serde_json::from_value(value.clone())?;
        let refs = &source.component_instance;
        let pin_current = |r: &crate::substrate::RevisionedRef| {
            model
                .objects
                .get(&r.object_id)
                .is_some_and(|o| o.object_revision == r.object_revision)
        };
        if source.schema_version != crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
            || refs.uuid != id
            || refs.object_revision != object.object_revision
            || refs.object_revision != current.object_revision
            || value["component_instance"] != *previous_component_instance
            || refs.placed_package_refs.len() != 1
            || !refs.placed_package_refs.iter().all(pin_current)
            || !refs.placed_symbol_refs.iter().all(pin_current)
            || !refs.part_ref.as_ref().is_none_or(pin_current)
            || !refs.library_bindings.values().all(|b| {
                model
                    .objects
                    .get(&b.target_object_id)
                    .is_some_and(|o| o.object_revision == b.pinned_object_revision)
                    && b.local_override_refs.iter().all(pin_current)
            })
        {
            return Err(invalid(
                "stale original CI preimage or source-reference closure",
            ));
        }
        // Commit admission has validated and stripped object guards. The exact
        // current source preimage above remains independently required here.
        let package = refs.placed_package_refs[0].object_id;
        let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
        if board["component_pads"]
            .as_object()
            .into_iter()
            .flat_map(|m| m.iter())
            .filter(|(key, _)| **key != package.to_string())
            .flat_map(|(_, v)| v.as_array().into_iter().flatten())
            .any(|p| {
                p["source_binding"]["component_instance"]["object_id"] == serde_json::json!(id)
            })
        {
            return Err(invalid(
                "prospective CI refresh has an unsupported other-package dependency",
            ));
        }
        let materializations = batch
            .operations
            .iter()
            .filter(|op| {
                matches!(op,
            Operation::SetBoardPackagePackage {package_id,..} if *package_id==package)
            })
            .count();
        if materializations != 1 {
            return Err(invalid(
                "CI refresh requires exactly one corresponding package transition",
            ));
        }
        let mut expected = previous_component_instance.clone();
        expected["object_revision"] = serde_json::to_value(ObjectRevision(
            refs.object_revision
                .0
                .checked_add(1)
                .ok_or_else(|| invalid("CI revision overflow"))?,
        ))?;
        expected["placed_package_refs"][0]["object_revision"] =
            serde_json::to_value(ObjectRevision(
                refs.placed_package_refs[0]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("package revision overflow"))?,
            ))?;
        if expected != *component_instance {
            return Err(invalid(
                "prospective CI operation changes more than validated reference versions",
            ));
        }
        let final_ci =
            crate::substrate::component_instance::persisted_component_instance_from_value(
                component_instance,
            )?;
        result.insert(id, final_ci);
    }
    if !result.is_empty() {
        let ci = *result.keys().next().unwrap();
        if result.len() != 1
            || batch
                .operations
                .iter()
                .filter(|op| matches!(op, Operation::SetComponentInstance { .. }))
                .count()
                != 1
            || batch
                .operations
                .iter()
                .filter(|op| matches!(op, Operation::SetBoardPackagePackage { .. }))
                .count()
                != 1
        {
            return Err(invalid(
                "prospective refresh requires exactly one CI and package",
            ));
        }
        let mut required = BTreeMap::new();
        for record in model.electrical_identities.values() {
            if let crate::substrate::ElectricalIdentity::NetRelationship {
                logical_net,
                evidence,
                ..
            } = &record.identity
                && evidence
                    .iter()
                    .any(|e| e.component_instance.object_id == ci)
            {
                if crate::substrate::net_correspondence_status(model, *logical_net)
                    != crate::substrate::NetCorrespondenceStatus::Complete
                {
                    return Err(invalid(
                        "prospective refresh cannot recertify unavailable Net evidence",
                    ));
                }
                required.insert(record.id, *logical_net);
            }
        }
        let mut supplied = BTreeSet::new();
        for op in &batch.operations {
            if let Operation::SetElectricalIdentity { previous, record } = op {
                if !required.contains_key(&record.id)
                    || !supplied.insert(record.id)
                    || model.electrical_identities.get(&record.id) != Some(previous.as_ref())
                    || electrical_semantics(previous, ci) != electrical_semantics(record, ci)
                {
                    return Err(invalid(
                        "changed, duplicate or foreign prospective Net dependency",
                    ));
                }
            }
        }
        if supplied != required.keys().copied().collect() {
            return Err(invalid("missing prospective Net dependency"));
        }
        let mut prospective = model.clone();
        prospective.commit_without_direct_policy(batch.clone())?;
        let basis = crate::substrate::electrical_basis::ElectricalBasis::new(
            &prospective,
            &batch.operations,
        )?;
        let membership =
            crate::substrate::electrical_net_membership::index(&prospective, &batch.operations)?;
        for net in required.values() {
            if crate::substrate::electrical_identity_validation::correspondence_with_basis(
                &prospective,
                *net,
                Some(&basis),
                Some(&membership),
            ) != crate::substrate::NetCorrespondenceStatus::Complete
            {
                return Err(invalid("prospective Net dependency graph is incomplete"));
            }
        }
    }
    Ok(result)
}

fn electrical_semantics(record: &crate::substrate::ElectricalIdentityRecord, ci: Uuid) -> Value {
    let mut value = serde_json::to_value(record).expect("typed electrical record");
    value["object_revision"] = Value::Null;
    if let Some(evidence) = value["identity"]["evidence"].as_array_mut() {
        for e in evidence {
            if e["component_instance"]["object_id"] == serde_json::json!(ci) {
                e["component_instance"]["object_revision"] = Value::Null;
                e["board_pad"]["object_revision"] = Value::Null;
            }
        }
    }
    value
}
