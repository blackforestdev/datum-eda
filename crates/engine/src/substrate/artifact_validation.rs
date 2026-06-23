use std::path::{Component, Path};

use super::artifact::ArtifactMetadata;

pub(super) fn validate_artifact_metadata(metadata: &ArtifactMetadata) -> Result<(), String> {
    if metadata
        .output_dir
        .as_ref()
        .is_some_and(|path| path.as_os_str().is_empty())
    {
        return Err("artifact output_dir must not be empty".to_string());
    }
    for file in &metadata.files {
        validate_artifact_file_path(&file.path)?;
        validate_sha256_digest("artifact file sha256", &file.sha256)?;
    }
    for projection in &metadata.production_projections {
        if projection.projection_kind.trim().is_empty() {
            return Err("artifact production projection kind must not be blank".to_string());
        }
        if projection.projection_contract.trim().is_empty() {
            return Err("artifact production projection contract must not be blank".to_string());
        }
        if projection.model_revision != metadata.model_revision {
            return Err(format!(
                "artifact production projection model_revision {} does not match artifact model_revision {}",
                projection.model_revision.0, metadata.model_revision.0
            ));
        }
        validate_sha256_digest("artifact production projection sha256", &projection.sha256)?;
    }
    Ok(())
}

fn validate_sha256_digest(label: &str, value: &str) -> Result<(), String> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(format!("{label} must be a sha256:<64 lowercase hex> value"));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(format!("{label} must be a sha256:<64 lowercase hex> value"));
    }
    Ok(())
}

fn validate_artifact_file_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("artifact file path must not be empty".to_string());
    }
    if path.is_absolute() {
        return Err(format!(
            "artifact file path must be relative: {}",
            path.display()
        ));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(format!(
                    "artifact file path contains unsafe component: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}
