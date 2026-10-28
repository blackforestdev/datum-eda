use std::fs::{DirBuilder, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::ir::serialization::to_json_bytes;

use super::model::{
    CANONICALIZATION_VERSION, CURRENT_WRITE_VERSION, GenerationManifest, GenerationRef, HeadRecord,
    MutationReceipt, REPOSITORY_FORMAT_VERSION_V1, RepositoryError, RepositorySnapshot,
    RepositoryState, UnknownEnvelopeRef, UnreadableEvidence, is_supported_format_version,
};

pub(crate) const HEAD_FILE: &str = "head.json";
const MANIFEST_FILE: &str = "manifest.json";
const LEASE_FILE: &str = "writer.lock";

pub(crate) fn digest_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    format!("sha256:{hex}")
}

pub(crate) fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, RepositoryError> {
    Ok(to_json_bytes(value)?)
}

pub(crate) fn generation_dir(root: &Path, generation: u64) -> PathBuf {
    root.join("generations").join(format!("g{generation:020}"))
}

pub(crate) fn next_generation_number(root: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(root.join("generations")) else {
        return 0;
    };
    entries
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix('g'))
                .and_then(|number| number.parse::<u64>().ok())
        })
        .max()
        .map_or(0, |generation| generation + 1)
}

pub(crate) fn read_head(root: &Path) -> Result<Option<(HeadRecord, Vec<u8>)>, RepositoryError> {
    let path = root.join(HEAD_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    let head = serde_json::from_slice(&bytes)?;
    Ok(Some((head, bytes)))
}

pub(crate) fn repository_is_pristine(root: &Path) -> bool {
    if !root.exists() {
        return true;
    }
    std::fs::read_dir(root).is_ok_and(|entries| {
        entries
            .flatten()
            .all(|entry| entry.file_name() == std::ffi::OsStr::new(LEASE_FILE))
    })
}

pub(crate) fn load_snapshot(
    root: &Path,
    generation: &GenerationRef,
) -> Result<RepositorySnapshot, RepositoryError> {
    if !is_supported_format_version(generation.format_version) {
        return Err(RepositoryError::UnsupportedRepositoryVersion(
            generation.format_version,
        ));
    }
    if generation.canonicalization_version != CANONICALIZATION_VERSION {
        return Err(RepositoryError::Invariant(format!(
            "unsupported canonicalization version {}",
            generation.canonicalization_version
        )));
    }
    let directory = generation_dir(root, generation.generation);
    let manifest_bytes = std::fs::read(directory.join(MANIFEST_FILE))?;
    if digest_bytes(&manifest_bytes) != generation.canonical_manifest_digest {
        return Err(RepositoryError::Invariant(
            "generation manifest digest mismatch".to_owned(),
        ));
    }
    let manifest: GenerationManifest = serde_json::from_slice(&manifest_bytes)?;
    validate_manifest(generation, &manifest)?;
    validate_unknown_payloads(&directory, &manifest.state)?;
    let receipts = reconstruct_receipts(root, generation, &manifest)?;
    Ok(RepositorySnapshot {
        generation: generation.clone(),
        installation: manifest.state.installation,
        user: manifest.state.user,
        unknown_envelopes: manifest.state.unknown_envelopes,
        receipts,
    })
}

/// Reconstruct the complete ordered audit log for `head`.
///
/// A v1 generation embeds the full accumulated log, so it is returned directly.
/// A v2 generation stores only its own receipt delta; the full log is the
/// concatenation of every ancestor's delta, assembled by walking the parent
/// chain newest-first and verifying each parent against the digest the child
/// recorded (GP_C04 sections 4/5: the parent/reference graph validated by
/// integrity facts). The walk stops at the genesis generation or at a v1
/// ancestor, whose manifest already carries the complete earlier prefix — which
/// is how a store that began on v1 and advanced to v2 reconstructs correctly.
pub(crate) fn reconstruct_receipts(
    root: &Path,
    head: &GenerationRef,
    head_manifest: &GenerationManifest,
) -> Result<Vec<MutationReceipt>, RepositoryError> {
    if head.format_version == REPOSITORY_FORMAT_VERSION_V1 {
        return Ok(head_manifest.state.receipts.clone());
    }
    let mut segments = vec![head_manifest.state.receipts.clone()];
    visit_ancestors(
        root,
        head,
        head_manifest,
        |_generation, _bytes, manifest| {
            segments.push(manifest.state.receipts.clone());
            Ok(())
        },
    )?;
    segments.reverse();
    Ok(segments.into_iter().flatten().collect())
}

/// Walk a v2 head's parent chain newest-first, invoking `visit` with each
/// verified ancestor (its generation number, its manifest bytes, the parsed
/// manifest). This is the single admission gate shared by receipt
/// reconstruction and exact-backup copying: an ancestor must match the digest
/// its child recorded and carry a supported format/canonicalization version and
/// the repository's own identity, or the walk refuses (fail closed). It stops at
/// genesis or at the first v1 ancestor, whose manifest already holds the
/// complete earlier prefix. Each parent link must strictly descend in generation
/// number, so a malformed or cyclic chain is refused and the walk always
/// terminates.
pub(crate) fn visit_ancestors(
    root: &Path,
    head: &GenerationRef,
    head_manifest: &GenerationManifest,
    mut visit: impl FnMut(u64, &[u8], &GenerationManifest) -> Result<(), RepositoryError>,
) -> Result<(), RepositoryError> {
    if head.format_version == REPOSITORY_FORMAT_VERSION_V1 {
        return Ok(());
    }
    let mut parent_generation = head_manifest.parent_generation;
    let mut parent_digest = head_manifest.parent_canonical_manifest_digest.clone();
    let mut child_generation = head.generation;
    while let Some(generation) = parent_generation {
        if generation >= child_generation {
            return Err(RepositoryError::Invariant(
                "generation chain parent does not strictly precede its child".to_owned(),
            ));
        }
        let expected = parent_digest.ok_or_else(|| {
            RepositoryError::Invariant(
                "v2 generation references a parent without an integrity digest".to_owned(),
            )
        })?;
        let bytes = std::fs::read(generation_dir(root, generation).join(MANIFEST_FILE))?;
        if digest_bytes(&bytes) != expected {
            return Err(RepositoryError::Invariant(
                "generation chain parent digest mismatch".to_owned(),
            ));
        }
        let manifest: GenerationManifest = serde_json::from_slice(&bytes)?;
        if manifest.generation != generation
            || manifest.repository_id != head.repository_id
            || manifest.canonicalization_version != CANONICALIZATION_VERSION
            || !is_supported_format_version(manifest.format_version)
        {
            return Err(RepositoryError::Invariant(
                "generation chain parent failed identity or format validation".to_owned(),
            ));
        }
        visit(generation, &bytes, &manifest)?;
        if manifest.format_version == REPOSITORY_FORMAT_VERSION_V1 {
            break;
        }
        child_generation = generation;
        parent_generation = manifest.parent_generation;
        parent_digest = manifest.parent_canonical_manifest_digest.clone();
    }
    Ok(())
}

fn validate_manifest(
    expected: &GenerationRef,
    manifest: &GenerationManifest,
) -> Result<(), RepositoryError> {
    let matches = manifest.format_version == expected.format_version
        && manifest.canonicalization_version == expected.canonicalization_version
        && manifest.repository_id == expected.repository_id
        && manifest.generation == expected.generation
        && manifest.parent_generation == expected.parent_generation
        && manifest.committed_order == expected.committed_order
        && manifest.writer_instance == expected.writer_instance;
    if matches {
        Ok(())
    } else {
        Err(RepositoryError::Invariant(
            "head and generation manifest disagree".to_owned(),
        ))
    }
}

fn validate_unknown_payloads(
    directory: &Path,
    state: &RepositoryState,
) -> Result<(), RepositoryError> {
    for envelope in state.unknown_envelopes.values() {
        let payload = std::fs::read(opaque_path(directory, envelope))?;
        if payload.len() as u64 != envelope.payload_len
            || digest_bytes(&payload) != envelope.payload_digest
        {
            return Err(RepositoryError::Invariant(format!(
                "unknown envelope {} failed byte-integrity validation",
                envelope.identity
            )));
        }
    }
    Ok(())
}

pub(crate) fn opaque_path(directory: &Path, envelope: &UnknownEnvelopeRef) -> PathBuf {
    let digest = envelope
        .payload_digest
        .strip_prefix("sha256:")
        .unwrap_or(&envelope.payload_digest);
    directory.join("opaque").join(format!("{digest}.bin"))
}

pub(crate) struct WriterLease {
    _file: File,
}

impl WriterLease {
    pub(crate) fn acquire(root: &Path, writer: &str) -> Result<Self, RepositoryError> {
        let mut builder = DirBuilder::new();
        builder.recursive(true).mode(0o700);
        builder.create(root)?;
        let path = root.join(LEASE_FILE);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => RepositoryError::WriterLeaseUnavailable,
            std::fs::TryLockError::Error(error) => RepositoryError::Io(error),
        })?;
        file.set_len(0)?;
        file.write_all(writer.as_bytes())?;
        file.sync_all()?;
        sync_parent(&path)?;
        Ok(Self { _file: file })
    }
}

pub(crate) fn write_generation(
    root: &Path,
    repository_id: &str,
    generation: u64,
    parent: Option<&GenerationRef>,
    writer_instance: &str,
    state: &RepositoryState,
    unknown_bytes: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<GenerationRef, RepositoryError> {
    let parent_generation = parent.map(|parent| parent.generation);
    // v2 persists only this generation's receipt delta; the complete audit log
    // lives as the parent-digest-verified chain of deltas (see
    // reconstruct_receipts). Values and unknown envelopes stay complete per
    // generation — they are bounded by the key set, not by generation count, so
    // only the receipt log needed deduplication to stop quadratic growth.
    let manifest_state = RepositoryState {
        installation: state.installation.clone(),
        user: state.user.clone(),
        unknown_envelopes: state.unknown_envelopes.clone(),
        receipts: state
            .receipts
            .iter()
            .filter(|receipt| receipt.resulting_generation == generation)
            .cloned()
            .collect(),
    };
    let manifest = GenerationManifest {
        format_version: CURRENT_WRITE_VERSION,
        canonicalization_version: CANONICALIZATION_VERSION,
        repository_id: repository_id.to_owned(),
        generation,
        parent_generation,
        parent_canonical_manifest_digest: parent
            .map(|parent| parent.canonical_manifest_digest.clone()),
        committed_order: generation,
        writer_instance: writer_instance.to_owned(),
        state: manifest_state,
    };
    let manifest_bytes = canonical_bytes(&manifest)?;
    let generation_ref = GenerationRef {
        repository_id: repository_id.to_owned(),
        generation,
        parent_generation,
        canonical_manifest_digest: digest_bytes(&manifest_bytes),
        committed_order: generation,
        writer_instance: writer_instance.to_owned(),
        format_version: CURRENT_WRITE_VERSION,
        canonicalization_version: CANONICALIZATION_VERSION,
    };

    let generations = root.join("generations");
    std::fs::create_dir_all(&generations)?;
    let stage = generations.join(format!(".stage-{}", Uuid::new_v4()));
    std::fs::create_dir(&stage)?;
    for envelope in state.unknown_envelopes.values() {
        let bytes = unknown_bytes.get(&envelope.identity).ok_or_else(|| {
            RepositoryError::Invariant(format!(
                "missing staged bytes for unknown envelope {}",
                envelope.identity
            ))
        })?;
        if digest_bytes(bytes) != envelope.payload_digest
            || bytes.len() as u64 != envelope.payload_len
        {
            return Err(RepositoryError::Invariant(format!(
                "staged bytes changed for unknown envelope {}",
                envelope.identity
            )));
        }
        write_immutable_content(&opaque_path(&stage, envelope), bytes)?;
    }
    write_new_file(&stage.join(MANIFEST_FILE), &manifest_bytes)?;
    validate_unknown_payloads(&stage, state)?;
    let staged_manifest: GenerationManifest =
        serde_json::from_slice(&std::fs::read(stage.join(MANIFEST_FILE))?)?;
    validate_manifest(&generation_ref, &staged_manifest)?;
    sync_tree(&stage)?;

    let target = generation_dir(root, generation);
    if target.exists() {
        return Err(RepositoryError::Invariant(format!(
            "immutable generation already exists: {generation}"
        )));
    }
    std::fs::rename(&stage, &target)?;
    sync_parent(&target)?;
    Ok(generation_ref)
}

pub(crate) fn promote_head(root: &Path, generation: &GenerationRef) -> Result<(), RepositoryError> {
    write_atomic(
        &root.join(HEAD_FILE),
        &canonical_bytes(&HeadRecord {
            generation: generation.clone(),
        })?,
    )
}

pub(crate) fn copy_unknown_bytes(
    root: &Path,
    snapshot: &RepositorySnapshot,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>, RepositoryError> {
    let directory = generation_dir(root, snapshot.generation.generation);
    snapshot
        .unknown_envelopes
        .iter()
        .map(|(identity, envelope)| {
            Ok((
                identity.clone(),
                std::fs::read(opaque_path(&directory, envelope))?,
            ))
        })
        .collect()
}

pub(crate) fn unreadable_evidence(
    root: &Path,
    head_digest: String,
    source_path: PathBuf,
    exact_bytes: Vec<u8>,
    reason: String,
) -> UnreadableEvidence {
    let identity = digest_bytes(&exact_bytes);
    let recovery_candidates = list_valid_generations(root);
    UnreadableEvidence {
        identity,
        head_digest,
        source_path,
        exact_bytes,
        reason,
        recovery_candidates,
    }
}

pub(crate) fn list_valid_generations(root: &Path) -> Vec<GenerationRef> {
    let Ok(entries) = std::fs::read_dir(root.join("generations")) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(bytes) = std::fs::read(path.join(MANIFEST_FILE)) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_slice::<GenerationManifest>(&bytes) else {
            continue;
        };
        let generation = GenerationRef {
            repository_id: manifest.repository_id.clone(),
            generation: manifest.generation,
            parent_generation: manifest.parent_generation,
            canonical_manifest_digest: digest_bytes(&bytes),
            committed_order: manifest.committed_order,
            writer_instance: manifest.writer_instance.clone(),
            format_version: manifest.format_version,
            canonicalization_version: manifest.canonicalization_version,
        };
        // Admit a recovery candidate only if it loads completely, including a
        // valid parent chain: a descendant whose ancestor is missing or corrupt
        // cannot actually be restored, so it must not be advertised even though
        // its own manifest validates in isolation.
        if load_snapshot(root, &generation).is_ok() {
            result.push(generation);
        }
    }
    result.sort_by_key(|generation| generation.generation);
    result
}

pub(crate) fn suspect_generation_bytes(
    root: &Path,
    generation: &GenerationRef,
) -> (PathBuf, Vec<u8>) {
    let directory = generation_dir(root, generation.generation);
    let manifest_path = directory.join(MANIFEST_FILE);
    let manifest_bytes = std::fs::read(&manifest_path).unwrap_or_default();
    let Ok(manifest) = serde_json::from_slice::<GenerationManifest>(&manifest_bytes) else {
        return (manifest_path, manifest_bytes);
    };
    if digest_bytes(&manifest_bytes) != generation.canonical_manifest_digest {
        return (manifest_path, manifest_bytes);
    }
    for envelope in manifest.state.unknown_envelopes.values() {
        let path = opaque_path(&directory, envelope);
        let bytes = std::fs::read(&path).unwrap_or_default();
        if bytes.len() as u64 != envelope.payload_len
            || digest_bytes(&bytes) != envelope.payload_digest
        {
            return (path, bytes);
        }
    }
    (manifest_path, manifest_bytes)
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), RepositoryError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    write_new_file(&temporary, bytes)?;
    std::fs::rename(&temporary, path)?;
    sync_parent(path)
}

pub(crate) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), RepositoryError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    sync_parent(path)
}

pub(crate) fn write_immutable_content(path: &Path, bytes: &[u8]) -> Result<(), RepositoryError> {
    if path.exists() {
        if std::fs::read(path)? == bytes {
            return Ok(());
        }
        return Err(RepositoryError::Invariant(format!(
            "immutable content conflict: {}",
            path.display()
        )));
    }
    write_new_file(path, bytes)
}

pub(crate) fn sync_parent(path: &Path) -> Result<(), RepositoryError> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn sync_tree(path: &Path) -> Result<(), RepositoryError> {
    let opaque = path.join("opaque");
    if opaque.exists() {
        File::open(opaque)?.sync_all()?;
    }
    File::open(path)?.sync_all()?;
    Ok(())
}
