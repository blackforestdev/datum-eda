use crate::*;

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
    /// Legacy initial package-pad UUID; prefer creating a Footprint for land-pattern pads
    #[arg(long = "pad")]
    pub(crate) pad: Option<Uuid>,
    /// Legacy referenced padstack UUID; required only when --pad is supplied
    #[arg(long = "padstack")]
    pub(crate) padstack: Option<Uuid>,
    /// Legacy human-readable pad name
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
pub(crate) struct ProposalCreatePoolFootprintArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Human-readable footprint name
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
pub(crate) struct ProposalGenerateIpc7351bTwoTerminalChipArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Generated Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Generated Padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack: Uuid,
    /// Generated pad 1 UUID
    #[arg(long = "pad-a")]
    pub(crate) pad_a: Uuid,
    /// Generated pad 2 UUID
    #[arg(long = "pad-b")]
    pub(crate) pad_b: Uuid,
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalGenerateIpc7351bSoicArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Generated Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package: Uuid,
    /// Generated Padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack: Uuid,
    /// Generated pad UUIDs in package pin order; count must equal --pin-count
    #[arg(long = "pad", required = true)]
    pub(crate) pads: Vec<Uuid>,
    /// Optional human-readable footprint name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// IPC package code, e.g. SOIC-8_NARROW
    #[arg(long = "package-code")]
    pub(crate) package_code: String,
    /// SOIC pin count; must be even and at least 4
    #[arg(long = "pin-count")]
    pub(crate) pin_count: u32,
    /// Lead pitch in nanometers
    #[arg(long = "pitch-nm")]
    pub(crate) pitch_nm: i64,
    /// Component body length in nanometers
    #[arg(long = "body-length-nm")]
    pub(crate) body_length_nm: i64,
    /// Component body width in nanometers
    #[arg(long = "body-width-nm")]
    pub(crate) body_width_nm: i64,
    /// Overall lead span in nanometers
    #[arg(long = "lead-span-nm")]
    pub(crate) lead_span_nm: i64,
    /// Terminal/lead land length in nanometers
    #[arg(long = "terminal-length-nm")]
    pub(crate) terminal_length_nm: i64,
    /// Terminal/lead width in nanometers
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePoolPinPadMapArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// PinPadMap UUID
    #[arg(long = "map")]
    pub(crate) map_uuid: Uuid,
    /// Part UUID this PinPadMap binds
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Optional Footprint UUID; if omitted mappings target package pads
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Option<Uuid>,
    /// Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous
    #[arg(long = "entry", required = true)]
    pub(crate) entries: Vec<String>,
    /// Also set this map as the part default_pin_pad_map in the same proposal batch
    #[arg(long = "set-default")]
    pub(crate) set_default: bool,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolPinPadMapArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// PinPadMap UUID
    #[arg(long = "map")]
    pub(crate) map_uuid: Uuid,
    /// Merge listed mappings or replace the full mapping table
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous
    #[arg(long = "entry", required = true)]
    pub(crate) entries: Vec<String>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolFootprintPadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
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
pub(crate) struct ProposalSetPoolFootprintCourtyardRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalSetPoolFootprintCourtyardPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
    /// Polygon vertices as x,y;x,y;... in nanometers
    #[arg(long)]
    pub(crate) vertices: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalAddPoolFootprintSilkscreenLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalAddPoolFootprintSilkscreenRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalAddPoolFootprintSilkscreenCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
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
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalAddPoolFootprintSilkscreenPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path
    #[arg(long = "pool", default_value = "pool")]
    pub(crate) pool: String,
    /// Footprint UUID
    #[arg(long = "footprint")]
    pub(crate) footprint: Uuid,
    /// Polygon vertices as x,y;x,y;... in nanometers
    #[arg(long)]
    pub(crate) vertices: String,
    /// Whether this primitive is a closed polygon rather than an open polyline
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub(crate) closed: bool,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
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
