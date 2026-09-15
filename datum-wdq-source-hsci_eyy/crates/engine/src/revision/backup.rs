use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    canonical::{canonical_bytes, digest_bytes},
    model::{AlgorithmQualifiedDigest, IntegrityHead, REVISION_STORE_SCHEMA_VERSION},
    store::{ProjectWriteLease, RevisionAuthorityStore},
};

const BACKUP_MANIFEST_NAME: &str = "backup-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupEntry {
    pub relative_path: String,
    pub byte_length: u64,
    pub digest: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupManifest {
    pub schema_version: u64,
    pub project_id: Uuid,
    pub head: IntegrityHead,
    pub files: Vec<BackupEntry>,
    pub completeness_root: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupManifestMaterial {
    schema_version: u64,
    project_id: Uuid,
    head: IntegrityHead,
    files: Vec<BackupEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorePreview {
    pub project_id: Uuid,
    pub incoming_head: IntegrityHead,
    pub current_head: Option<IntegrityHead>,
    pub file_count: usize,
    pub completeness_root: AlgorithmQualifiedDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreReceipt {
    pub schema_version: u64,
    pub project_id: Uuid,
    pub restored_head: IntegrityHead,
    pub previous_store: Option<PathBuf>,
    pub completeness_root: AlgorithmQualifiedDigest,
}

pub fn export_backup(
    project_root: &Path,
    project_id: Uuid,
    destination: &Path,
) -> Result<BackupManifest, EngineError> {
    let _lease = ProjectWriteLease::acquire(project_root)?;
    let store = RevisionAuthorityStore::new(project_root);
    let head = store.read_head().transpose()?.ok_or_else(|| {
        EngineError::Validation("cannot back up an empty revision integrity store".to_string())
    })?;
    if head.project_id != project_id {
        return Err(EngineError::Validation(
            "backup project identity mismatch".to_string(),
        ));
    }
    if destination.exists() {
        return Err(EngineError::Operation(format!(
            "backup destination already exists: {}",
            destination.display()
        )));
    }
    std::fs::create_dir_all(destination)?;

    let mut source_paths = Vec::new();
    collect_files(&store.root, &store.root, &mut source_paths)?;
    source_paths.retain(|relative| {
        !relative.starts_with("stage/") && relative != "write.lock" && !relative.ends_with(".tmp")
    });
    source_paths.sort();
    let mut files = Vec::with_capacity(source_paths.len());
    for relative in source_paths {
        let bytes = std::fs::read(store.root.join(&relative))?;
        write_new(&destination.join(&relative), &bytes)?;
        files.push(BackupEntry {
            relative_path: relative,
            byte_length: bytes.len() as u64,
            digest: digest_bytes(&bytes),
        });
    }
    let material = BackupManifestMaterial {
        schema_version: REVISION_STORE_SCHEMA_VERSION,
        project_id,
        head,
        files,
    };
    let completeness_root = digest_bytes(&canonical_bytes(&material)?);
    let manifest = BackupManifest {
        schema_version: material.schema_version,
        project_id: material.project_id,
        head: material.head,
        files: material.files,
        completeness_root,
    };
    write_new(
        &destination.join(BACKUP_MANIFEST_NAME),
        &canonical_bytes(&manifest)?,
    )?;
    sync_dir(destination)?;
    Ok(manifest)
}

pub fn verify_backup(path: &Path) -> Result<BackupManifest, EngineError> {
    let manifest: BackupManifest =
        serde_json::from_slice(&std::fs::read(path.join(BACKUP_MANIFEST_NAME))?)?;
    if manifest.schema_version != REVISION_STORE_SCHEMA_VERSION {
        return Err(EngineError::Validation(format!(
            "unsupported revision backup schema_version {}",
            manifest.schema_version
        )));
    }
    let material = BackupManifestMaterial {
        schema_version: manifest.schema_version,
        project_id: manifest.project_id,
        head: manifest.head.clone(),
        files: manifest.files.clone(),
    };
    if digest_bytes(&canonical_bytes(&material)?) != manifest.completeness_root {
        return Err(EngineError::Validation(
            "revision backup completeness root mismatch".to_string(),
        ));
    }
    let mut declared = BTreeSet::new();
    for entry in &manifest.files {
        validate_relative_path(&entry.relative_path)?;
        if !declared.insert(entry.relative_path.clone()) {
            return Err(EngineError::Validation(format!(
                "duplicate revision backup entry `{}`",
                entry.relative_path
            )));
        }
        let bytes = std::fs::read(path.join(&entry.relative_path))?;
        if bytes.len() as u64 != entry.byte_length || digest_bytes(&bytes) != entry.digest {
            return Err(EngineError::Validation(format!(
                "revision backup entry `{}` failed independent verification",
                entry.relative_path
            )));
        }
    }
    let mut actual = Vec::new();
    collect_files(path, path, &mut actual)?;
    actual.retain(|relative| relative != BACKUP_MANIFEST_NAME);
    let actual: BTreeSet<_> = actual.into_iter().collect();
    if actual != declared {
        return Err(EngineError::Validation(
            "revision backup file set does not match its completeness manifest".to_string(),
        ));
    }
    let restored_store = RevisionAuthorityStore::from_store_root(path.to_path_buf());
    let generation: super::IntegrityGeneration = serde_json::from_slice(&std::fs::read(
        restored_store.generation_path(&manifest.head.integrity_root),
    )?)?;
    restored_store.verify_generation(&generation)?;
    Ok(manifest)
}

pub fn preview_restore(project_root: &Path, backup: &Path) -> Result<RestorePreview, EngineError> {
    let manifest = verify_backup(backup)?;
    let current_head = RevisionAuthorityStore::new(project_root)
        .read_head()
        .transpose()?;
    Ok(RestorePreview {
        project_id: manifest.project_id,
        incoming_head: manifest.head,
        current_head,
        file_count: manifest.files.len(),
        completeness_root: manifest.completeness_root,
    })
}

pub fn restore_backup(project_root: &Path, backup: &Path) -> Result<RestoreReceipt, EngineError> {
    let preview = preview_restore(project_root, backup)?;
    let _lease = ProjectWriteLease::acquire(project_root)?;
    let datum_root = project_root.join(".datum/revision");
    std::fs::create_dir_all(&datum_root)?;
    let active = datum_root.join("v1");
    let stage = datum_root.join(format!("restore-stage-{}", Uuid::new_v4()));
    let previous = datum_root.join(format!("restore-previous-{}", Uuid::new_v4()));
    copy_verified_backup(backup, &stage)?;

    let previous_store = if active.exists() {
        std::fs::rename(&active, &previous)?;
        Some(previous.clone())
    } else {
        None
    };
    if let Err(error) = std::fs::rename(&stage, &active) {
        if previous_store.is_some() {
            let _ = std::fs::rename(&previous, &active);
        }
        return Err(error.into());
    }
    sync_dir(&datum_root)?;
    Ok(RestoreReceipt {
        schema_version: REVISION_STORE_SCHEMA_VERSION,
        project_id: preview.project_id,
        restored_head: preview.incoming_head,
        previous_store,
        completeness_root: preview.completeness_root,
    })
}

pub fn undo_restore(project_root: &Path, receipt: &RestoreReceipt) -> Result<(), EngineError> {
    let Some(previous) = &receipt.previous_store else {
        return Err(EngineError::Operation(
            "restore has no previous store to reinstate".to_string(),
        ));
    };
    let _lease = ProjectWriteLease::acquire(project_root)?;
    let active = project_root.join(".datum/revision/v1");
    let displaced = project_root
        .join(".datum/revision")
        .join(format!("restore-displaced-{}", Uuid::new_v4()));
    std::fs::rename(&active, &displaced)?;
    if let Err(error) = std::fs::rename(previous, &active) {
        let _ = std::fs::rename(&displaced, &active);
        return Err(error.into());
    }
    sync_dir(&project_root.join(".datum/revision"))
}

fn copy_verified_backup(source: &Path, destination: &Path) -> Result<(), EngineError> {
    let manifest = verify_backup(source)?;
    std::fs::create_dir_all(destination)?;
    for entry in manifest.files {
        write_new(
            &destination.join(&entry.relative_path),
            &std::fs::read(source.join(entry.relative_path))?,
        )?;
    }
    sync_dir(destination)
}

fn collect_files(
    root: &Path,
    directory: &Path,
    output: &mut Vec<String>,
) -> Result<(), EngineError> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            return Err(EngineError::Validation(format!(
                "revision backup/store contains unsupported symlink: {}",
                entry.path().display()
            )));
        }
        if file_type.is_dir() {
            collect_files(root, &entry.path(), output)?;
        } else if file_type.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|_| {
                    EngineError::Validation("revision backup path escaped store root".to_string())
                })?
                .to_string_lossy()
                .replace('\\', "/");
            output.push(relative);
        } else {
            return Err(EngineError::Validation(format!(
                "revision backup/store contains unsupported filesystem entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), EngineError> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || path.is_empty()
        || candidate
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(EngineError::Validation(format!(
            "unsafe revision backup path `{path}`"
        )));
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        sync_dir(parent)?;
    }
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), EngineError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
