use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ExportDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output CSV path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(Args)]
pub(crate) struct ValidateDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drill CSV path to validate
    #[arg(long = "drill")]
    pub(crate) drill: PathBuf,
}

#[derive(Args)]
pub(crate) struct CompareDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drill CSV path to compare
    #[arg(long = "drill")]
    pub(crate) drill: PathBuf,
}

#[derive(Args)]
pub(crate) struct ExportExcellonDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output drill path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(Args)]
pub(crate) struct InspectDrillArgs {
    /// Drill CSV path to inspect
    pub(crate) path: PathBuf,
}

#[derive(Args)]
pub(crate) struct CompareExcellonDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drill path to compare
    #[arg(long = "drill")]
    pub(crate) drill: PathBuf,
}

#[derive(Args)]
pub(crate) struct ValidateExcellonDrillArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drill path to validate
    #[arg(long = "drill")]
    pub(crate) drill: PathBuf,
}

#[derive(Args)]
pub(crate) struct ReportDrillHoleClassesArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}
