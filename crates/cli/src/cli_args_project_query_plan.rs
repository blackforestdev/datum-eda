use super::*;

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
    /// Current persisted 3D model refs for one native board component
    #[command(name = "board-component-models-3d")]
    BoardComponentModels3d(BoardComponentModels3dArgs),
    /// Current persisted package-pad subset for one native board component
    #[command(name = "board-component-pads")]
    BoardComponentPads(BoardComponentPadsArgs),
    /// Current persisted package silkscreen subset for one native board component
    #[command(name = "board-component-silkscreen")]
    BoardComponentSilkscreen(BoardComponentSilkscreenArgs),
    /// Current persisted package mechanical subset for one native board component
    #[command(name = "board-component-mechanical")]
    BoardComponentMechanical(BoardComponentMechanicalArgs),
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
