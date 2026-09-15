// Shared CLI arg/report types that belong to no single arg family. The
// legacy cli_args_surface.rs re-export shim was dissolved into args/mod.rs;
// only these type definitions remain.
use crate::*;

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeLabelKindArg {
    Local,
    Global,
    Hierarchical,
    Power,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileFingerprint {
    pub(crate) path: PathBuf,
    pub(crate) source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifest {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) board_path: PathBuf,
    pub(crate) board_source_hash: String,
    pub(crate) libraries: Vec<ManifestFileFingerprint>,
    pub(crate) plan: ScopedComponentReplacementPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ManifestDriftStatus {
    Match,
    Drifted,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileInspection {
    pub(crate) path: PathBuf,
    pub(crate) recorded_source_hash: String,
    pub(crate) current_source_hash: Option<String>,
    pub(crate) status: ManifestDriftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestInspection {
    pub(crate) manifest_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
    pub(crate) all_inputs_match: bool,
    pub(crate) board: ManifestFileInspection,
    pub(crate) libraries: Vec<ManifestFileInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestUpgradeReport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) all_inputs_match: bool,
    pub(crate) board_status: ManifestDriftStatus,
    pub(crate) drifted_libraries: usize,
    pub(crate) missing_libraries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationSummary {
    pub(crate) manifests_checked: usize,
    pub(crate) manifests_passing: usize,
    pub(crate) manifests_failing: usize,
    pub(crate) reports: Vec<ScopedReplacementPlanManifestValidationReport>,
}
