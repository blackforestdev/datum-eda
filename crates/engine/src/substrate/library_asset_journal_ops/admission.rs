//! Library admission over exact pending sources and a canonical evaluated frame.
//! No source writes, fabricated journal entries, publisher approval or policy defaults.
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::EngineError,
    library_assets::{
        AssetKind, AssetRevisionRef, VerifiedAsset,
        lock::{ComponentBinding, ProjectLibraryLock},
        payload_reference_slots,
    },
    revision::ProjectWriteLease,
    substrate::{
        ComponentCounterpartStatus, ComponentInstanceAuthority, ComponentInstanceShard,
        DesignModel, ElectricalOccurrence, ElectricalSelectionSnapshot, ModelRevision, Operation,
        OperationBatch, SourceShardKind,
        library_asset_operations::{LibraryAssetOperation, refusal},
        library_asset_resolver::{
            LibraryAssetLimits, LibraryAssetSnapshot, counter_policy, read_state, resolve_state,
            restrict_limits,
        },
        pool_journal_ops::apply_pool_shard_operation,
        source_shard::validate_source_shard_ownership_path,
    },
};

mod bindings;
mod physical;
mod projection;
use bindings::{coherent_component_records, validate_frame};
pub(in crate::substrate) use physical::geometry_only as qualify_geometry_delta;
pub(crate) use physical::planned_sources as planned_physical_sources;
use projection::{pending_pool, pool_index, project_header};

/// An input preflight is deliberately not a completed admission. The canonical
/// transaction owner must finish it against its evaluated final source frame
/// before publishing; raw operations cannot supply that sealed frame on the wire.
pub(in crate::substrate) struct PreparedLibraryAdmission {
    project: Uuid,
    before: ModelRevision,
    batch: Uuid,
    batch_digest: [u8; 32],
    operations_digest: [u8; 32],
    snapshot: LibraryAssetSnapshot,
    pool: Option<BTreeMap<String, Value>>,
    physical_before: Option<BTreeMap<String, Value>>,
    changed: BTreeSet<Uuid>,
    limits: LibraryAssetLimits,
    manifest_hash: String,
}

pub(in crate::substrate) struct LibraryAdmissionProof {
    _private: (),
}

/// Only a lease-qualified admission can authorize provisional application.
/// This is neither a metadata proof nor permission to publish the candidate.
pub(in crate::substrate) struct LibraryEvaluationPermit<'a> {
    admission: &'a PreparedLibraryAdmission,
}
impl LibraryEvaluationPermit<'_> {
    pub(in crate::substrate) fn validate_operations(
        &self,
        operations: &[Operation],
    ) -> Result<(), EngineError> {
        if fingerprint(&operations)? != self.admission.operations_digest {
            return Err(refusal(
                "library evaluation operations changed after admission",
            ));
        }
        Ok(())
    }
}

pub(in crate::substrate) fn prepare_under_lease(
    root: &Path,
    lease: &ProjectWriteLease,
    model: &DesignModel,
    batch: &OperationBatch,
    limits: LibraryAssetLimits,
) -> Result<Option<PreparedLibraryAdmission>, EngineError> {
    lease.validate(root)?;
    let operations: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    if operations.is_empty() && model.project.schema_version != Some(2) {
        return Ok(None);
    }
    if batch
        .operations
        .iter()
        .any(|op| matches!(op, Operation::GuardObjectRevision { .. }))
    {
        return Err(refusal(
            "library admission requires commit-owned operations after guard validation and stripping",
        ));
    }
    let limits = restrict_limits(limits)?;
    let manifest_hash = project_header(root, model, &batch.operations, limits)?;
    if operations.len() > 1 {
        return Err(refusal("multiple library state transitions in one batch"));
    }
    let previous = if model.project.schema_version == Some(2) {
        Some(read_state(root, limits)?)
    } else {
        None
    };
    let old_lock = previous
        .as_ref()
        .map(|state| state.decode(limits.metadata_bytes).map(|(lock, _)| lock))
        .transpose()?;
    let (state, files) = if let Some(operation) = operations.first() {
        if operation.owner_id() != model.project.project_id {
            return Err(refusal("foreign project library owner"));
        }
        if matches!(operation, LibraryAssetOperation::AdoptProfile { .. })
            && batch.operations.iter().any(|op| {
                !matches!(
                    op,
                    Operation::LibraryAssets { .. } | Operation::GuardObjectRevision { .. }
                )
            })
        {
            return Err(refusal(
                "format adoption must be a separate format-only transaction",
            ));
        }
        super::validate_preimages(root, operation, limits)?;
        if let LibraryAssetOperation::AdoptProfile { preimage, .. } = operation {
            crate::library_assets::verify_native_snapshot(
                preimage.as_bytes(),
                &serde_json::from_str(preimage)?,
            )?;
        }
        (
            operation.state()?,
            operation.content_map(
                limits.metadata_bytes,
                limits.blob_bytes,
                limits.files,
                limits.total_bytes,
            )?,
        )
    } else {
        (
            previous
                .as_ref()
                .ok_or_else(|| refusal("managed state missing"))?,
            BTreeMap::new(),
        )
    };
    let snapshot = resolve_state(root, state, &files, limits)?;
    coherent_component_records(snapshot.lock())?;
    let managed_paths: BTreeSet<_> = snapshot
        .lock()
        .assets
        .iter()
        .chain(old_lock.iter().flat_map(|lock| &lock.assets))
        .map(|entry| entry.source_path.clone())
        .collect();
    let (pool, before_pool) = if managed_paths.is_empty() {
        (BTreeMap::new(), BTreeMap::new())
    } else {
        pending_pool(root, model, &batch.operations, limits, &managed_paths)?
    };
    let index = pool_index(&pool)?;
    let mut changed = BTreeSet::new();
    for entry in &snapshot.lock().assets {
        let asset = &snapshot.assets()[&entry.asset];
        let value = pool
            .get(&entry.source_path)
            .ok_or_else(|| refusal("locked materialization missing from pending batch"))?;
        if index.get(&entry.asset.asset_id) != Some(&entry.source_path) {
            return Err(refusal("locked identity and materialization path disagree"));
        }
        let counters = counter_policy(
            asset.revision().kind,
            asset.revision().payload_schema_version,
        )?;
        asset.payload().verify_materialization(value, counters)?;
        if before_pool.get(&entry.source_path).is_none_or(|old| {
            asset
                .payload()
                .verify_materialization(old, counters)
                .is_err()
        }) {
            changed.insert(entry.asset.asset_id);
        }
    }
    if let Some(old) = old_lock {
        let retained: BTreeSet<_> = snapshot
            .lock()
            .assets
            .iter()
            .map(|a| a.asset.asset_id)
            .collect();
        for asset in old
            .assets
            .iter()
            .filter(|a| !retained.contains(&a.asset.asset_id))
        {
            if pool.contains_key(&asset.source_path) {
                return Err(refusal(
                    "managed assets cannot silently become legacy while their source remains",
                ));
            }
            changed.insert(asset.asset.asset_id);
        }
    }
    Ok(Some(PreparedLibraryAdmission {
        project: model.project.project_id,
        before: model.model_revision.clone(),
        batch: batch.batch_id,
        batch_digest: fingerprint(&(batch.batch_id, &batch.provenance, &batch.operations))?,
        operations_digest: fingerprint(&batch.operations)?,
        snapshot,
        pool: (!managed_paths.is_empty()).then_some(pool),
        physical_before: operations
            .iter()
            .any(|op| matches!(op, LibraryAssetOperation::SetState { .. }))
            .then_some(before_pool),
        changed,
        limits,
        manifest_hash,
    }))
}

impl PreparedLibraryAdmission {
    pub(in crate::substrate) fn derive_component_bindings(
        &self,
        root: &Path,
        lease: &ProjectWriteLease,
        before: &DesignModel,
        candidate: &DesignModel,
        writes: &[crate::substrate::journal::StagedShardWrite],
    ) -> Result<Vec<ComponentBinding>, EngineError> {
        let requested = ElectricalSelectionSnapshot::required_pending_sources(candidate);
        let frame =
            crate::substrate::journal_source_frame::VerifiedPendingSourceFrame::capture_evaluated(
                root,
                lease,
                std::sync::Arc::new(before.clone()),
                std::sync::Arc::new(candidate.clone()),
                writes,
                &requested,
                crate::substrate::journal_source_frame::SourceFrameLimits {
                    per_source: self.limits.metadata_bytes.max(self.limits.blob_bytes),
                    total_bytes: self.limits.total_bytes,
                    max_sources: self.limits.files,
                },
            )?;
        let sources = ElectricalSelectionSnapshot::capture_pending(frame).map_err(query_error)?;
        sources.validate_pending_electrical().map_err(query_error)?;
        bindings::derive_frame(
            candidate,
            &self.snapshot,
            &sources,
            &self.changed,
            self.limits,
        )
    }

    pub(in crate::substrate) fn evaluation_permit(
        &self,
        model: &DesignModel,
        batch: &OperationBatch,
    ) -> Result<LibraryEvaluationPermit<'_>, EngineError> {
        if model.project.project_id != self.project
            || model.model_revision != self.before
            || fingerprint(&(batch.batch_id, &batch.provenance, &batch.operations))?
                != self.batch_digest
        {
            return Err(refusal("library candidate differs from its admitted input"));
        }
        Ok(LibraryEvaluationPermit { admission: self })
    }

    /// Commit owns the actual evaluated candidate and staged files. A caller
    /// cannot substitute a snapshot assembled before canonical application.
    pub(in crate::substrate) fn finish_evaluated(
        self,
        root: &Path,
        lease: &ProjectWriteLease,
        before: &DesignModel,
        candidate: &DesignModel,
        writes: &[crate::substrate::journal::StagedShardWrite],
    ) -> Result<LibraryAdmissionProof, EngineError> {
        let (sources, frame) = {
            let requested = ElectricalSelectionSnapshot::required_pending_sources(candidate);
            let frame = crate::substrate::journal_source_frame::VerifiedPendingSourceFrame::capture_evaluated(
                root, lease, std::sync::Arc::new(before.clone()),
                std::sync::Arc::new(candidate.clone()), writes, &requested,
                crate::substrate::journal_source_frame::SourceFrameLimits {
                    per_source: self.limits.metadata_bytes.max(self.limits.blob_bytes),
                    total_bytes: self.limits.total_bytes,
                    max_sources: self.limits.files,
                },
            )?;
            (
                ElectricalSelectionSnapshot::capture_pending(frame.clone()).map_err(query_error)?,
                frame,
            )
        };
        sources.validate_pending_electrical().map_err(query_error)?;
        if self.physical_before.is_some() {
            physical::verify_final(&frame)?;
        }
        if let Some(before_pool) = &self.physical_before {
            let impacts = super::physical_impact::assess(
                candidate,
                &sources,
                before_pool,
                self.snapshot.lock(),
                self.snapshot.assets(),
            )?;
            super::physical_impact::require_supported(&impacts)?;
        }
        self.finish(root, lease, before, candidate, Some(&sources))
    }

    fn finish(
        self,
        root: &Path,
        lease: &ProjectWriteLease,
        before: &DesignModel,
        candidate: &DesignModel,
        sources: Option<&ElectricalSelectionSnapshot>,
    ) -> Result<LibraryAdmissionProof, EngineError> {
        lease.validate(root)?;
        if before.project.project_id != self.project
            || before.model_revision != self.before
            || candidate.project.project_id != self.project
        {
            return Err(refusal("library admission model basis changed"));
        }
        if candidate.model_revision
            != crate::substrate::compute_model_revision(
                &self.project,
                &candidate.source_shards,
                &candidate.objects,
            )
        {
            return Err(refusal(
                "pending library frame has an incoherent model revision",
            ));
        }
        let transaction = candidate
            .journal
            .last()
            .ok_or_else(|| refusal("canonical pending transaction missing"))?;
        if transaction.batch_id != self.batch
            || transaction.before_model_revision != self.before
            || transaction.after_model_revision != candidate.model_revision
            || before.journal_cursor.applied_transaction_count != before.journal.len()
            || candidate.journal_cursor != before.journal_cursor
            || candidate.journal.len() != before.journal.len() + 1
            || candidate.journal[..before.journal.len()] != before.journal[..]
            || fingerprint(&(
                transaction.batch_id,
                &transaction.provenance,
                &transaction.operations,
            ))? != self.batch_digest
        {
            return Err(refusal(
                "canonical transaction differs from preflight operations",
            ));
        }
        verify_metadata_heads(candidate, &self.snapshot, &self.manifest_hash)?;
        if let Some(pool) = &self.pool {
            let sources = sources
                .ok_or_else(|| refusal("canonical evaluated pending source frame required"))?;
            let heads: BTreeSet<_> = candidate
                .source_shards
                .iter()
                .filter(|s| s.kind == SourceShardKind::Pool)
                .map(|s| s.relative_path.clone())
                .collect();
            if heads != pool.keys().cloned().collect() {
                return Err(refusal("pending pool source inventory differs"));
            }
            for shard in candidate
                .source_shards
                .iter()
                .filter(|s| s.kind == SourceShardKind::Pool)
            {
                let value = sources
                    .source_shard_value(&candidate.model_revision, shard.shard_id)
                    .map_err(query_error)?;
                if pool.get(&shard.relative_path) != Some(value) {
                    return Err(refusal(
                        "evaluated source differs from exact pending pool projection",
                    ));
                }
            }
            // The evaluated source frame owns physical qualification. The
            // unchanged-pool compensation gate validate_library_basis must not
            // be invoked or widened for a library adoption transaction.
            validate_frame(
                candidate,
                &self.snapshot,
                sources,
                &self.changed,
                self.limits,
            )?;
        }
        Ok(LibraryAdmissionProof { _private: () })
    }
}

/// Cold resolution uses the same managed projection and occurrence checks after
/// integrity recovery. This is not an assertion that legacy assets were migrated.
pub(in crate::substrate) fn validate_resolved(
    root: &Path,
    lease: &ProjectWriteLease,
    model: &DesignModel,
    snapshot: &LibraryAssetSnapshot,
    sources: Option<&ElectricalSelectionSnapshot>,
    limits: LibraryAssetLimits,
) -> Result<LibraryAdmissionProof, EngineError> {
    lease.validate(root)?;
    coherent_component_records(snapshot.lock())?;
    if model.journal_cursor.applied_transaction_count != model.journal.len()
        || model.model_revision
            != crate::substrate::compute_model_revision(
                &model.project.project_id,
                &model.source_shards,
                &model.objects,
            )
    {
        return Err(refusal(
            "cold managed model differs from its complete canonical revision",
        ));
    }
    let limits = restrict_limits(limits)?;
    let manifest_hash = project_header(root, model, &[], limits)?;
    verify_metadata_heads(model, snapshot, &manifest_hash)?;
    if snapshot.lock().assets.is_empty() && snapshot.lock().bindings.is_empty() {
        return Ok(LibraryAdmissionProof { _private: () });
    }
    let paths = snapshot
        .lock()
        .assets
        .iter()
        .map(|entry| entry.source_path.clone())
        .collect();
    pending_pool(root, model, &[], limits, &paths)?;
    let sources = sources.ok_or_else(|| refusal("canonical current source frame required"))?;
    validate_frame(model, snapshot, sources, &BTreeSet::new(), limits)?;
    Ok(LibraryAdmissionProof { _private: () })
}

fn verify_metadata_heads(
    model: &DesignModel,
    snapshot: &LibraryAssetSnapshot,
    manifest_hash: &str,
) -> Result<(), EngineError> {
    if model.project.schema_version != Some(2) {
        return Err(refusal(
            "managed library requires the adopted project profile",
        ));
    }
    if model
        .source_shards
        .iter()
        .any(|shard| shard.kind == SourceShardKind::LibraryContent)
    {
        return Err(refusal(
            "immutable library content cannot become an independent source head",
        ));
    }
    for (kind, path, bytes) in [
        (
            SourceShardKind::LibraryLock,
            crate::library_assets::lock::LOCK_RELATIVE_PATH,
            crate::library_assets::encode_metadata(snapshot.lock())?,
        ),
        (
            SourceShardKind::LibraryCatalog,
            crate::substrate::library_asset_operations::CATALOG_RELATIVE_PATH,
            crate::library_assets::encode_metadata(snapshot.catalog())?,
        ),
    ] {
        let heads: Vec<_> = model
            .source_shards
            .iter()
            .filter(|s| s.kind == kind || s.relative_path == path)
            .collect();
        let [head] = heads.as_slice() else {
            return Err(refusal(
                "library metadata source head is absent or duplicated",
            ));
        };
        if head.kind != kind
            || head.relative_path != path
            || head.content_hash != crate::substrate::sha256_hex(&bytes)
        {
            return Err(refusal(
                "library metadata source head differs from exact retained state",
            ));
        }
    }
    let manifest = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::ProjectManifest)
        .ok_or_else(|| refusal("project manifest source head missing"))?;
    if manifest.content_hash != manifest_hash {
        return Err(refusal(
            "project header differs from certified library postimage",
        ));
    }
    Ok(())
}

fn fingerprint(value: &impl Serialize) -> Result<[u8; 32], EngineError> {
    struct Sink(Sha256);
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink(Sha256::new());
    serde_json::to_writer(&mut sink, value)?;
    Ok(sink.0.finalize().into())
}
fn query_error(error: crate::substrate::ElectricalQueryFailure) -> EngineError {
    refusal(format!("unqualified canonical source frame: {error:?}"))
}

#[cfg(test)]
mod tests;
