use std::ffi::OsString;

use crate::*;

#[derive(Parser)]
#[command(name = "datum-eda", about = "PCB design analysis and automation")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Output format
    #[arg(long, default_value = "text")]
    pub(crate) format: OutputFormat,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Commands {
    /// Inspect Datum session/context discovery state
    Context {
        #[command(subcommand)]
        action: ContextCommands,
    },
    /// Import a KiCad or Eagle design
    Import {
        /// Path to design file (.kicad_pcb, .brd, .lbr)
        path: PathBuf,
    },
    /// Query resolver-backed native project data
    Query {
        #[command(subcommand)]
        action: QueryCommands,
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
    /// Run check, waiver, deviation, and standards-repair workflows
    Check {
        #[command(subcommand)]
        action: CheckCommands,
    },
    /// Search the component pool
    Pool {
        #[command(subcommand)]
        action: PoolCommands,
    },
    /// Review and apply persisted native project proposals
    Proposal {
        #[command(subcommand)]
        action: ProposalCommands,
    },
    /// Inspect and move the native project journal cursor
    Journal {
        #[command(subcommand)]
        action: JournalCommands,
    },
    /// Generate, inspect, and validate production artifacts
    Artifact {
        #[command(subcommand)]
        action: ArtifactCommands,
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
    /// Aggregated native project summary
    Summary(QueryPathArgs),
    /// Resolver-derived native project relationships
    Relationships(QueryPathArgs),
    /// Resolver-derived native project variants
    Variants(QueryPathArgs),
    /// Resolver-validated import-key identity sidecars
    ImportMap(QueryPathArgs),
    /// Resolver-derived native project component instances
    ComponentInstances(QueryPathArgs),
    /// Native schematic sheet index
    Sheets(QueryPathArgs),
    /// Native schematic symbols
    Symbols(QueryPathArgs),
    /// Native schematic labels
    Labels(QueryPathArgs),
    /// Native schematic hierarchical ports
    Ports(QueryPathArgs),
    /// Native schematic buses
    Buses(QueryPathArgs),
    /// Native schematic bus entries
    #[command(name = "bus-entries")]
    BusEntries(QueryPathArgs),
    /// Native schematic no-connect markers
    Noconnects(QueryPathArgs),
    /// Native schematic hierarchy links
    Hierarchy(QueryPathArgs),
    /// Native schematic connectivity nets
    #[command(name = "schematic-nets")]
    SchematicNets(QueryPathArgs),
    /// Native schematic connectivity diagnostics
    #[command(name = "connectivity-diagnostics")]
    ConnectivityDiagnostics(QueryPathArgs),
    /// Resolver-derived native project zone-fill state
    ZoneFills(QueryPathArgs),
    /// Resolver-discovered authored panel projections
    PanelProjections(QueryPathArgs),
    /// Resolver-discovered authored manufacturing plans
    ManufacturingPlans(QueryPathArgs),
    /// Resolver-discovered authored output jobs
    OutputJobs(QueryPathArgs),
    /// Legacy imported-design query compatibility
    Imported {
        /// Path to design file
        path: PathBuf,
        /// What to query
        #[command(subcommand)]
        what: ImportedQueryCommands,
    },
    /// Legacy `query <path> <what>` imported-design compatibility
    #[command(external_subcommand)]
    LegacyImported(Vec<OsString>),
}

#[derive(clap::Args)]
pub(crate) struct QueryPathArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(Parser)]
pub(crate) struct ImportedQueryCommandParser {
    #[command(subcommand)]
    pub(crate) what: ImportedQueryCommands,
}

#[derive(Subcommand)]
pub(crate) enum ImportedQueryCommands {
    /// Board summary (dimensions, counts)
    Summary,
    /// List the canonical netlist view
    Netlist,
    /// List all nets
    Nets,
    /// List schematic net inventory only
    SchematicNets,
    /// List all components
    Components,
    /// List schematic sheets
    Sheets,
    /// List schematic symbols
    Symbols,
    /// List schematic labels
    Labels,
    /// List schematic ports
    Ports,
    /// List schematic buses
    Buses,
    /// List schematic bus entries
    BusEntries,
    /// List schematic no-connect markers
    Noconnects,
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
