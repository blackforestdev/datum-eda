//! Current source-qualified occurrences, exact retained pins and legacy users.
use super::*;

pub(super) fn validate_frame(
    model: &DesignModel,
    snapshot: &LibraryAssetSnapshot,
    sources: &ElectricalSelectionSnapshot,
    changed: &BTreeSet<Uuid>,
    limits: LibraryAssetLimits,
) -> Result<(), EngineError> {
    let expected = derive_frame(model, snapshot, sources, changed, limits)?;
    if expected != snapshot.lock().bindings {
        return Err(refusal(
            "lock and exact current component occurrences disagree",
        ));
    }
    Ok(())
}

/// The same complete occurrence and native-pin checks serve composition and
/// admission. Deriving a proposed lock never grants publication authority.
pub(super) fn derive_frame(
    model: &DesignModel,
    snapshot: &LibraryAssetSnapshot,
    sources: &ElectricalSelectionSnapshot,
    changed: &BTreeSet<Uuid>,
    limits: LibraryAssetLimits,
) -> Result<Vec<ComponentBinding>, EngineError> {
    if sources.project_id() != model.project.project_id {
        return Err(refusal("foreign evaluated source frame"));
    }
    coherent_component_records(snapshot.lock())?;
    let mut values = BTreeMap::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
    {
        values.insert(
            shard.relative_path.clone(),
            sources
                .source_shard_value(&model.model_revision, shard.shard_id)
                .map_err(query_error)?
                .clone(),
        );
    }
    let index = pool_index(&values)?;
    let mut managed = BTreeMap::new();
    for entry in &snapshot.lock().assets {
        let asset = &snapshot.assets()[&entry.asset];
        let object = model
            .objects
            .get(&entry.local_object_id)
            .filter(|o| {
                o.domain == "pool" && Some(o.kind.as_str()) == asset.revision().kind.pool_kind()
            })
            .ok_or_else(|| refusal("locked object is absent or has the wrong native kind"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.shard_id == object.source_shard_id
                    && s.kind == SourceShardKind::Pool
                    && s.relative_path == entry.source_path
            })
            .ok_or_else(|| refusal("locked native source ownership changed"))?;
        if index.get(&entry.asset.asset_id) != Some(&entry.source_path) {
            return Err(refusal("ambiguous locked native identity"));
        }
        let value = sources
            .source_shard_value(&model.model_revision, shard.shard_id)
            .map_err(query_error)?;
        asset.payload().verify_materialization(
            value,
            counter_policy(
                asset.revision().kind,
                asset.revision().payload_schema_version,
            )?,
        )?;
        managed.insert(entry.asset.asset_id, asset);
    }
    let authored = sources.authored_sources().map_err(query_error)?;
    let mut expected = Vec::new();
    let mut managed_occurrences = BTreeSet::new();
    for ci in model.component_instances.values() {
        let part = ci.part_ref.and_then(|id| managed.get(&id));
        let ci_source = model.objects.get(&ci.id).and_then(|o| {
            model.source_shards.iter().find(|s| {
                s.kind == SourceShardKind::ComponentInstance && s.shard_id == o.source_shard_id
            })
        });
        let Some(part) = part else {
            if ci
                .library_bindings
                .values()
                .any(|b| changed.contains(&b.target_object_id))
            {
                return Err(refusal(
                    "changed library asset has an untracked placed user",
                ));
            }
            if let Some(part) = ci.part_ref
                && !changed.is_empty()
            {
                let kind = AssetKind::Part;
                let path = index
                    .get(&part)
                    .ok_or_else(|| refusal("legacy Part source unavailable during update"))?;
                if reaches_changed(
                    part,
                    kind,
                    &values[path],
                    &values,
                    &index,
                    changed,
                    &mut BTreeSet::new(),
                    &mut BTreeMap::new(),
                    limits.depth,
                )? {
                    return Err(refusal(
                        "changed shared dependency has a legacy component user; include its Part in update review",
                    ));
                }
            }
            continue;
        };
        if part.revision().kind != AssetKind::Part
            || ci.authority != ComponentInstanceAuthority::Authored
        {
            return Err(refusal(
                "managed component does not have an authored Part binding",
            ));
        }
        let source =
            ci_source.ok_or_else(|| refusal("authored managed component source missing"))?;
        let raw = sources
            .source_shard_value(&model.model_revision, source.shard_id)
            .map_err(query_error)?;
        let declaration: ComponentInstanceShard = serde_json::from_value(raw.clone())?;
        let persisted = declaration.component_instance;
        let mut part_pins: Vec<_> = persisted
            .part_ref
            .iter()
            .map(|p| (p.object_id, p.object_revision))
            .collect();
        part_pins.extend(
            persisted
                .library_bindings
                .values()
                .filter(|b| b.binding_role == crate::substrate::LibraryBindingRole::Part)
                .map(|b| (b.target_object_id, b.pinned_object_revision)),
        );
        if part_pins.is_empty() {
            return Err(refusal("managed component lacks an exact Part revision"));
        }
        for (id, revision) in part_pins {
            if id != part.reference().asset_id {
                return Err(refusal(
                    "component Part differs from locked portable identity",
                ));
            }
            verify_pin(model, sources, part, id, revision)?;
        }
        for binding in persisted.library_bindings.values() {
            if let Some(asset) = managed.get(&binding.target_object_id) {
                verify_pin(
                    model,
                    sources,
                    asset,
                    binding.target_object_id,
                    binding.pinned_object_revision,
                )?;
                for override_ref in &binding.local_override_refs {
                    if override_ref.object_id != asset.reference().asset_id {
                        return Err(refusal(
                            "an effective local override needs an explicit portable binding",
                        ));
                    }
                    verify_pin(
                        model,
                        sources,
                        asset,
                        override_ref.object_id,
                        override_ref.object_revision,
                    )?;
                }
            }
        }
        let origins: BTreeSet<ElectricalOccurrence> = authored
            .iter()
            .filter(|s| {
                s.occurrence.class == "symbols"
                    && ci.placed_symbol_refs.contains(&s.occurrence.source_id)
            })
            .map(|s| s.occurrence.clone())
            .collect();
        if origins.is_empty() {
            return Err(refusal("managed component has no schematic occurrence"));
        }
        let context = sources
            .component_selection_context(&model.model_revision, &origins)
            .map_err(query_error)?;
        managed_occurrences.extend(origins.iter().cloned());
        managed_occurrences.extend(
            context
                .relations
                .iter()
                .filter(|r| r.component_instance == ci.id)
                .map(|r| r.counterpart.clone()),
        );
        for origin in &origins {
            let proof = context
                .bindings
                .get(origin)
                .filter(|binding| binding.component_instance == ci.id)
                .ok_or_else(|| refusal("component occurrence is repeated, stale or ambiguous"))?;
            if proof.counterpart_status == ComponentCounterpartStatus::Unavailable {
                return Err(refusal(
                    "managed component physical counterpart is not qualified",
                ));
            }
            let object = model
                .objects
                .get(&origin.source_id)
                .ok_or_else(|| refusal("placed symbol missing"))?;
            let source = model
                .source_shards
                .iter()
                .find(|s| {
                    s.shard_id == object.source_shard_id
                        && matches!(
                            s.kind,
                            SourceShardKind::SchematicSheet
                                | SourceShardKind::SchematicRoot
                                | SourceShardKind::SchematicDefinition
                        )
                })
                .ok_or_else(|| refusal("schematic owning source is absent"))?;
            expected.push(ComponentBinding {
                component_instance_id: ci.id,
                source_path: source.relative_path.clone(),
                instance_path: origin.instance_path.clone(),
                component: part.reference().clone(),
            });
        }
    }
    expected.sort_by(|a, b| {
        (&a.source_path, &a.instance_path, a.component_instance_id).cmp(&(
            &b.source_path,
            &b.instance_path,
            b.component_instance_id,
        ))
    });
    expected.dedup();
    if !changed.is_empty() {
        for source in &authored {
            let origin = &source.occurrence;
            if !matches!(origin.class.as_str(), "symbols" | "packages")
                || managed_occurrences.contains(origin)
            {
                continue;
            }
            let object = model
                .objects
                .get(&origin.source_id)
                .ok_or_else(|| refusal("legacy placed source missing"))?;
            let row = sources
                .source_shard_value(&model.model_revision, object.source_shard_id)
                .map_err(query_error)?;
            let key = origin.source_id.to_string();
            let mut members = Vec::new();
            if let Some(member) = row.get(&origin.class).and_then(|rows| rows.get(&key)) {
                members.push(member);
            }
            if let Some(sheets) = row.get("sheets").and_then(Value::as_object) {
                members.extend(
                    sheets.values().filter_map(|sheet| {
                        sheet.get(&origin.class).and_then(|rows| rows.get(&key))
                    }),
                );
            }
            let [member] = members.as_slice() else {
                return Err(refusal("legacy placed source membership is ambiguous"));
            };
            let slots = if origin.class == "symbols" {
                vec![
                    ("part", AssetKind::Part),
                    ("entity", AssetKind::Entity),
                    ("lib_id", AssetKind::Symbol),
                ]
            } else {
                vec![("part", AssetKind::Part), ("package", AssetKind::Package)]
            };
            for (field, kind) in slots {
                let Some(id) = member
                    .get(field)
                    .and_then(Value::as_str)
                    .and_then(|v| v.parse::<Uuid>().ok())
                else {
                    continue;
                };
                if id.is_nil() {
                    continue;
                }
                let path = index
                    .get(&id)
                    .ok_or_else(|| refusal("legacy placed library source is unavailable"))?;
                if reaches_changed(
                    id,
                    kind,
                    &values[path],
                    &values,
                    &index,
                    changed,
                    &mut BTreeSet::new(),
                    &mut BTreeMap::new(),
                    limits.depth,
                )? {
                    return Err(refusal(
                        "shared asset update has an unbound legacy placed user; review its component binding",
                    ));
                }
            }
        }
    }
    Ok(expected)
}

fn verify_pin(
    model: &DesignModel,
    sources: &ElectricalSelectionSnapshot,
    asset: &VerifiedAsset,
    id: Uuid,
    revision: crate::substrate::ObjectRevision,
) -> Result<(), EngineError> {
    let kind = asset
        .revision()
        .kind
        .pool_kind()
        .ok_or_else(|| refusal("unsupported owned model pin"))?;
    // Required canonical-owner seam: this must resolve both published and
    // evaluated pending pins from certified values/accepted history. It must not
    // stamp the pending model's integrity cursor or fall back to latest content.
    let pinned = sources
        .pinned_library_definition(&model.model_revision, id, kind, revision)
        .map_err(query_error)?;
    asset.payload().verify_materialization(
        &pinned,
        counter_policy(
            asset.revision().kind,
            asset.revision().payload_schema_version,
        )?,
    )
}

pub(super) fn coherent_component_records(lock: &ProjectLibraryLock) -> Result<(), EngineError> {
    lock.validate()?;
    let mut parts: BTreeMap<Uuid, &AssetRevisionRef> = BTreeMap::new();
    for binding in &lock.bindings {
        if parts
            .insert(binding.component_instance_id, &binding.component)
            .is_some_and(|old| old != &binding.component)
        {
            return Err(refusal(
                "one component has conflicting portable Parts across source records",
            ));
        }
    }
    Ok(())
}

fn reaches_changed(
    id: Uuid,
    kind: AssetKind,
    value: &Value,
    values: &BTreeMap<String, Value>,
    index: &BTreeMap<Uuid, String>,
    changed: &BTreeSet<Uuid>,
    visiting: &mut BTreeSet<Uuid>,
    memo: &mut BTreeMap<Uuid, bool>,
    max_depth: usize,
) -> Result<bool, EngineError> {
    if changed.contains(&id) {
        return Ok(true);
    }
    if let Some(changed) = memo.get(&id) {
        return Ok(*changed);
    }
    if visiting.len() >= max_depth {
        return Err(refusal(
            "legacy dependency walk exceeds service depth policy",
        ));
    }
    if !visiting.insert(id) {
        return Err(refusal("legacy forward dependency cycle"));
    }
    for slot in payload_reference_slots(kind, value)? {
        let path = index
            .get(&slot.target_id)
            .ok_or_else(|| refusal("legacy dependency cannot be qualified"))?;
        if reaches_changed(
            slot.target_id,
            slot.kind,
            &values[path],
            values,
            index,
            changed,
            visiting,
            memo,
            max_depth,
        )? {
            memo.insert(id, true);
            return Ok(true);
        }
    }
    visiting.remove(&id);
    memo.insert(id, false);
    Ok(false)
}
