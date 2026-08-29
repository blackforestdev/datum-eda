use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::EngineError,
    substrate::{TransactionRecord, transaction_journal_path},
};

use super::{
    canonical::{canonical_bytes, digest_bytes, validate_digest},
    model::{
        AffectedShard, AffectedShardAction, AlgorithmQualifiedDigest, CANONICAL_ENCODING,
        DIGEST_ALGORITHM, IntegrityDiagnostic, IntegrityGeneration, IntegrityGenerationMaterial,
        IntegrityHead, REVISION_STORE_SCHEMA_VERSION, RevisionStoreState, StagedShardPostimage,
        TechnicalValidationState,
    },
};

const STORE_RELATIVE_PATH: &str = ".datum/revision/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegrityCommitFaultPoint {
    AuthorityStage,
    JournalAppend,
    ShardPromotion,
    AuthorityPromotion,
    CursorWrite,
}

impl IntegrityCommitFaultPoint {
    pub(crate) fn inject(self, point: Self) -> Result<(), EngineError> {
        if self == point {
            return Err(EngineError::Operation(format!(
                "injected revision integrity commit interruption at {point:?}"
            )));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProjectWriteLease {
    path: PathBuf,
    nonce: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LeaseRecord {
    schema_version: u64,
    pid: u32,
    nonce: Uuid,
}

impl ProjectWriteLease {
    pub fn acquire(project_root: &Path) -> Result<Self, EngineError> {
        let lease_root = project_root.join(".datum/revision");
        std::fs::create_dir_all(&lease_root)?;
        let path = lease_root.join("write.lock");
        let nonce = Uuid::new_v4();
        let record = LeaseRecord {
            schema_version: REVISION_STORE_SCHEMA_VERSION,
            pid: std::process::id(),
            nonce,
        };
        let bytes = canonical_bytes(&record)?;
        for attempt in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(&bytes)?;
                    file.sync_all()?;
                    sync_parent(&path)?;
                    return Ok(Self { path, nonce });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt == 0 => {
                    if !lease_owner_is_live(&path) {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    return Err(EngineError::Operation(format!(
                        "project write refused: live writer owns {}",
                        path.display()
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(EngineError::Operation(format!(
                        "project write refused: write lease {} could not be acquired",
                        path.display()
                    )));
                }
                Err(error) => return Err(error.into()),
            }
        }
        unreachable!("bounded write-lease attempts always return")
    }
}

impl Drop for ProjectWriteLease {
    fn drop(&mut self) {
        let owned = std::fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<LeaseRecord>(&bytes).ok())
            .is_some_and(|record| record.nonce == self.nonce);
        if owned {
            let _ = std::fs::remove_file(&self.path);
            if let Some(parent) = self.path.parent() {
                let _ = File::open(parent).and_then(|file| file.sync_all());
            }
        }
    }
}

fn lease_owner_is_live(path: &Path) -> bool {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<LeaseRecord>(&bytes).ok())
        .is_some_and(|record| Path::new("/proc").join(record.pid.to_string()).exists())
}

#[derive(Debug, Clone)]
pub struct RevisionAuthorityStore {
    pub(crate) root: PathBuf,
}

#[derive(Debug)]
pub struct StagedIntegrityGeneration {
    store: RevisionAuthorityStore,
    stage_root: PathBuf,
    pub generation: IntegrityGeneration,
}

impl RevisionAuthorityStore {
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.join(STORE_RELATIVE_PATH),
        }
    }

    pub(crate) fn from_store_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn stage_transaction(
        &self,
        project_id: Uuid,
        transaction: &TransactionRecord,
        parent_transaction_id: Option<Uuid>,
        postimages: Vec<StagedShardPostimage>,
    ) -> Result<StagedIntegrityGeneration, EngineError> {
        let current_head = self.read_head().transpose()?;
        if let Some(head) = &current_head
            && Some(head.transaction_id) != parent_transaction_id
        {
            return Err(EngineError::Operation(format!(
                "revision authority head mismatch: expected parent {:?}, current {}",
                parent_transaction_id, head.transaction_id
            )));
        }

        let stage_root = self
            .root
            .join("stage")
            .join(transaction.transaction_id.to_string());
        if stage_root.exists() {
            std::fs::remove_dir_all(&stage_root)?;
        }
        std::fs::create_dir_all(stage_root.join("blobs/sha256"))?;

        let transaction_bytes = canonical_bytes(transaction)?;
        let transaction_blob = self.stage_blob(&stage_root, &transaction_bytes)?;
        let authority_snapshot_blob = if let Some(head) = &current_head {
            let generation: IntegrityGeneration =
                read_json(&self.generation_path(&head.integrity_root))?;
            generation.authority_snapshot_blob
        } else {
            None
        };
        let mut affected_shards = Vec::with_capacity(postimages.len());
        for postimage in postimages {
            let (action, postimage_blob) = match postimage.bytes {
                Some(bytes) => (
                    AffectedShardAction::Write,
                    Some(self.stage_blob(&stage_root, &bytes)?),
                ),
                None => (AffectedShardAction::Delete, None),
            };
            affected_shards.push(AffectedShard {
                relative_path: postimage.relative_path,
                action,
                postimage_blob,
            });
        }
        affected_shards.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        let cache_invalidations = affected_shards
            .iter()
            .map(|shard| format!("source_shard:{}", shard.relative_path))
            .collect();
        let material = IntegrityGenerationMaterial {
            schema_version: REVISION_STORE_SCHEMA_VERSION,
            canonical_encoding: CANONICAL_ENCODING.to_string(),
            digest_algorithm: DIGEST_ALGORITHM.to_string(),
            project_id,
            transaction_id: transaction.transaction_id,
            parent_transaction_id,
            parent_integrity_root: current_head.map(|head| head.integrity_root),
            before_model_revision: transaction.before_model_revision.clone(),
            after_model_revision: transaction.after_model_revision.clone(),
            transaction_blob,
            authority_snapshot_blob,
            affected_shards,
            validation_state: TechnicalValidationState::AcceptedByMutationGuards,
            cache_invalidations,
        };
        let integrity_root = digest_bytes(&canonical_bytes(&material)?);
        let generation = IntegrityGeneration {
            schema_version: material.schema_version,
            canonical_encoding: material.canonical_encoding,
            digest_algorithm: material.digest_algorithm,
            project_id: material.project_id,
            transaction_id: material.transaction_id,
            parent_transaction_id: material.parent_transaction_id,
            parent_integrity_root: material.parent_integrity_root,
            before_model_revision: material.before_model_revision,
            after_model_revision: material.after_model_revision,
            transaction_blob: material.transaction_blob,
            authority_snapshot_blob: material.authority_snapshot_blob,
            affected_shards: material.affected_shards,
            validation_state: material.validation_state,
            cache_invalidations: material.cache_invalidations,
            integrity_root,
        };
        write_new_file(
            &stage_root.join("generation.json"),
            &canonical_bytes(&generation)?,
        )?;
        sync_tree(&stage_root)?;
        Ok(StagedIntegrityGeneration {
            store: self.clone(),
            stage_root,
            generation,
        })
    }

    fn stage_blob(
        &self,
        stage_root: &Path,
        bytes: &[u8],
    ) -> Result<AlgorithmQualifiedDigest, EngineError> {
        let digest = digest_bytes(bytes);
        let path = stage_root.join("blobs/sha256").join(digest_hex(&digest)?);
        if !path.exists() {
            write_new_file(&path, bytes)?;
        }
        Ok(digest)
    }

    pub fn inspect_and_recover(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
    ) -> RevisionStoreState {
        match self.inspect_and_recover_inner(project_id, journal) {
            Ok(state) => state,
            Err(error) => RevisionStoreState::ReadOnlyDiagnostic {
                last_complete: self.find_last_complete(project_id, journal).ok().flatten(),
                diagnostics: vec![IntegrityDiagnostic {
                    code: "revision_integrity_unavailable".to_string(),
                    message: error.to_string(),
                    path: Some(self.root.clone()),
                }],
            },
        }
    }

    fn inspect_and_recover_inner(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
    ) -> Result<RevisionStoreState, EngineError> {
        if !self.root.exists() {
            return Ok(RevisionStoreState::Empty);
        }
        let current_head = self.read_head().transpose()?;
        let current_index = current_head.as_ref().and_then(|head| {
            journal
                .iter()
                .position(|record| record.transaction_id == head.transaction_id)
        });
        let journal_ids: BTreeSet<_> = journal.iter().map(|record| record.transaction_id).collect();
        let mut recovered_stages = 0;
        let stage_parent = self.root.join("stage");
        if let Ok(entries) = std::fs::read_dir(&stage_parent) {
            let mut candidates = Vec::new();
            for entry in entries.flatten() {
                let manifest_path = entry.path().join("generation.json");
                let Ok(generation) = read_json::<IntegrityGeneration>(&manifest_path) else {
                    continue;
                };
                if generation.project_id == project_id
                    && journal_ids.contains(&generation.transaction_id)
                    && self.verify_staged(&entry.path(), &generation).is_ok()
                {
                    let index = journal
                        .iter()
                        .position(|record| record.transaction_id == generation.transaction_id)
                        .unwrap_or(usize::MAX);
                    if current_index.is_none_or(|current| index >= current) {
                        candidates.push((index, entry.path(), generation));
                    }
                }
            }
            candidates.sort_by_key(|(index, _, _)| *index);
            if !candidates.is_empty() {
                let _lease = ProjectWriteLease::acquire(
                    self.root
                        .parent()
                        .and_then(Path::parent)
                        .and_then(Path::parent)
                        .ok_or_else(|| {
                            EngineError::Validation(
                                "revision store has no Project root".to_string(),
                            )
                        })?,
                )?;
                for (_, stage_root, generation) in candidates {
                    StagedIntegrityGeneration {
                        store: self.clone(),
                        stage_root,
                        generation,
                    }
                    .promote()?;
                    recovered_stages += 1;
                }
            }
        }

        let Some(head) = self.read_head().transpose()? else {
            return Ok(RevisionStoreState::Empty);
        };
        self.verify_head(project_id, journal, &head)?;
        if recovered_stages == 0 {
            Ok(RevisionStoreState::Complete { head })
        } else {
            Ok(RevisionStoreState::Recovered {
                head,
                recovered_stages,
            })
        }
    }

    fn verify_head(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
        head: &IntegrityHead,
    ) -> Result<IntegrityGeneration, EngineError> {
        if head.schema_version != REVISION_STORE_SCHEMA_VERSION || head.project_id != project_id {
            return Err(EngineError::Validation(
                "revision integrity head schema/project mismatch".to_string(),
            ));
        }
        if transaction_tip(journal) != Some(head.transaction_id) {
            return Err(EngineError::Validation(format!(
                "revision integrity head transaction {} is not the accepted journal tip",
                head.transaction_id
            )));
        }
        let generation =
            read_json::<IntegrityGeneration>(&self.generation_path(&head.integrity_root))?;
        self.verify_generation(&generation)?;
        if generation.transaction_id != head.transaction_id
            || generation.integrity_root != head.integrity_root
        {
            return Err(EngineError::Validation(
                "revision integrity head does not name its generation".to_string(),
            ));
        }
        Ok(generation)
    }

    fn find_last_complete(
        &self,
        project_id: Uuid,
        journal: &[TransactionRecord],
    ) -> Result<Option<IntegrityHead>, EngineError> {
        let generations = self.root.join("generations");
        let Ok(entries) = std::fs::read_dir(generations) else {
            return Ok(None);
        };
        let journal_positions: BTreeMap<_, _> = journal
            .iter()
            .enumerate()
            .map(|(index, record)| (record.transaction_id, index))
            .collect();
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let Ok(generation) = read_json::<IntegrityGeneration>(&entry.path()) else {
                continue;
            };
            let Some(index) = journal_positions.get(&generation.transaction_id).copied() else {
                continue;
            };
            if generation.project_id == project_id && self.verify_generation(&generation).is_ok() {
                candidates.push((index, generation));
            }
        }
        candidates.sort_by_key(|(index, _)| *index);
        Ok(candidates.pop().map(|(_, generation)| IntegrityHead {
            schema_version: REVISION_STORE_SCHEMA_VERSION,
            project_id,
            transaction_id: generation.transaction_id,
            integrity_root: generation.integrity_root,
        }))
    }

    pub(crate) fn verify_generation(
        &self,
        generation: &IntegrityGeneration,
    ) -> Result<(), EngineError> {
        if generation.schema_version != REVISION_STORE_SCHEMA_VERSION
            || generation.canonical_encoding != CANONICAL_ENCODING
            || generation.digest_algorithm != DIGEST_ALGORITHM
        {
            return Err(EngineError::Validation(
                "unsupported revision integrity generation encoding".to_string(),
            ));
        }
        validate_digest(&generation.integrity_root)?;
        let calculated = digest_bytes(&canonical_bytes(&generation.material())?);
        if calculated != generation.integrity_root {
            return Err(EngineError::Validation(
                "revision integrity root mismatch".to_string(),
            ));
        }
        let mut digests = vec![&generation.transaction_blob];
        digests.extend(generation.authority_snapshot_blob.iter());
        digests.extend(
            generation
                .affected_shards
                .iter()
                .filter_map(|entry| entry.postimage_blob.as_ref()),
        );
        for digest in digests {
            let bytes = std::fs::read(self.blob_path(digest))?;
            if digest_bytes(&bytes) != *digest {
                return Err(EngineError::Validation(format!(
                    "revision blob {} is corrupt",
                    digest.0
                )));
            }
        }
        let transaction: TransactionRecord = serde_json::from_slice(&std::fs::read(
            self.blob_path(&generation.transaction_blob),
        )?)?;
        if transaction.transaction_id != generation.transaction_id
            || transaction.before_model_revision != generation.before_model_revision
            || transaction.after_model_revision != generation.after_model_revision
        {
            return Err(EngineError::Validation(
                "revision transaction blob does not match generation".to_string(),
            ));
        }
        Ok(())
    }

    fn verify_staged(
        &self,
        stage_root: &Path,
        generation: &IntegrityGeneration,
    ) -> Result<(), EngineError> {
        let calculated = digest_bytes(&canonical_bytes(&generation.material())?);
        if calculated != generation.integrity_root {
            return Err(EngineError::Validation(
                "staged revision integrity root mismatch".to_string(),
            ));
        }
        for digest in std::iter::once(&generation.transaction_blob)
            .chain(generation.authority_snapshot_blob.iter())
            .chain(
                generation
                    .affected_shards
                    .iter()
                    .filter_map(|entry| entry.postimage_blob.as_ref()),
            )
        {
            let staged_path = stage_root.join("blobs/sha256").join(digest_hex(digest)?);
            let bytes = std::fs::read(if staged_path.exists() {
                staged_path
            } else {
                self.blob_path(digest)
            })?;
            if digest_bytes(&bytes) != *digest {
                return Err(EngineError::Validation(
                    "staged revision blob digest mismatch".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn read_head(&self) -> Option<Result<IntegrityHead, EngineError>> {
        let path = self.root.join("head.json");
        path.exists().then(|| read_json(&path))
    }

    pub(crate) fn blob_path(&self, digest: &AlgorithmQualifiedDigest) -> PathBuf {
        self.root
            .join("blobs/sha256")
            .join(digest_hex(digest).unwrap_or_else(|_| "invalid".to_string()))
    }

    pub(crate) fn generation_path(&self, digest: &AlgorithmQualifiedDigest) -> PathBuf {
        self.root.join("generations").join(format!(
            "{}.json",
            digest_hex(digest).unwrap_or_else(|_| "invalid".to_string())
        ))
    }
}

impl StagedIntegrityGeneration {
    pub(crate) fn promote(self) -> Result<IntegrityHead, EngineError> {
        self.store
            .verify_staged(&self.stage_root, &self.generation)?;
        if let Some(current) = self.store.read_head().transpose()? {
            if current.transaction_id == self.generation.transaction_id
                && current.integrity_root == self.generation.integrity_root
            {
                let _ = std::fs::remove_dir_all(&self.stage_root);
                return Ok(current);
            }
            if Some(current.transaction_id) != self.generation.parent_transaction_id
                || Some(current.integrity_root) != self.generation.parent_integrity_root
            {
                return Err(EngineError::Validation(
                    "staged revision generation does not extend the active head".to_string(),
                ));
            }
        } else if self.generation.parent_integrity_root.is_some() {
            return Err(EngineError::Validation(
                "staged revision generation has no active parent head".to_string(),
            ));
        }
        std::fs::create_dir_all(self.store.root.join("blobs/sha256"))?;
        std::fs::create_dir_all(self.store.root.join("generations"))?;
        for digest in std::iter::once(&self.generation.transaction_blob)
            .chain(self.generation.authority_snapshot_blob.iter())
            .chain(
                self.generation
                    .affected_shards
                    .iter()
                    .filter_map(|entry| entry.postimage_blob.as_ref()),
            )
        {
            let source = self
                .stage_root
                .join("blobs/sha256")
                .join(digest_hex(digest)?);
            let destination = self.store.blob_path(digest);
            if !source.exists() && destination.exists() {
                if digest_bytes(&std::fs::read(&destination)?) != *digest {
                    return Err(EngineError::Validation(format!(
                        "carried revision blob {} is corrupt",
                        digest.0
                    )));
                }
                continue;
            }
            promote_immutable(&source, &destination)?;
        }
        let generation_path = self.store.generation_path(&self.generation.integrity_root);
        promote_immutable(&self.stage_root.join("generation.json"), &generation_path)?;
        let head = IntegrityHead {
            schema_version: REVISION_STORE_SCHEMA_VERSION,
            project_id: self.generation.project_id,
            transaction_id: self.generation.transaction_id,
            integrity_root: self.generation.integrity_root.clone(),
        };
        write_atomic(&self.store.root.join("head.json"), &canonical_bytes(&head)?)?;
        let _ = std::fs::remove_dir_all(&self.stage_root);
        sync_tree(&self.store.root)?;
        Ok(head)
    }
}

pub fn transaction_tip(journal: &[TransactionRecord]) -> Option<Uuid> {
    journal.last().map(|record| record.transaction_id)
}

fn digest_hex(digest: &AlgorithmQualifiedDigest) -> Result<String, EngineError> {
    validate_digest(digest)?;
    Ok(digest.0[7..].to_string())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, EngineError> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    sync_parent(path)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    write_new_file(&temporary, bytes)?;
    std::fs::rename(&temporary, path)?;
    sync_parent(path)
}

fn promote_immutable(source: &Path, destination: &Path) -> Result<(), EngineError> {
    if destination.exists() {
        if std::fs::read(source)? != std::fs::read(destination)? {
            return Err(EngineError::Validation(format!(
                "immutable revision path conflict: {}",
                destination.display()
            )));
        }
        std::fs::remove_file(source)?;
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(source, destination)?;
    sync_parent(destination)
}

fn sync_parent(path: &Path) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn sync_tree(path: &Path) -> Result<(), EngineError> {
    let mut dirs = BTreeMap::new();
    dirs.insert(path.to_path_buf(), ());
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            dirs.insert(entry.path(), ());
        }
    }
    for directory in dirs.into_keys() {
        File::open(directory)?.sync_all()?;
    }
    Ok(())
}

#[allow(dead_code)]
fn journal_exists(project_root: &Path) -> bool {
    transaction_journal_path(project_root).exists()
}
