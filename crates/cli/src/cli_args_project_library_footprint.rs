use super::*;

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum IpcDensityLevelArg {
    Most,
    Nominal,
    Least,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolFootprintArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Human-readable footprint name
    #[arg(long)]
    pub(crate) name: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectGenerateIpc7351bTwoTerminalChipArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Generated Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Generated Padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack_uuid: Uuid,
    /// Generated pad 1 UUID
    #[arg(long = "pad-a")]
    pub(crate) pad_a_uuid: Uuid,
    /// Generated pad 2 UUID
    #[arg(long = "pad-b")]
    pub(crate) pad_b_uuid: Uuid,
    /// Optional human-readable footprint name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// IPC metric package code, e.g. 0603 or 1608
    #[arg(long = "metric-code")]
    pub(crate) metric_code: String,
    /// Component body length in nanometers
    #[arg(long = "body-length-nm")]
    pub(crate) body_length_nm: i64,
    /// Component body width in nanometers
    #[arg(long = "body-width-nm")]
    pub(crate) body_width_nm: i64,
    /// Terminal length in nanometers
    #[arg(long = "terminal-length-nm")]
    pub(crate) terminal_length_nm: i64,
    /// Terminal width in nanometers
    #[arg(long = "terminal-width-nm")]
    pub(crate) terminal_width_nm: i64,
    /// IPC density level
    #[arg(long = "density", value_enum, default_value = "nominal")]
    pub(crate) density: IpcDensityLevelArg,
    /// Solder-mask expansion in nanometers
    #[arg(long = "mask-expansion-nm", default_value_t = 50_000)]
    pub(crate) mask_expansion_nm: i64,
    /// Paste aperture reduction in nanometers
    #[arg(long = "paste-reduction-nm", default_value_t = 50_000)]
    pub(crate) paste_reduction_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolFootprintPadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
    /// Referenced padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack_uuid: Uuid,
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
    #[arg(long, default_value_t = 1)]
    pub(crate) layer: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolFootprintCourtyardRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Minimum X bound in nanometers
    #[arg(long = "min-x-nm")]
    pub(crate) min_x_nm: i64,
    /// Minimum Y bound in nanometers
    #[arg(long = "min-y-nm")]
    pub(crate) min_y_nm: i64,
    /// Maximum X bound in nanometers
    #[arg(long = "max-x-nm")]
    pub(crate) max_x_nm: i64,
    /// Maximum Y bound in nanometers
    #[arg(long = "max-y-nm")]
    pub(crate) max_y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolFootprintCourtyardPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Polygon vertices as x,y;x,y;... in nanometers
    #[arg(long)]
    pub(crate) vertices: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolFootprintSilkscreenLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Line start X coordinate in nanometers
    #[arg(long)]
    pub(crate) from_x_nm: i64,
    /// Line start Y coordinate in nanometers
    #[arg(long)]
    pub(crate) from_y_nm: i64,
    /// Line end X coordinate in nanometers
    #[arg(long)]
    pub(crate) to_x_nm: i64,
    /// Line end Y coordinate in nanometers
    #[arg(long)]
    pub(crate) to_y_nm: i64,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolFootprintSilkscreenRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Minimum X bound in nanometers
    #[arg(long = "min-x-nm")]
    pub(crate) min_x_nm: i64,
    /// Minimum Y bound in nanometers
    #[arg(long = "min-y-nm")]
    pub(crate) min_y_nm: i64,
    /// Maximum X bound in nanometers
    #[arg(long = "max-x-nm")]
    pub(crate) max_x_nm: i64,
    /// Maximum Y bound in nanometers
    #[arg(long = "max-y-nm")]
    pub(crate) max_y_nm: i64,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolFootprintSilkscreenCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Circle center X coordinate in nanometers
    #[arg(long = "center-x-nm")]
    pub(crate) center_x_nm: i64,
    /// Circle center Y coordinate in nanometers
    #[arg(long = "center-y-nm")]
    pub(crate) center_y_nm: i64,
    /// Circle radius in nanometers
    #[arg(long)]
    pub(crate) radius_nm: i64,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolFootprintSilkscreenPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Uuid,
    /// Polygon vertices as x,y;x,y;... in nanometers
    #[arg(long)]
    pub(crate) vertices: String,
    /// Whether this primitive is a closed polygon rather than an open polyline
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub(crate) closed: bool,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}
