//! Guarded current-reference closure for one unchanged authored binding.
use super::*;
use crate::substrate::{
    ComponentInstanceAuthority, ComponentInstanceShard, ElectricalIdentity,
    NetCorrespondenceStatus, ObjectRevision, RevisionedRef, net_correspondence_status,
};

fn next(value: ObjectRevision) -> Result<ObjectRevision, EngineError> {
    value
        .0
        .checked_add(1)
        .map(ObjectRevision)
        .ok_or_else(|| invalid("binding revision overflow"))
}

pub(super) fn refresh(
    model: &DesignModel,
    request: &ComponentPadMaterializationRequest,
    board: &serde_json::Value,
    pads: &mut [ComponentPadSource],
) -> Result<Vec<Operation>, EngineError> {
    let Some(id) = request.component_instance else {
        return Ok(vec![]);
    };
    let owner = model
        .component_instances
        .get(&id)
        .ok_or_else(|| invalid("missing authored ComponentInstance"))?;
    if owner.authority != ComponentInstanceAuthority::Authored
        || owner.placed_package_refs != vec![request.placement.uuid]
        || owner.part_ref != Some(request.placement.part)
    {
        return Err(invalid(
            "reference refresh requires one unchanged authored package binding",
        ));
    }
    let object = model
        .objects
        .get(&id)
        .ok_or_else(|| invalid("missing ComponentInstance source"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.shard_id == object.source_shard_id && s.kind == SourceShardKind::ComponentInstance
        })
        .ok_or_else(|| invalid("missing authored binding shard"))?;
    let source = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    let parsed: ComponentInstanceShard = serde_json::from_value(source.clone())?;
    let ci = parsed.component_instance;
    let current = |reference: &RevisionedRef| {
        model
            .objects
            .get(&reference.object_id)
            .is_some_and(|o| o.object_revision == reference.object_revision)
    };
    if parsed.schema_version != crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
        || ci.uuid != id
        || ci.object_revision != object.object_revision
        || ci.object_revision != owner.object_revision
        || ci.placed_package_refs.len() != 1
        || ci.placed_package_refs[0].object_id != request.placement.uuid
        || !ci.placed_symbol_refs.iter().all(current)
        || !ci.placed_package_refs.iter().all(current)
        || !ci.part_ref.as_ref().is_none_or(current)
        || !ci.library_bindings.values().all(|b| {
            model
                .objects
                .get(&b.target_object_id)
                .is_some_and(|o| o.object_revision == b.pinned_object_revision)
                && b.local_override_refs.iter().all(current)
        })
    {
        return Err(invalid(
            "cannot recertify stale component source or reference pins",
        ));
    }
    // Advancing this CI must not strand another package's Pad bindings.
    if board["component_pads"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.iter())
        .filter(|(key, _)| **key != request.placement.uuid.to_string())
        .flat_map(|(_, v)| v.as_array().into_iter().flatten())
        .any(|p| p["source_binding"]["component_instance"]["object_id"] == serde_json::json!(id))
    {
        return Err(invalid(
            "component reference refresh has another package dependency",
        ));
    }
    let post_ci = RevisionedRef {
        object_id: id,
        object_revision: next(ci.object_revision)?,
    };
    let previous = source["component_instance"].clone();
    let mut updated = previous.clone();
    updated["object_revision"] = serde_json::to_value(post_ci.object_revision)?;
    let package_revision = next(model.objects[&request.placement.uuid].object_revision)?;
    for reference in updated["placed_package_refs"]
        .as_array_mut()
        .ok_or_else(|| invalid("missing package references"))?
    {
        reference["object_revision"] = serde_json::to_value(package_revision)?;
    }
    for pad in pads.iter_mut() {
        let binding = pad
            .source_binding
            .as_mut()
            .ok_or_else(|| invalid("missing new Pad binding"))?;
        if binding
            .component_instance
            .as_ref()
            .is_none_or(|r| r.object_id != id || r.object_revision != ci.object_revision)
        {
            return Err(invalid(
                "new materialization targets a different component binding",
            ));
        }
        binding.component_instance = Some(post_ci.clone());
    }
    let mut operations = vec![Operation::SetComponentInstance {
        component_instance_id: id,
        previous_component_instance: previous,
        component_instance: updated,
    }];
    // Only complete, unchanged evidence can advance its source version pins.
    // A retained Pad requires explicit continuity; names never recover a target.
    for record in model.electrical_identities.values() {
        let ElectricalIdentity::NetRelationship {
            logical_net,
            evidence,
            ..
        } = &record.identity
        else {
            continue;
        };
        if !evidence
            .iter()
            .any(|e| e.component_instance.object_id == id)
        {
            continue;
        }
        if net_correspondence_status(model, *logical_net) != NetCorrespondenceStatus::Complete {
            return Err(invalid(
                "cannot recertify an unavailable electrical relationship",
            ));
        }
        let mut updated = record.clone();
        updated.object_revision = next(record.object_revision)?;
        let ElectricalIdentity::NetRelationship { evidence, .. } = &mut updated.identity else {
            unreachable!();
        };
        for evidence in evidence
            .iter_mut()
            .filter(|e| e.component_instance.object_id == id)
        {
            if evidence.component_instance.object_revision != ci.object_revision
                || !current(&evidence.board_pad)
            {
                return Err(invalid("stale preexisting terminal correspondence"));
            }
            let pad = pads
                .iter()
                .find(|p| p.uuid == evidence.board_pad.object_id)
                .ok_or_else(|| {
                    invalid("electrically bound Pad has no explicit retained identity")
                })?;
            let binding = pad.source_binding.as_ref().unwrap();
            if evidence.library_pad.as_ref().is_none_or(|r| {
                r.object_id != binding.library_pad.object_id
                    || r.object_revision != binding.library_pad.object_revision
            }) || binding.pin_pad_map.as_ref().is_none_or(|p| {
                p.object_id != evidence.pin_pad_map.object_id
                    || p.object_revision != evidence.pin_pad_map.object_revision
            }) {
                return Err(invalid(
                    "reference refresh changes electrical mapping semantics",
                ));
            }
            evidence.component_instance = post_ci.clone();
            evidence.board_pad.object_revision = next(evidence.board_pad.object_revision)?;
        }
        operations.push(Operation::SetElectricalIdentity {
            previous: Box::new(record.clone()),
            record: updated,
        });
    }
    Ok(operations)
}
