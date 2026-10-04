//! Atomic board-source adoption and revision-bound imported occupied copper.
use super::*;

/// Build the single atomic batch for a KiCad board import: create every
/// resolver-unknown net/package/pad/track/via/zone (sorted by id within each
/// family), rewrite the board outline/stackup when the imported values
/// differ from the materialized board root, and write the import-map sidecar
/// shard when `import_map_entries` is non-empty.
pub fn build_kicad_board_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    board: &Board,
    import_map_entries: Vec<ImportMapEntry>,
    source: &Path,
) -> Result<ImportWrite, EngineError> {
    let mut operations = Vec::new();

    let mut nets: Vec<_> = board.nets.values().collect();
    nets.sort_by_key(|net| net.uuid);
    for net in nets {
        if model.objects.contains_key(&net.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_net(
            model,
            provenance.clone(),
            net,
        )?));
    }
    let mut packages: Vec<_> = board.packages.values().collect();
    packages.sort_by_key(|package| package.uuid);
    for package in packages {
        if model.objects.contains_key(&package.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_package(
            model,
            provenance.clone(),
            &BoardPackagePlacement {
                package: package.clone(),
                materialized: serde_json::json!({}),
            },
        )?));
    }
    let mut pads: Vec<_> = board.pads.values().collect();
    pads.sort_by_key(|pad| pad.uuid);
    for pad in pads {
        if model.objects.contains_key(&pad.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_pad(
            model,
            provenance.clone(),
            pad,
        )?));
    }
    let mut tracks: Vec<_> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);
    for track in tracks {
        if model.objects.contains_key(&track.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_track(
            model,
            provenance.clone(),
            track,
        )?));
    }
    let mut vias: Vec<_> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);
    for via in vias {
        if model.objects.contains_key(&via.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_via(
            model,
            provenance.clone(),
            via,
        )?));
    }
    let mut zones: Vec<_> = board.zones.values().collect();
    zones.sort_by_key(|zone| zone.uuid);
    for zone in zones {
        if model.objects.contains_key(&zone.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_zone(
            model,
            provenance.clone(),
            zone,
        )?));
    }

    let board_root = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let imported_outline = serde_json::to_value(&board.outline)?;
    if board_root.get("outline") != Some(&imported_outline) {
        operations.extend(unguarded_operations(build_set_board_outline(
            model,
            provenance.clone(),
            board_id,
            &board.outline,
        )?));
    }
    let imported_stackup = serde_json::to_value(&board.stackup)?;
    if board_root.get("stackup") != Some(&imported_stackup) {
        operations.extend(unguarded_operations(build_set_board_stackup(
            model,
            provenance.clone(),
            board_id,
            &board.stackup.layers,
        )?));
    }

    let created_object_count = operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                Operation::CreateBoardPackage { .. }
                    | Operation::CreateBoardPad { .. }
                    | Operation::CreateBoardTrack { .. }
                    | Operation::CreateBoardVia { .. }
                    | Operation::CreateBoardZone { .. }
                    | Operation::CreateBoardNet { .. }
            )
        })
        .count();
    if !import_map_entries.is_empty() {
        operations.push(import_map_shard_operation(
            kicad_board_import_map_relative_path(source),
            import_map_entries,
        )?);
    }
    finish_import_write(model, provenance, operations, created_object_count)
}

/// Adopt saved copper only as generated evidence in the same source transaction.
#[allow(clippy::too_many_arguments)]
pub fn build_kicad_board_import_with_fills(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    board: &Board,
    legacy_zones: &std::collections::HashMap<Uuid, crate::board::Zone>,
    fills: &std::collections::HashMap<Uuid, Vec<crate::ir::geometry::Polygon>>,
    import_map_entries: Vec<ImportMapEntry>,
    source: &Path,
    source_hash: &str,
) -> Result<ImportWrite, EngineError> {
    let value = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut incomplete_source = false;
    let mut pad_upgrades = Vec::new();
    // Saved source copper cannot certify a different native conductive design.
    for (class, imported) in [
        ("tracks", serde_json::to_value(&board.tracks)?),
        ("vias", serde_json::to_value(&board.vias)?),
        ("pads", serde_json::to_value(&board.pads)?),
    ] {
        if !fills.is_empty()
            && let Some(native) = value[class].as_object()
        {
            for (id, object) in native {
                if imported.get(id) != Some(object) {
                    let object_id = id.parse::<Uuid>().ok();
                    if class == "pads"
                        && object.get("layer_connection").is_none()
                        && let (Some(id), Some(mut incoming)) =
                            (object_id, imported.get(id).cloned())
                    {
                        incoming.as_object_mut().unwrap().remove("layer_connection");
                        let owned = model
                            .objects
                            .get(&id)
                            .is_some_and(|o| o.object_revision.0 == 0)
                            && model.import_map.values().any(|entry| {
                                entry.object_id == id
                                    && entry.source_hash == source_hash
                                    && entry.source_path == source.display().to_string()
                            });
                        if &incoming == object && owned {
                            pad_upgrades.extend(unguarded_operations(
                                super::super::board_routing::build_set_board_pad(
                                    model,
                                    provenance.clone(),
                                    &board.pads[&id],
                                )?,
                            ));
                            continue;
                        }
                    }
                    if imported.get(id).is_none()
                        && object_id.is_some_and(|id| {
                            model
                                .objects
                                .get(&id)
                                .is_some_and(|o| o.object_revision.0 == 0)
                                && model.import_map.values().any(|entry| {
                                    entry.object_id == id
                                        && entry.source_path == source.display().to_string()
                                })
                        })
                    {
                        // Existing non-destructive import tracks disappearance;
                        // it does not delete the retained native source.
                        incomplete_source = true;
                        continue;
                    }
                    return Err(EngineError::Validation(format!(
                        "saved fill source differs from native {class}/{id}; explicit source reconciliation required"
                    )));
                }
            }
        }
    }
    let existing: std::collections::HashMap<Uuid, crate::board::Zone> =
        serde_json::from_value(value["zones"].clone())?;
    if !fills.is_empty()
        && existing
            .keys()
            .any(|id| !board.zones.contains_key(id) && !legacy_zones.contains_key(id))
    {
        return Err(EngineError::Validation(
            "saved fill cannot certify unrelated native Zones".into(),
        ));
    }
    let mut write = build_kicad_board_import(
        model,
        provenance.clone(),
        board_id,
        board,
        import_map_entries,
        source,
    )?;
    let mut extra = pad_upgrades;
    let mut zones: Vec<_> = board.zones.values().collect();
    zones.sort_by_key(|zone| zone.uuid);
    for zone in &zones {
        if let Some(old) = existing.get(&zone.uuid)
            && serde_json::to_value(old)? != serde_json::to_value(zone)?
        {
            let owned = model.import_map.values().any(|e| {
                e.object_id == zone.uuid
                    && e.source_hash == source_hash
                    && e.source_path == source.display().to_string()
            });
            if !owned
                || legacy_zones.get(&zone.uuid).is_none_or(|legacy| {
                    serde_json::to_value(legacy).ok() != serde_json::to_value(old).ok()
                })
            {
                return Err(EngineError::Validation(
                    "imported Zone migration conflicts with edited native source".into(),
                ));
            }
            extra.extend(unguarded_operations(
                super::super::board_routing::build_set_board_zone(model, provenance.clone(), zone)?,
            ));
        }
    }
    let mut legacy_order: Vec<_> = legacy_zones.iter().collect();
    legacy_order.sort_by_key(|(id, _)| **id);
    for (id, legacy) in legacy_order {
        if board.zones.contains_key(id) {
            continue;
        }
        if let Some(old) = existing.get(id) {
            if serde_json::to_value(old)? != serde_json::to_value(legacy)? {
                return Err(EngineError::Validation(
                    "legacy Zone island migration conflicts with edited source".into(),
                ));
            }
            extra.extend(unguarded_operations(
                super::super::board_routing::build_delete_board_zone(
                    model,
                    provenance.clone(),
                    *id,
                    serde_json::to_value(old)?,
                )?,
            ));
        }
    }
    let mut prepared = match write.prepared.take() {
        Some(mut p) => {
            p.batch.operations.extend(extra);
            p
        }
        None => super::super::context::BatchComposer::compose(model, provenance.clone())
            .push_ops(extra)
            .finish()?,
    };
    prepared.batch = super::super::guards::guarded_operation_batch(model, prepared.batch)?;
    let basis = crate::substrate::project_board_basis(model, &prepared.batch)?;
    if fills.keys().any(|id| !board.zones.contains_key(id)) {
        return Err(EngineError::Validation(
            "saved fill has no imported source Zone".into(),
        ));
    }
    for zone in zones {
        let id = &zone.uuid;
        let islands = fills.get(id);
        if islands.is_none()
            && model.zone_fills.get(id).is_some_and(|fill| {
                fill.state == crate::substrate::ZoneFillState::Filled
                    && !fill
                        .provenance
                        .as_deref()
                        .unwrap_or("")
                        .starts_with("datum:kicad-saved-fill:v1;")
            })
        {
            continue;
        }
        // A read/reimport cannot revive evidence invalidated by native edits.
        if model
            .zone_fills
            .get(id)
            .is_some_and(|f| f.state == crate::substrate::ZoneFillState::Stale)
        {
            continue;
        }
        let object = basis
            .objects
            .get(id)
            .ok_or_else(|| EngineError::Validation("imported fill source Zone is absent".into()))?;
        let fill = crate::substrate::ZoneFill {
            schema_version: crate::substrate::ZONE_FILL_SCHEMA_VERSION,
            zone_id: *id,
            state: if islands.is_some() && incomplete_source {
                crate::substrate::ZoneFillState::Stale
            } else if islands.is_some() {
                crate::substrate::ZoneFillState::Filled
            } else {
                crate::substrate::ZoneFillState::Unfilled
            },
            source_zone_revision: object.object_revision,
            model_revision: basis.model_revision.clone(),
            islands: islands.cloned().unwrap_or_default(),
            provenance: Some(format!(
                "datum:kicad-saved-fill:v1; source={}; sha256={source_hash}; correspondence={}",
                source.display(),
                if incomplete_source {
                    "missing-source-retained-native"
                } else {
                    "complete"
                }
            )),
        };
        let fill_value = serde_json::to_value(&fill)?;
        if model
            .zone_fills
            .get(id)
            .is_some_and(|old| serde_json::to_value(old).ok() == Some(fill_value.clone()))
        {
            continue;
        }
        prepared.batch.operations.extend(unguarded_operations(
            super::super::board_routing::build_set_zone_fills(model, provenance.clone(), &[fill])?,
        ));
    }
    write.prepared = if prepared.batch.operations.is_empty() {
        None
    } else {
        Some(prepared)
    };
    Ok(write)
}
