use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
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
    /// Text height in nm
    #[arg(long = "height-nm", default_value_t = 1_000_000)]
    pub(crate) height_nm: i64,
    /// Stroke width in nm
    #[arg(long = "stroke-width-nm", default_value_t = 100_000)]
    pub(crate) stroke_width_nm: i64,
    /// Layer identifier
    #[arg(long)]
    pub(crate) layer: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBoardTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Text UUID
    #[arg(long = "text")]
    pub(crate) text_uuid: Uuid,
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
    /// Replacement text height in nm
    #[arg(long = "height-nm")]
    pub(crate) height_nm: Option<i64>,
    /// Replacement stroke width in nm
    #[arg(long = "stroke-width-nm")]
    pub(crate) stroke_width_nm: Option<i64>,
    /// Replacement layer identifier
    #[arg(long)]
    pub(crate) layer: Option<i32>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardTextArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Text UUID
    #[arg(long = "text")]
    pub(crate) text_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardKeepoutArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Polygon vertices as x_nm:y_nm tuples
    #[arg(long = "vertex")]
    pub(crate) vertices: Vec<String>,
    /// Layer identifiers
    #[arg(long = "layer")]
    pub(crate) layers: Vec<i32>,
    /// Keepout kind label
    #[arg(long = "kind")]
    pub(crate) kind: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBoardKeepoutArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Keepout UUID
    #[arg(long = "keepout")]
    pub(crate) keepout_uuid: Uuid,
    /// Replacement polygon vertices as x_nm:y_nm tuples
    #[arg(long = "vertex")]
    pub(crate) vertices: Vec<String>,
    /// Replacement layer identifiers
    #[arg(long = "layer")]
    pub(crate) layers: Vec<i32>,
    /// Replacement keepout kind label
    #[arg(long = "kind")]
    pub(crate) kind: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardKeepoutArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Keepout UUID
    #[arg(long = "keepout")]
    pub(crate) keepout_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetBoardOutlineArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Polygon vertices as x_nm:y_nm tuples
    #[arg(long = "vertex")]
    pub(crate) vertices: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetBoardStackupArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Layer specification: <id>:<name>:<type>:<thickness_nm>
    #[arg(long = "layer")]
    pub(crate) layers: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAddDefaultTopStackupArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardNetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net name
    #[arg(long)]
    pub(crate) name: String,
    /// Assigned net-class UUID
    #[arg(long = "class")]
    pub(crate) class_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBoardNetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Replacement net name
    #[arg(long = "name")]
    pub(crate) name: Option<String>,
    /// Replacement net-class UUID
    #[arg(long = "class")]
    pub(crate) class_uuid: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardNetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDrawBoardTrackArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Start X coordinate in nm
    #[arg(long = "from-x-nm")]
    pub(crate) from_x_nm: i64,
    /// Start Y coordinate in nm
    #[arg(long = "from-y-nm")]
    pub(crate) from_y_nm: i64,
    /// End X coordinate in nm
    #[arg(long = "to-x-nm")]
    pub(crate) to_x_nm: i64,
    /// End Y coordinate in nm
    #[arg(long = "to-y-nm")]
    pub(crate) to_y_nm: i64,
    /// Track width in nm
    #[arg(long = "width-nm")]
    pub(crate) width_nm: i64,
    /// Layer identifier
    #[arg(long)]
    pub(crate) layer: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardTrackArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Track UUID
    #[arg(long = "track")]
    pub(crate) track_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardViaArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// X coordinate in nm
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Via drill in nm
    #[arg(long = "drill-nm")]
    pub(crate) drill_nm: i64,
    /// Via diameter in nm
    #[arg(long = "diameter-nm")]
    pub(crate) diameter_nm: i64,
    /// Starting layer identifier
    #[arg(long = "from-layer")]
    pub(crate) from_layer: i32,
    /// Ending layer identifier
    #[arg(long = "to-layer")]
    pub(crate) to_layer: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardViaArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Via UUID
    #[arg(long = "via")]
    pub(crate) via_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardZoneArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
    /// Polygon vertices as x_nm:y_nm
    #[arg(long = "vertex")]
    pub(crate) vertices: Vec<String>,
    /// Layer identifier
    #[arg(long)]
    pub(crate) layer: i32,
    /// Zone priority
    #[arg(long, default_value_t = 0)]
    pub(crate) priority: u32,
    /// Thermal relief enabled
    #[arg(
        long = "thermal-relief",
        default_value_t = true,
        action = clap::ArgAction::Set
    )]
    pub(crate) thermal_relief: bool,
    /// Thermal gap in nm
    #[arg(long = "thermal-gap-nm")]
    pub(crate) thermal_gap_nm: i64,
    /// Thermal spoke width in nm
    #[arg(long = "thermal-spoke-width-nm")]
    pub(crate) thermal_spoke_width_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardZoneArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Zone UUID
    #[arg(long = "zone")]
    pub(crate) zone_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetBoardPadNetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
    /// Net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearBoardPadNetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBoardPadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
    /// Replacement X coordinate in nm
    #[arg(long = "x-nm")]
    pub(crate) x_nm: Option<i64>,
    /// Replacement Y coordinate in nm
    #[arg(long = "y-nm")]
    pub(crate) y_nm: Option<i64>,
    /// Replacement layer identifier
    #[arg(long)]
    pub(crate) layer: Option<i32>,
    /// Replacement pad shape (`circle` or `rect`)
    #[arg(long)]
    pub(crate) shape: Option<String>,
    /// Replacement circular copper diameter in nm
    #[arg(long = "diameter-nm")]
    pub(crate) diameter_nm: Option<i64>,
    /// Replacement rectangular copper width in nm
    #[arg(long = "width-nm")]
    pub(crate) width_nm: Option<i64>,
    /// Replacement rectangular copper height in nm
    #[arg(long = "height-nm")]
    pub(crate) height_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardPadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Pad name
    #[arg(long)]
    pub(crate) name: String,
    /// X coordinate in nm
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Layer identifier
    #[arg(long)]
    pub(crate) layer: i32,
    /// Pad shape (`circle` or `rect`)
    #[arg(long)]
    pub(crate) shape: Option<String>,
    /// Circular copper diameter in nm
    #[arg(long = "diameter-nm")]
    pub(crate) diameter_nm: Option<i64>,
    /// Rectangular copper width in nm
    #[arg(long = "width-nm")]
    pub(crate) width_nm: Option<i64>,
    /// Rectangular copper height in nm
    #[arg(long = "height-nm")]
    pub(crate) height_nm: Option<i64>,
    /// Optional net UUID
    #[arg(long = "net")]
    pub(crate) net_uuid: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardPadArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Pad UUID
    #[arg(long = "pad")]
    pub(crate) pad_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardComponentArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Reference designator
    #[arg(long)]
    pub(crate) reference: String,
    /// Value text
    #[arg(long)]
    pub(crate) value: String,
    /// X coordinate in nm
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
    /// Layer identifier
    #[arg(long)]
    pub(crate) layer: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectMoveBoardComponentArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// X coordinate in nm
    #[arg(long = "x-nm")]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long = "y-nm")]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRotateBoardComponentArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Rotation in degrees
    #[arg(long = "rotation-deg")]
    pub(crate) rotation_deg: i32,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardComponentArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetBoardComponentLockedArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectClearBoardComponentLockedArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBoardNetClassArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net-class name
    #[arg(long)]
    pub(crate) name: String,
    /// Clearance in nm
    #[arg(long = "clearance-nm")]
    pub(crate) clearance_nm: i64,
    /// Track width in nm
    #[arg(long = "track-width-nm")]
    pub(crate) track_width_nm: i64,
    /// Via drill in nm
    #[arg(long = "via-drill-nm")]
    pub(crate) via_drill_nm: i64,
    /// Via diameter in nm
    #[arg(long = "via-diameter-nm")]
    pub(crate) via_diameter_nm: i64,
    /// Differential-pair width in nm
    #[arg(long = "diffpair-width-nm", default_value_t = 0)]
    pub(crate) diffpair_width_nm: i64,
    /// Differential-pair gap in nm
    #[arg(long = "diffpair-gap-nm", default_value_t = 0)]
    pub(crate) diffpair_gap_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBoardNetClassArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net-class UUID
    #[arg(long = "net-class")]
    pub(crate) net_class_uuid: Uuid,
    /// Replacement net-class name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Replacement clearance in nm
    #[arg(long = "clearance-nm")]
    pub(crate) clearance_nm: Option<i64>,
    /// Replacement track width in nm
    #[arg(long = "track-width-nm")]
    pub(crate) track_width_nm: Option<i64>,
    /// Replacement via drill in nm
    #[arg(long = "via-drill-nm")]
    pub(crate) via_drill_nm: Option<i64>,
    /// Replacement via diameter in nm
    #[arg(long = "via-diameter-nm")]
    pub(crate) via_diameter_nm: Option<i64>,
    /// Replacement differential-pair width in nm
    #[arg(long = "diffpair-width-nm")]
    pub(crate) diffpair_width_nm: Option<i64>,
    /// Replacement differential-pair gap in nm
    #[arg(long = "diffpair-gap-nm")]
    pub(crate) diffpair_gap_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardNetClassArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Net-class UUID
    #[arg(long = "net-class")]
    pub(crate) net_class_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBoardDimensionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Dimension UUID
    #[arg(long = "dimension")]
    pub(crate) dimension_uuid: Uuid,
}
