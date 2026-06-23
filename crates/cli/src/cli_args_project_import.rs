use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectImportKiCadFootprintArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// KiCad .kicad_mod source file
    #[arg(long)]
    pub(crate) source: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
}
