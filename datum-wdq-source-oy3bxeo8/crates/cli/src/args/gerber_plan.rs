use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct PlanGerberExportArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ExportGerberSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to write the planned artifact set into
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct CompareGerberExportPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to compare against the planned artifact set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ValidateGerberSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to validate against the planned artifact set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct CompareGerberSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to compare semantically against the planned artifact set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}
