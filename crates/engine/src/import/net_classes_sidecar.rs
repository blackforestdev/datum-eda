use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::NetClass;
use crate::error::EngineError;
use crate::ir::serialization::to_json_deterministic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetClassesSidecar {
    pub schema_version: u32,
    pub source_file: String,
    pub source_hash: String,
    pub classes: Vec<NetClass>,
    pub assignments: BTreeMap<Uuid, Uuid>,
}

impl NetClassesSidecar {
    pub fn new(
        source_file: impl Into<String>,
        source_hash: impl Into<String>,
        mut classes: Vec<NetClass>,
        assignments: BTreeMap<Uuid, Uuid>,
    ) -> Self {
        classes.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        Self {
            schema_version: 1,
            source_file: source_file.into(),
            source_hash: source_hash.into(),
            classes,
            assignments,
        }
    }
}

pub fn sidecar_path_for_source(source_file: impl AsRef<Path>) -> PathBuf {
    let source = source_file.as_ref();
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "design".to_string());
    source.with_file_name(format!("{file_name}.net-classes.json"))
}

pub fn write_sidecar(
    path: impl AsRef<Path>,
    sidecar: &NetClassesSidecar,
) -> Result<(), EngineError> {
    let json = to_json_deterministic(sidecar)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn read_sidecar(path: impl AsRef<Path>) -> Result<NetClassesSidecar, EngineError> {
    let text = std::fs::read_to_string(path)?;
    let sidecar: NetClassesSidecar = serde_json::from_str(&text)
        .map_err(|e| EngineError::Validation(format!("invalid net-class sidecar JSON: {e}")))?;
    if sidecar.schema_version != 1 {
        return Err(EngineError::Validation(format!(
            "unsupported net-class sidecar schema version: {}",
            sidecar.schema_version
        )));
    }
    Ok(sidecar)
}
