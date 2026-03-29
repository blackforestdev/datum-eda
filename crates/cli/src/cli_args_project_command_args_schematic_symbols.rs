use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Reference designator
    #[arg(long)]
    pub(crate) reference: String,
    /// Display value
    #[arg(long)]
    pub(crate) value: String,
    /// Optional library identifier for future resolution
    #[arg(long = "lib-id")]
    pub(crate) lib_id: Option<String>,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Rotation in degrees
    #[arg(long = "rotation-deg", default_value_t = 0)]
    pub(crate) rotation_deg: i32,
    /// Mirror the symbol about its local Y axis
    #[arg(long, default_value_t = false)]
    pub(crate) mirrored: bool,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectMoveSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// New X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// New Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRotateSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// New rotation in degrees
    #[arg(long = "rotation-deg")]
    pub(crate) rotation_deg: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectMirrorSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolReferenceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// New reference value
    #[arg(long)]
    pub(crate) reference: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolValueArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// New symbol value
    #[arg(long)]
    pub(crate) value: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolDisplayModeArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Display mode
    #[arg(long = "mode", value_enum)]
    pub(crate) display_mode: NativeSymbolDisplayModeArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolHiddenPowerBehaviorArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Hidden-power behavior
    #[arg(long = "behavior", value_enum)]
    pub(crate) hidden_power_behavior: NativeHiddenPowerBehaviorArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolUnitArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Replacement unit selection token
    #[arg(long = "unit")]
    pub(crate) unit_selection: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearSymbolUnitArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolGateArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Gate UUID
    #[arg(long = "gate")]
    pub(crate) gate_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearSymbolGateArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolEntityArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Entity UUID
    #[arg(long = "entity")]
    pub(crate) entity_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearSymbolEntityArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolPartArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearSymbolPartArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetSymbolLibIdArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
    /// New library identifier
    #[arg(long = "lib-id")]
    pub(crate) lib_id: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearSymbolLibIdArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long = "symbol")]
    pub(crate) symbol_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPinOverrideArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Pin UUID
    #[arg(long = "pin")]
    pub(crate) pin_uuid: Uuid,
    /// Replacement visible state
    #[arg(long, action = clap::ArgAction::Set)]
    pub(crate) visible: bool,
    /// Replacement pin X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: Option<i64>,
    /// Replacement pin Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearPinOverrideArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Pin UUID
    #[arg(long = "pin")]
    pub(crate) pin_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddSymbolFieldArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Field key
    #[arg(long)]
    pub(crate) key: String,
    /// Field value
    #[arg(long)]
    pub(crate) value: String,
    /// Mark the field hidden instead of visible
    #[arg(long, default_value_t = false)]
    pub(crate) hidden: bool,
    /// Optional field X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: Option<i64>,
    /// Optional field Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditSymbolFieldArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Field UUID
    #[arg(long = "field")]
    pub(crate) field: Uuid,
    /// Replacement field key
    #[arg(long)]
    pub(crate) key: Option<String>,
    /// Replacement field value
    #[arg(long)]
    pub(crate) value: Option<String>,
    /// Replacement visible state
    #[arg(long)]
    pub(crate) visible: Option<bool>,
    /// Replacement field X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: Option<i64>,
    /// Replacement field Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteSymbolFieldArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Field UUID
    #[arg(long)]
    pub(crate) field: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Text content
    #[arg(long)]
    pub(crate) text: String,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Rotation in degrees
    #[arg(long = "rotation-deg", default_value_t = 0)]
    pub(crate) rotation_deg: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Text UUID
    #[arg(long)]
    pub(crate) text: Uuid,
    /// Replacement text content
    #[arg(long)]
    pub(crate) value: Option<String>,
    /// Replacement X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: Option<i64>,
    /// Replacement Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: Option<i64>,
    /// Replacement rotation in degrees
    #[arg(long = "rotation-deg")]
    pub(crate) rotation_deg: Option<i32>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Text UUID
    #[arg(long)]
    pub(crate) text: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceDrawingLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Start X coordinate in nm
    #[arg(long)]
    pub(crate) from_x_nm: i64,
    /// Start Y coordinate in nm
    #[arg(long)]
    pub(crate) from_y_nm: i64,
    /// End X coordinate in nm
    #[arg(long)]
    pub(crate) to_x_nm: i64,
    /// End Y coordinate in nm
    #[arg(long)]
    pub(crate) to_y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceDrawingRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Minimum X coordinate in nm
    #[arg(long)]
    pub(crate) min_x_nm: i64,
    /// Minimum Y coordinate in nm
    #[arg(long)]
    pub(crate) min_y_nm: i64,
    /// Maximum X coordinate in nm
    #[arg(long)]
    pub(crate) max_x_nm: i64,
    /// Maximum Y coordinate in nm
    #[arg(long)]
    pub(crate) max_y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceDrawingCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Center X coordinate in nm
    #[arg(long)]
    pub(crate) center_x_nm: i64,
    /// Center Y coordinate in nm
    #[arg(long)]
    pub(crate) center_y_nm: i64,
    /// Radius in nm
    #[arg(long)]
    pub(crate) radius_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceDrawingArcArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Center X coordinate in nm
    #[arg(long)]
    pub(crate) center_x_nm: i64,
    /// Center Y coordinate in nm
    #[arg(long)]
    pub(crate) center_y_nm: i64,
    /// Radius in nm
    #[arg(long)]
    pub(crate) radius_nm: i64,
    /// Start angle in millidegrees
    #[arg(long)]
    pub(crate) start_angle_mdeg: i32,
    /// End angle in millidegrees
    #[arg(long)]
    pub(crate) end_angle_mdeg: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditDrawingLineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drawing UUID
    #[arg(long = "drawing")]
    pub(crate) drawing: Uuid,
    /// Replacement start X coordinate in nm
    #[arg(long)]
    pub(crate) from_x_nm: Option<i64>,
    /// Replacement start Y coordinate in nm
    #[arg(long)]
    pub(crate) from_y_nm: Option<i64>,
    /// Replacement end X coordinate in nm
    #[arg(long)]
    pub(crate) to_x_nm: Option<i64>,
    /// Replacement end Y coordinate in nm
    #[arg(long)]
    pub(crate) to_y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditDrawingRectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drawing UUID
    #[arg(long = "drawing")]
    pub(crate) drawing: Uuid,
    /// Replacement minimum X coordinate in nm
    #[arg(long)]
    pub(crate) min_x_nm: Option<i64>,
    /// Replacement minimum Y coordinate in nm
    #[arg(long)]
    pub(crate) min_y_nm: Option<i64>,
    /// Replacement maximum X coordinate in nm
    #[arg(long)]
    pub(crate) max_x_nm: Option<i64>,
    /// Replacement maximum Y coordinate in nm
    #[arg(long)]
    pub(crate) max_y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditDrawingCircleArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drawing UUID
    #[arg(long = "drawing")]
    pub(crate) drawing: Uuid,
    /// Replacement center X coordinate in nm
    #[arg(long)]
    pub(crate) center_x_nm: Option<i64>,
    /// Replacement center Y coordinate in nm
    #[arg(long)]
    pub(crate) center_y_nm: Option<i64>,
    /// Replacement radius in nm
    #[arg(long)]
    pub(crate) radius_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditDrawingArcArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drawing UUID
    #[arg(long = "drawing")]
    pub(crate) drawing: Uuid,
    /// Replacement center X coordinate in nm
    #[arg(long)]
    pub(crate) center_x_nm: Option<i64>,
    /// Replacement center Y coordinate in nm
    #[arg(long)]
    pub(crate) center_y_nm: Option<i64>,
    /// Replacement radius in nm
    #[arg(long)]
    pub(crate) radius_nm: Option<i64>,
    /// Replacement start angle in millidegrees
    #[arg(long)]
    pub(crate) start_angle_mdeg: Option<i32>,
    /// Replacement end angle in millidegrees
    #[arg(long)]
    pub(crate) end_angle_mdeg: Option<i32>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteDrawingArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Drawing UUID
    #[arg(long = "drawing")]
    pub(crate) drawing: Uuid,
}
