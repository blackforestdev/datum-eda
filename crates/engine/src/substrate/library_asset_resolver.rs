//! Read-only resolution of the lock/catalog and their immutable content.
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

use super::library_asset_operations::{CATALOG_RELATIVE_PATH, LibraryStateValues, refusal};
use crate::{
    error::EngineError,
    library_assets::{
        AssetDigest, AssetRevisionRef, ClosureLimits, LocalCounters, VerifiedAsset,
        inspect_closure,
        lock::{LOCK_RELATIVE_PATH, ProjectLibraryLock},
        manifest::{LibraryDiscovery, LibraryReleaseManifest},
        review::AssetReview,
    },
};

pub(crate) mod release;

/// Explicit service/transport limits; no hidden performance acceptance numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryAssetLimits {
    pub metadata_bytes: usize,
    pub blob_bytes: usize,
    pub files: usize,
    pub depth: usize,
    pub total_bytes: usize,
}

static SERVER_POLICY: OnceLock<LibraryAssetLimits> = OnceLock::new();

/// Only trusted server initialization configures policy. No wire request or
/// publisher metadata may enable the service or raise its resource bounds.
pub(crate) fn configure_server_policy(mut policy: LibraryAssetLimits) -> Result<(), EngineError> {
    if [
        policy.metadata_bytes,
        policy.blob_bytes,
        policy.files,
        policy.depth,
        policy.total_bytes,
    ]
    .contains(&0)
    {
        return Err(refusal(
            "server library policy must have explicit positive limits",
        ));
    }
    policy.metadata_bytes = policy.metadata_bytes.min(policy.total_bytes);
    policy.blob_bytes = policy.blob_bytes.min(policy.total_bytes);
    let actual = SERVER_POLICY.get_or_init(|| policy);
    if *actual != policy {
        return Err(refusal(
            "server library policy already configured differently",
        ));
    }
    Ok(())
}

pub(crate) fn server_policy() -> Result<LibraryAssetLimits, EngineError> {
    SERVER_POLICY
        .get()
        .copied()
        .ok_or_else(|| refusal("unsupported library profile: server policy unconfigured"))
}

pub(crate) fn restrict_limits(
    requested: LibraryAssetLimits,
) -> Result<LibraryAssetLimits, EngineError> {
    let cap = server_policy()?;
    let mut limits = LibraryAssetLimits {
        metadata_bytes: requested.metadata_bytes.min(cap.metadata_bytes),
        blob_bytes: requested.blob_bytes.min(cap.blob_bytes),
        files: requested.files.min(cap.files),
        depth: requested.depth.min(cap.depth),
        total_bytes: requested.total_bytes.min(cap.total_bytes),
    };
    limits.metadata_bytes = limits.metadata_bytes.min(limits.total_bytes);
    limits.blob_bytes = limits.blob_bytes.min(limits.total_bytes);
    if [
        limits.metadata_bytes,
        limits.blob_bytes,
        limits.files,
        limits.depth,
        limits.total_bytes,
    ]
    .contains(&0)
    {
        return Err(refusal("library request limits must be positive"));
    }
    Ok(limits)
}

pub(crate) struct LibraryAssetSnapshot {
    lock: ProjectLibraryLock,
    catalog: LibraryDiscovery,
    manifest: LibraryReleaseManifest,
    assets: BTreeMap<AssetRevisionRef, VerifiedAsset>,
    reviews: BTreeMap<AssetDigest, AssetReview>,
}

impl LibraryAssetSnapshot {
    pub(crate) fn lock(&self) -> &ProjectLibraryLock {
        &self.lock
    }
    pub(crate) fn catalog(&self) -> &LibraryDiscovery {
        &self.catalog
    }
    pub(crate) fn manifest(&self) -> &LibraryReleaseManifest {
        &self.manifest
    }
    pub(crate) fn assets(&self) -> &BTreeMap<AssetRevisionRef, VerifiedAsset> {
        &self.assets
    }
    pub(crate) fn reviews(&self) -> &BTreeMap<AssetDigest, AssetReview> {
        &self.reviews
    }
}

pub(crate) fn read_state(
    root: &Path,
    limits: LibraryAssetLimits,
) -> Result<LibraryStateValues, EngineError> {
    let limits = restrict_limits(limits)?;

    let empty = BTreeMap::new();
    let mut used = 0;
    Ok(LibraryStateValues {
        lock: read_charge(
            root,
            LOCK_RELATIVE_PATH,
            &empty,
            limits.metadata_bytes,
            &mut used,
            limits.total_bytes,
        )?,
        catalog: read_charge(
            root,
            CATALOG_RELATIVE_PATH,
            &empty,
            limits.metadata_bytes,
            &mut used,
            limits.total_bytes,
        )?,
    })
}

pub(crate) fn resolve_state(
    root: &Path,
    state: &LibraryStateValues,
    pending: &BTreeMap<String, &[u8]>,
    limits: LibraryAssetLimits,
) -> Result<LibraryAssetSnapshot, EngineError> {
    let limits = restrict_limits(limits)?;
    let mut used = state
        .lock
        .capacity()
        .checked_add(state.catalog.capacity())
        .ok_or_else(|| refusal("aggregate library input size overflow"))?;
    for bytes in pending.values() {
        used = used
            .checked_add(bytes.len())
            .ok_or_else(|| refusal("aggregate library input size overflow"))?;
    }
    if used > limits.total_bytes {
        return Err(refusal("aggregate library raw input exceeds server policy"));
    }

    if limits.metadata_bytes == 0
        || limits.blob_bytes == 0
        || limits.files == 0
        || limits.depth == 0
    {
        return Err(refusal(
            "library service limits must be explicitly positive",
        ));
    }
    let (lock, catalog) = state.decode(limits.metadata_bytes)?;
    let active: Vec<_> = lock.assets.iter().map(|a| a.asset.clone()).collect();
    let release::LibraryReleaseSnapshot {
        manifest,
        assets,
        reviews,
    } = release::load(
        root, "library/", &catalog, pending, &active, limits, &mut used,
    )?;
    let bounds = ClosureLimits {
        max_nodes: limits.files,
        max_depth: limits.depth,
    };
    let closure = inspect_closure(&active, &assets, bounds)?;
    lock.verify_closure(&closure)?;
    if !closure.graph_report().valid {
        return Err(refusal(format!(
            "active library graph invalid: {:?}",
            closure.graph_report().diagnostics
        )));
    }
    Ok(LibraryAssetSnapshot {
        lock,
        catalog,
        manifest,
        assets,
        reviews,
    })
}

pub(super) fn blob_path(digest: &AssetDigest) -> String {
    format!("library/blobs/sha256/{}", digest.hex())
}

pub(crate) use super::source_shard::io::owned_path;

pub(super) fn read_content(
    root: &Path,
    relative: &str,
    pending: &BTreeMap<String, &[u8]>,
    limit: usize,
) -> Result<Vec<u8>, EngineError> {
    if let Some(bytes) = pending.get(relative) {
        if bytes.len() > limit {
            return Err(refusal("staged content exceeds declared read limit"));
        }
        return Ok(bytes.to_vec());
    }
    super::source_shard::io::read_regular_file(root, relative, limit)
}

/// Exact input bytes for guarded forward-only format adoption.
pub(crate) fn manifest_preimage(root: &Path, limit: usize) -> Result<String, EngineError> {
    String::from_utf8(read_content(root, "project.json", &BTreeMap::new(), limit)?)
        .map_err(|_| refusal("project manifest is not UTF-8"))
}

pub(crate) fn validate_profile_header(value: &serde_json::Value) -> Result<(), EngineError> {
    match value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(2)
            if value.get("library_lock")
                == Some(&serde_json::json!({"path":LOCK_RELATIVE_PATH,"schema_version":1})) =>
        {
            server_policy()?;
            Ok(())
        }
        None | Some(0 | 1) if value.get("library_lock").is_none() => Ok(()),
        _ => Err(refusal(
            "unsupported native project schema_version/library profile declaration",
        )),
    }
}

/// Only native schema1's eight root kinds reserve object_revision as a local
/// counter. model_revision is not a declared library-payload header; preserve it
/// as unknown authored data. Owned attachment details have no root counter.
pub(crate) fn counter_policy(
    kind: crate::library_assets::AssetKind,
    version: u32,
) -> Result<LocalCounters, EngineError> {
    use crate::library_assets::AssetKind;
    if version != 1 {
        return Err(refusal("unsupported owning payload counter schema"));
    }
    Ok(match kind {
        AssetKind::Unit
        | AssetKind::Symbol
        | AssetKind::Entity
        | AssetKind::Package
        | AssetKind::Padstack
        | AssetKind::Footprint
        | AssetKind::Part
        | AssetKind::PinPadMap => LocalCounters {
            object_revision: true,
            model_revision: false,
        },
        AssetKind::ModelAttachment => LocalCounters::default(),
    })
}

fn read_charge(
    root: &Path,
    path: &str,
    pending: &BTreeMap<String, &[u8]>,
    per_file: usize,
    used: &mut usize,
    total: usize,
) -> Result<Vec<u8>, EngineError> {
    let remaining = total
        .checked_sub(*used)
        .ok_or_else(|| refusal("aggregate raw allocation exhausted"))?;
    let bytes = read_content(root, path, pending, per_file.min(remaining))?;
    *used = used
        .checked_add(bytes.capacity())
        .ok_or_else(|| refusal("aggregate raw allocation overflow"))?;
    if *used > total {
        return Err(refusal(
            "aggregate library raw allocation exceeds server policy",
        ));
    }
    Ok(bytes)
}

/// The declared profile owns exactly two mutable source heads. Immutable
/// content is reached through their verified inventory, never registered as CAD UUIDs.
pub(in crate::substrate) fn discover_profile(
    root: &Path,
    manifest: &serde_json::Value,
    heads: &mut Vec<super::SourceShardRef>,
) -> Result<Option<LibraryAssetSnapshot>, EngineError> {
    validate_profile_header(manifest)?;
    if manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(2)
    {
        return Ok(None);
    }
    let limits = server_policy()?;
    let state = read_state(root, limits)?;
    let snapshot = resolve_state(root, &state, &BTreeMap::new(), limits)?;
    for (kind, path, bytes) in [
        (
            super::SourceShardKind::LibraryLock,
            LOCK_RELATIVE_PATH,
            &state.lock,
        ),
        (
            super::SourceShardKind::LibraryCatalog,
            CATALOG_RELATIVE_PATH,
            &state.catalog,
        ),
    ] {
        if heads
            .iter()
            .any(|head| head.relative_path == path || head.kind == kind)
        {
            return Err(refusal("duplicate library metadata source head"));
        }
        let head = super::source_shard_ref_builders::source_shard_ref_for_bytes(
            kind,
            root.join(path),
            path.into(),
            Some(1),
            bytes,
            "library_metadata_unavailable",
        )
        .map_err(|diagnostic| refusal(diagnostic.message))?;
        heads.push(head);
    }
    Ok(Some(snapshot))
}

/// Cold validation runs after integrity recovery and native object/history
/// resolution. It grants no online dependency or migration of legacy assets.
pub(in crate::substrate) fn validate_profile_model(
    root: &Path,
    lease: &crate::revision::ProjectWriteLease,
    model: &super::DesignModel,
    snapshot: &LibraryAssetSnapshot,
) -> Result<(), EngineError> {
    let sources = super::ElectricalSelectionSnapshot::capture(model)
        .map_err(|error| refusal(format!("cold library sources unavailable: {error:?}")))?;
    super::library_asset_journal_ops::admission::validate_resolved(
        root,
        lease,
        model,
        snapshot,
        Some(&sources),
        server_policy()?,
    )?;
    Ok(())
}
