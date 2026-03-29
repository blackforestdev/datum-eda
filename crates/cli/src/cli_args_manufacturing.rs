use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ReportManufacturingArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional Gerber artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ExportManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to write the current supported manufacturing set into
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ValidateManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to validate against the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct CompareManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to compare semantically against the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ManifestManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory containing or intended to contain the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct InspectManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory containing the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}
