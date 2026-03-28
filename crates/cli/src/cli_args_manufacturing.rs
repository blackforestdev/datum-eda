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
