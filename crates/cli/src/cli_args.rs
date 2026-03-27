use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eda_engine::api::ScopedComponentReplacementPlan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "eda", about = "PCB design analysis and automation")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Output format
    #[arg(long, default_value = "text")]
    pub(crate) format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum FailOn {
    Info,
    Warning,
    Error,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Commands {
    /// Import a KiCad or Eagle design
    Import {
        /// Path to design file (.kicad_pcb, .brd, .lbr)
        path: PathBuf,
    },
    /// Query design data
    Query {
        /// Path to design file
        path: PathBuf,
        /// What to query
        #[command(subcommand)]
        what: QueryCommands,
    },
    /// Run design rule checks
    Drc {
        /// Path to design file
        path: String,
    },
    /// Run electrical rule checks on a schematic
    Erc {
        /// Path to schematic file (.kicad_sch in current M1 slice)
        path: PathBuf,
    },
    /// Run the current unified check surface for an imported design
    Check {
        /// Path to design file
        path: PathBuf,

        /// Exit nonzero if the check report status meets or exceeds this level
        #[arg(long, value_enum)]
        fail_on: Option<FailOn>,
    },
    /// Search the component pool
    Pool {
        #[command(subcommand)]
        action: PoolCommands,
    },
    /// Create and manage native projects
    Project {
        #[command(subcommand)]
        action: Box<ProjectCommands>,
    },
    /// Persist and reuse scoped replacement workflow artifacts
    Plan {
        #[command(subcommand)]
        action: PlanCommands,
    },
    /// Apply the current minimal M3 board modification surface
    Modify {
        /// Path to board design file
        path: PathBuf,

        /// Delete one track by UUID
        #[arg(long = "delete-track")]
        delete_track: Vec<Uuid>,

        /// Delete one via by UUID
        #[arg(long = "delete-via")]
        delete_via: Vec<Uuid>,

        /// Delete one component by UUID
        #[arg(long = "delete-component")]
        delete_component: Vec<Uuid>,

        /// Load Eagle libraries into the in-memory pool before applying modify ops
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,

        /// Move one component: <uuid>:<x_mm>:<y_mm>[:<rotation_deg>]
        #[arg(long = "move-component")]
        move_component: Vec<String>,

        /// Rotate one component: <uuid>:<rotation_deg>
        #[arg(long = "rotate-component")]
        rotate_component: Vec<String>,

        /// Set one component value: <uuid>:<value>
        #[arg(long = "set-value")]
        set_value: Vec<String>,

        /// Assign one component part: <uuid>:<part_uuid>
        #[arg(long = "assign-part")]
        assign_part: Vec<String>,

        /// Set one component package: <uuid>:<package_uuid>
        #[arg(long = "set-package")]
        set_package: Vec<String>,

        /// Set one component package with an explicit compatible part: <uuid>:<package_uuid>:<part_uuid>
        #[arg(long = "set-package-with-part")]
        set_package_with_part: Vec<String>,

        /// Replace one component with an explicit compatible part+package: <uuid>:<package_uuid>:<part_uuid>
        #[arg(long = "replace-component")]
        replace_component: Vec<String>,

        /// Apply replacement-plan selection: <uuid>:package:<package_uuid> | <uuid>:part:<part_uuid> | <uuid>:package:<package_uuid>:part:<part_uuid>
        #[arg(long = "apply-replacement-plan")]
        apply_replacement_plan: Vec<String>,

        /// Apply replacement policy: <uuid>:package | <uuid>:part
        #[arg(long = "apply-replacement-policy")]
        apply_replacement_policy: Vec<String>,

        /// Apply scoped replacement policy: package|part[:ref_prefix=<text>][:value=<text>][:package_uuid=<uuid>][:part_uuid=<uuid>]
        #[arg(long = "apply-scoped-replacement-policy")]
        apply_scoped_replacement_policy: Vec<String>,

        /// Apply a previously exported scoped replacement preview JSON file without re-resolving policy
        #[arg(long = "apply-scoped-replacement-plan-file")]
        apply_scoped_replacement_plan_file: Vec<PathBuf>,

        /// Apply a versioned scoped replacement manifest and automatically load its recorded libraries
        #[arg(long = "apply-scoped-replacement-manifest")]
        apply_scoped_replacement_manifest: Vec<PathBuf>,

        /// Set one net class: <net_uuid>:<class_name>:<clearance_nm>:<track_width_nm>:<via_drill_nm>:<via_diameter_nm>[:<diffpair_width_nm>:<diffpair_gap_nm>]
        #[arg(long = "set-net-class")]
        set_net_class: Vec<String>,

        /// Set one component reference: <uuid>:<reference>
        #[arg(long = "set-reference")]
        set_reference: Vec<String>,

        /// Undo the most recent transaction count times
        #[arg(long, default_value_t = 0)]
        undo: usize,

        /// Redo the most recent undone transaction count times
        #[arg(long, default_value_t = 0)]
        redo: usize,

        /// Save modifications to a new path
        #[arg(long)]
        save: Option<PathBuf>,

        /// Set the default all-scope copper clearance rule minimum in nm
        #[arg(long)]
        set_clearance_min_nm: Option<i64>,

        /// Save back to the original imported file path
        #[arg(long, default_value_t = false)]
        save_original: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum QueryCommands {
    /// Board summary (dimensions, counts)
    Summary,
    /// List all nets
    Nets,
    /// List all components
    Components,
    /// List schematic labels
    Labels,
    /// List schematic ports
    Ports,
    /// Show schematic hierarchy
    Hierarchy,
    /// Show schematic connectivity diagnostics
    Diagnostics,
    /// Show unrouted connections
    Unrouted,
    /// Show design rules
    DesignRules,
    /// Show package-change compatibility candidates for a component UUID
    PackageChangeCandidates {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying candidates
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show part-change compatibility candidates for a component UUID
    PartChangeCandidates {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying candidates
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show a unified replacement-planning report for a component UUID
    ComponentReplacementPlan {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying the plan
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show the resolved replacements a scoped policy would apply
    ScopedReplacementPlan {
        /// Replacement policy to resolve
        #[arg(value_enum)]
        policy: ReplacementPolicyArg,
        /// Restrict matches by current reference prefix
        #[arg(long = "ref-prefix")]
        ref_prefix: Option<String>,
        /// Restrict matches by current value
        #[arg(long = "value")]
        value: Option<String>,
        /// Restrict matches by current package UUID
        #[arg(long = "package-uuid")]
        package_uuid: Option<Uuid>,
        /// Restrict matches by current part UUID
        #[arg(long = "part-uuid")]
        part_uuid: Option<Uuid>,
        /// Exclude one component UUID from the previewed plan
        #[arg(long = "exclude-component")]
        exclude_component: Vec<Uuid>,
        /// Override one component target: <component_uuid>:<target_package_uuid>:<target_part_uuid>
        #[arg(long = "override-component")]
        override_component: Vec<String>,
        /// Load Eagle libraries into the in-memory pool before querying the plan
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum ReplacementPolicyArg {
    Package,
    Part,
}

#[derive(Subcommand)]
pub(crate) enum PoolCommands {
    /// Search for parts
    Search {
        /// Search query
        query: String,

        /// Eagle library files to load into the in-memory pool for this search
        #[arg(long = "library", required = true)]
        libraries: Vec<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ProjectCommands {
    /// Create a deterministic native project scaffold
    New {
        /// Project root directory
        path: PathBuf,
        /// Project display name; defaults to the directory basename
        #[arg(long)]
        name: Option<String>,
    },
    /// Inspect a native project scaffold and report resolved file layout
    Inspect {
        /// Project root directory
        path: PathBuf,
    },
    /// Query native project data from the on-disk scaffold
    Query {
        /// Project root directory
        path: PathBuf,
        /// What to query
        #[command(subcommand)]
        what: NativeProjectQueryCommands,
    },
    /// Export a native project BOM as deterministic CSV from persisted board components
    ExportBom {
        /// Project root directory
        path: PathBuf,
        /// Output CSV path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Compare a BOM CSV against the current native board-component inventory
    CompareBom {
        /// Project root directory
        path: PathBuf,
        /// BOM CSV path to compare
        #[arg(long = "bom")]
        bom: PathBuf,
    },
    /// Export a native project pick-and-place file as deterministic CSV from persisted board components
    ExportPnp {
        /// Project root directory
        path: PathBuf,
        /// Output CSV path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Compare a PnP CSV against the current native board-component inventory
    ComparePnp {
        /// Project root directory
        path: PathBuf,
        /// PnP CSV path to compare
        #[arg(long = "pnp")]
        pnp: PathBuf,
    },
    /// Export a native project drill file as deterministic CSV from persisted vias
    ExportDrill {
        /// Project root directory
        path: PathBuf,
        /// Output CSV path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Export a native project drill file as narrow Excellon from persisted vias
    ExportExcellonDrill {
        /// Project root directory
        path: PathBuf,
        /// Output drill path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Inspect a narrow Excellon drill file and report its tool table and hit counts
    InspectExcellonDrill {
        /// Drill path to inspect
        path: PathBuf,
    },
    /// Compare a narrow Excellon drill file against the current native via inventory
    CompareExcellonDrill {
        /// Project root directory
        path: PathBuf,
        /// Drill path to compare
        #[arg(long = "drill")]
        drill: PathBuf,
    },
    /// Report native drill hole classes from via span and stackup data
    ReportDrillHoleClasses {
        /// Project root directory
        path: PathBuf,
    },
    /// Export the native board outline as a narrow RS-274X Gerber file
    ExportGerberOutline {
        /// Project root directory
        path: PathBuf,
        /// Output Gerber path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Export one native board copper layer as a narrow RS-274X Gerber file
    ExportGerberCopperLayer {
        /// Project root directory
        path: PathBuf,
        /// Layer identifier
        #[arg(long = "layer")]
        layer: i32,
        /// Output Gerber path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Validate a narrow RS-274X board-outline Gerber against the current native board outline
    ValidateGerberOutline {
        /// Project root directory
        path: PathBuf,
        /// Gerber path to validate
        #[arg(long = "gerber")]
        gerber: PathBuf,
    },
    /// Validate a narrow RS-274X copper-layer Gerber against the current native board tracks on one layer
    ValidateGerberCopperLayer {
        /// Project root directory
        path: PathBuf,
        /// Layer identifier
        #[arg(long = "layer")]
        layer: i32,
        /// Gerber path to validate
        #[arg(long = "gerber")]
        gerber: PathBuf,
    },
    /// Compare a narrow RS-274X board-outline Gerber semantically against the current native board outline
    CompareGerberOutline {
        /// Project root directory
        path: PathBuf,
        /// Gerber path to compare
        #[arg(long = "gerber")]
        gerber: PathBuf,
    },
    /// Compare a narrow RS-274X copper-layer Gerber semantically against the current native copper geometry on one layer
    CompareGerberCopperLayer {
        /// Project root directory
        path: PathBuf,
        /// Layer identifier
        #[arg(long = "layer")]
        layer: i32,
        /// Gerber path to compare
        #[arg(long = "gerber")]
        gerber: PathBuf,
    },
    /// Validate a narrow Excellon drill file against the current native via inventory
    ValidateExcellonDrill {
        /// Project root directory
        path: PathBuf,
        /// Drill path to validate
        #[arg(long = "drill")]
        drill: PathBuf,
    },
    /// Plan the native Gerber export artifact set from the current board outline and stackup
    PlanGerberExport {
        /// Project root directory
        path: PathBuf,
        /// Optional artifact filename prefix; defaults to the board name
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Compare the planned native Gerber artifact set against an output directory
    CompareGerberExportPlan {
        /// Project root directory
        path: PathBuf,
        /// Directory to compare against the planned artifact set
        #[arg(long = "output-dir")]
        output_dir: PathBuf,
        /// Optional artifact filename prefix; defaults to the board name
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Place one schematic symbol into an existing native sheet file
    PlaceSymbol {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Reference designator
        #[arg(long)]
        reference: String,
        /// Display value
        #[arg(long)]
        value: String,
        /// Optional library identifier for future resolution
        #[arg(long = "lib-id")]
        lib_id: Option<String>,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
        /// Rotation in degrees
        #[arg(long = "rotation-deg", default_value_t = 0)]
        rotation_deg: i32,
        /// Mirror the symbol about its local Y axis
        #[arg(long, default_value_t = false)]
        mirrored: bool,
    },
    /// Move one schematic symbol in a native sheet file
    MoveSymbol {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// New X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// New Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Rotate one schematic symbol in a native sheet file
    RotateSymbol {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// New rotation in degrees
        #[arg(long = "rotation-deg")]
        rotation_deg: i32,
    },
    /// Mirror one schematic symbol in a native sheet file
    MirrorSymbol {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Delete one schematic symbol from a native sheet file
    DeleteSymbol {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol reference in a native sheet file
    SetSymbolReference {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement reference designator
        #[arg(long)]
        reference: String,
    },
    /// Set one schematic symbol value in a native sheet file
    SetSymbolValue {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement value text
        #[arg(long)]
        value: String,
    },
    /// Set one schematic symbol library identifier in a native sheet file
    SetSymbolLibId {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement library identifier
        #[arg(long = "lib-id")]
        lib_id: String,
    },
    /// Clear one schematic symbol library identifier in a native sheet file
    ClearSymbolLibId {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol unresolved entity identifier in a native sheet file
    SetSymbolEntity {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement entity UUID
        #[arg(long = "entity")]
        entity_uuid: Uuid,
    },
    /// Clear one schematic symbol unresolved entity identifier in a native sheet file
    ClearSymbolEntity {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol resolved part identifier in a native sheet file
    SetSymbolPart {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement part UUID
        #[arg(long = "part")]
        part_uuid: Uuid,
    },
    /// Clear one schematic symbol resolved part identifier in a native sheet file
    ClearSymbolPart {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol unit selection in a native sheet file
    SetSymbolUnit {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement unit selection token
        #[arg(long = "unit")]
        unit_selection: String,
    },
    /// Clear one schematic symbol unit selection in a native sheet file
    ClearSymbolUnit {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol gate selection in a native sheet file
    SetSymbolGate {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Gate UUID
        #[arg(long = "gate")]
        gate_uuid: Uuid,
    },
    /// Clear one schematic symbol gate selection in a native sheet file
    ClearSymbolGate {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Set one schematic symbol display mode in a native sheet file
    SetSymbolDisplayMode {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement display mode
        #[arg(long = "mode", value_enum)]
        display_mode: NativeSymbolDisplayModeArg,
    },
    /// Set one per-pin display override in a native schematic symbol
    SetPinOverride {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Pin UUID
        #[arg(long = "pin")]
        pin_uuid: Uuid,
        /// Replacement visible state
        #[arg(long, action = clap::ArgAction::Set)]
        visible: bool,
        /// Replacement pin X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// Replacement pin Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
    },
    /// Clear one per-pin display override in a native schematic symbol
    ClearPinOverride {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Pin UUID
        #[arg(long = "pin")]
        pin_uuid: Uuid,
    },
    /// Set one schematic symbol hidden-power behavior in a native sheet file
    SetSymbolHiddenPowerBehavior {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Replacement hidden-power behavior
        #[arg(long = "behavior", value_enum)]
        hidden_power_behavior: NativeHiddenPowerBehaviorArg,
    },
    /// Add one field to a native schematic symbol
    AddSymbolField {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Field key
        #[arg(long)]
        key: String,
        /// Field value
        #[arg(long)]
        value: String,
        /// Mark the field hidden instead of visible
        #[arg(long, default_value_t = false)]
        hidden: bool,
        /// Optional field X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// Optional field Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
    },
    /// Edit one field on a native schematic symbol
    EditSymbolField {
        /// Project root directory
        path: PathBuf,
        /// Field UUID
        #[arg(long)]
        field: Uuid,
        /// Replacement field key
        #[arg(long)]
        key: Option<String>,
        /// Replacement field value
        #[arg(long)]
        value: Option<String>,
        /// Replacement visible state
        #[arg(long)]
        visible: Option<bool>,
        /// Replacement field X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// Replacement field Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
    },
    /// Delete one field from a native schematic symbol
    DeleteSymbolField {
        /// Project root directory
        path: PathBuf,
        /// Field UUID
        #[arg(long)]
        field: Uuid,
    },
    /// Place one schematic text object into an existing native sheet file
    PlaceText {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Text content
        #[arg(long)]
        text: String,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
        /// Rotation in degrees
        #[arg(long = "rotation-deg", default_value_t = 0)]
        rotation_deg: i32,
    },
    /// Edit one schematic text object in a native sheet file
    EditText {
        /// Project root directory
        path: PathBuf,
        /// Text UUID
        #[arg(long)]
        text: Uuid,
        /// Replacement text content
        #[arg(long)]
        value: Option<String>,
        /// Replacement X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// Replacement Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
        /// Replacement rotation in degrees
        #[arg(long = "rotation-deg")]
        rotation_deg: Option<i32>,
    },
    /// Delete one schematic text object from a native sheet file
    DeleteText {
        /// Project root directory
        path: PathBuf,
        /// Text UUID
        #[arg(long)]
        text: Uuid,
    },
    /// Place one schematic drawing line into an existing native sheet file
    PlaceDrawingLine {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Start X coordinate in nm
        #[arg(long)]
        from_x_nm: i64,
        /// Start Y coordinate in nm
        #[arg(long)]
        from_y_nm: i64,
        /// End X coordinate in nm
        #[arg(long)]
        to_x_nm: i64,
        /// End Y coordinate in nm
        #[arg(long)]
        to_y_nm: i64,
    },
    /// Place one schematic drawing rectangle into an existing native sheet file
    PlaceDrawingRect {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Minimum X coordinate in nm
        #[arg(long)]
        min_x_nm: i64,
        /// Minimum Y coordinate in nm
        #[arg(long)]
        min_y_nm: i64,
        /// Maximum X coordinate in nm
        #[arg(long)]
        max_x_nm: i64,
        /// Maximum Y coordinate in nm
        #[arg(long)]
        max_y_nm: i64,
    },
    /// Place one schematic drawing circle into an existing native sheet file
    PlaceDrawingCircle {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Center X coordinate in nm
        #[arg(long)]
        center_x_nm: i64,
        /// Center Y coordinate in nm
        #[arg(long)]
        center_y_nm: i64,
        /// Radius in nm
        #[arg(long)]
        radius_nm: i64,
    },
    /// Place one schematic drawing arc into an existing native sheet file
    PlaceDrawingArc {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Center X coordinate in nm
        #[arg(long)]
        center_x_nm: i64,
        /// Center Y coordinate in nm
        #[arg(long)]
        center_y_nm: i64,
        /// Radius in nm
        #[arg(long)]
        radius_nm: i64,
        /// Start angle in millidegrees
        #[arg(long)]
        start_angle_mdeg: i32,
        /// End angle in millidegrees
        #[arg(long)]
        end_angle_mdeg: i32,
    },
    /// Edit one schematic drawing line in a native sheet file
    EditDrawingLine {
        /// Project root directory
        path: PathBuf,
        /// Drawing UUID
        #[arg(long = "drawing")]
        drawing: Uuid,
        /// Replacement start X coordinate in nm
        #[arg(long)]
        from_x_nm: Option<i64>,
        /// Replacement start Y coordinate in nm
        #[arg(long)]
        from_y_nm: Option<i64>,
        /// Replacement end X coordinate in nm
        #[arg(long)]
        to_x_nm: Option<i64>,
        /// Replacement end Y coordinate in nm
        #[arg(long)]
        to_y_nm: Option<i64>,
    },
    /// Edit one schematic drawing rectangle in a native sheet file
    EditDrawingRect {
        /// Project root directory
        path: PathBuf,
        /// Drawing UUID
        #[arg(long = "drawing")]
        drawing: Uuid,
        /// Replacement minimum X coordinate in nm
        #[arg(long)]
        min_x_nm: Option<i64>,
        /// Replacement minimum Y coordinate in nm
        #[arg(long)]
        min_y_nm: Option<i64>,
        /// Replacement maximum X coordinate in nm
        #[arg(long)]
        max_x_nm: Option<i64>,
        /// Replacement maximum Y coordinate in nm
        #[arg(long)]
        max_y_nm: Option<i64>,
    },
    /// Edit one schematic drawing circle in a native sheet file
    EditDrawingCircle {
        /// Project root directory
        path: PathBuf,
        /// Drawing UUID
        #[arg(long = "drawing")]
        drawing: Uuid,
        /// Replacement center X coordinate in nm
        #[arg(long)]
        center_x_nm: Option<i64>,
        /// Replacement center Y coordinate in nm
        #[arg(long)]
        center_y_nm: Option<i64>,
        /// Replacement radius in nm
        #[arg(long)]
        radius_nm: Option<i64>,
    },
    /// Edit one schematic drawing arc in a native sheet file
    EditDrawingArc {
        /// Project root directory
        path: PathBuf,
        /// Drawing UUID
        #[arg(long = "drawing")]
        drawing: Uuid,
        /// Replacement center X coordinate in nm
        #[arg(long)]
        center_x_nm: Option<i64>,
        /// Replacement center Y coordinate in nm
        #[arg(long)]
        center_y_nm: Option<i64>,
        /// Replacement radius in nm
        #[arg(long)]
        radius_nm: Option<i64>,
        /// Replacement start angle in millidegrees
        #[arg(long)]
        start_angle_mdeg: Option<i32>,
        /// Replacement end angle in millidegrees
        #[arg(long)]
        end_angle_mdeg: Option<i32>,
    },
    /// Delete one schematic drawing primitive from a native sheet file
    DeleteDrawing {
        /// Project root directory
        path: PathBuf,
        /// Drawing UUID
        #[arg(long = "drawing")]
        drawing: Uuid,
    },
    /// Place one schematic label into an existing native sheet file
    PlaceLabel {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Label name
        #[arg(long)]
        name: String,
        /// Label kind
        #[arg(long, value_enum, default_value = "local")]
        kind: NativeLabelKindArg,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Rename one schematic label in a native sheet file
    RenameLabel {
        /// Project root directory
        path: PathBuf,
        /// Label UUID
        #[arg(long)]
        label: Uuid,
        /// New label name
        #[arg(long)]
        name: String,
    },
    /// Delete one schematic label from a native sheet file
    DeleteLabel {
        /// Project root directory
        path: PathBuf,
        /// Label UUID
        #[arg(long)]
        label: Uuid,
    },
    /// Draw one schematic wire into an existing native sheet file
    DrawWire {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Start X coordinate in nm
        #[arg(long)]
        from_x_nm: i64,
        /// Start Y coordinate in nm
        #[arg(long)]
        from_y_nm: i64,
        /// End X coordinate in nm
        #[arg(long)]
        to_x_nm: i64,
        /// End Y coordinate in nm
        #[arg(long)]
        to_y_nm: i64,
    },
    /// Delete one schematic wire from a native sheet file
    DeleteWire {
        /// Project root directory
        path: PathBuf,
        /// Wire UUID
        #[arg(long)]
        wire: Uuid,
    },
    /// Place one schematic junction into an existing native sheet file
    PlaceJunction {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Delete one schematic junction from a native sheet file
    DeleteJunction {
        /// Project root directory
        path: PathBuf,
        /// Junction UUID
        #[arg(long)]
        junction: Uuid,
    },
    /// Place one hierarchical port into an existing native sheet file
    PlacePort {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Port name
        #[arg(long)]
        name: String,
        /// Port direction
        #[arg(long, value_enum)]
        direction: NativePortDirectionArg,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Edit one hierarchical port in a native sheet file
    EditPort {
        /// Project root directory
        path: PathBuf,
        /// Port UUID
        #[arg(long)]
        port: Uuid,
        /// New port name
        #[arg(long)]
        name: Option<String>,
        /// New port direction
        #[arg(long, value_enum)]
        direction: Option<NativePortDirectionArg>,
        /// New X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// New Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
    },
    /// Delete one hierarchical port from a native sheet file
    DeletePort {
        /// Project root directory
        path: PathBuf,
        /// Port UUID
        #[arg(long)]
        port: Uuid,
    },
    /// Create one bus in an existing native sheet file
    CreateBus {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Bus name
        #[arg(long)]
        name: String,
        /// Bus members
        #[arg(long = "member")]
        members: Vec<String>,
    },
    /// Edit one bus member list in a native sheet file
    EditBusMembers {
        /// Project root directory
        path: PathBuf,
        /// Bus UUID
        #[arg(long)]
        bus: Uuid,
        /// Replacement member list
        #[arg(long = "member")]
        members: Vec<String>,
    },
    /// Place one bus entry in an existing native sheet file
    PlaceBusEntry {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Parent bus UUID
        #[arg(long)]
        bus: Uuid,
        /// Optional attached wire UUID
        #[arg(long)]
        wire: Option<Uuid>,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Delete one bus entry from a native sheet file
    DeleteBusEntry {
        /// Project root directory
        path: PathBuf,
        /// Bus entry UUID
        #[arg(long = "bus-entry")]
        bus_entry: Uuid,
    },
    /// Place one no-connect marker into an existing native sheet file
    #[command(name = "place-noconnect")]
    PlaceNoConnect {
        /// Project root directory
        path: PathBuf,
        /// Target sheet UUID
        #[arg(long)]
        sheet: Uuid,
        /// Target symbol UUID
        #[arg(long)]
        symbol: Uuid,
        /// Target pin UUID
        #[arg(long)]
        pin: Uuid,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
    },
    /// Delete one no-connect marker from a native sheet file
    #[command(name = "delete-noconnect")]
    DeleteNoConnect {
        /// Project root directory
        path: PathBuf,
        /// No-connect UUID
        #[arg(long = "noconnect")]
        noconnect: Uuid,
    },
    /// Place one board text object into the native board file
    PlaceBoardText {
        /// Project root directory
        path: PathBuf,
        /// Text content
        #[arg(long)]
        text: String,
        /// X coordinate in nm
        #[arg(long)]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long)]
        y_nm: i64,
        /// Rotation in degrees
        #[arg(long = "rotation-deg", default_value_t = 0)]
        rotation_deg: i32,
        /// Layer identifier
        #[arg(long)]
        layer: i32,
    },
    /// Edit one board text object in the native board file
    EditBoardText {
        /// Project root directory
        path: PathBuf,
        /// Board text UUID
        #[arg(long = "text")]
        text_uuid: Uuid,
        /// Replacement text content
        #[arg(long)]
        value: Option<String>,
        /// Replacement X coordinate in nm
        #[arg(long)]
        x_nm: Option<i64>,
        /// Replacement Y coordinate in nm
        #[arg(long)]
        y_nm: Option<i64>,
        /// Replacement rotation in degrees
        #[arg(long = "rotation-deg")]
        rotation_deg: Option<i32>,
        /// Replacement layer identifier
        #[arg(long)]
        layer: Option<i32>,
    },
    /// Delete one board text object from the native board file
    DeleteBoardText {
        /// Project root directory
        path: PathBuf,
        /// Board text UUID
        #[arg(long = "text")]
        text_uuid: Uuid,
    },
    /// Place one board keepout into the native board file
    PlaceBoardKeepout {
        /// Project root directory
        path: PathBuf,
        /// Keepout kind
        #[arg(long)]
        kind: String,
        /// Board layers affected by the keepout
        #[arg(long = "layer")]
        layers: Vec<i32>,
        /// Polygon vertices as x_nm:y_nm
        #[arg(long = "vertex")]
        vertices: Vec<String>,
    },
    /// Edit one board keepout in the native board file
    EditBoardKeepout {
        /// Project root directory
        path: PathBuf,
        /// Keepout UUID
        #[arg(long = "keepout")]
        keepout_uuid: Uuid,
        /// Replacement keepout kind
        #[arg(long)]
        kind: Option<String>,
        /// Replacement board layers
        #[arg(long = "layer")]
        layers: Vec<i32>,
        /// Replacement polygon vertices as x_nm:y_nm
        #[arg(long = "vertex")]
        vertices: Vec<String>,
    },
    /// Delete one board keepout from the native board file
    DeleteBoardKeepout {
        /// Project root directory
        path: PathBuf,
        /// Keepout UUID
        #[arg(long = "keepout")]
        keepout_uuid: Uuid,
    },
    /// Replace the native board outline polygon
    SetBoardOutline {
        /// Project root directory
        path: PathBuf,
        /// Polygon vertices as x_nm:y_nm
        #[arg(long = "vertex")]
        vertices: Vec<String>,
    },
    /// Replace the native board stackup layer list
    SetBoardStackup {
        /// Project root directory
        path: PathBuf,
        /// Layer spec as id:name:type:thickness_nm
        #[arg(long = "layer")]
        layers: Vec<String>,
    },
    /// Create one native board net class
    PlaceBoardNetClass {
        /// Project root directory
        path: PathBuf,
        /// Net class name
        #[arg(long)]
        name: String,
        /// Clearance in nm
        #[arg(long = "clearance-nm")]
        clearance_nm: i64,
        /// Track width in nm
        #[arg(long = "track-width-nm")]
        track_width_nm: i64,
        /// Via drill in nm
        #[arg(long = "via-drill-nm")]
        via_drill_nm: i64,
        /// Via diameter in nm
        #[arg(long = "via-diameter-nm")]
        via_diameter_nm: i64,
        /// Differential pair width in nm
        #[arg(long = "diffpair-width-nm", default_value_t = 0)]
        diffpair_width_nm: i64,
        /// Differential pair gap in nm
        #[arg(long = "diffpair-gap-nm", default_value_t = 0)]
        diffpair_gap_nm: i64,
    },
    /// Create one native board net
    PlaceBoardNet {
        /// Project root directory
        path: PathBuf,
        /// Net name
        #[arg(long)]
        name: String,
        /// Assigned net-class UUID
        #[arg(long = "class")]
        class_uuid: Uuid,
    },
    /// Place one native board component/package
    PlaceBoardComponent {
        /// Project root directory
        path: PathBuf,
        /// Part UUID
        #[arg(long = "part")]
        part_uuid: Uuid,
        /// Package UUID
        #[arg(long = "package")]
        package_uuid: Uuid,
        /// Reference designator
        #[arg(long)]
        reference: String,
        /// Value text
        #[arg(long)]
        value: String,
        /// X coordinate in nm
        #[arg(long = "x-nm")]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long = "y-nm")]
        y_nm: i64,
        /// Layer identifier
        #[arg(long)]
        layer: i32,
    },
    /// Edit one native board net class
    EditBoardNetClass {
        /// Project root directory
        path: PathBuf,
        /// Net class UUID
        #[arg(long = "net-class")]
        net_class_uuid: Uuid,
        /// Replacement net class name
        #[arg(long)]
        name: Option<String>,
        /// Replacement clearance in nm
        #[arg(long = "clearance-nm")]
        clearance_nm: Option<i64>,
        /// Replacement track width in nm
        #[arg(long = "track-width-nm")]
        track_width_nm: Option<i64>,
        /// Replacement via drill in nm
        #[arg(long = "via-drill-nm")]
        via_drill_nm: Option<i64>,
        /// Replacement via diameter in nm
        #[arg(long = "via-diameter-nm")]
        via_diameter_nm: Option<i64>,
        /// Replacement differential pair width in nm
        #[arg(long = "diffpair-width-nm")]
        diffpair_width_nm: Option<i64>,
        /// Replacement differential pair gap in nm
        #[arg(long = "diffpair-gap-nm")]
        diffpair_gap_nm: Option<i64>,
    },
    /// Edit one native board net
    EditBoardNet {
        /// Project root directory
        path: PathBuf,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
        /// Replacement net name
        #[arg(long)]
        name: Option<String>,
        /// Replacement net-class UUID
        #[arg(long = "class")]
        class_uuid: Option<Uuid>,
    },
    /// Move one native board component/package
    MoveBoardComponent {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
        /// X coordinate in nm
        #[arg(long = "x-nm")]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long = "y-nm")]
        y_nm: i64,
    },
    /// Set the part UUID on one native board component/package
    SetBoardComponentPart {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
        /// Replacement part UUID
        #[arg(long = "part")]
        part_uuid: Uuid,
    },
    /// Set the package UUID on one native board component/package
    SetBoardComponentPackage {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
        /// Replacement package UUID
        #[arg(long = "package")]
        package_uuid: Uuid,
    },
    /// Rotate one native board component/package
    RotateBoardComponent {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
        /// Rotation in degrees
        #[arg(long = "rotation-deg")]
        rotation_deg: i32,
    },
    /// Lock one native board component/package
    SetBoardComponentLocked {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
    },
    /// Clear the locked state on one native board component/package
    ClearBoardComponentLocked {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
    },
    /// Delete one native board component/package
    DeleteBoardComponent {
        /// Project root directory
        path: PathBuf,
        /// Component UUID
        #[arg(long = "component")]
        component_uuid: Uuid,
    },
    /// Apply one supported forward-annotation proposal action by stable action ID
    ApplyForwardAnnotationAction {
        /// Project root directory
        path: PathBuf,
        /// Stable proposal action ID
        #[arg(long = "action-id")]
        action_id: String,
        /// Explicit package UUID for add_component actions
        #[arg(long = "package")]
        package_uuid: Option<Uuid>,
        /// Explicit part UUID override for add_component actions
        #[arg(long = "part")]
        part_uuid: Option<Uuid>,
        /// Placement X coordinate in nm for add_component actions
        #[arg(long = "x-nm")]
        x_nm: Option<i64>,
        /// Placement Y coordinate in nm for add_component actions
        #[arg(long = "y-nm")]
        y_nm: Option<i64>,
        /// Placement layer for add_component actions
        #[arg(long = "layer")]
        layer: Option<i32>,
    },
    /// Apply all currently self-sufficient forward-annotation proposal actions while honoring persisted defer/reject review state
    ApplyForwardAnnotationReviewed {
        /// Project root directory
        path: PathBuf,
    },
    /// Export the current forward-annotation proposal and review state as a versioned artifact
    ExportForwardAnnotationProposal {
        /// Project root directory
        path: PathBuf,
        /// Output artifact path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Export a selected subset of the current forward-annotation proposal and matching review state as a versioned artifact
    ExportForwardAnnotationProposalSelection {
        /// Project root directory
        path: PathBuf,
        /// Stable proposal action IDs to retain
        #[arg(long = "action-id")]
        action_ids: Vec<String>,
        /// Output artifact path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Select a subset of actions from an existing forward-annotation proposal artifact
    SelectForwardAnnotationProposalArtifact {
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
        /// Stable proposal action IDs to retain
        #[arg(long = "action-id")]
        action_ids: Vec<String>,
        /// Output artifact path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Inspect a versioned forward-annotation proposal artifact
    InspectForwardAnnotationProposalArtifact {
        /// Artifact path
        path: PathBuf,
    },
    /// Compare a forward-annotation proposal artifact against the current live project proposal state
    CompareForwardAnnotationProposalArtifact {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
    },
    /// Filter a forward-annotation proposal artifact down to actions still applicable against the current live project proposal
    FilterForwardAnnotationProposalArtifact {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
        /// Output artifact path
        #[arg(long = "out")]
        out: PathBuf,
    },
    /// Plan artifact-driven forward-annotation apply without mutating project state
    PlanForwardAnnotationProposalArtifactApply {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
    },
    /// Apply one filtered forward-annotation proposal artifact when all retained actions are still self-sufficient
    ApplyForwardAnnotationProposalArtifact {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
    },
    /// Import forward-annotation review decisions from an artifact into the current live project state
    ImportForwardAnnotationArtifactReview {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
    },
    /// Replace live forward-annotation review state with validated review decisions from an artifact
    ReplaceForwardAnnotationArtifactReview {
        /// Project root directory
        path: PathBuf,
        /// Artifact path
        #[arg(long = "artifact")]
        artifact: PathBuf,
    },
    /// Mark one forward-annotation proposal action as deferred by stable action ID
    DeferForwardAnnotationAction {
        /// Project root directory
        path: PathBuf,
        /// Stable proposal action ID
        #[arg(long = "action-id")]
        action_id: String,
    },
    /// Mark one forward-annotation proposal action as rejected by stable action ID
    RejectForwardAnnotationAction {
        /// Project root directory
        path: PathBuf,
        /// Stable proposal action ID
        #[arg(long = "action-id")]
        action_id: String,
    },
    /// Clear one persisted forward-annotation review decision by stable action ID
    ClearForwardAnnotationActionReview {
        /// Project root directory
        path: PathBuf,
        /// Stable proposal action ID
        #[arg(long = "action-id")]
        action_id: String,
    },
    /// Draw one native board track
    DrawBoardTrack {
        /// Project root directory
        path: PathBuf,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
        /// Start X coordinate in nm
        #[arg(long = "from-x-nm")]
        from_x_nm: i64,
        /// Start Y coordinate in nm
        #[arg(long = "from-y-nm")]
        from_y_nm: i64,
        /// End X coordinate in nm
        #[arg(long = "to-x-nm")]
        to_x_nm: i64,
        /// End Y coordinate in nm
        #[arg(long = "to-y-nm")]
        to_y_nm: i64,
        /// Track width in nm
        #[arg(long = "width-nm")]
        width_nm: i64,
        /// Layer identifier
        #[arg(long)]
        layer: i32,
    },
    /// Delete one native board track
    DeleteBoardTrack {
        /// Project root directory
        path: PathBuf,
        /// Track UUID
        #[arg(long = "track")]
        track_uuid: Uuid,
    },
    /// Place one native board via
    PlaceBoardVia {
        /// Project root directory
        path: PathBuf,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
        /// X coordinate in nm
        #[arg(long = "x-nm")]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long = "y-nm")]
        y_nm: i64,
        /// Via drill in nm
        #[arg(long = "drill-nm")]
        drill_nm: i64,
        /// Via diameter in nm
        #[arg(long = "diameter-nm")]
        diameter_nm: i64,
        /// Starting layer identifier
        #[arg(long = "from-layer")]
        from_layer: i32,
        /// Ending layer identifier
        #[arg(long = "to-layer")]
        to_layer: i32,
    },
    /// Delete one native board via
    DeleteBoardVia {
        /// Project root directory
        path: PathBuf,
        /// Via UUID
        #[arg(long = "via")]
        via_uuid: Uuid,
    },
    /// Place one native board zone
    PlaceBoardZone {
        /// Project root directory
        path: PathBuf,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
        /// Polygon vertices as x_nm:y_nm
        #[arg(long = "vertex")]
        vertices: Vec<String>,
        /// Layer identifier
        #[arg(long)]
        layer: i32,
        /// Zone priority
        #[arg(long, default_value_t = 0)]
        priority: u32,
        /// Thermal relief enabled
        #[arg(
            long = "thermal-relief",
            default_value_t = true,
            action = clap::ArgAction::Set
        )]
        thermal_relief: bool,
        /// Thermal gap in nm
        #[arg(long = "thermal-gap-nm")]
        thermal_gap_nm: i64,
        /// Thermal spoke width in nm
        #[arg(long = "thermal-spoke-width-nm")]
        thermal_spoke_width_nm: i64,
    },
    /// Delete one native board zone
    DeleteBoardZone {
        /// Project root directory
        path: PathBuf,
        /// Zone UUID
        #[arg(long = "zone")]
        zone_uuid: Uuid,
    },
    /// Set one native board pad net assignment
    SetBoardPadNet {
        /// Project root directory
        path: PathBuf,
        /// Pad UUID
        #[arg(long = "pad")]
        pad_uuid: Uuid,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
    },
    /// Clear one native board pad net assignment
    ClearBoardPadNet {
        /// Project root directory
        path: PathBuf,
        /// Pad UUID
        #[arg(long = "pad")]
        pad_uuid: Uuid,
    },
    /// Edit one native board pad position and/or layer
    EditBoardPad {
        /// Project root directory
        path: PathBuf,
        /// Pad UUID
        #[arg(long = "pad")]
        pad_uuid: Uuid,
        /// Replacement X coordinate in nm
        #[arg(long = "x-nm")]
        x_nm: Option<i64>,
        /// Replacement Y coordinate in nm
        #[arg(long = "y-nm")]
        y_nm: Option<i64>,
        /// Replacement layer identifier
        #[arg(long)]
        layer: Option<i32>,
        /// Replacement circular copper diameter in nm
        #[arg(long = "diameter-nm")]
        diameter_nm: Option<i64>,
    },
    /// Place one native board pad
    PlaceBoardPad {
        /// Project root directory
        path: PathBuf,
        /// Package UUID
        #[arg(long = "package")]
        package_uuid: Uuid,
        /// Pad name
        #[arg(long)]
        name: String,
        /// X coordinate in nm
        #[arg(long = "x-nm")]
        x_nm: i64,
        /// Y coordinate in nm
        #[arg(long = "y-nm")]
        y_nm: i64,
        /// Layer identifier
        #[arg(long)]
        layer: i32,
        /// Circular copper diameter in nm
        #[arg(long = "diameter-nm")]
        diameter_nm: i64,
        /// Optional net UUID
        #[arg(long = "net")]
        net_uuid: Option<Uuid>,
    },
    /// Delete one native board pad
    DeleteBoardPad {
        /// Project root directory
        path: PathBuf,
        /// Pad UUID
        #[arg(long = "pad")]
        pad_uuid: Uuid,
    },
    /// Delete one native board net class
    DeleteBoardNetClass {
        /// Project root directory
        path: PathBuf,
        /// Net class UUID
        #[arg(long = "net-class")]
        net_class_uuid: Uuid,
    },
    /// Delete one native board net
    DeleteBoardNet {
        /// Project root directory
        path: PathBuf,
        /// Net UUID
        #[arg(long = "net")]
        net_uuid: Uuid,
    },
    /// Place one board dimension into the native board file
    PlaceBoardDimension {
        /// Project root directory
        path: PathBuf,
        /// Start X coordinate in nm
        #[arg(long = "from-x-nm")]
        from_x_nm: i64,
        /// Start Y coordinate in nm
        #[arg(long = "from-y-nm")]
        from_y_nm: i64,
        /// End X coordinate in nm
        #[arg(long = "to-x-nm")]
        to_x_nm: i64,
        /// End Y coordinate in nm
        #[arg(long = "to-y-nm")]
        to_y_nm: i64,
        /// Optional dimension text
        #[arg(long)]
        text: Option<String>,
    },
    /// Edit one board dimension in the native board file
    EditBoardDimension {
        /// Project root directory
        path: PathBuf,
        /// Dimension UUID
        #[arg(long = "dimension")]
        dimension_uuid: Uuid,
        /// Replacement start X coordinate in nm
        #[arg(long = "from-x-nm")]
        from_x_nm: Option<i64>,
        /// Replacement start Y coordinate in nm
        #[arg(long = "from-y-nm")]
        from_y_nm: Option<i64>,
        /// Replacement end X coordinate in nm
        #[arg(long = "to-x-nm")]
        to_x_nm: Option<i64>,
        /// Replacement end Y coordinate in nm
        #[arg(long = "to-y-nm")]
        to_y_nm: Option<i64>,
        /// Replacement dimension text
        #[arg(long)]
        text: Option<String>,
        /// Clear stored dimension text
        #[arg(long = "clear-text", default_value_t = false)]
        clear_text: bool,
    },
    /// Delete one board dimension from the native board file
    DeleteBoardDimension {
        /// Project root directory
        path: PathBuf,
        /// Dimension UUID
        #[arg(long = "dimension")]
        dimension_uuid: Uuid,
    },
}

#[derive(Subcommand)]
pub(crate) enum NativeProjectQueryCommands {
    /// Aggregated native project summary
    Summary,
    /// Current native design rules payload
    DesignRules,
    /// Current native schematic symbols
    Symbols,
    /// Current fields for one native schematic symbol
    SymbolFields {
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Current semantic selection state for one native schematic symbol
    SymbolSemantics {
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Current stored pins for one native schematic symbol
    SymbolPins {
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
    },
    /// Current native schematic text objects
    Texts,
    /// Current native schematic drawing primitives
    Drawings,
    /// Current native schematic labels
    Labels,
    /// Current native schematic wires
    Wires,
    /// Current native schematic junctions
    Junctions,
    /// Current native schematic hierarchical ports
    Ports,
    /// Current native schematic buses
    Buses,
    /// Current native schematic bus entries
    BusEntries,
    /// Current native schematic no-connect markers
    Noconnects,
    /// Current native schematic connectivity nets
    Nets,
    /// Current native schematic connectivity diagnostics
    Diagnostics,
    /// Current native schematic ERC findings
    Erc,
    /// Current native combined schematic check report
    Check,
    /// Current native board text objects
    BoardTexts,
    /// Current native board keepouts
    BoardKeepouts,
    /// Current native board outline polygon
    BoardOutline,
    /// Current native board stackup
    BoardStackup,
    /// Current native board placed packages/components
    BoardComponents,
    /// Current native board tracks
    BoardTracks,
    /// Current native board vias
    BoardVias,
    /// Current native board zones
    BoardZones,
    /// Current native board connectivity diagnostics
    BoardDiagnostics,
    /// Current native board unrouted airwires
    BoardUnrouted,
    /// Current native combined board check report
    BoardCheck,
    /// Current forward-annotation audit between native schematic and board state
    ForwardAnnotationAudit,
    /// Current read-only forward-annotation ECO proposal between native schematic and board state
    ForwardAnnotationProposal,
    /// Current persisted forward-annotation review decisions by stable action ID
    ForwardAnnotationReview,
    /// Current native board pads
    BoardPads,
    /// Current native board nets
    BoardNets,
    /// Current native board net classes
    BoardNetClasses,
    /// Current native board dimensions
    BoardDimensions,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeLabelKindArg {
    Local,
    Global,
    Hierarchical,
    Power,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativePortDirectionArg {
    Input,
    Output,
    Bidirectional,
    Passive,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeSymbolDisplayModeArg {
    LibraryDefault,
    ShowHiddenPins,
    HideOptionalPins,
}

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeHiddenPowerBehaviorArg {
    SourceDefinedImplicit,
    ExplicitPowerObject,
    PreservedAsImportedMetadata,
}

#[derive(Subcommand)]
pub(crate) enum PlanCommands {
    /// Export a versioned scoped replacement manifest
    ExportScopedReplacementManifest {
        /// Path to board design file
        path: PathBuf,
        /// Output manifest path
        #[arg(long)]
        out: PathBuf,
        /// Replacement policy to resolve
        #[arg(value_enum)]
        policy: ReplacementPolicyArg,
        /// Restrict matches by current reference prefix
        #[arg(long = "ref-prefix")]
        ref_prefix: Option<String>,
        /// Restrict matches by current value
        #[arg(long = "value")]
        value: Option<String>,
        /// Restrict matches by current package UUID
        #[arg(long = "package-uuid")]
        package_uuid: Option<Uuid>,
        /// Restrict matches by current part UUID
        #[arg(long = "part-uuid")]
        part_uuid: Option<Uuid>,
        /// Exclude one component UUID from the previewed plan
        #[arg(long = "exclude-component")]
        exclude_component: Vec<Uuid>,
        /// Override one component target: <component_uuid>:<target_package_uuid>:<target_part_uuid>
        #[arg(long = "override-component")]
        override_component: Vec<String>,
        /// Load Eagle libraries into the in-memory pool before querying the plan
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Inspect a scoped replacement manifest and report current provenance/drift status
    InspectScopedReplacementManifest {
        /// Manifest path
        path: PathBuf,
    },
    /// Validate a scoped replacement manifest for drift/missing inputs
    ValidateScopedReplacementManifest {
        /// Manifest path(s)
        paths: Vec<PathBuf>,
    },
    /// Rewrite a scoped replacement manifest into the current schema version
    UpgradeScopedReplacementManifest {
        /// Input manifest path
        path: PathBuf,
        /// Output manifest path
        #[arg(long)]
        out: Option<PathBuf>,
        /// Rewrite the input manifest in place
        #[arg(long, default_value_t = false)]
        in_place: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileFingerprint {
    pub(crate) path: PathBuf,
    pub(crate) source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifest {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) board_path: PathBuf,
    pub(crate) board_source_hash: String,
    pub(crate) libraries: Vec<ManifestFileFingerprint>,
    pub(crate) plan: ScopedComponentReplacementPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ManifestDriftStatus {
    Match,
    Drifted,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileInspection {
    pub(crate) path: PathBuf,
    pub(crate) recorded_source_hash: String,
    pub(crate) current_source_hash: Option<String>,
    pub(crate) status: ManifestDriftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestInspection {
    pub(crate) manifest_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
    pub(crate) all_inputs_match: bool,
    pub(crate) board: ManifestFileInspection,
    pub(crate) libraries: Vec<ManifestFileInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestUpgradeReport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) all_inputs_match: bool,
    pub(crate) board_status: ManifestDriftStatus,
    pub(crate) drifted_libraries: usize,
    pub(crate) missing_libraries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationSummary {
    pub(crate) manifests_checked: usize,
    pub(crate) manifests_passing: usize,
    pub(crate) manifests_failing: usize,
    pub(crate) reports: Vec<ScopedReplacementPlanManifestValidationReport>,
}
