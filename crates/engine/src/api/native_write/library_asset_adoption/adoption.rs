//! Exact retained revisions become active only through a separately reviewed batch.
use super::*;
use crate::{
    library_assets::lock::{ComponentBinding, LockedAsset, ProjectLibraryLock},
    substrate::{ComponentInstanceAuthority, ElectricalOccurrence, ElectricalSelectionSnapshot},
};

pub(crate) struct AdoptionRequest {
    pub(crate) expected_catalog: AssetDigest,
    /// Complete desired active closure roots, selected by the update preview.
    pub(crate) roots: Vec<AssetRevisionRef>,
    geometry: GeometryRequest,
}

enum GeometryRequest {
    None,
    Discover,
    Apply(crate::substrate::component_pad_source::PhysicalGeometryIntent),
}
impl AdoptionRequest {
    pub(crate) fn new(
        expected_catalog: AssetDigest,
        roots: Vec<AssetRevisionRef>,
        intent: Option<crate::substrate::component_pad_source::PhysicalGeometryIntent>,
    ) -> Self {
        Self {
            expected_catalog,
            roots,
            geometry: intent.map_or(GeometryRequest::None, GeometryRequest::Apply),
        }
    }
    pub(crate) fn discover(expected_catalog: AssetDigest, roots: Vec<AssetRevisionRef>) -> Self {
        Self {
            expected_catalog,
            roots,
            geometry: GeometryRequest::Discover,
        }
    }
}

pub(crate) struct AdoptionPlan {
    pub(crate) prepared: Option<PreparedWrite>,
    pub(crate) before: Vec<AssetRevisionRef>,
    pub(crate) after: Vec<AssetRevisionRef>,
    pub(crate) changed_materializations: Vec<Uuid>,
    pub(crate) affected_components: Vec<Uuid>,
    pub(crate) physical_geometry: crate::substrate::component_pad_source::PhysicalGeometryIntent,
    pub(crate) physical_geometry_updates: Vec<Value>,
}

pub(crate) fn prepare_adoption(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: AdoptionRequest,
    limits: LibraryAssetLimits,
) -> Result<AdoptionPlan, EngineError> {
    let limits = restrict_limits(limits)?;
    if context.model.project.schema_version != Some(2) {
        return Err(refusal("separate forward-only format adoption is required"));
    }
    let previous = read_state(context.project_root, limits)?;
    let snapshot = resolve_state(context.project_root, &previous, &BTreeMap::new(), limits)?;
    if request.expected_catalog != snapshot.catalog().manifest_ref {
        return Err(refusal("catalog changed since update preview"));
    }
    let closure = inspect_closure(
        &request.roots,
        snapshot.assets(),
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    if !closure.graph_report().valid {
        return Err(refusal(
            "requested adoption has an invalid native library graph",
        ));
    }
    let by_id: BTreeMap<_, _> = closure
        .assets()
        .keys()
        .map(|r| (r.asset_id, r.clone()))
        .collect();
    if snapshot
        .lock()
        .assets
        .iter()
        .any(|a| !by_id.contains_key(&a.asset.asset_id))
    {
        return Err(refusal(
            "update must retain currently managed assets; include their dependent roots",
        ));
    }
    let mut operations = Vec::new();
    let selection = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|error| refusal(format!("{error:?}")))?;
    let before_pool: BTreeMap<_, _> = context
        .model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool)
        .map(|s| {
            Ok((
                s.relative_path.clone(),
                selection
                    .source_shard_value(&context.model.model_revision, s.shard_id)
                    .map_err(|error| refusal(format!("{error:?}")))?
                    .clone(),
            ))
        })
        .collect::<Result<_, EngineError>>()?;
    let mut locked = Vec::new();
    let mut next_revisions = BTreeMap::new();
    for (reference, asset) in closure.assets() {
        let kind = asset.revision().kind;
        let pool_kind = kind
            .pool_kind()
            .ok_or_else(|| refusal("owned model transport remains unsupported"))?;
        let relative = format!("pool/{pool_kind}/{}.json", reference.asset_id);
        let mut next = asset.payload().value().clone();
        let owner = context.model.objects.get(&reference.asset_id);
        if let Some(owner) = owner {
            if owner.domain != "pool" || owner.kind != pool_kind {
                return Err(refusal(
                    "adopted asset collides with a different native object",
                ));
            }
            let sources: Vec<_> = context
                .model
                .source_shards
                .iter()
                .filter(|s| {
                    s.kind == SourceShardKind::Pool
                        && s.shard_id == owner.source_shard_id
                        && s.relative_path == relative
                })
                .collect();
            let [_] = sources.as_slice() else {
                return Err(refusal("adoption source ownership is ambiguous"));
            };
            let old = before_pool
                .get(&relative)
                .ok_or_else(|| refusal("captured adoption source is absent"))?
                .clone();
            if asset
                .payload()
                .verify_materialization(&old, counter_policy(kind, 1)?)
                .is_err()
            {
                let revision = owner
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| refusal("native revision overflow"))?;
                // Counter presence follows the existing owning source schema;
                // authored portable payload fields are otherwise exact.
                if old.get("object_revision").is_some() {
                    next["object_revision"] = revision.into();
                }
                operations.push(Operation::SetPoolLibraryObject {
                    object_id: reference.asset_id,
                    relative_path: relative.clone(),
                    object_kind: pool_kind.into(),
                    previous_object: old,
                    object: next,
                });
                next_revisions.insert(reference.asset_id, revision);
            }
        } else {
            if context
                .model
                .source_shards
                .iter()
                .any(|s| s.relative_path == relative)
            {
                return Err(refusal(
                    "adoption destination already has an unregistered source",
                ));
            }
            operations.push(Operation::CreatePoolLibraryObject {
                object_id: reference.asset_id,
                relative_path: relative.clone(),
                object_kind: pool_kind.into(),
                object: next,
            });
            next_revisions.insert(reference.asset_id, 0);
        }
        locked.push(LockedAsset {
            asset: (*reference).clone(),
            local_object_id: reference.asset_id,
            source_path: relative,
            payload_ref: asset.revision().payload_ref.clone(),
        });
    }
    locked.sort_by(|a, b| a.asset.cmp(&b.asset));
    let (bindings, affected_components, mut component_ops) =
        component_bindings(context, &by_id, &next_revisions, &selection)?;
    operations.append(&mut component_ops);
    let mut source_manifests = snapshot.lock().source_manifests.clone();
    source_manifests.push(snapshot.catalog().manifest_ref.clone());
    source_manifests.sort();
    source_manifests.dedup();
    let lock = ProjectLibraryLock {
        schema_version: 1,
        profile: PROFILE.into(),
        bindings,
        assets: locked,
        source_manifests,
        closure_digest: closure.digest().clone(),
        extensions: snapshot.lock().extensions.clone(),
    };
    lock.verify_closure(&closure)?;
    let impacts = crate::substrate::library_asset_journal_ops::physical_impact::assess(
        context.model,
        &selection,
        &before_pool,
        &lock,
        snapshot.assets(),
    )?;
    let state = LibraryStateValues {
        lock: encode_metadata(&lock)?,
        catalog: previous.catalog.clone(),
    };
    operations.push(Operation::LibraryAssets {
        operation: LibraryAssetOperation::SetState {
            project_id: context.model.project.project_id,
            previous,
            state,
            content: vec![],
        },
    });
    let mut batch = build_batch(context.model, provenance.clone(), operations)?;
    let mut physical_geometry =
        crate::substrate::component_pad_source::PhysicalGeometryIntent { placements: vec![] };
    let mut physical_geometry_updates = vec![];
    if !impacts.is_empty() {
        if matches!(request.geometry, GeometryRequest::None) {
            crate::substrate::library_asset_journal_ops::physical_impact::require_supported(
                &impacts,
            )?;
        }
        let target = crate::substrate::library_asset_journal_ops::planned_physical_sources(
            context.project_root,
            context.model,
            &batch,
            limits,
        )?;
        let geometry = super::super::board_components::pad_materialization::plan_geometry(
            context.model,
            provenance.clone(),
            &target,
        )?;
        physical_geometry = geometry.intent;
        physical_geometry_updates = geometry.updates;
        if let GeometryRequest::Apply(intent) = &request.geometry {
            if intent != &physical_geometry {
                return Err(refusal(
                    "reviewed geometry correspondence or affected users changed",
                ));
            }
        }
        let updated: BTreeSet<_> = geometry
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::SetComponentInstance {
                    component_instance_id,
                    ..
                } => Some(*component_instance_id),
                _ => None,
            })
            .collect();
        batch.operations.retain(|op|!matches!(op,Operation::SetComponentInstance {component_instance_id,..} if updated.contains(component_instance_id)));
        batch.operations.extend(geometry.operations);
        batch = build_batch(context.model, provenance.clone(), batch.operations)?;
        crate::substrate::library_asset_journal_ops::qualify_physical_plan(
            context.model,
            &batch
                .operations
                .iter()
                .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
                .cloned()
                .collect::<Vec<_>>(),
        )?;
    } else if let GeometryRequest::Apply(intent) = &request.geometry {
        if intent != &physical_geometry {
            return Err(refusal(
                "reviewed geometry update no longer has those affected users",
            ));
        }
    }
    let before = snapshot
        .lock()
        .assets
        .iter()
        .map(|a| a.asset.clone())
        .collect();
    let after = lock.assets.iter().map(|a| a.asset.clone()).collect();
    Ok(AdoptionPlan {
        prepared: if matches!(request.geometry, GeometryRequest::Discover) {
            None
        } else {
            Some(PreparedWrite {
                batch,
                primary_object_id: None,
            })
        },
        before,
        after,
        changed_materializations: next_revisions.into_keys().collect(),
        affected_components,
        physical_geometry,
        physical_geometry_updates,
    })
}

fn component_bindings(
    context: &NativeWriteContext<'_>,
    active: &BTreeMap<Uuid, AssetRevisionRef>,
    changed: &BTreeMap<Uuid, u64>,
    selection: &ElectricalSelectionSnapshot,
) -> Result<(Vec<ComponentBinding>, Vec<Uuid>, Vec<Operation>), EngineError> {
    let authored = selection
        .authored_sources()
        .map_err(|e| refusal(format!("{e:?}")))?;
    let mut bindings = Vec::new();
    let mut affected = Vec::new();
    let mut operations = Vec::new();
    for ci in context.model.component_instances.values() {
        let touches = ci
            .library_bindings
            .values()
            .any(|b| changed.contains_key(&b.target_object_id));
        let part = ci.part_ref.and_then(|id| active.get(&id));
        if touches && part.is_none() {
            return Err(refusal(
                "a changed shared asset has a placed user outside the selected managed closure",
            ));
        }
        if touches
            && ci.library_bindings.values().any(|binding| {
                binding
                    .local_override_refs
                    .iter()
                    .any(|r| changed.contains_key(&r.object_id))
            })
        {
            return Err(refusal(
                "an explicit local override needs its own reviewed update intent",
            ));
        }
        let Some(part) = part else {
            continue;
        };
        if ci.authority != ComponentInstanceAuthority::Authored || ci.placed_symbol_refs.is_empty()
        {
            return Err(refusal(
                "managed component needs an authored schematic occurrence",
            ));
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
            return Err(refusal(
                "managed component has no current schematic occurrence",
            ));
        }
        let proven = selection
            .component_selection_context(&context.model.model_revision, &origins)
            .map_err(|e| refusal(format!("{e:?}")))?;
        for origin in &origins {
            if proven
                .bindings
                .get(origin)
                .is_none_or(|b| b.component_instance != ci.id)
            {
                return Err(refusal(
                    "component occurrence is repeated, stale or ambiguous",
                ));
            }
            let object = context
                .model
                .objects
                .get(&origin.source_id)
                .ok_or_else(|| refusal("symbol source registration missing"))?;
            let source = context
                .model
                .source_shards
                .iter()
                .find(|s| {
                    s.shard_id == object.source_shard_id
                        && matches!(
                            s.kind,
                            SourceShardKind::SchematicRoot
                                | SourceShardKind::SchematicSheet
                                | SourceShardKind::SchematicDefinition
                        )
                })
                .ok_or_else(|| refusal("symbol owning schematic source missing"))?;
            bindings.push(ComponentBinding {
                component_instance_id: ci.id,
                source_path: source.relative_path.clone(),
                instance_path: origin.instance_path.clone(),
                component: part.clone(),
            });
        }
        affected.push(ci.id);
        if touches {
            let owner = context
                .model
                .objects
                .get(&ci.id)
                .ok_or_else(|| refusal("component registration missing"))?;
            let source = context
                .model
                .source_shards
                .iter()
                .find(|s| {
                    s.shard_id == owner.source_shard_id
                        && s.kind == SourceShardKind::ComponentInstance
                })
                .ok_or_else(|| refusal("authored component source missing"))?;
            let old = context
                .model
                .materialized_source_shard_value_by_relative_path(&source.relative_path)?;
            let previous = old
                .get("component_instance")
                .ok_or_else(|| refusal("component source has no declaration"))?
                .clone();
            let mut next = previous.clone();
            let revision = ci
                .object_revision
                .0
                .checked_add(1)
                .ok_or_else(|| refusal("component revision overflow"))?;
            next["object_revision"] = revision.into();
            if let Some(part_id) = ci.part_ref
                && let Some(revision) = changed.get(&part_id)
                && next.get("part_ref").is_some_and(|v| !v.is_null())
            {
                next["part_ref"]["object_revision"] = (*revision).into();
            }
            if let Some(items) = next
                .get_mut("library_bindings")
                .and_then(Value::as_object_mut)
            {
                for value in items.values_mut() {
                    if let Some(revision) = value
                        .get("target_object_id")
                        .and_then(Value::as_str)
                        .and_then(|id| id.parse::<Uuid>().ok())
                        .and_then(|id| changed.get(&id))
                    {
                        value["pinned_object_revision"] = (*revision).into();
                    }
                }
            }
            operations.push(Operation::SetComponentInstance {
                component_instance_id: ci.id,
                previous_component_instance: previous,
                component_instance: next,
            });
        }
    }
    bindings.sort_by(|a, b| {
        (&a.source_path, &a.instance_path, a.component_instance_id).cmp(&(
            &b.source_path,
            &b.instance_path,
            b.component_instance_id,
        ))
    });
    bindings.dedup();
    Ok((bindings, affected, operations))
}
