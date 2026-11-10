//! Shared saved-definition source identity and pinned revision collision checks.
use super::*;

pub(super) fn load(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    kind: &str,
) -> Result<(PoolLibraryObjectTarget, Value), EngineError> {
    let target = PoolLibraryObjectTarget::new("pool", kind, id);
    let (path, object) = reference(model, id, kind)?;
    if path != target.relative_path {
        return Err(refusal(
            "Saved definition is not owned by the canonical project pool path",
        ));
    }
    Ok((target, object))
}

// Reading a qualified reference does not grant ownership of its write path.
// One existing source scan serves both reference reads and canonical saved edits.
pub(super) fn reference(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    kind: &str,
) -> Result<(String, Value), EngineError> {
    let mut candidates = Vec::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        if let Ok(raw) =
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)
            && raw
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<Uuid>().ok())
                == Some(id)
        {
            candidates.push((shard, raw));
        }
    }
    if candidates.len() != 1 {
        return Err(refusal(
            "Saved definition identity has missing or ambiguous source candidates",
        ));
    }
    let source = model
        .objects
        .get(&id)
        .filter(|o| o.domain == "pool" && o.kind == kind)
        .ok_or_else(|| refusal("Saved definition has no matching canonical source"))?;
    let (shard, object) = candidates.pop().unwrap();
    if source.source_shard_id != shard.shard_id {
        return Err(refusal(
            "Saved definition is not owned by the canonical project pool path",
        ));
    }
    Ok((shard.relative_path.clone(), object))
}

pub(super) fn preserve_pinned_revision(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    kind: &str,
    replacement: &Value,
) -> Result<(), EngineError> {
    let role = match kind {
        "parts" => crate::substrate::LibraryBindingRole::Part,
        "packages" => crate::substrate::LibraryBindingRole::Package,
        "pin_pad_maps" => crate::substrate::LibraryBindingRole::PinPadMap,
        _ => {
            return Err(refusal(
                "Pinned revision guard requires a Part, Package or PinPadMap",
            ));
        }
    };
    let next = model.objects[&id]
        .object_revision
        .0
        .checked_add(1)
        .ok_or_else(|| refusal("Definition revision counter is exhausted"))?;
    let mut pinned = model
        .component_instances
        .values()
        .flat_map(|ci| ci.library_bindings.values())
        .any(|b| {
            b.binding_role == role
                && ((b.target_object_id == id && b.pinned_object_revision.0 == next)
                    || b.local_override_refs
                        .iter()
                        .any(|r| r.object_id == id && r.object_revision.0 == next))
        });
    // Frozen BoardRoot Package pins remain users even without a CI binding.
    if kind == "packages" {
        for shard in model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::BoardRoot)
        {
            let board =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            if let Some(placements) = board.get("component_pads").and_then(Value::as_object) {
                for pad in placements.values().filter_map(Value::as_array).flatten() {
                    for field in ["package", "land_pattern"] {
                        if let Some(pin) = pad
                            .get("source_binding")
                            .and_then(|b| b.get(field))
                            .filter(|p| !p.is_null())
                        {
                            let pin: crate::substrate::component_pad_source::PinnedPadLibrarySource = serde_json::from_value(pin.clone())?;
                            pinned |= pin.object_id == id
                                && pin.owner_id == id
                                && pin.object_revision.0 == next;
                        }
                    }
                }
            }
        }
    }
    // Legacy authored CIs can declare only part_ref. Its revision is retained in
    // the canonical shard, while the resolved CI carries only the Part UUID.
    for ci in model.component_instances.values().filter(|ci| {
        kind == "parts"
            && ci.authority == crate::substrate::ComponentInstanceAuthority::Authored
            && ci.part_ref == Some(id)
            && !ci
                .library_bindings
                .values()
                .any(|b| b.binding_role == crate::substrate::LibraryBindingRole::Part)
    }) {
        let owner = model
            .objects
            .get(&ci.id)
            .filter(|o| o.domain == "component_instance" && o.kind == "component_instance")
            .ok_or_else(|| refusal("Declared Part reference has no canonical CI owner"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.shard_id == owner.source_shard_id && s.kind == SourceShardKind::ComponentInstance
            })
            .ok_or_else(|| refusal("Declared Part reference source is unavailable"))?;
        let source: crate::substrate::ComponentInstanceShard = serde_json::from_value(
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?,
        )?;
        if source.schema_version != crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
            || source.component_instance.uuid != ci.id
            || source.component_instance.object_revision != owner.object_revision
        {
            return Err(refusal("Declared Part reference source identity differs"));
        }
        pinned |= source
            .component_instance
            .part_ref
            .is_some_and(|r| r.object_id == id && r.object_revision.0 == next);
    }
    if pinned
        && &model.materialized_pool_object_revision(
            id,
            kind,
            crate::substrate::ObjectRevision(next),
        )? != replacement
    {
        return Err(refusal(
            "Edit would make an existing pinned definition revision ambiguous after restored history",
        ));
    }
    Ok(())
}
