use std::path::{Path, PathBuf};

use serde::Serialize;
use uuid::Uuid;

use super::ResolveDiagnostic;
use crate::ir::serialization::to_json_deterministic;

pub(super) fn persist_generated_evidence<T: Serialize>(
    project_root: &Path,
    relative_dir: &str,
    id: &Uuid,
    value: &T,
) -> Result<PathBuf, crate::error::EngineError> {
    let directory = project_root.join(relative_dir);
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{id}.json"));
    let temp_path = directory.join(format!("{id}.json.tmp"));
    let json = to_json_deterministic(value)?;
    let bytes = format!("{json}\n");
    std::fs::write(&temp_path, bytes.as_bytes())?;
    std::fs::File::open(&temp_path)?.sync_all()?;
    std::fs::rename(&temp_path, &path)?;
    sync_directory(&directory)?;
    Ok(path)
}

pub(super) fn validate_filename_uuid(
    path: &Path,
    expected: Uuid,
    code: &str,
) -> Result<(), ResolveDiagnostic> {
    let actual = path.file_stem().and_then(|value| value.to_str());
    let expected = expected.to_string();
    if actual == Some(expected.as_str()) {
        return Ok(());
    }
    Err(ResolveDiagnostic {
        code: code.to_string(),
        message: format!(
            "manifest filename does not match embedded id {expected}: {}",
            path.display()
        ),
        path: Some(path.to_path_buf()),
    })
}

fn sync_directory(path: &Path) -> Result<(), crate::error::EngineError> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}
