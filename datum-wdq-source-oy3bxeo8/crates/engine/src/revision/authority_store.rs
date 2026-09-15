use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AlgorithmQualifiedDigest, AuthoritySnapshot, IntegrityGeneration, IntegrityHead,
    ProjectWriteLease, REVISION_STORE_SCHEMA_VERSION, RevisionAuthorityStore,
    canonical::{canonical_bytes, digest_bytes},
};

impl RevisionAuthorityStore {
    #[allow(dead_code)]
    pub(crate) fn commit_authority_snapshot(
        &self,
        project_id: Uuid,
        expected_integrity_root: &AlgorithmQualifiedDigest,
        snapshot: &AuthoritySnapshot,
    ) -> Result<IntegrityHead, EngineError> {
        if snapshot.project_id != project_id || !snapshot.validate().is_empty() {
            return Err(EngineError::Validation(
                "typed revision authority snapshot failed validation".to_string(),
            ));
        }
        let project_root = self
            .root
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| {
                EngineError::Validation("revision store has no Project root".to_string())
            })?;
        let _lease = ProjectWriteLease::acquire(project_root)?;
        let head = self.read_head().transpose()?.ok_or_else(|| {
            EngineError::Validation(
                "authority mutation requires an accepted Project transaction".to_string(),
            )
        })?;
        if head.project_id != project_id || &head.integrity_root != expected_integrity_root {
            return Err(EngineError::Operation(
                "revision authority generation changed before commit".to_string(),
            ));
        }
        let mut generation: IntegrityGeneration =
            serde_json::from_slice(&std::fs::read(self.generation_path(&head.integrity_root))?)?;
        self.verify_generation(&generation)?;
        let snapshot_bytes = snapshot.canonical_bytes()?;
        let snapshot_digest = digest_bytes(&snapshot_bytes);
        write_immutable(&self.blob_path(&snapshot_digest), &snapshot_bytes)?;
        generation.authority_snapshot_blob = Some(snapshot_digest);
        generation.integrity_root = digest_bytes(&canonical_bytes(&generation.material())?);
        let generation_bytes = canonical_bytes(&generation)?;
        write_immutable(
            &self.generation_path(&generation.integrity_root),
            &generation_bytes,
        )?;
        let replacement = IntegrityHead {
            schema_version: REVISION_STORE_SCHEMA_VERSION,
            project_id,
            transaction_id: generation.transaction_id,
            integrity_root: generation.integrity_root,
        };
        write_atomic(
            &self.root.join("head.json"),
            &canonical_bytes(&replacement)?,
        )?;
        sync_directory(&self.root)?;
        Ok(replacement)
    }
}

fn write_immutable(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    if path.exists() {
        if std::fs::read(path)? != bytes {
            return Err(EngineError::Validation(format!(
                "immutable revision authority path conflict: {}",
                path.display()
            )));
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    sync_parent(path)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    write_immutable(&temporary, bytes)?;
    std::fs::rename(&temporary, path)?;
    sync_parent(path)
}

fn sync_parent(path: &Path) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), EngineError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
