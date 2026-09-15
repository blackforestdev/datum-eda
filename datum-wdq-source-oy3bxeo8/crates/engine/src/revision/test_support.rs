use std::{fs::OpenOptions, io::Write, path::Path};

use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AuthoritySnapshot, IntegrityGeneration, IntegrityHead, REVISION_STORE_SCHEMA_VERSION,
    RevisionAuthorityStore,
    canonical::{canonical_bytes, digest_bytes},
};

impl RevisionAuthorityStore {
    pub(crate) fn install_authority_fixture(
        &self,
        project_id: Uuid,
        authority_snapshot: &AuthoritySnapshot,
    ) -> Result<IntegrityHead, EngineError> {
        if authority_snapshot.project_id != project_id || !authority_snapshot.validate().is_empty()
        {
            return Err(EngineError::Validation(
                "authority fixture failed structural validation".to_string(),
            ));
        }
        self.install_authority_bytes_for_test(project_id, &authority_snapshot.canonical_bytes()?)
    }

    pub(crate) fn install_authority_bytes_for_test(
        &self,
        project_id: Uuid,
        bytes: &[u8],
    ) -> Result<IntegrityHead, EngineError> {
        let head = self.read_head().transpose()?.ok_or_else(|| {
            EngineError::Validation(
                "authority fixture requires an integrity generation".to_string(),
            )
        })?;
        let mut generation: IntegrityGeneration =
            serde_json::from_slice(&std::fs::read(self.generation_path(&head.integrity_root))?)?;
        self.verify_generation(&generation)?;
        let digest = digest_bytes(bytes);
        let blob_path = self.blob_path(&digest);
        if !blob_path.exists() {
            write_new(&blob_path, bytes)?;
        }
        generation.authority_snapshot_blob = Some(digest);
        generation.integrity_root = digest_bytes(&canonical_bytes(&generation.material())?);
        let generation_path = self.generation_path(&generation.integrity_root);
        if !generation_path.exists() {
            write_new(&generation_path, &canonical_bytes(&generation)?)?;
        }
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
        Ok(replacement)
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    write_new(&temporary, bytes)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}
