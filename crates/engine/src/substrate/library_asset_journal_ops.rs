//! Immutable content and lock/catalog staging, invoked only by the journal.
pub(in crate::substrate) mod admission;
pub(in crate::substrate) mod binding_history;
pub(in crate::substrate) mod composition;
pub(in crate::substrate) mod history;
pub(crate) mod physical_impact;
pub(crate) use admission::planned_physical_sources;
pub(crate) use history::qualify_plan as qualify_physical_plan;

use std::{io::Read, path::Path};

use super::{
    EngineError, OperationBatch, SourceShardKind,
    journal::{StagedShardWrite, stage_new_shard_write},
    library_asset_operations::{CATALOG_RELATIVE_PATH, LibraryAssetOperation, refusal},
    library_asset_resolver::{LibraryAssetLimits, owned_path, resolve_state},
    sha256_hex,
    source_shard::validate_source_shard_ownership_path,
};
use crate::library_assets::lock::LOCK_RELATIVE_PATH;

/// Ordinary preflight checks precede the sealed library admission and its
/// mandatory evaluated-source proof in the canonical journaled commit. A raw
/// typed operation or compensation cannot bypass either stage.
pub(super) fn validate_batch(
    model: &super::DesignModel,
    root: &Path,
    batch: &OperationBatch,
) -> Result<(), EngineError> {
    let operations: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            super::Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    if operations.is_empty() {
        return Ok(());
    }
    let limits = super::library_asset_resolver::server_policy()?;
    if operations.len() != 1 {
        return Err(refusal(
            "one library state transition is allowed per transaction",
        ));
    }
    let op = operations[0];
    if op.owner_id() != model.project.project_id {
        return Err(refusal("foreign library project owner"));
    }
    if matches!(op, LibraryAssetOperation::AdoptProfile { .. }) && batch.operations.len() != 1 {
        return Err(refusal(
            "profile adoption requires a separate format-only transaction",
        ));
    }
    validate_preimages(root, op, limits)?;
    Ok(())
}

/// LibraryContent writes are immutable referenced partitions, not independent
/// source heads. Parent journal hash registration must skip that kind; lock and
/// catalog hashes bind the references and the resolver verifies their bytes.
pub(super) fn stage(
    root: &Path,
    batch: &OperationBatch,
    op: &LibraryAssetOperation,
    limits: LibraryAssetLimits,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    let limits = super::library_asset_resolver::restrict_limits(limits)?;
    let files = op.content_map(
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.total_bytes,
    )?;
    let state = op.state()?;
    resolve_state(root, state, &files, limits)?;
    for (relative, bytes) in files {
        let destination = owned_path(root, &relative)?;
        if destination.exists() {
            if !existing_matches(&destination, bytes)? {
                return Err(refusal("existing immutable content has different bytes"));
            }
            continue;
        }
        validate_source_shard_ownership_path(&SourceShardKind::LibraryContent, &relative)?;
        let path = root
            .join(".datum/stage")
            .join(batch.batch_id.to_string())
            .join(&relative);
        if let Some(parent) = path.parent() {
            crate::revision::create_directory_durable(parent)?;
        }
        std::fs::write(&path, bytes)?;
        std::fs::File::open(&path)?.sync_all()?;
        if let Some(parent) = path.parent() {
            super::journal_io::sync_directory(parent)?;
        }
        staged.push(StagedShardWrite {
            destination,
            staged: Some(path),
            kind: SourceShardKind::LibraryContent,
            relative_path: relative,
            content_hash: sha256_hex(bytes),
            schema_version: None,
            delete: false,
        });
    }
    for (kind, relative, bytes) in [
        (
            SourceShardKind::LibraryLock,
            LOCK_RELATIVE_PATH,
            &state.lock,
        ),
        (
            SourceShardKind::LibraryCatalog,
            CATALOG_RELATIVE_PATH,
            &state.catalog,
        ),
    ] {
        let value = serde_json::from_slice(bytes)?;
        owned_path(root, relative)?;
        staged.push(stage_new_shard_write(root, batch, kind, relative, &value)?);
    }
    Ok(())
}

fn existing_matches(path: &Path, expected: &[u8]) -> Result<bool, EngineError> {
    let mut file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != expected.len() as u64 {
        return Ok(false);
    }
    let mut chunk = [0u8; 16 * 1024];
    let mut offset = 0;
    while offset < expected.len() {
        let size = chunk.len().min(expected.len() - offset);
        let n = file.read(&mut chunk[..size])?;
        if n == 0 || chunk[..n] != expected[offset..offset + n] {
            return Ok(false);
        }
        offset += n;
    }
    let mut extra = [0u8; 1];
    Ok(file.read(&mut extra)? == 0)
}

pub(super) fn current_preimages(
    root: &Path,
    limits: LibraryAssetLimits,
) -> Result<super::library_asset_operations::LibraryStateValues, EngineError> {
    super::library_asset_resolver::read_state(root, limits)
}

/// Admission uses complete state preimages. It does not equate a candidate save
/// with project adoption; unchanged lock bytes leave the project pinned.
pub(crate) fn validate_preimages(
    root: &Path,
    op: &LibraryAssetOperation,
    limits: LibraryAssetLimits,
) -> Result<(), EngineError> {
    let limits = super::library_asset_resolver::restrict_limits(limits)?;
    match op {
        LibraryAssetOperation::AdoptProfile {
            preimage, state, ..
        } => {
            let actual =
                super::library_asset_resolver::manifest_preimage(root, limits.metadata_bytes)?;
            if actual != *preimage {
                return Err(refusal("stale exact project manifest preimage"));
            }
            let manifest: serde_json::Value = serde_json::from_str(preimage)?;
            if manifest
                .get("schema_version")
                .and_then(serde_json::Value::as_u64)
                != Some(1)
                || manifest.get("library_lock").is_some()
                || root.join(LOCK_RELATIVE_PATH).exists()
                || root.join(CATALOG_RELATIVE_PATH).exists()
            {
                return Err(refusal(
                    "profile already adopted or reserved metadata paths exist",
                ));
            }
            let (lock, _) = state.decode(limits.metadata_bytes)?;
            if !lock.assets.is_empty()
                || !lock.bindings.is_empty()
                || !lock.source_manifests.is_empty()
            {
                return Err(refusal(
                    "format adoption must establish a separate empty managed baseline",
                ));
            }
        }
        LibraryAssetOperation::SetState { previous, .. } => {
            if current_preimages(root, limits)? != *previous {
                return Err(refusal("stale library lock/catalog preimage"));
            }
        }
        LibraryAssetOperation::RefuseProfileInverse { .. } => {
            return Err(refusal("profile adoption is forward-only"));
        }
    }
    let files = op.content_map(
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.total_bytes,
    )?;
    let snapshot = resolve_state(root, op.state()?, &files, limits)?;
    if let LibraryAssetOperation::AdoptProfile { .. } = op
        && (!snapshot.manifest().roots.is_empty()
            || !snapshot.manifest().objects.is_empty()
            || !snapshot.manifest().blobs.is_empty())
    {
        return Err(refusal(
            "format adoption cannot silently acquire library assets",
        ));
    }
    Ok(())
}

/// Replay the mutable metadata postimages through their existing source owner.
/// Content bytes remain immutable inventory members and integrity postimages.
pub(in crate::substrate) fn replay_metadata_sources(
    root: &Path,
    heads: &mut Vec<super::SourceShardRef>,
    journal: &[super::TransactionRecord],
) -> Result<(), EngineError> {
    let state = journal
        .iter()
        .rev()
        .flat_map(|record| record.operations.iter().rev())
        .find_map(|op| match op {
            super::Operation::LibraryAssets { operation } => Some(operation.state()),
            _ => None,
        })
        .transpose()?;
    let Some(state) = state else {
        return Ok(());
    };
    let limits = super::library_asset_resolver::server_policy()?;
    resolve_state(root, state, &std::collections::BTreeMap::new(), limits)?;
    for (kind, path, bytes) in [
        (
            SourceShardKind::LibraryLock,
            LOCK_RELATIVE_PATH,
            &state.lock,
        ),
        (
            SourceShardKind::LibraryCatalog,
            CATALOG_RELATIVE_PATH,
            &state.catalog,
        ),
    ] {
        let head = super::source_shard_ref_builders::source_shard_ref_for_bytes(
            kind.clone(),
            root.join(path),
            path.into(),
            Some(1),
            bytes,
            "library_metadata_unavailable",
        )
        .map_err(|diagnostic| refusal(diagnostic.message))?;
        if let Some(previous) = heads.iter_mut().find(|s| s.relative_path == path) {
            if previous.kind != kind {
                return Err(refusal("replayed library metadata owner collision"));
            }
            *previous = head;
        } else {
            heads.push(head);
        }
    }
    Ok(())
}
