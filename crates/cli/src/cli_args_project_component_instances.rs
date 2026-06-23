use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectBindComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional ComponentInstance UUID; deterministic from refs when omitted
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Option<Uuid>,
    /// Schematic symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol: Uuid,
    /// Board package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ComponentInstance UUID
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Uuid,
    /// Replacement schematic symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol: Uuid,
    /// Replacement board package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ComponentInstance UUID
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Uuid,
}
