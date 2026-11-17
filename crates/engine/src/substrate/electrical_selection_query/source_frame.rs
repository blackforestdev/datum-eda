//! One value-map evaluator for accepted reads and certified pending sources.
use super::*;
use crate::substrate::journal_source_frame::VerifiedPendingSourceFrame;

pub(super) fn evaluate(
    model: Arc<DesignModel>,
    mut values: BTreeMap<Uuid, serde_json::Value>,
    mut source_basis: Vec<SelectionSourceBasis>,
    pending_frame: Option<Arc<VerifiedPendingSourceFrame>>,
) -> Result<ElectricalSelectionSnapshot, ElectricalQueryFailure> {
    source_basis.sort_by_key(|v| v.shard_id);
    let read = |shard: &crate::substrate::SourceShardRef| {
        values
            .get(&shard.shard_id)
            .cloned()
            .ok_or_else(|| EngineError::Validation("missing captured electrical source".into()))
    };
    let failure = |e: EngineError| ElectricalQueryFailure::UnavailableBasis {
        sources: source_basis.iter().map(|s| s.shard_id).collect(),
        reason: e.to_string(),
    };
    let basis = if let Some(frame) = &pending_frame {
        crate::substrate::electrical_basis::ElectricalBasis::with_pending_frame(frame.clone())
            .map_err(failure)?
    } else {
        crate::substrate::electrical_basis::ElectricalBasis::with_reader(&model, &[], &read)
            .map_err(failure)?
    };
    let (logical, physical, representations, schematic) = if model
        .source_shards
        .iter()
        .any(|s| s.kind == SourceShardKind::SchematicRoot)
    {
        let schematic =
            crate::substrate::electrical_topology_source::schematic_with_reader(&model, &[], &read)
                .map_err(failure)?;
        let groups = crate::connectivity::partitions(&schematic).map_err(failure)?;
        let logical = crate::substrate::electrical_transaction::previous(&model, &groups)
            .map_err(failure)?
            .into_iter()
            .map(|g| (g.net_id, g.members))
            .collect();
        (
            logical,
            crate::connectivity::physical_partitions(&schematic).map_err(failure)?,
            crate::connectivity::representation_occurrences(&schematic).map_err(failure)?,
            Some(schematic),
        )
    } else {
        (BTreeMap::new(), vec![], BTreeSet::new(), None)
    };
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::BoardRoot)
    {
        // The basis owns the final projection after assignment qualification.
        // Source queries borrow that same tree rather than retaining a copy.
        values.remove(&shard.shard_id);
    }
    Ok(ElectricalSelectionSnapshot {
        model,
        basis,
        logical,
        physical,
        schematic,
        representations,
        values,
        source_basis,
        pending_frame,
    })
}

pub(super) fn needed(kind: &SourceShardKind) -> bool {
    matches!(
        kind,
        SourceShardKind::SchematicRoot
            | SourceShardKind::SchematicSheet
            | SourceShardKind::SchematicDefinition
            | SourceShardKind::BoardRoot
            | SourceShardKind::Pool
            | SourceShardKind::ElectricalIdentity
            | SourceShardKind::ComponentInstance
            | SourceShardKind::ZoneFill
    )
}

pub(super) fn pending(
    frame: Arc<VerifiedPendingSourceFrame>,
) -> Result<ElectricalSelectionSnapshot, ElectricalQueryFailure> {
    let model = frame.candidate().clone();
    let mut values = BTreeMap::new();
    let mut basis = Vec::new();
    for head in model.source_shards.iter().filter(|s| needed(&s.kind)) {
        let value = frame
            .value(head)
            .map_err(|e| ElectricalQueryFailure::UnavailableBasis {
                sources: vec![head.shard_id],
                reason: e.to_string(),
            })?;
        values.insert(head.shard_id, value);
        basis.push(SelectionSourceBasis {
            shard_id: head.shard_id,
            content_hash: head.content_hash.clone(),
        });
    }
    evaluate(model, values, basis, Some(frame))
}

/// A historical package anchor may survive only the existing qualified pose
/// history. Compare its result to this snapshot's captured Board, not live files.
pub(super) fn qualified_package_pose(
    snapshot: &ElectricalSelectionSnapshot,
    reference: &crate::substrate::RevisionedRef,
) -> Result<(), EngineError> {
    let invalid =
        |message: &str| EngineError::Validation(format!("captured package pose: {message}"));
    let model: &DesignModel = if let Some(frame) = &snapshot.pending_frame {
        if !Arc::ptr_eq(&snapshot.model, frame.candidate()) {
            return Err(invalid(
                "pending snapshot does not own its certified candidate",
            ));
        }
        frame.candidate()
    } else {
        if snapshot.model.journal_cursor.applied_transaction_count != snapshot.model.journal.len() {
            return Err(invalid("accepted journal tip is not the source basis"));
        }
        &snapshot.model
    };
    let object = model
        .objects
        .get(&reference.object_id)
        .filter(|object| object.domain == "board")
        .ok_or_else(|| invalid("package source is missing or foreign"))?;
    let heads: Vec<_> = model
        .source_shards
        .iter()
        .filter(|head| {
            head.shard_id == object.source_shard_id && head.kind == SourceShardKind::BoardRoot
        })
        .collect();
    let [head] = heads.as_slice() else {
        return Err(invalid("package Board ownership is ambiguous"));
    };
    let board = if let Some(frame) = &snapshot.pending_frame {
        frame.value(head)?
    } else {
        snapshot
            .source_shard_value(snapshot.revision(), head.shard_id)
            .cloned()
            .map_err(|error| invalid(&format!("captured Board unavailable: {error:?}")))?
    };
    let current = board
        .get("packages")
        .and_then(|packages| packages.get(reference.object_id.to_string()))
        .ok_or_else(|| invalid("package is not in the captured Board"))?;
    if current
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|id| id.parse::<Uuid>().ok())
        != Some(reference.object_id)
    {
        return Err(invalid("package identity differs from source membership"));
    }
    let (recorded, epoch) =
        crate::substrate::component_instance::placed_package_history::recorded_package(
            &model.journal,
            reference,
        )?;
    if epoch != object.object_revision || recorded != *current {
        return Err(invalid(
            "recorded pose or epoch differs from captured source",
        ));
    }
    Ok(())
}

pub(super) fn pinned(
    snapshot: &ElectricalSelectionSnapshot,
    id: Uuid,
    kind: &str,
    revision: crate::substrate::ObjectRevision,
) -> Result<serde_json::Value, ElectricalQueryFailure> {
    let value = if let Some(frame) = &snapshot.pending_frame {
        frame.pinned_definition(id, kind, revision)
    } else {
        let read = |head: &crate::substrate::SourceShardRef| {
            snapshot
                .source_shard_value(snapshot.revision(), head.shard_id)
                .cloned()
                .map_err(|e| {
                    EngineError::Validation(format!("captured pinned source unavailable: {e:?}"))
                })
        };
        if matches!(kind, "footprints" | "padstacks") {
            // Only this actual accepted-snapshot branch uses an empty pending
            // batch. The eight-kind owner authenticates recorded source history;
            // evaluated snapshots above always retain their real pending frame.
            let basis = crate::substrate::component_instance::pool_history::PoolHistoryBasis {
                model: &snapshot.model,
                operations: &[],
                applied_operations: &[],
            };
            if snapshot.model.journal_cursor.applied_transaction_count
                != snapshot.model.journal.len()
            {
                Err(EngineError::Validation(
                    "journal tip is not the resolved source basis".into(),
                ))
            } else {
                basis
                    .projected_definition_value_with_reader(
                        &snapshot.model.objects,
                        id,
                        revision,
                        kind,
                        &read,
                    )
                    .and_then(|value| {
                        value.ok_or_else(|| {
                            EngineError::Validation(
                                "captured physical definition revision unavailable".into(),
                            )
                        })
                    })
            }
        } else {
            snapshot
                .model
                .pool_object_revision_with_reader(id, kind, revision, &read)
        }
    };
    value.map_err(|e| ElectricalQueryFailure::UnavailableBasis {
        sources: snapshot.source_basis.iter().map(|s| s.shard_id).collect(),
        reason: e.to_string(),
    })
}

/// Full final identity and declaration checks share the ordinary rule owners.
/// The frame supplies final postimages, so no pending operations apply twice.
pub(super) fn validate(
    snapshot: &ElectricalSelectionSnapshot,
) -> Result<(), ElectricalQueryFailure> {
    if snapshot.pending_frame.is_none() {
        return Err(ElectricalQueryFailure::UnavailableBasis {
            sources: vec![],
            reason: "certified pending sources required".into(),
        });
    }
    let check = || -> Result<(), EngineError> {
        if let Some(source) = &snapshot.schematic {
            crate::substrate::schematic_net_declarations::validate_source(&snapshot.model, source)?;
        }
        crate::substrate::electrical_identity_validation::validate_basis(
            &snapshot.model,
            &snapshot.basis,
            &snapshot.logical,
        )
    };
    check().map_err(|error| ElectricalQueryFailure::UnavailableBasis {
        sources: snapshot.source_basis.iter().map(|s| s.shard_id).collect(),
        reason: error.to_string(),
    })
}
