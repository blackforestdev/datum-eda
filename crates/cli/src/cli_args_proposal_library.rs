use super::*;

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Pool object kind, e.g. symbols, units, parts, packages
    #[arg(long = "kind")]
    pub(crate) kind: String,
    /// Pool object UUID
    #[arg(long = "object")]
    pub(crate) object: Uuid,
    /// JSON payload for the pool object
    #[arg(long = "from-json")]
    pub(crate) from_json: PathBuf,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolUnitArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Unit UUID
    #[arg(long = "unit")]
    pub(crate) unit: Uuid,
    /// Unit name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Unit manufacturer
    #[arg(long = "manufacturer", default_value = "")]
    pub(crate) manufacturer: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol: Uuid,
    /// Referenced unit UUID
    #[arg(long = "unit")]
    pub(crate) unit: Uuid,
    /// Symbol name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolEntityArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Entity UUID
    #[arg(long = "entity")]
    pub(crate) entity: Uuid,
    /// Gate UUID
    #[arg(long = "gate")]
    pub(crate) gate: Uuid,
    /// Referenced unit UUID
    #[arg(long = "unit")]
    pub(crate) unit: Uuid,
    /// Referenced symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol: Uuid,
    /// Entity name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Reference prefix
    #[arg(long = "prefix")]
    pub(crate) prefix: String,
    /// Entity manufacturer
    #[arg(long = "manufacturer", default_value = "")]
    pub(crate) manufacturer: String,
    /// Gate name
    #[arg(long = "gate-name", default_value = "A")]
    pub(crate) gate_name: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolPadstackArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack: Uuid,
    /// Padstack name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Padstack aperture kind: circle or rect
    #[arg(long = "aperture")]
    pub(crate) aperture: Option<String>,
    /// Circle aperture diameter in nm
    #[arg(long = "diameter-nm")]
    pub(crate) diameter_nm: Option<i64>,
    /// Rect aperture width in nm
    #[arg(long = "width-nm")]
    pub(crate) width_nm: Option<i64>,
    /// Rect aperture height in nm
    #[arg(long = "height-nm")]
    pub(crate) height_nm: Option<i64>,
    /// Optional drill diameter in nm
    #[arg(long = "drill-nm")]
    pub(crate) drill_nm: Option<i64>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolPackageArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Human-readable package name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Initial pad UUID
    #[arg(long = "pad")]
    pub(crate) pad: Uuid,
    /// Referenced padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack: Uuid,
    /// Human-readable pad name
    #[arg(long = "pad-name", default_value = "1")]
    pub(crate) pad_name: String,
    /// Pad X position in nanometers
    #[arg(long = "x-nm", default_value_t = 0)]
    pub(crate) x_nm: i64,
    /// Pad Y position in nanometers
    #[arg(long = "y-nm", default_value_t = 0)]
    pub(crate) y_nm: i64,
    /// Numeric layer id; 1 is top copper
    #[arg(long = "layer", default_value_t = 1)]
    pub(crate) layer: i32,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolPackagePadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad: Uuid,
    /// Referenced padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack: Uuid,
    /// Human-readable pad name
    #[arg(long = "pad-name", default_value = "1")]
    pub(crate) pad_name: String,
    /// Pad X position in nanometers
    #[arg(long = "x-nm", default_value_t = 0)]
    pub(crate) x_nm: i64,
    /// Pad Y position in nanometers
    #[arg(long = "y-nm", default_value_t = 0)]
    pub(crate) y_nm: i64,
    /// Numeric layer id; 1 is top copper
    #[arg(long = "layer", default_value_t = 1)]
    pub(crate) layer: i32,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolPackageCourtyardRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Minimum X coordinate in nanometers
    #[arg(long = "min-x-nm")]
    pub(crate) min_x_nm: i64,
    /// Minimum Y coordinate in nanometers
    #[arg(long = "min-y-nm")]
    pub(crate) min_y_nm: i64,
    /// Maximum X coordinate in nanometers
    #[arg(long = "max-x-nm")]
    pub(crate) max_x_nm: i64,
    /// Maximum Y coordinate in nanometers
    #[arg(long = "max-y-nm")]
    pub(crate) max_y_nm: i64,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolPackageCourtyardPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Vertices as x,y pairs separated by semicolons: x,y;x,y;x,y
    #[arg(long = "vertices")]
    pub(crate) vertices: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}
