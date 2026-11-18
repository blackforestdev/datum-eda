//! Shared immutable collection loading; project locks remain with their owner.
use super::{
    AssetDigest, AssetRevisionRef, ClosureLimits, LibraryAssetLimits, LibraryDiscovery,
    LibraryReleaseManifest, VerifiedAsset, counter_policy, inspect_closure, read_charge, refusal,
    restrict_limits, server_policy,
};
use crate::{
    error::EngineError,
    library_assets::{AssetRevision, decode_metadata, manifest::ObjectKind, review::AssetReview},
};
use std::{borrow::Cow, collections::BTreeMap, path::Path};

pub(crate) struct LibraryReleaseSnapshot {
    pub(crate) manifest: LibraryReleaseManifest,
    pub(crate) assets: BTreeMap<AssetRevisionRef, VerifiedAsset>,
    pub(crate) reviews: BTreeMap<AssetDigest, AssetReview>,
    pub(crate) manifest_bytes: Vec<u8>,
    pub(crate) objects: BTreeMap<AssetDigest, Vec<u8>>,
    pub(crate) blobs: BTreeMap<AssetDigest, Vec<u8>>,
}

pub(super) fn load(
    root: &Path,
    prefix: &str,
    catalog: &LibraryDiscovery,
    pending: &BTreeMap<String, &[u8]>,
    extra_refs: &[AssetRevisionRef],
    limits: LibraryAssetLimits,
    used: &mut usize,
) -> Result<LibraryReleaseSnapshot, EngineError> {
    let bytes = read_charge(
        root,
        &record_path(prefix, &catalog.manifest_ref),
        pending,
        limits.metadata_bytes,
        used,
        limits.total_bytes,
    )?;
    let manifest: LibraryReleaseManifest = decode_metadata(&bytes, limits.metadata_bytes)?;
    catalog.verify_manifest(&manifest)?;
    if manifest.objects.len().saturating_add(manifest.blobs.len()) > limits.files
        || extra_refs.len() > limits.files
    {
        return Err(refusal("library inventory exceeds declared file limit"));
    }
    let mut objects = BTreeMap::new();
    let mut blobs = BTreeMap::new();
    for entry in &manifest.objects {
        objects.insert(
            entry.digest.clone(),
            read_charge(
                root,
                &record_path(prefix, &entry.digest),
                pending,
                limits.metadata_bytes,
                used,
                limits.total_bytes,
            )?,
        );
    }
    for entry in &manifest.blobs {
        blobs.insert(
            entry.digest.clone(),
            read_charge(
                root,
                &payload_path(prefix, &entry.digest),
                pending,
                limits.blob_bytes,
                used,
                limits.total_bytes,
            )?,
        );
    }
    manifest.verify_inventory(&objects, &blobs, limits.metadata_bytes)?;
    // These exact bytes were already loaded, charged and inventory-verified.
    // Retaining their decoded claims adds no acquisition or review authority.
    let reviews = manifest
        .objects
        .iter()
        .filter(|entry| entry.kind == ObjectKind::Review)
        .map(|entry| {
            Ok((
                entry.digest.clone(),
                decode_metadata::<AssetReview>(&objects[&entry.digest], limits.metadata_bytes)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, EngineError>>()?;
    let mut refs = extra_refs.to_vec();
    for entry in manifest
        .objects
        .iter()
        .filter(|o| o.kind == ObjectKind::AssetRevision)
    {
        let revision: AssetRevision =
            decode_metadata(&objects[&entry.digest], limits.metadata_bytes)?;
        refs.push(revision.reference()?);
    }
    let mut assets = BTreeMap::new();
    for reference in refs {
        if assets.contains_key(&reference) {
            continue;
        }
        let record = match objects.get(&reference.revision_id) {
            Some(bytes) => Cow::Borrowed(bytes.as_slice()),
            None => Cow::Owned(read_charge(
                root,
                &record_path(prefix, &reference.revision_id),
                pending,
                limits.metadata_bytes,
                used,
                limits.total_bytes,
            )?),
        };
        let revision: AssetRevision = decode_metadata(&record, limits.metadata_bytes)?;
        let provenance = match objects.get(&revision.provenance_ref) {
            Some(bytes) => Cow::Borrowed(bytes.as_slice()),
            None => Cow::Owned(read_charge(
                root,
                &record_path(prefix, &revision.provenance_ref),
                pending,
                limits.metadata_bytes,
                used,
                limits.total_bytes,
            )?),
        };
        let payload = match blobs.get(&revision.payload_ref) {
            Some(bytes) => Cow::Borrowed(bytes.as_slice()),
            None => Cow::Owned(read_charge(
                root,
                &payload_path(prefix, &revision.payload_ref),
                pending,
                limits.blob_bytes,
                used,
                limits.total_bytes,
            )?),
        };
        // VerifiedPayload retains its own raw Box<[u8]>. Inventory buffers are
        // borrowed above, but that upcoming allocation still consumes the
        // existing aggregate raw-byte budget. Refuse before allocating it.
        *used = used
            .checked_add(payload.len())
            .ok_or_else(|| refusal("aggregate retained payload allocation overflow"))?;
        if *used > limits.total_bytes {
            return Err(refusal(
                "aggregate retained payload allocation exceeds server policy",
            ));
        }
        let asset = VerifiedAsset::read(
            &reference,
            &record,
            &provenance,
            &payload,
            limits.metadata_bytes.max(limits.blob_bytes),
            counter_policy(revision.kind, revision.payload_schema_version)?,
        )?;
        for field in ["models_3d", "behavioural_models"] {
            if asset.payload().value().get(field).is_some_and(|value| {
                !value.is_null()
                    && !value.as_array().is_some_and(Vec::is_empty)
                    && !value.as_object().is_some_and(serde_json::Map::is_empty)
            }) {
                return Err(refusal(format!(
                    "unsupported owned model dependencies in {field}; offline closure cannot be verified",
                )));
            }
        }
        assets.insert(reference, asset);
    }
    let bounds = ClosureLimits {
        max_nodes: limits.files,
        max_depth: limits.depth,
    };
    let heads = inspect_closure(&manifest.roots, &assets, bounds)?;
    manifest.verify_closure(&heads)?;
    Ok(LibraryReleaseSnapshot {
        manifest,
        assets,
        reviews,
        manifest_bytes: bytes,
        objects,
        blobs,
    })
}

fn record_path(prefix: &str, digest: &AssetDigest) -> String {
    format!("{prefix}objects/sha256/{}.json", digest.hex())
}
fn payload_path(prefix: &str, digest: &AssetDigest) -> String {
    format!("{prefix}blobs/sha256/{}", digest.hex())
}

/// Inspect the exchange directory itself, with no project or fabricated lock.
pub(crate) fn read_candidate(
    root: &Path,
) -> Result<(LibraryDiscovery, LibraryReleaseSnapshot), EngineError> {
    let (catalog, release, _) = read_candidate_bytes(root)?;
    Ok((catalog, release))
}

/// Retain the discovery bytes from the same bounded verification snapshot.
pub(super) fn read_candidate_bytes(
    root: &Path,
) -> Result<(LibraryDiscovery, LibraryReleaseSnapshot, Vec<u8>), EngineError> {
    let metadata = std::fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(refusal("candidate root must be a regular directory"));
    }
    let limits = restrict_limits(server_policy()?)?;
    let pending = BTreeMap::new();
    let mut used = 0;
    let bytes = read_charge(
        root,
        crate::library_assets::manifest::DISCOVERY_PATH,
        &pending,
        limits.metadata_bytes,
        &mut used,
        limits.total_bytes,
    )?;
    let catalog: LibraryDiscovery = decode_metadata(&bytes, limits.metadata_bytes)?;
    let release = load(root, "", &catalog, &pending, &[], limits, &mut used)?;
    Ok((catalog, release, bytes))
}
