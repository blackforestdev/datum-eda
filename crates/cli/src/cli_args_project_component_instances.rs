use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectBindComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional ComponentInstance UUID; deterministic from refs when omitted
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Option<Uuid>,
    /// Schematic symbol UUID; repeat for multi-unit component instances
    #[arg(long = "symbol", required = true)]
    pub(crate) symbols: Vec<Uuid>,
    /// Board package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Optional native pool part UUID for this electrical-to-physical component identity
    #[arg(long = "part")]
    pub(crate) part: Option<Uuid>,
    /// Per-symbol role metadata as <symbol-uuid>=<role>[:label]
    #[arg(long = "symbol-role")]
    pub(crate) symbol_roles: Vec<String>,
    /// Per-package role metadata as <package-uuid>=<role>[:label]
    #[arg(long = "package-role")]
    pub(crate) package_roles: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ComponentInstance UUID
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Uuid,
    /// Replacement schematic symbol UUID; repeat for multi-unit component instances
    #[arg(long = "symbol", required = true)]
    pub(crate) symbols: Vec<Uuid>,
    /// Replacement board package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Replacement native pool part UUID; omitted preserves/clears no authored part reference
    #[arg(long = "part")]
    pub(crate) part: Option<Uuid>,
    /// Replacement per-symbol role metadata as <symbol-uuid>=<role>[:label]
    #[arg(long = "symbol-role")]
    pub(crate) symbol_roles: Vec<String>,
    /// Replacement per-package role metadata as <package-uuid>=<role>[:label]
    #[arg(long = "package-role")]
    pub(crate) package_roles: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteComponentInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ComponentInstance UUID
    #[arg(long = "component-instance")]
    pub(crate) component_instance: Uuid,
}
