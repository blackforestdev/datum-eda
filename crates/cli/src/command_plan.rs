use super::*;
use serde::Deserialize;

const SCOPED_REPLACEMENT_MANIFEST_KIND: &str = "scoped_component_replacement_plan_manifest";
const SCOPED_REPLACEMENT_MANIFEST_VERSION: u32 = 1;

#[derive(Deserialize)]
struct LegacyScopedReplacementPlanManifestV0 {
    board_path: PathBuf,
    board_source_hash: String,
    libraries: Vec<ManifestFileFingerprint>,
    plan: ScopedComponentReplacementPlan,
}

pub(super) fn parse_scoped_replacement_override_arg(
    value: &str,
) -> Result<ScopedComponentReplacementOverride> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 {
        bail!(
            "--override-component expects <component_uuid>:<target_package_uuid>:<target_part_uuid>"
        );
    }
    Ok(ScopedComponentReplacementOverride {
        component_uuid: Uuid::parse_str(parts[0])?,
        target_package_uuid: Uuid::parse_str(parts[1])?,
        target_part_uuid: Uuid::parse_str(parts[2])?,
    })
}

pub(super) fn scoped_replacement_manifest_from_parts(
    board_path: &Path,
    libraries: &[PathBuf],
    plan: ScopedComponentReplacementPlan,
) -> Result<ScopedReplacementPlanManifest> {
    Ok(ScopedReplacementPlanManifest {
        kind: SCOPED_REPLACEMENT_MANIFEST_KIND.to_string(),
        version: SCOPED_REPLACEMENT_MANIFEST_VERSION,
        board_path: board_path.to_path_buf(),
        board_source_hash: eda_engine::import::ids_sidecar::compute_source_hash_file(board_path)?,
        libraries: libraries
            .iter()
            .map(|path| {
                Ok(ManifestFileFingerprint {
                    path: path.clone(),
                    source_hash: eda_engine::import::ids_sidecar::compute_source_hash_file(path)?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        plan,
    })
}

pub(super) fn load_scoped_replacement_manifest(path: &Path) -> Result<ScopedReplacementPlanManifest> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read scoped replacement manifest {}", path.display()))?;
    let value = serde_json::from_str::<serde_json::Value>(&contents)
        .with_context(|| format!("failed to parse scoped replacement manifest {}", path.display()))?;

    let kind = value.get("kind").and_then(serde_json::Value::as_str);
    if let Some(kind) = kind {
        if kind != SCOPED_REPLACEMENT_MANIFEST_KIND {
            bail!(
                "unsupported scoped replacement manifest kind '{}' in {}",
                kind,
                path.display()
            );
        }
    }

    let version = match value.get("version") {
        Some(version) => {
            let raw = version.as_u64().ok_or_else(|| {
                let msg = format!(
                    "invalid scoped replacement manifest version in {}",
                    path.display()
                );
                anyhow::Error::msg(msg)
            })?;
            u32::try_from(raw).map_err(|_| {
                let msg = format!(
                    "invalid scoped replacement manifest version in {}",
                    path.display()
                );
                anyhow::Error::msg(msg)
            })?
        }
        None => 0,
    };

    match version {
        0 => {
            let manifest = serde_json::from_value::<LegacyScopedReplacementPlanManifestV0>(value)
                .with_context(|| {
                    format!(
                        "failed to parse legacy scoped replacement manifest {}",
                        path.display()
                    )
                })?;
            Ok(ScopedReplacementPlanManifest {
                kind: SCOPED_REPLACEMENT_MANIFEST_KIND.to_string(),
                version: SCOPED_REPLACEMENT_MANIFEST_VERSION,
                board_path: manifest.board_path,
                board_source_hash: manifest.board_source_hash,
                libraries: manifest.libraries,
                plan: manifest.plan,
            })
        }
        SCOPED_REPLACEMENT_MANIFEST_VERSION => {
            let manifest = serde_json::from_value::<ScopedReplacementPlanManifest>(value)
                .with_context(|| {
                    format!("failed to parse scoped replacement manifest {}", path.display())
                })?;
            if manifest.kind != SCOPED_REPLACEMENT_MANIFEST_KIND {
                bail!(
                    "unsupported scoped replacement manifest kind '{}' in {}",
                    manifest.kind,
                    path.display()
                );
            }
            Ok(manifest)
        }
        _ => {
            bail!(
                "unsupported scoped replacement manifest version {} in {}",
                version,
                path.display()
            );
        }
    }
}

pub(super) fn validate_scoped_replacement_manifest(
    manifest: &ScopedReplacementPlanManifest,
    board_path: &Path,
) -> Result<()> {
    let board_hash = eda_engine::import::ids_sidecar::compute_source_hash_file(board_path)?;
    if board_hash != manifest.board_source_hash {
        bail!(
            "scoped replacement manifest board hash drifted for {}; refresh the manifest before apply",
            board_path.display()
        );
    }
    for library in &manifest.libraries {
        let current_hash = eda_engine::import::ids_sidecar::compute_source_hash_file(&library.path)?;
        if current_hash != library.source_hash {
            bail!(
                "scoped replacement manifest library hash drifted for {}; refresh the manifest before apply",
                library.path.display()
            );
        }
    }
    Ok(())
}

fn inspect_manifest_file(
    path: &Path,
    recorded_source_hash: &str,
) -> Result<ManifestFileInspection> {
    if !path.exists() {
        return Ok(ManifestFileInspection {
            path: path.to_path_buf(),
            recorded_source_hash: recorded_source_hash.to_string(),
            current_source_hash: None,
            status: ManifestDriftStatus::Missing,
        });
    }

    let current_source_hash = eda_engine::import::ids_sidecar::compute_source_hash_file(path)?;
    let status = if current_source_hash == recorded_source_hash {
        ManifestDriftStatus::Match
    } else {
        ManifestDriftStatus::Drifted
    };
    Ok(ManifestFileInspection {
        path: path.to_path_buf(),
        recorded_source_hash: recorded_source_hash.to_string(),
        current_source_hash: Some(current_source_hash),
        status,
    })
}

pub(super) fn inspect_scoped_replacement_manifest(
    manifest_path: &Path,
) -> Result<ScopedReplacementPlanManifestInspection> {
    let manifest = load_scoped_replacement_manifest(manifest_path)?;
    let board = inspect_manifest_file(&manifest.board_path, &manifest.board_source_hash)?;
    let libraries = manifest
        .libraries
        .iter()
        .map(|library| inspect_manifest_file(&library.path, &library.source_hash))
        .collect::<Result<Vec<_>>>()?;
    let all_inputs_match = board.status == ManifestDriftStatus::Match
        && libraries
            .iter()
            .all(|library| library.status == ManifestDriftStatus::Match);

    Ok(ScopedReplacementPlanManifestInspection {
        manifest_path: manifest_path.to_path_buf(),
        kind: manifest.kind,
        version: manifest.version,
        replacements: manifest.plan.replacements.len(),
        all_inputs_match,
        board,
        libraries,
    })
}
