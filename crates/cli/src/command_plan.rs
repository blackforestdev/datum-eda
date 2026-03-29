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

pub(super) struct LoadedScopedReplacementManifest {
    pub(super) manifest_path: PathBuf,
    pub(super) manifest: ScopedReplacementPlanManifest,
    pub(super) source_version: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ScopedReplacementPlanManifestArtifactValidationReport {
    pub(super) manifest_path: PathBuf,
    pub(super) kind: String,
    pub(super) source_version: u32,
    pub(super) version: u32,
    pub(super) migration_applied: bool,
    pub(super) replacements: usize,
    pub(super) matches_expected: bool,
    pub(super) canonical_bytes_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ScopedReplacementPlanManifestArtifactInspection {
    pub(super) manifest_path: PathBuf,
    pub(super) kind: String,
    pub(super) source_version: u32,
    pub(super) version: u32,
    pub(super) migration_applied: bool,
    pub(super) replacements: usize,
    pub(super) board_path: PathBuf,
    pub(super) libraries: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ScopedReplacementPlanManifestArtifactComparisonReport {
    pub(super) manifest_path: PathBuf,
    pub(super) artifact_path: PathBuf,
    pub(super) manifest_source_version: u32,
    pub(super) artifact_source_version: u32,
    pub(super) manifest_version: u32,
    pub(super) artifact_version: u32,
    pub(super) manifest_migration_applied: bool,
    pub(super) artifact_migration_applied: bool,
    pub(super) manifest_replacements: usize,
    pub(super) artifact_replacements: usize,
    pub(super) matches_artifact: bool,
    pub(super) drift_fields: Vec<String>,
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

pub(super) fn render_scoped_replacement_manifest_export_text(
    path: &Path,
    kind: &str,
    version: u32,
    replacements: usize,
) -> String {
    [
        format!("manifest: {}", path.display()),
        format!("kind: {kind}"),
        format!("version: {version}"),
        format!("replacements: {replacements}"),
    ]
    .join("\n")
}

pub(super) fn load_scoped_replacement_manifest(
    path: &Path,
) -> Result<ScopedReplacementPlanManifest> {
    Ok(load_scoped_replacement_manifest_with_metadata(path)?.manifest)
}

pub(super) fn load_scoped_replacement_manifest_with_metadata(
    path: &Path,
) -> Result<LoadedScopedReplacementManifest> {
    let contents = std::fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read scoped replacement manifest {}",
            path.display()
        )
    })?;
    let value = serde_json::from_str::<serde_json::Value>(&contents).with_context(|| {
        format!(
            "failed to parse scoped replacement manifest {}",
            path.display()
        )
    })?;

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
            Ok(LoadedScopedReplacementManifest {
                manifest_path: path.to_path_buf(),
                manifest: ScopedReplacementPlanManifest {
                    kind: SCOPED_REPLACEMENT_MANIFEST_KIND.to_string(),
                    version: SCOPED_REPLACEMENT_MANIFEST_VERSION,
                    board_path: manifest.board_path,
                    board_source_hash: manifest.board_source_hash,
                    libraries: manifest.libraries,
                    plan: manifest.plan,
                },
                source_version: 0,
            })
        }
        SCOPED_REPLACEMENT_MANIFEST_VERSION => {
            let manifest = serde_json::from_value::<ScopedReplacementPlanManifest>(value)
                .with_context(|| {
                    format!(
                        "failed to parse scoped replacement manifest {}",
                        path.display()
                    )
                })?;
            if manifest.kind != SCOPED_REPLACEMENT_MANIFEST_KIND {
                bail!(
                    "unsupported scoped replacement manifest kind '{}' in {}",
                    manifest.kind,
                    path.display()
                );
            }
            Ok(LoadedScopedReplacementManifest {
                manifest_path: path.to_path_buf(),
                manifest,
                source_version: SCOPED_REPLACEMENT_MANIFEST_VERSION,
            })
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

pub(super) fn upgrade_scoped_replacement_manifest(
    input_path: &Path,
    output_path: &Path,
) -> Result<ScopedReplacementPlanManifestUpgradeReport> {
    let loaded = load_scoped_replacement_manifest_with_metadata(input_path)?;
    let payload = serde_json::to_string_pretty(&loaded.manifest)
        .expect("manifest serialization must succeed");
    std::fs::write(output_path, payload).with_context(|| {
        format!(
            "failed to write upgraded manifest {}",
            output_path.display()
        )
    })?;
    Ok(ScopedReplacementPlanManifestUpgradeReport {
        input_path: input_path.to_path_buf(),
        output_path: output_path.to_path_buf(),
        kind: loaded.manifest.kind.clone(),
        source_version: loaded.source_version,
        version: loaded.manifest.version,
        migration_applied: loaded.source_version != loaded.manifest.version,
        replacements: loaded.manifest.plan.replacements.len(),
    })
}

pub(super) fn render_scoped_replacement_manifest_upgrade_text(
    report: &ScopedReplacementPlanManifestUpgradeReport,
) -> String {
    [
        format!("input: {}", report.input_path.display()),
        format!("output: {}", report.output_path.display()),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("replacements: {}", report.replacements),
    ]
    .join("\n")
}

pub(super) fn validate_scoped_replacement_manifest_artifact(
    manifest_path: &Path,
) -> Result<ScopedReplacementPlanManifestArtifactValidationReport> {
    let contents = std::fs::read_to_string(manifest_path).with_context(|| {
        format!(
            "failed to read scoped replacement manifest {}",
            manifest_path.display()
        )
    })?;
    let loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
    let expected = serde_json::to_string_pretty(&loaded.manifest)
        .expect("manifest serialization must succeed");
    let canonical_bytes_match = contents == expected;
    Ok(ScopedReplacementPlanManifestArtifactValidationReport {
        manifest_path: loaded.manifest_path,
        kind: loaded.manifest.kind,
        source_version: loaded.source_version,
        version: loaded.manifest.version,
        migration_applied: loaded.source_version != loaded.manifest.version,
        replacements: loaded.manifest.plan.replacements.len(),
        matches_expected: canonical_bytes_match,
        canonical_bytes_match,
    })
}

pub(super) fn inspect_scoped_replacement_manifest_artifact(
    manifest_path: &Path,
) -> Result<ScopedReplacementPlanManifestArtifactInspection> {
    let loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
    Ok(ScopedReplacementPlanManifestArtifactInspection {
        manifest_path: loaded.manifest_path,
        kind: loaded.manifest.kind,
        source_version: loaded.source_version,
        version: loaded.manifest.version,
        migration_applied: loaded.source_version != loaded.manifest.version,
        replacements: loaded.manifest.plan.replacements.len(),
        board_path: loaded.manifest.board_path,
        libraries: loaded.manifest.libraries.len(),
    })
}

pub(super) fn render_scoped_replacement_manifest_artifact_inspection_text(
    report: &ScopedReplacementPlanManifestArtifactInspection,
) -> String {
    [
        format!("manifest: {}", report.manifest_path.display()),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("replacements: {}", report.replacements),
        format!("board_path: {}", report.board_path.display()),
        format!("libraries: {}", report.libraries),
    ]
    .join("\n")
}

pub(super) fn render_scoped_replacement_manifest_artifact_validation_text(
    report: &ScopedReplacementPlanManifestArtifactValidationReport,
) -> String {
    [
        format!("manifest: {}", report.manifest_path.display()),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("replacements: {}", report.replacements),
        format!("matches_expected: {}", report.matches_expected),
        format!("canonical_bytes_match: {}", report.canonical_bytes_match),
    ]
    .join("\n")
}

pub(super) fn compare_scoped_replacement_manifest_artifact(
    manifest_path: &Path,
    artifact_path: &Path,
) -> Result<ScopedReplacementPlanManifestArtifactComparisonReport> {
    let manifest_loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
    let artifact_loaded = load_scoped_replacement_manifest_with_metadata(artifact_path)?;

    let mut drift_fields = Vec::new();
    if manifest_loaded.manifest.board_path != artifact_loaded.manifest.board_path {
        drift_fields.push("board_path".to_string());
    }
    if manifest_loaded.manifest.board_source_hash != artifact_loaded.manifest.board_source_hash {
        drift_fields.push("board_source_hash".to_string());
    }
    if manifest_loaded.manifest.libraries != artifact_loaded.manifest.libraries {
        drift_fields.push("libraries".to_string());
    }
    if manifest_loaded.manifest.plan.scope != artifact_loaded.manifest.plan.scope {
        drift_fields.push("scope".to_string());
    }
    if manifest_loaded.manifest.plan.policy != artifact_loaded.manifest.plan.policy {
        drift_fields.push("policy".to_string());
    }
    if manifest_loaded.manifest.plan.replacements != artifact_loaded.manifest.plan.replacements {
        drift_fields.push("replacements".to_string());
    }

    Ok(ScopedReplacementPlanManifestArtifactComparisonReport {
        manifest_path: manifest_loaded.manifest_path,
        artifact_path: artifact_loaded.manifest_path,
        manifest_source_version: manifest_loaded.source_version,
        artifact_source_version: artifact_loaded.source_version,
        manifest_version: manifest_loaded.manifest.version,
        artifact_version: artifact_loaded.manifest.version,
        manifest_migration_applied: manifest_loaded.source_version != manifest_loaded.manifest.version,
        artifact_migration_applied: artifact_loaded.source_version != artifact_loaded.manifest.version,
        manifest_replacements: manifest_loaded.manifest.plan.replacements.len(),
        artifact_replacements: artifact_loaded.manifest.plan.replacements.len(),
        matches_artifact: drift_fields.is_empty(),
        drift_fields,
    })
}

pub(super) fn render_scoped_replacement_manifest_artifact_comparison_text(
    report: &ScopedReplacementPlanManifestArtifactComparisonReport,
) -> String {
    [
        format!("manifest: {}", report.manifest_path.display()),
        format!("artifact: {}", report.artifact_path.display()),
        format!("manifest_source_version: {}", report.manifest_source_version),
        format!("artifact_source_version: {}", report.artifact_source_version),
        format!("manifest_version: {}", report.manifest_version),
        format!("artifact_version: {}", report.artifact_version),
        format!(
            "manifest_migration_applied: {}",
            report.manifest_migration_applied
        ),
        format!(
            "artifact_migration_applied: {}",
            report.artifact_migration_applied
        ),
        format!("manifest_replacements: {}", report.manifest_replacements),
        format!("artifact_replacements: {}", report.artifact_replacements),
        format!("matches_artifact: {}", report.matches_artifact),
        format!("drift_fields: {}", report.drift_fields.join(", ")),
    ]
    .join("\n")
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
        let current_hash =
            eda_engine::import::ids_sidecar::compute_source_hash_file(&library.path)?;
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
    let loaded = load_scoped_replacement_manifest_with_metadata(manifest_path)?;
    let board = inspect_manifest_file(
        &loaded.manifest.board_path,
        &loaded.manifest.board_source_hash,
    )?;
    let libraries = loaded
        .manifest
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
        kind: loaded.manifest.kind,
        source_version: loaded.source_version,
        version: loaded.manifest.version,
        migration_applied: loaded.source_version != loaded.manifest.version,
        replacements: loaded.manifest.plan.replacements.len(),
        all_inputs_match,
        board,
        libraries,
    })
}

pub(super) fn render_scoped_replacement_manifest_inspection_text(
    inspection: &ScopedReplacementPlanManifestInspection,
) -> String {
    let mut lines = vec![
        format!("manifest: {}", inspection.manifest_path.display()),
        format!("kind: {}", inspection.kind),
        format!("source_version: {}", inspection.source_version),
        format!("version: {}", inspection.version),
        format!("migration_applied: {}", inspection.migration_applied),
        format!("replacements: {}", inspection.replacements),
        format!("all_inputs_match: {}", inspection.all_inputs_match),
        "board:".to_string(),
        format!(
            "  {} [{}]",
            inspection.board.path.display(),
            render_manifest_drift_status(inspection.board.status)
        ),
    ];
    if !inspection.libraries.is_empty() {
        lines.push("libraries:".to_string());
        for library in &inspection.libraries {
            lines.push(format!(
                "  {} [{}]",
                library.path.display(),
                render_manifest_drift_status(library.status)
            ));
        }
    }
    lines.join("\n")
}

fn render_manifest_drift_status(status: ManifestDriftStatus) -> &'static str {
    match status {
        ManifestDriftStatus::Match => "match",
        ManifestDriftStatus::Drifted => "drifted",
        ManifestDriftStatus::Missing => "missing",
    }
}

pub(super) fn validate_scoped_replacement_manifest_inputs(
    manifest_path: &Path,
) -> Result<ScopedReplacementPlanManifestValidationReport> {
    let inspection = inspect_scoped_replacement_manifest(manifest_path)?;
    let drifted_libraries = inspection
        .libraries
        .iter()
        .filter(|library| library.status == ManifestDriftStatus::Drifted)
        .count();
    let missing_libraries = inspection
        .libraries
        .iter()
        .filter(|library| library.status == ManifestDriftStatus::Missing)
        .count();
    Ok(ScopedReplacementPlanManifestValidationReport {
        manifest_path: inspection.manifest_path,
        source_version: inspection.source_version,
        version: inspection.version,
        migration_applied: inspection.migration_applied,
        all_inputs_match: inspection.all_inputs_match,
        board_status: inspection.board.status,
        drifted_libraries,
        missing_libraries,
    })
}

pub(super) fn validate_scoped_replacement_manifest_inputs_batch(
    manifest_paths: &[PathBuf],
) -> Result<ScopedReplacementPlanManifestValidationSummary> {
    let reports = manifest_paths
        .iter()
        .map(|path| validate_scoped_replacement_manifest_inputs(path))
        .collect::<Result<Vec<_>>>()?;
    let manifests_checked = reports.len();
    let manifests_passing = reports
        .iter()
        .filter(|report| report.all_inputs_match)
        .count();
    Ok(ScopedReplacementPlanManifestValidationSummary {
        manifests_checked,
        manifests_passing,
        manifests_failing: manifests_checked - manifests_passing,
        reports,
    })
}

pub(super) fn render_scoped_replacement_manifest_validation_text(
    summary: &ScopedReplacementPlanManifestValidationSummary,
) -> String {
    let mut lines = vec![
        format!("manifests_checked: {}", summary.manifests_checked),
        format!("manifests_passing: {}", summary.manifests_passing),
        format!("manifests_failing: {}", summary.manifests_failing),
    ];
    for report in &summary.reports {
        lines.push(format!("manifest: {}", report.manifest_path.display()));
        lines.push(format!("  source_version: {}", report.source_version));
        lines.push(format!("  version: {}", report.version));
        lines.push(format!("  migration_applied: {}", report.migration_applied));
        lines.push(format!("  all_inputs_match: {}", report.all_inputs_match));
        lines.push(format!(
            "  board_status: {}",
            render_manifest_drift_status(report.board_status)
        ));
        lines.push(format!("  drifted_libraries: {}", report.drifted_libraries));
        lines.push(format!("  missing_libraries: {}", report.missing_libraries));
    }
    lines.join("\n")
}
