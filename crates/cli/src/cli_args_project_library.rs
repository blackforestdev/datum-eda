use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
    /// JSON file containing the object payload
    #[arg(long = "from-json")]
    pub(crate) from_json: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeletePoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
    /// JSON file containing the replacement object payload
    #[arg(long = "from-json")]
    pub(crate) from_json: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolUnitArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Unit UUID
    #[arg(long = "unit")]
    pub(crate) unit_uuid: Uuid,
    /// Human-readable unit name
    #[arg(long)]
    pub(crate) name: String,
    /// Unit manufacturer/source namespace
    #[arg(long, default_value = "")]
    pub(crate) manufacturer: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolUnitPinArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Unit UUID
    #[arg(long = "unit")]
    pub(crate) unit_uuid: Uuid,
    /// Pin UUID
    #[arg(long = "pin")]
    pub(crate) pin_uuid: Uuid,
    /// Human-readable pin name/number
    #[arg(long)]
    pub(crate) name: String,
    /// Pin electrical direction enum
    #[arg(long, default_value = "Passive")]
    pub(crate) direction: String,
    /// Pin-swap group; zero means not swappable
    #[arg(long = "swap-group", default_value_t = 0)]
    pub(crate) swap_group: u32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Referenced unit UUID
    #[arg(long = "unit")]
    pub(crate) unit_uuid: Uuid,
    /// Human-readable symbol name
    #[arg(long)]
    pub(crate) name: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Line start X coordinate in nanometers
    #[arg(long = "from-x-nm")]
    pub(crate) from_x_nm: i64,
    /// Line start Y coordinate in nanometers
    #[arg(long = "from-y-nm")]
    pub(crate) from_y_nm: i64,
    /// Line end X coordinate in nanometers
    #[arg(long = "to-x-nm")]
    pub(crate) to_x_nm: i64,
    /// Line end Y coordinate in nanometers
    #[arg(long = "to-y-nm")]
    pub(crate) to_y_nm: i64,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Vertices as x,y pairs separated by semicolons: x,y;x,y;x,y
    #[arg(long)]
    pub(crate) vertices: String,
    /// Whether the primitive is closed polygon geometry; false authors a polyline
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub(crate) closed: bool,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Rectangle minimum X coordinate in nanometers
    #[arg(long = "min-x-nm")]
    pub(crate) min_x_nm: i64,
    /// Rectangle minimum Y coordinate in nanometers
    #[arg(long = "min-y-nm")]
    pub(crate) min_y_nm: i64,
    /// Rectangle maximum X coordinate in nanometers
    #[arg(long = "max-x-nm")]
    pub(crate) max_x_nm: i64,
    /// Rectangle maximum Y coordinate in nanometers
    #[arg(long = "max-y-nm")]
    pub(crate) max_y_nm: i64,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Circle center X coordinate in nanometers
    #[arg(long = "center-x-nm")]
    pub(crate) center_x_nm: i64,
    /// Circle center Y coordinate in nanometers
    #[arg(long = "center-y-nm")]
    pub(crate) center_y_nm: i64,
    /// Circle radius in nanometers
    #[arg(long = "radius-nm")]
    pub(crate) radius_nm: i64,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolArcArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Arc center X coordinate in nanometers
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Arc center Y coordinate in nanometers
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Arc radius in nanometers
    #[arg(long = "radius-nm")]
    pub(crate) radius_nm: i64,
    /// Arc start angle in tenths of degrees
    #[arg(long)]
    pub(crate) start_angle: i32,
    /// Arc end angle in tenths of degrees
    #[arg(long)]
    pub(crate) end_angle: i32,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolSymbolTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Symbol drawing text
    #[arg(long)]
    pub(crate) text: String,
    /// Text X coordinate in nanometers
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Text Y coordinate in nanometers
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Text rotation in tenths of degrees
    #[arg(long, default_value_t = 0)]
    pub(crate) rotation: i32,
}

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
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolEntityArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Entity UUID
    #[arg(long = "entity")]
    pub(crate) entity_uuid: Uuid,
    /// Initial gate UUID
    #[arg(long = "gate")]
    pub(crate) gate_uuid: Uuid,
    /// Referenced unit UUID
    #[arg(long = "unit")]
    pub(crate) unit_uuid: Uuid,
    /// Referenced symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// Human-readable entity name
    #[arg(long)]
    pub(crate) name: String,
    /// Reference prefix for placed components
    #[arg(long)]
    pub(crate) prefix: String,
    /// Entity manufacturer/source namespace
    #[arg(long, default_value = "")]
    pub(crate) manufacturer: String,
    /// Human-readable gate name
    #[arg(long = "gate-name", default_value = "A")]
    pub(crate) gate_name: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolPadstackArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Padstack UUID
    #[arg(long = "padstack")]
    pub(crate) padstack_uuid: Uuid,
    /// Human-readable padstack name
    #[arg(long)]
    pub(crate) name: String,
    /// Optional aperture kind: circle or rect
    #[arg(long)]
    pub(crate) aperture: Option<String>,
    /// Circle aperture diameter in nanometers
    #[arg(long = "diameter-nm")]
    pub(crate) diameter_nm: Option<i64>,
    /// Rect aperture width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: Option<i64>,
    /// Rect aperture height in nanometers
    #[arg(long = "height-nm")]
    pub(crate) height_nm: Option<i64>,
    /// Optional plated drill diameter in nanometers
    #[arg(long = "drill-nm")]
    pub(crate) drill_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolPackageArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Human-readable package name
    #[arg(long)]
    pub(crate) name: String,
    /// Initial pad UUID
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
pub(crate) struct ProjectSetPoolPackagePadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
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
pub(crate) struct ProjectSetPoolPackageCourtyardRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Minimum X coordinate in nanometers
    #[arg(long)]
    pub(crate) min_x_nm: i64,
    /// Minimum Y coordinate in nanometers
    #[arg(long)]
    pub(crate) min_y_nm: i64,
    /// Maximum X coordinate in nanometers
    #[arg(long)]
    pub(crate) max_x_nm: i64,
    /// Maximum Y coordinate in nanometers
    #[arg(long)]
    pub(crate) max_y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPackageCourtyardPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Vertices as x,y pairs separated by semicolons: x,y;x,y;x,y
    #[arg(long)]
    pub(crate) vertices: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageSilkscreenLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
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
pub(crate) struct ProjectAddPoolPackageSilkscreenRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Minimum X coordinate in nanometers
    #[arg(long)]
    pub(crate) min_x_nm: i64,
    /// Minimum Y coordinate in nanometers
    #[arg(long)]
    pub(crate) min_y_nm: i64,
    /// Maximum X coordinate in nanometers
    #[arg(long)]
    pub(crate) max_x_nm: i64,
    /// Maximum Y coordinate in nanometers
    #[arg(long)]
    pub(crate) max_y_nm: i64,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageSilkscreenCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Circle center X coordinate in nanometers
    #[arg(long)]
    pub(crate) center_x_nm: i64,
    /// Circle center Y coordinate in nanometers
    #[arg(long)]
    pub(crate) center_y_nm: i64,
    /// Circle radius in nanometers
    #[arg(long)]
    pub(crate) radius_nm: i64,
    /// Stroke width in nanometers
    #[arg(long)]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageSilkscreenArcArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Arc center X coordinate in nanometers
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Arc center Y coordinate in nanometers
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Arc radius in nanometers
    #[arg(long = "radius-nm")]
    pub(crate) radius_nm: i64,
    /// Arc start angle in tenths of degrees
    #[arg(long)]
    pub(crate) start_angle: i32,
    /// Arc end angle in tenths of degrees
    #[arg(long)]
    pub(crate) end_angle: i32,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageSilkscreenPolygonArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Vertices as x,y pairs separated by semicolons: x,y;x,y;x,y
    #[arg(long)]
    pub(crate) vertices: String,
    /// Whether the primitive is closed polygon geometry; false authors a polyline
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub(crate) closed: bool,
    /// Stroke width in nanometers
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageSilkscreenTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Text content
    #[arg(long)]
    pub(crate) text: String,
    /// Text anchor X coordinate in nanometers
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Text anchor Y coordinate in nanometers
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Rotation in tenths of a degree
    #[arg(long, default_value = "0")]
    pub(crate) rotation: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddPoolPackageModel3dArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Project-relative model asset path
    #[arg(long = "model-path")]
    pub(crate) model_path: String,
    /// Model format enum: Step, Wrl, Iges, Obj, or Gltf
    #[arg(long = "format")]
    pub(crate) format: Option<String>,
    /// Optional JSON transform payload
    #[arg(long = "transform-json")]
    pub(crate) transform_json: Option<String>,
    /// Translation X in nanometers
    #[arg(long = "tx-nm")]
    pub(crate) tx_nm: Option<i64>,
    /// Translation Y in nanometers
    #[arg(long = "ty-nm")]
    pub(crate) ty_nm: Option<i64>,
    /// Translation Z in nanometers
    #[arg(long = "tz-nm")]
    pub(crate) tz_nm: Option<i64>,
    /// Roll in tenths of a degree
    #[arg(long = "roll-tenths-deg")]
    pub(crate) roll_tenths_deg: Option<i32>,
    /// Pitch in tenths of a degree
    #[arg(long = "pitch-tenths-deg")]
    pub(crate) pitch_tenths_deg: Option<i32>,
    /// Yaw in tenths of a degree
    #[arg(long = "yaw-tenths-deg")]
    pub(crate) yaw_tenths_deg: Option<i32>,
    /// Uniform model scale as a positive JSON number
    #[arg(long = "scale")]
    pub(crate) scale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPackageBodyHeightsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Clear both body-height fields before applying supplied values
    #[arg(long)]
    pub(crate) clear: bool,
    /// Tallest authored body height in nanometers
    #[arg(long = "body-height-nm")]
    pub(crate) body_height_nm: Option<i64>,
    /// Mounted body height in nanometers
    #[arg(long = "body-height-mounted-nm")]
    pub(crate) body_height_mounted_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolPartArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Referenced entity UUID
    #[arg(long = "entity")]
    pub(crate) entity_uuid: Uuid,
    /// Referenced package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Manufacturer part number
    #[arg(long)]
    pub(crate) mpn: String,
    /// Part manufacturer/source namespace
    #[arg(long, default_value = "")]
    pub(crate) manufacturer: String,
    /// Display/electrical value
    #[arg(long, default_value = "")]
    pub(crate) value: String,
    /// Human-readable description
    #[arg(long, default_value = "")]
    pub(crate) description: String,
    /// Datasheet URI or path
    #[arg(long, default_value = "")]
    pub(crate) datasheet: String,
    /// Lifecycle enum: Active, Nrnd, Eol, Obsolete, or Unknown
    #[arg(long, default_value = "Active")]
    pub(crate) lifecycle: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartMetadataArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Manufacturer part number
    #[arg(long)]
    pub(crate) mpn: Option<String>,
    /// Part manufacturer/source namespace
    #[arg(long)]
    pub(crate) manufacturer: Option<String>,
    /// JEDEC JEP106 manufacturer ID
    #[arg(long = "manufacturer-jep106")]
    pub(crate) manufacturer_jep106: Option<u16>,
    /// Display/electrical value
    #[arg(long)]
    pub(crate) value: Option<String>,
    /// Human-readable description
    #[arg(long)]
    pub(crate) description: Option<String>,
    /// Datasheet URI or path
    #[arg(long)]
    pub(crate) datasheet: Option<String>,
    /// Lifecycle enum: Active, Nrnd, Eol, Obsolete, or Unknown
    #[arg(long)]
    pub(crate) lifecycle: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartParametricArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed parameters or replace the full parametric map
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Parametric entry as key=value; repeat for multiple parameters
    #[arg(long = "param", required = true)]
    pub(crate) params: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartOrderableMpnsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed orderable MPNs or replace the full list
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Orderable manufacturer part number; repeat for multiple values
    #[arg(long = "mpn", required = true)]
    pub(crate) mpns: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartPackagingOptionsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed packaging options or replace the full list
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Packaging option JSON object; repeat for multiple values
    #[arg(long = "option", required = true)]
    pub(crate) options: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartBehaviouralModelsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed behavioural model attachments or replace the full list
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// ModelAttachment JSON object; repeat for multiple values
    #[arg(long = "model", required = true)]
    pub(crate) models: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAttachPoolPartModelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Vendor model file to copy into pool/models
    #[arg(long = "source")]
    pub(crate) source: PathBuf,
    /// ModelRole enum: Spice, Ibis, IbisIss, IbisAmi, Touchstone, VerilogA, VerilogAms, VhdlAms, or CompactThermal
    #[arg(long)]
    pub(crate) role: String,
    /// Optional SPICE dialect enum
    #[arg(long)]
    pub(crate) dialect: Option<String>,
    /// Model or subcircuit name; repeat for multiple names
    #[arg(long = "model-name")]
    pub(crate) model_names: Vec<String>,
    /// Mark the attached model as encrypted
    #[arg(long)]
    pub(crate) encrypted: bool,
    /// Optional EncryptionScheme JSON value
    #[arg(long = "encryption-scheme")]
    pub(crate) encryption_scheme: Option<String>,
    /// Optional canonical vendor/source name
    #[arg(long)]
    pub(crate) vendor: Option<String>,
    /// Optional fetch timestamp string for provenance
    #[arg(long = "fetched-at")]
    pub(crate) fetched_at: Option<String>,
    /// Optional ModelFormatMetadata JSON object; defaults to {"kind":"none"}
    #[arg(long = "format-metadata-json")]
    pub(crate) format_metadata_json: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDetachPoolPartModelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Attachment UUID to remove from the part behavioural model list
    #[arg(long = "attachment")]
    pub(crate) attachment_uuid: Option<Uuid>,
    /// Model UUID whose attachments should be removed from the part
    #[arg(long = "model")]
    pub(crate) model_uuid: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectGcPoolModelsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Optional pool/models role directory filter, e.g. spice
    #[arg(long)]
    pub(crate) role: Option<String>,
    /// Optional model content hash filter
    #[arg(long)]
    pub(crate) sha256: Option<String>,
    /// Delete orphaned model blobs instead of returning a dry-run plan
    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartThermalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Clear the existing thermal spec before applying supplied fields
    #[arg(long)]
    pub(crate) clear: bool,
    /// Junction-to-ambient thermal resistance in C/W
    #[arg(long = "theta-ja-c-per-w", allow_hyphen_values = true)]
    pub(crate) theta_ja_c_per_w: Option<String>,
    /// Junction-to-case-top thermal resistance in C/W
    #[arg(long = "theta-jc-top-c-per-w", allow_hyphen_values = true)]
    pub(crate) theta_jc_top_c_per_w: Option<String>,
    /// Junction-to-case-bottom thermal resistance in C/W
    #[arg(long = "theta-jc-bot-c-per-w", allow_hyphen_values = true)]
    pub(crate) theta_jc_bot_c_per_w: Option<String>,
    /// Junction-to-board thermal resistance in C/W
    #[arg(long = "theta-jb-c-per-w", allow_hyphen_values = true)]
    pub(crate) theta_jb_c_per_w: Option<String>,
    /// Maximum junction temperature in C
    #[arg(long = "max-junction-c", allow_hyphen_values = true)]
    pub(crate) max_junction_c: Option<String>,
    /// Reference condition or standard for the thermal data
    #[arg(long = "thermal-reference")]
    pub(crate) thermal_reference: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartSupplyChainArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Clear supply-chain offer cache and last checked timestamp
    #[arg(long)]
    pub(crate) clear: bool,
    /// Last supply-chain check timestamp string
    #[arg(long = "checked-at")]
    pub(crate) checked_at: Option<String>,
    /// SupplyOffer JSON object; repeat for multiple offers
    #[arg(long = "offer")]
    pub(crate) offers: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartTagsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed tags or replace the full list
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Part tag; repeat for multiple values
    #[arg(long = "tag", required = true)]
    pub(crate) tags: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartPadMapEntryArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Package pad UUID to map
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
    /// Entity gate UUID to map
    #[arg(long = "gate")]
    pub(crate) gate_uuid: Uuid,
    /// Unit pin UUID to map
    #[arg(long = "pin")]
    pub(crate) pin_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartPadMapArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Merge listed entries or replace the full pad map
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Pad-map entry as pad_uuid:gate_uuid:pin_uuid; repeat for multiple entries
    #[arg(long = "entry", required = true)]
    pub(crate) entries: Vec<String>,
}
