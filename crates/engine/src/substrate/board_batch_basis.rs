//! Pure final board-source basis, using canonical application and serialization.
use super::*;

pub(crate) fn project_board_basis(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<DesignModel, EngineError> {
    let mut source = batch.clone();
    source.operations.retain(|op| {
        !matches!(
            op,
            Operation::SetZoneFill { .. } | Operation::DeleteZoneFill { .. }
        )
    });
    let mut candidate = model.clone();
    if !source.operations.is_empty() {
        let mut board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
        let mut touched = false;
        {
            let mut pass = super::board_list_journal_ops::BoardPass::new(&mut board);
            for op in &source.operations {
                touched |= pass.apply(op)?;
            }
        }
        candidate.commit_without_direct_policy(source)?;
        if touched {
            let shard = candidate
                .source_shards
                .iter_mut()
                .find(|s| s.kind == SourceShardKind::BoardRoot)
                .ok_or_else(|| EngineError::Validation("missing board source".into()))?;
            shard.content_hash = super::journal::canonical_json_hash(&board)?;
        }
    }
    super::journal::sort_source_shards(&mut candidate.source_shards);
    candidate.model_revision = compute_model_revision(
        &candidate.project.project_id,
        &candidate.source_shards,
        &candidate.objects,
    );
    Ok(candidate)
}

pub(super) fn compute_model_revision(
    project_id: &Uuid,
    shards: &[SourceShardRef],
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> ModelRevision {
    let mut hasher = Sha256::new();
    hasher.update(project_id.as_bytes());
    for shard in shards {
        if matches!(
            shard.kind,
            SourceShardKind::ArtifactMetadata
                | SourceShardKind::ArtifactRun
                | SourceShardKind::OutputJobRun
                | SourceShardKind::CheckRun
                | SourceShardKind::ZoneFill
                | SourceShardKind::ImportMap
                | SourceShardKind::ProposalMetadata
                | SourceShardKind::ForwardAnnotationReview
        ) {
            continue;
        }
        hasher.update(shard.relative_path.as_bytes());
        hasher.update(shard.content_hash.as_bytes());
    }
    for (object_id, object) in objects {
        hasher.update(object_id.as_bytes());
        hasher.update(object.object_revision.0.to_be_bytes());
        hasher.update(object.source_shard_id.as_bytes());
    }
    ModelRevision(sha256_digest_hex(hasher.finalize().as_slice()))
}

// History restores the exact saved geometry, but source-object revisions may
// advance during compensation. Bind only imported evidence to that actual basis.
pub(super) fn bind_imported_fill_history(
    model: &DesignModel,
    batch: &mut OperationBatch,
) -> Result<(), EngineError> {
    if !batch.operations.iter().any(|op| matches!(op, Operation::SetZoneFill{zone_fill,..} if zone_fill["provenance"].as_str().is_some_and(|p|p.starts_with("datum:kicad-saved-fill:v1;")))) {return Ok(());}
    let basis = project_board_basis(model, batch)?;
    for op in &mut batch.operations {
        if let Operation::SetZoneFill {
            zone_id, zone_fill, ..
        } = op
            && zone_fill["provenance"]
                .as_str()
                .is_some_and(|p| p.starts_with("datum:kicad-saved-fill:v1;"))
        {
            let object = basis.objects.get(zone_id).ok_or_else(|| {
                EngineError::Validation("imported fill history source missing".into())
            })?;
            zone_fill["model_revision"] = serde_json::to_value(&basis.model_revision)?;
            zone_fill["source_zone_revision"] = serde_json::to_value(object.object_revision)?;
        }
    }
    Ok(())
}

// The geometry and provenance remain byte-exact inverse payloads. Only the
// certified final revision fields may differ during imported-fill restoration.
pub(super) fn matches_imported_fill_compensation(
    transaction: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    let mut expected = target.inverse_operations.clone();
    for operation in &mut expected {
        if let Operation::SetZoneFill {
            zone_id, zone_fill, ..
        } = operation
        {
            if !zone_fill["provenance"]
                .as_str()
                .is_some_and(|p| p.starts_with("datum:kicad-saved-fill:v1;"))
            {
                continue;
            }
            let mut revision = Some(0_u64);
            for record in prior.iter().chain(std::iter::once(transaction)) {
                if record.diff.deleted.contains(zone_id) {
                    revision = None;
                }
                if record.diff.created.contains(zone_id) {
                    revision = Some(0_u64);
                }
                if record.diff.modified.contains(zone_id) {
                    revision = revision.and_then(|r| r.checked_add(1));
                }
            }
            let Some(revision) = revision else {
                return false;
            };
            zone_fill["model_revision"] = serde_json::json!(transaction.after_model_revision);
            zone_fill["source_zone_revision"] = serde_json::json!(revision);
        }
    }
    transaction.operations == expected
}
