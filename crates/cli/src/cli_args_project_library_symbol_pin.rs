use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolSymbolPinAnchorArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Unit pin UUID to anchor on the symbol
    #[arg(long = "pin")]
    pub(crate) pin_uuid: Uuid,
    /// Pin anchor X coordinate in nanometers
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Pin anchor Y coordinate in nanometers
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Pin anchor orientation: Left, Right, Up, or Down
    #[arg(long, default_value = "Right")]
    pub(crate) orientation: String,
    /// Pin line length in nanometers; omit to use the renderer default
    #[arg(long = "length-nm")]
    pub(crate) length_nm: Option<i64>,
    /// Pin anchor decoration: none, inverted, clock, or inverted_clock
    #[arg(long, default_value = "none")]
    pub(crate) decoration: String,
}
