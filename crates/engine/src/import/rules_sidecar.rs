use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::EngineError;
use crate::ir::serialization::to_json_deterministic;
use crate::rules::ast::Rule;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSidecar {
    pub schema_version: u32,
    pub source_file: String,
    pub source_hash: String,
    pub rules: Vec<Rule>,
}

impl RuleSidecar {
    pub fn new(
        source_file: impl Into<String>,
        source_hash: impl Into<String>,
        mut rules: Vec<Rule>,
    ) -> Self {
        rules.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.uuid.cmp(&b.uuid))
        });
        Self {
            schema_version: 1,
            source_file: source_file.into(),
            source_hash: source_hash.into(),
            rules,
        }
    }
}

pub fn sidecar_path_for_source(source_file: impl AsRef<Path>) -> PathBuf {
    let path = source_file.as_ref();
    let filename = path
        .file_name()
        .expect("source file must have filename")
        .to_string_lossy();
    path.with_file_name(format!("{filename}.rules.json"))
}

pub fn write_sidecar(path: impl AsRef<Path>, sidecar: &RuleSidecar) -> Result<(), EngineError> {
    let json = to_json_deterministic(sidecar)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn read_sidecar(path: impl AsRef<Path>) -> Result<RuleSidecar, EngineError> {
    let text = fs::read_to_string(path)?;
    let sidecar: RuleSidecar = serde_json::from_str(&text)
        .map_err(|e| EngineError::Validation(format!("invalid rule sidecar JSON: {e}")))?;
    if sidecar.schema_version != 1 {
        return Err(EngineError::Validation(format!(
            "unsupported rule sidecar schema version: {}",
            sidecar.schema_version
        )));
    }
    Ok(sidecar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ast::{RuleParams, RuleScope, RuleType};
    use uuid::Uuid;

    #[test]
    fn sidecar_path_appends_rules_json() {
        let path = sidecar_path_for_source("/tmp/example/board.kicad_pcb");
        assert!(path.ends_with("board.kicad_pcb.rules.json"));
    }

    #[test]
    fn write_read_sidecar_round_trip() {
        let dir = std::env::temp_dir().join(format!("eda-rule-sidecar-{}", std::process::id()));
        let path = dir.join("example.rules.json");
        fs::create_dir_all(&dir).unwrap();

        let sidecar = RuleSidecar::new(
            "board.kicad_pcb",
            "sha256:test",
            vec![Rule {
                uuid: Uuid::from_u128(1),
                name: "clearance".into(),
                scope: RuleScope::All,
                priority: 1,
                enabled: true,
                rule_type: RuleType::ClearanceCopper,
                parameters: RuleParams::Clearance { min: 100_000 },
            }],
        );

        write_sidecar(&path, &sidecar).unwrap();
        let decoded = read_sidecar(&path).unwrap();
        assert_eq!(sidecar, decoded);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&dir);
    }
}
