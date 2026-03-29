use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ExportBomArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output CSV path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(Args)]
pub(crate) struct CompareBomArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// BOM CSV path to compare
    #[arg(long = "bom")]
    pub(crate) bom: PathBuf,
}

#[derive(Args)]
pub(crate) struct ValidateBomArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// BOM CSV path to validate
    #[arg(long = "bom")]
    pub(crate) bom: PathBuf,
}

#[derive(Args)]
pub(crate) struct InspectBomArgs {
    /// BOM CSV path to inspect
    pub(crate) path: PathBuf,
}

#[derive(Args)]
pub(crate) struct ExportPnpArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Output CSV path
    #[arg(long = "out")]
    pub(crate) out: PathBuf,
}

#[derive(Args)]
pub(crate) struct ComparePnpArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// PnP CSV path to compare
    #[arg(long = "pnp")]
    pub(crate) pnp: PathBuf,
}

#[derive(Args)]
pub(crate) struct ValidatePnpArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// PnP CSV path to validate
    #[arg(long = "pnp")]
    pub(crate) pnp: PathBuf,
}

#[derive(Args)]
pub(crate) struct InspectPnpArgs {
    /// PnP CSV path to inspect
    pub(crate) path: PathBuf,
}
