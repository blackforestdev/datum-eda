//! Exact planned source projection is read-only. Only final admission may
//! qualify actual committed geometry from the sealed evaluated source frame.
use super::*;
use crate::substrate::{
    component_pad_source::PlannedPadSources,
    operation_application_objects::apply_operation_to_objects,
};

pub(crate) fn planned_sources<'a>(
    root: &Path,
    model: &'a DesignModel,
    batch: &OperationBatch,
    limits: LibraryAssetLimits,
) -> Result<PlannedPadSources<'a>, EngineError> {
    let limits = restrict_limits(limits)?;
    if model.journal_cursor.applied_transaction_count != model.journal.len()
        || batch.expected_model_revision.as_ref() != Some(&model.model_revision)
    {
        return Err(refusal(
            "physical planning requires the exact accepted source basis",
        ));
    }
    let transitions: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    let [
        operation @ LibraryAssetOperation::SetState {
            previous,
            state,
            content,
            ..
        },
    ] = transitions.as_slice()
    else {
        return Err(refusal(
            "physical planning requires one exact library adoption",
        ));
    };
    if operation.owner_id() != model.project.project_id || !content.is_empty() {
        return Err(refusal(
            "physical planning adopts existing local records only",
        ));
    }
    if previous.catalog != state.catalog {
        return Err(refusal(
            "physical adoption must retain the reviewed local catalog",
        ));
    }
    super::super::validate_preimages(root, operation, limits)?;
    let (old_lock, _) = previous.decode(limits.metadata_bytes)?;
    let snapshot = resolve_state(root, state, &BTreeMap::new(), limits)?;
    if old_lock
        .assets
        .iter()
        .map(|entry| entry.local_object_id)
        .collect::<BTreeSet<_>>()
        != snapshot
            .lock()
            .assets
            .iter()
            .map(|entry| entry.local_object_id)
            .collect()
    {
        return Err(refusal(
            "physical adoption must retain the complete managed asset inventory",
        ));
    }
    let paths = snapshot
        .lock()
        .assets
        .iter()
        .map(|entry| entry.source_path.clone())
        .collect();
    let (pool, before_pool) = pending_pool(root, model, &batch.operations, limits, &paths)?;
    let mut objects = model.objects.clone();
    let mut components = BTreeMap::new();
    let mut applied = Vec::new();
    for op in &batch.operations {
        if matches!(op, Operation::GuardObjectRevision { .. }) {
            apply_operation_to_objects(&mut objects, op, None)?;
        }
    }
    for op in &batch.operations {
        match op {
            Operation::GuardObjectRevision { .. } => {}
            Operation::SetPoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                previous_object,
                object,
            } => {
                if !matches!(object_kind.as_str(), "footprints" | "padstacks") {
                    return Err(refusal(
                        "geometry adoption requires separate review of other native definition changes",
                    ));
                }
                crate::substrate::pool_journal_ops::validate_pool_library_object(
                    *object_id,
                    relative_path,
                    object_kind,
                    object,
                )?;
                geometry_only(previous_object, object, object_kind)?;
                apply_operation_to_objects(&mut objects, op, None)?;
                applied.push(op.clone());
            }
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                ..
            } => {
                let owner = model
                    .objects
                    .get(component_instance_id)
                    .ok_or_else(|| refusal("physical component source is absent"))?;
                let head = model
                    .source_shards
                    .iter()
                    .find(|head| {
                        head.shard_id == owner.source_shard_id
                            && head.kind == SourceShardKind::ComponentInstance
                    })
                    .ok_or_else(|| refusal("physical component source is foreign"))?;
                let source =
                    model.materialized_source_shard_value_by_relative_path(&head.relative_path)?;
                if source["component_instance"] != *previous_component_instance
                    || components
                        .insert(*component_instance_id, previous_component_instance.clone())
                        .is_some()
                {
                    return Err(refusal(
                        "physical component preimage is stale or duplicated",
                    ));
                }
            }
            Operation::LibraryAssets { .. } => {}
            Operation::SetBoardPackagePackage { .. } | Operation::SetElectricalIdentity { .. } => {}
            _ => {
                return Err(refusal(
                    "unsupported operation in planned geometry adoption",
                ));
            }
        }
    }
    let closure = crate::library_assets::inspect_closure(
        &snapshot
            .lock()
            .assets
            .iter()
            .map(|entry| entry.asset.clone())
            .collect::<Vec<_>>(),
        snapshot.assets(),
        crate::library_assets::ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    snapshot.lock().verify_closure(&closure)?;
    if !closure.graph_report().valid {
        return Err(refusal("planned geometry closure is invalid"));
    }
    for entry in &snapshot.lock().assets {
        let asset = &snapshot.assets()[&entry.asset];
        let old = old_lock
            .assets
            .iter()
            .find(|old| old.local_object_id == entry.local_object_id)
            .ok_or_else(|| {
                refusal("new managed source requires separate acquisition before geometry adoption")
            })?;
        if old.source_path != entry.source_path || !before_pool.contains_key(&entry.source_path) {
            return Err(refusal("geometry adoption changes source ownership"));
        }
        let object = objects
            .get(&entry.local_object_id)
            .ok_or_else(|| refusal("planned library object is absent"))?;
        let kind = asset
            .revision()
            .kind
            .pool_kind()
            .ok_or_else(|| refusal("owned model geometry transport is unsupported"))?;
        if object.domain != "pool"
            || object.kind != kind
            || model
                .source_shards
                .iter()
                .filter(|head| {
                    head.shard_id == object.source_shard_id
                        && head.kind == SourceShardKind::Pool
                        && head.relative_path == entry.source_path
                })
                .count()
                != 1
        {
            return Err(refusal("planned library source ownership is ambiguous"));
        }
        asset.payload().verify_materialization(
            &pool[&entry.source_path],
            counter_policy(
                asset.revision().kind,
                asset.revision().payload_schema_version,
            )?,
        )?;
    }
    // Derive only the existing adoption's native-library pin increment. Incoming
    // final CI/package/electrical payloads are never authority for this plan;
    // the complete physical transition is independently recomputed below.
    components.retain(|id, previous| {
        let _ = id;
        let mut changed = false;
        if let Some(bindings) = previous["library_bindings"].as_object_mut() {
            for binding in bindings.values_mut() {
                if let Some(id) = binding["target_object_id"]
                    .as_str()
                    .and_then(|s| s.parse::<Uuid>().ok())
                {
                    if let Some((old, new)) = model.objects.get(&id).zip(objects.get(&id)) {
                        if old.object_revision != new.object_revision {
                            binding["pinned_object_revision"] =
                                serde_json::json!(new.object_revision);
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    });
    for (id, component) in &mut components {
        let next = model.objects[id]
            .object_revision
            .0
            .checked_add(1)
            .ok_or_else(|| refusal("component revision overflow"))?;
        component["object_revision"] = serde_json::json!(next);
    }
    let selection = ElectricalSelectionSnapshot::capture(model).map_err(query_error)?;
    let impacts = super::super::physical_impact::assess(
        model,
        &selection,
        &before_pool,
        snapshot.lock(),
        snapshot.assets(),
    )?;
    Ok(PlannedPadSources::new(
        model,
        objects,
        pool,
        applied,
        snapshot.lock().clone(),
        components,
        impacts,
    ))
}

/// This increment changes pad position/aperture/drill dimensions only. Full
/// land-pattern graphics, layer membership, map changes and rebirth stay refused.
pub(in crate::substrate) fn geometry_only(
    before: &Value,
    after: &Value,
    kind: &str,
) -> Result<(), EngineError> {
    let mut old = before.clone();
    let mut new = after.clone();
    for value in [&mut old, &mut new] {
        let raw = value
            .as_object_mut()
            .ok_or_else(|| refusal("geometry source is not an object"))?;
        let counters = counter_policy(
            if kind == "footprints" {
                AssetKind::Footprint
            } else {
                AssetKind::Padstack
            },
            1,
        )?;
        if counters.object_revision {
            raw.remove("object_revision");
        }
        if counters.model_revision {
            raw.remove("model_revision");
        }
        raw.remove("name");
        if kind == "footprints" {
            raw.remove("tags");
            let pads = raw
                .get_mut("pads")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| refusal("geometry footprint has no pad map"))?;
            for pad in pads.values_mut() {
                let position = pad
                    .get("position")
                    .ok_or_else(|| refusal("geometry pad position is absent"))?;
                let _: crate::ir::geometry::Point = serde_json::from_value(position.clone())?;
                let position = pad
                    .get_mut("position")
                    .and_then(Value::as_object_mut)
                    .ok_or_else(|| refusal("geometry pad position is not an object"))?;
                position.remove("x");
                position.remove("y");
            }
        } else {
            let aperture = raw
                .get_mut("aperture")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| refusal("geometry padstack aperture is absent"))?;
            for (shape, dimensions) in [
                ("circle", &["diameter_nm"][..]),
                ("rect", &["width_nm", "height_nm"][..]),
                ("round_rect", &["width_nm", "height_nm", "rratio_ppm"][..]),
            ] {
                if let Some(shape) = aperture.get_mut(shape).and_then(Value::as_object_mut) {
                    for field in dimensions {
                        shape.remove(*field);
                    }
                }
            }
            if let Some(drill) = raw.get_mut("drill_nm") {
                if !drill.is_null() {
                    *drill = serde_json::json!(0);
                }
            }
        }
    }
    if old != new {
        return Err(refusal(
            "requested physical update changes topology, mapping, layers, process basis or other unsupported geometry",
        ));
    }
    Ok(())
}

pub(super) fn verify_final(
    frame: &crate::substrate::journal_source_frame::VerifiedPendingSourceFrame,
) -> Result<(), EngineError> {
    let packages: BTreeSet<_> = frame
        .candidate()
        .journal
        .last()
        .ok_or_else(|| refusal("missing actual physical transaction"))?
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetBoardPackagePackage { package_id, .. } => Some(*package_id),
            _ => None,
        })
        .collect();
    if packages.is_empty() {
        return Ok(());
    }
    let heads: Vec<_> = frame
        .candidate()
        .source_shards
        .iter()
        .filter(|head| head.kind == SourceShardKind::BoardRoot)
        .collect();
    let [head] = heads.as_slice() else {
        return Err(refusal("final physical Board ownership is ambiguous"));
    };
    let board = frame.value(head)?;
    let projected = crate::substrate::component_pad_source::pending::projection(frame, &board)?;
    for package in packages {
        let pads: Vec<crate::substrate::component_pad_source::ComponentPadSource> =
            serde_json::from_value(board["component_pads"][package.to_string()].clone())?;
        if pads.is_empty()||pads.iter().any(|pad|projected.statuses.get(&pad.uuid)!=Some(&crate::substrate::component_pad_source::ComponentPadMaterializationStatus::Current)) {
            return Err(refusal("actual evaluated physical library update has stale or unqualified pads"));
        }
    }
    Ok(())
}
