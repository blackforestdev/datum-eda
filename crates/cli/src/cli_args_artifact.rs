use super::*;

#[derive(Subcommand)]
pub(crate) enum ArtifactCommands {
    /// Generate derived production artifacts from include scopes
    Generate(ArtifactGenerateArgs),
    /// Persist a running OutputJobRun evidence record for one OutputJob
    StartOutputJobRun(ArtifactStartOutputJobRunArgs),
    /// Mark an existing OutputJobRun evidence record canceled
    CancelOutputJobRun(ArtifactCancelOutputJobRunArgs),
    /// List resolver-discovered generated artifact metadata
    List(ArtifactListArgs),
    /// Show one resolver-discovered generated artifact metadata record
    Show(ArtifactShowArgs),
    /// Show generated files and projection proofs for one artifact
    Files(ArtifactFilesArgs),
    /// Preview one generated artifact file through supported semantic readers
    Preview(ArtifactPreviewArgs),
    /// Compare two resolver-discovered artifact metadata records
    Compare(ArtifactCompareArgs),
    /// Validate one resolver-discovered artifact metadata record
    Validate(ArtifactValidateArgs),
    /// Export the current manufacturing set and persist artifact/run evidence
    ExportManufacturingSet(ExportManufacturingSetArgs),
    /// Validate a manufacturing set artifact directory against current project state
    ValidateManufacturingSet(ValidateManufacturingSetArgs),
}

#[derive(clap::Args)]
pub(crate) struct ArtifactGenerateArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Output directory for generated artifact files, or an authored OutputJob run override
    #[arg(long = "output-dir")]
    pub(crate) output_dir: Option<PathBuf>,

    /// Comma-separated include scopes: gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,

    /// Optional output filename prefix
    #[arg(long)]
    pub(crate) prefix: Option<String>,

    /// Execute one authored OutputJob instead of direct include-scope generation
    #[arg(long = "output-job", conflicts_with_all = ["include", "prefix"])]
    pub(crate) output_job: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactStartOutputJobRunArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// OutputJob UUID to mark running
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactCancelOutputJobRunArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// OutputJobRun UUID to mark canceled
    #[arg(long = "run")]
    pub(crate) run: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactListArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactShowArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Artifact UUID to inspect
    #[arg(long)]
    pub(crate) artifact: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactFilesArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Artifact UUID to inspect
    #[arg(long)]
    pub(crate) artifact: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactPreviewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Artifact UUID that owns the generated file
    #[arg(long)]
    pub(crate) artifact: Uuid,

    /// Directory containing generated artifact files; defaults to stored artifact output_dir
    #[arg(long = "artifact-dir")]
    pub(crate) artifact_dir: Option<PathBuf>,

    /// Relative generated file path from the artifact metadata
    #[arg(long)]
    pub(crate) file: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactCompareArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Baseline artifact UUID
    #[arg(long)]
    pub(crate) before: Uuid,

    /// Candidate artifact UUID
    #[arg(long)]
    pub(crate) after: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ArtifactValidateArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Artifact UUID to validate
    #[arg(long)]
    pub(crate) artifact: Uuid,
}
