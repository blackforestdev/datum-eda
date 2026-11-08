//! Shared canonical-source qualification and raw metadata patching for saved forms.
use super::*;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    kind: &str,
    allowed: &[&str],
    required_text: &[&str],
) -> Result<PreparedWrite, EngineError> {
    if request.kind != kind || !matches!(kind, "entities" | "parts") {
        return Err(refusal("Expected a saved Entity or Part metadata request"));
    }
    let target = PoolLibraryObjectTarget::new("pool", kind, request.object_id);
    let candidates = context
        .model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
        .filter(|s| {
            context
                .model
                .materialized_source_shard_value_by_relative_path(&s.relative_path)
                .ok()
                .and_then(|v| {
                    v.get("uuid")
                        .and_then(Value::as_str)
                        .and_then(|s| Uuid::parse_str(s).ok())
                })
                == Some(request.object_id)
        })
        .count();
    if candidates != 1 {
        return Err(refusal(
            "Saved definition identity has missing or ambiguous source candidates",
        ));
    }
    let source = context
        .model
        .objects
        .get(&request.object_id)
        .filter(|o| o.domain == "pool" && o.kind == kind)
        .ok_or_else(|| refusal("Saved definition has no matching canonical source"))?;
    if !context.model.source_shards.iter().any(|s| {
        s.shard_id == source.source_shard_id
            && s.kind == SourceShardKind::Pool
            && s.relative_path == target.relative_path
    }) {
        return Err(refusal(
            "Saved definition is not owned by the canonical project pool path",
        ));
    }
    let mut object = library::pool_library_object_payload(
        context.model,
        &target.relative_path,
        request.object_id,
    )?;
    for (key, value) in request.fields {
        if !allowed.contains(&key.as_str()) {
            return Err(refusal(
                "Saved metadata editing refuses structural, identity and policy fields",
            ));
        }
        if key == "tags" {
            let _: Vec<String> = serde_json::from_value(value.clone())?;
        } else {
            let text = value
                .as_str()
                .ok_or_else(|| refusal(format!("{key} must be text")))?;
            if required_text.contains(&key.as_str()) && text.trim().is_empty() {
                return Err(refusal(format!("Enter {key}")));
            }
        }
        object
            .as_object_mut()
            .ok_or_else(|| refusal("Definition payload is not an object"))?
            .insert(key, value);
    }
    match kind {
        "entities" => {
            let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
        }
        "parts" => {
            let _: crate::pool::Part = serde_json::from_value(object.clone())?;
            preserve_pinned_part_revision(context.model, request.object_id, &object)?;
        }
        _ => unreachable!(),
    }
    library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )
}

fn preserve_pinned_part_revision(
    model: &crate::substrate::DesignModel,
    id: Uuid,
    replacement: &Value,
) -> Result<(), EngineError> {
    let next = model.objects[&id]
        .object_revision
        .0
        .checked_add(1)
        .ok_or_else(|| refusal("Part revision counter is exhausted"))?;
    let mut pinned = model
        .component_instances
        .values()
        .flat_map(|ci| ci.library_bindings.values())
        .any(|b| {
            b.binding_role == crate::substrate::LibraryBindingRole::Part
                && ((b.target_object_id == id && b.pinned_object_revision.0 == next)
                    || b.local_override_refs
                        .iter()
                        .any(|r| r.object_id == id && r.object_revision.0 == next))
        });
    // Legacy authored CIs can declare only part_ref. Its revision is retained in
    // the canonical shard, while the resolved CI carries only the Part UUID.
    for ci in model.component_instances.values().filter(|ci| {
        ci.authority == crate::substrate::ComponentInstanceAuthority::Authored
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
            "parts",
            crate::substrate::ObjectRevision(next),
        )? != replacement
    {
        return Err(refusal(
            "Edit would make an existing pinned Part revision ambiguous after restored history",
        ));
    }
    Ok(())
}
