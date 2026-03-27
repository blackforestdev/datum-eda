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
        action: ProjectCommands,
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
    /// Delete one schematic symbol from a native sheet file
    DeleteSymbol {
        /// Project root directory
        path: PathBuf,
        /// Symbol UUID
        #[arg(long)]
        symbol: Uuid,
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
}

#[derive(Subcommand)]
pub(crate) enum NativeProjectQueryCommands {
    /// Aggregated native project summary
    Summary,
    /// Current native design rules payload
    DesignRules,
    /// Current native schematic symbols
    Symbols,
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
