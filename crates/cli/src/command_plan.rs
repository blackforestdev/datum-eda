use super::*;

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
        kind: "scoped_component_replacement_plan_manifest".to_string(),
        version: 1,
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
    let manifest = serde_json::from_str::<ScopedReplacementPlanManifest>(&contents)
        .with_context(|| format!("failed to parse scoped replacement manifest {}", path.display()))?;
    if manifest.kind != "scoped_component_replacement_plan_manifest" {
        bail!(
            "unsupported scoped replacement manifest kind '{}' in {}",
            manifest.kind,
            path.display()
        );
    }
    if manifest.version != 1 {
        bail!(
            "unsupported scoped replacement manifest version {} in {}",
            manifest.version,
            path.display()
        );
    }
    Ok(manifest)
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
