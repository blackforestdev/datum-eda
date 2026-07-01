use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectGenerateBoardComponentsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Commit generated board packages through the project journal
    #[arg(long)]
    pub(crate) apply: bool,
    /// Initial X coordinate in nm
    #[arg(long = "origin-x-nm", default_value_t = 0)]
    pub(crate) origin_x_nm: i64,
    /// Initial Y coordinate in nm
    #[arg(long = "origin-y-nm", default_value_t = 0)]
    pub(crate) origin_y_nm: i64,
    /// X pitch between generated packages in nm
    #[arg(long = "pitch-nm", default_value_t = 5_000_000)]
    pub(crate) pitch_nm: i64,
    /// Layer identifier
    #[arg(long, default_value_t = 1)]
    pub(crate) layer: i32,
}
