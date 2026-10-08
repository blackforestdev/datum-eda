//! Process-owned serialization for cooperating native Project writers.
use std::{
    fs::{File, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use crate::error::EngineError;

/// A non-cloneable guard for one canonical root. The coordination inode survives
/// release; ownership belongs to the descriptor, never to a PID record.
#[derive(Debug)]
pub struct ProjectWriteLease {
    root: PathBuf,
    path: PathBuf,
    file: File,
}

impl ProjectWriteLease {
    pub fn acquire(project_root: &Path) -> Result<Self, EngineError> {
        let root = project_root.canonicalize()?;
        if !root.is_dir() {
            return Err(refusal("Project root is not a directory"));
        }
        let datum = root.join(".datum");
        create_directory_durable(&datum)?;
        let directory = datum.join("revision");
        create_directory_durable(&directory)?;
        if std::fs::symlink_metadata(directory.join("write.lock")).is_ok() {
            return Err(refusal(
                "legacy write.lock requires reconciliation; mixed writers unsupported",
            ));
        }
        let path = directory.join("writer.lock");
        if let Ok(metadata) = std::fs::symlink_metadata(&path)
            && !metadata.is_file()
        {
            return Err(refusal("writer lock is not a regular file"));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&path)?;
        // std's Unix opens are close-on-exec. Never duplicate this descriptor.
        file.try_lock()
            .map_err(|error| refusal(&format!("live writer or unavailable lock: {error}")))?;
        let lease = Self { root, path, file };
        lease.validate(project_root)?;
        lease.file.sync_all()?;
        File::open(directory)?.sync_all()?;
        Ok(lease)
    }

    pub(crate) fn validate(&self, project_root: &Path) -> Result<(), EngineError> {
        if project_root.canonicalize()? != self.root {
            return Err(refusal("writer guard belongs to a different Project"));
        }
        for path in [self.root.join(".datum"), self.root.join(".datum/revision")] {
            if !std::fs::symlink_metadata(path)?.is_dir() {
                return Err(refusal("writer directory identity changed"));
            }
        }
        let named = std::fs::symlink_metadata(&self.path)?;
        let opened = self.file.metadata()?;
        if !named.is_file() || named.dev() != opened.dev() || named.ino() != opened.ino() {
            return Err(refusal("writer coordination inode identity changed"));
        }
        Ok(())
    }

    /// Probe ownership without creating a lock or mutating coordination bytes.
    pub(crate) fn is_busy(project_root: &Path) -> bool {
        if std::fs::symlink_metadata(project_root.join(".datum/revision/write.lock")).is_ok() {
            return true;
        }
        let path = project_root.join(".datum/revision/writer.lock");
        match std::fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Ok(metadata) if metadata.is_file() => OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .and_then(|file| file.try_lock().map_err(std::io::Error::from))
                .is_err(),
            _ => true,
        }
    }
}

impl Drop for ProjectWriteLease {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn refusal(message: &str) -> EngineError {
    EngineError::Operation(format!("project write refused: {message}"))
}

pub(crate) fn create_directory_durable(path: impl AsRef<Path>) -> Result<(), EngineError> {
    let path = path.as_ref();
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(refusal("writer directory is a symlink or non-directory")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            create_directory_durable(
                path.parent()
                    .ok_or_else(|| refusal("missing directory parent"))?,
            )?;
            match std::fs::create_dir(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !std::fs::symlink_metadata(path)?.is_dir() {
                        return Err(refusal("writer directory creation raced with unsafe path"));
                    }
                }
                Err(error) => return Err(error.into()),
            }
            File::open(path)?.sync_all()?;
            File::open(
                path.parent()
                    .ok_or_else(|| refusal("missing writer parent"))?,
            )?
            .sync_all()?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}
