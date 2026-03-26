use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::EngineError;
use crate::ir::serialization::to_json_deterministic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Kicad,
    Eagle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdSidecar {
    pub schema_version: u32,
    pub format: ImportFormat,
    pub source_file: String,
    pub source_hash: String,
    pub generated_at: String,
    pub mappings: HashMap<String, String>,
}

impl IdSidecar {
    pub fn new(
        format: ImportFormat,
        source_file: impl Into<String>,
        source_hash: impl Into<String>,
        generated_at: impl Into<String>,
        mappings: HashMap<String, String>,
    ) -> Self {
        Self {
            schema_version: 1,
            format,
            source_file: source_file.into(),
            source_hash: source_hash.into(),
            generated_at: generated_at.into(),
            mappings,
        }
    }

    pub fn mapping_uuid(&self, object_path: &str) -> Result<Option<Uuid>, EngineError> {
        match self.mappings.get(object_path) {
            Some(value) => Ok(Some(Uuid::parse_str(value).map_err(|e| {
                EngineError::Validation(format!("invalid UUID in sidecar: {e}"))
            })?)),
            None => Ok(None),
        }
    }
}

pub fn compute_source_hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{:x}", digest)
}

pub fn compute_source_hash_file(path: impl AsRef<Path>) -> Result<String, EngineError> {
    let bytes = fs::read(path)?;
    Ok(compute_source_hash_bytes(&bytes))
}

pub fn sidecar_path_for_source(source_file: impl AsRef<Path>) -> PathBuf {
    let path = source_file.as_ref();
    let filename = path
        .file_name()
        .expect("source file must have filename")
        .to_string_lossy();
    path.with_file_name(format!("{filename}.ids.json"))
}

pub fn write_sidecar(path: impl AsRef<Path>, sidecar: &IdSidecar) -> Result<(), EngineError> {
    let path = path.as_ref();
    let json = to_json_deterministic(sidecar)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn read_sidecar(path: impl AsRef<Path>) -> Result<IdSidecar, EngineError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    let sidecar: IdSidecar = serde_json::from_str(&text)
        .map_err(|e| EngineError::Validation(format!("invalid sidecar JSON: {e}")))?;
    if sidecar.schema_version != 1 {
        return Err(EngineError::Validation(format!(
            "unsupported sidecar schema version: {}",
            sidecar.schema_version
        )));
    }
    Ok(sidecar)
}

pub fn merge_mappings(
    computed_paths: impl IntoIterator<Item = String>,
    existing: Option<&IdSidecar>,
    namespace: &Uuid,
) -> HashMap<String, String> {
    computed_paths
        .into_iter()
        .map(|path| {
            let uuid = existing
                .and_then(|sidecar| sidecar.mappings.get(&path).cloned())
                .unwrap_or_else(|| Uuid::new_v5(namespace, path.as_bytes()).to_string());
            (path, uuid)
        })
        .collect()
}

pub fn restore_or_merge_mappings(
    computed_paths: impl IntoIterator<Item = String>,
    existing: Option<&IdSidecar>,
    current_source_hash: &str,
    namespace: &Uuid,
) -> HashMap<String, String> {
    match existing {
        Some(sidecar) if sidecar.source_hash == current_source_hash => sidecar.mappings.clone(),
        Some(sidecar) => merge_mappings(computed_paths, Some(sidecar), namespace),
        None => merge_mappings(computed_paths, None, namespace),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::ir::ids::namespace_eagle;

    fn sample_sidecar() -> IdSidecar {
        IdSidecar::new(
            ImportFormat::Kicad,
            "board.kicad_pcb",
            "sha256:abc123",
            "2026-03-24T12:00:00Z",
            HashMap::from([
                ("net:VCC".to_string(), Uuid::from_u128(1).to_string()),
                ("net:GND".to_string(), Uuid::from_u128(2).to_string()),
            ]),
        )
    }

    #[test]
    fn sidecar_path_appends_ids_json() {
        let path = sidecar_path_for_source("/tmp/example/board.kicad_pcb");
        assert!(path.ends_with("board.kicad_pcb.ids.json"));
    }

    #[test]
    fn source_hash_is_prefixed_sha256() {
        let hash = compute_source_hash_bytes(b"hello");
        assert!(hash.starts_with("sha256:"));
    }

    #[test]
    fn write_read_sidecar_round_trip() {
        let dir = std::env::temp_dir().join(format!("eda-sidecar-{}", std::process::id()));
        let path = dir.join("example.ids.json");
        fs::create_dir_all(&dir).unwrap();

        let sidecar = sample_sidecar();
        write_sidecar(&path, &sidecar).unwrap();
        let decoded = read_sidecar(&path).unwrap();
        assert_eq!(sidecar, decoded);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_exact_when_hash_matches() {
        let sidecar = sample_sidecar();
        let restored = restore_or_merge_mappings(
            vec!["net:VCC".to_string(), "net:GND".to_string()],
            Some(&sidecar),
            "sha256:abc123",
            &namespace_eagle(),
        );
        assert_eq!(restored, sidecar.mappings);
    }

    #[test]
    fn merge_preserves_existing_and_adds_new_paths() {
        let sidecar = sample_sidecar();
        let merged = restore_or_merge_mappings(
            vec![
                "net:VCC".to_string(),
                "net:GND".to_string(),
                "net:SIG".to_string(),
            ],
            Some(&sidecar),
            "sha256:different",
            &namespace_eagle(),
        );

        assert_eq!(
            merged.get("net:VCC").unwrap(),
            &Uuid::from_u128(1).to_string()
        );
        assert_eq!(
            merged.get("net:GND").unwrap(),
            &Uuid::from_u128(2).to_string()
        );
        assert!(Uuid::parse_str(merged.get("net:SIG").unwrap()).is_ok());
        assert_eq!(
            merged.keys().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                "net:VCC".to_string(),
                "net:GND".to_string(),
                "net:SIG".to_string()
            ])
        );
    }
}
