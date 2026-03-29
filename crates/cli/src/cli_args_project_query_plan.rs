use super::*;
use crate::cli_args::cli_args_board_component::BoardComponentArgs;

#[derive(Subcommand)]
pub(crate) enum NativeProjectQueryCommands {
    /// Aggregated native project summary
    Summary,
    /// Current resolved native project pool refs
    Pools,
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
    /// Deterministic routing-kernel substrate from persisted native board state
    #[command(name = "routing-substrate")]
    RoutingSubstrate,
    /// Deterministic single-net routing preflight from persisted native board state
    #[command(name = "route-preflight")]
    RoutePreflight {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
    },
    /// Deterministic single-net corridor geometry from persisted native board state
    #[command(name = "route-corridor")]
    RouteCorridor {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
    },
    /// Deterministic single-layer path candidate for one authored anchor pair
    #[command(name = "route-path-candidate")]
    RoutePathCandidate {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic single-via path candidate reusing one authored target-net via
    #[command(name = "route-path-candidate-via")]
    RoutePathCandidateVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic two-via path candidate reusing exactly two authored target-net vias
    #[command(name = "route-path-candidate-two-via")]
    RoutePathCandidateTwoVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic three-via path candidate reusing exactly three authored target-net vias
    #[command(name = "route-path-candidate-three-via")]
    RoutePathCandidateThreeVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic four-via path candidate reusing exactly four authored target-net vias
    #[command(name = "route-path-candidate-four-via")]
    RoutePathCandidateFourVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic five-via path candidate reusing exactly five authored target-net vias
    #[command(name = "route-path-candidate-five-via")]
    RoutePathCandidateFiveVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic six-via path candidate reusing exactly six authored target-net vias
    #[command(name = "route-path-candidate-six-via")]
    RoutePathCandidateSixVia {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic authored via-chain path candidate reusing persisted target-net vias only
    #[command(name = "route-path-candidate-authored-via-chain")]
    RoutePathCandidateAuthoredViaChain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current authored-via-chain path candidate result
    #[command(name = "route-path-candidate-authored-via-chain-explain")]
    RoutePathCandidateAuthoredViaChainExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic existing-authored-copper path candidate over persisted target-net tracks and vias
    #[command(name = "route-path-candidate-authored-copper-graph")]
    RoutePathCandidateAuthoredCopperGraph {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current existing-authored-copper path candidate result
    #[command(name = "route-path-candidate-authored-copper-graph-explain")]
    RoutePathCandidateAuthoredCopperGraphExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic existing-authored-copper path candidate including authored target-net zone continuity
    #[command(name = "route-path-candidate-authored-copper-graph-zone-aware")]
    RoutePathCandidateAuthoredCopperGraphZoneAware {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current zone-aware existing-authored-copper path candidate result
    #[command(name = "route-path-candidate-authored-copper-graph-zone-aware-explain")]
    RoutePathCandidateAuthoredCopperGraphZoneAwareExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic existing-authored-copper path candidate reusing only zone/track/via graph edges unblocked by current authored obstacles
    #[command(name = "route-path-candidate-authored-copper-graph-zone-obstacle-aware")]
    RoutePathCandidateAuthoredCopperGraphZoneObstacleAware {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic existing-authored-copper path candidate reusing only unblocked authored track/via geometry
    #[command(name = "route-path-candidate-authored-copper-graph-obstacle-aware")]
    RoutePathCandidateAuthoredCopperGraphObstacleAware {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current obstacle-aware existing-authored-copper path candidate result
    #[command(name = "route-path-candidate-authored-copper-graph-obstacle-aware-explain")]
    RoutePathCandidateAuthoredCopperGraphObstacleAwareExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current six-via path candidate result
    #[command(name = "route-path-candidate-six-via-explain")]
    RoutePathCandidateSixViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current five-via path candidate result
    #[command(name = "route-path-candidate-five-via-explain")]
    RoutePathCandidateFiveViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current four-via path candidate result
    #[command(name = "route-path-candidate-four-via-explain")]
    RoutePathCandidateFourViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current three-via path candidate result
    #[command(name = "route-path-candidate-three-via-explain")]
    RoutePathCandidateThreeViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current two-via path candidate result
    #[command(name = "route-path-candidate-two-via-explain")]
    RoutePathCandidateTwoViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current single-via path candidate result
    #[command(name = "route-path-candidate-via-explain")]
    RoutePathCandidateViaExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Deterministic explanation for the current single-layer path candidate result
    #[command(name = "route-path-candidate-explain")]
    RoutePathCandidateExplain {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
        /// Source anchor pad UUID
        #[arg(long = "from-anchor")]
        from_anchor: Uuid,
        /// Target anchor pad UUID
        #[arg(long = "to-anchor")]
        to_anchor: Uuid,
    },
    /// Current native board placed packages/components
    BoardComponents,
    /// Current native board component for one component UUID
    #[command(name = "board-component")]
    BoardComponent(BoardComponentArgs),
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
    /// Current native board net for one net UUID
    #[command(name = "board-net")]
    BoardNet {
        /// Net UUID
        #[arg(long = "net")]
        net: Uuid,
    },
    /// Current native board net classes
    BoardNetClasses,
    /// Current native board net class for one net-class UUID
    #[command(name = "board-net-class")]
    BoardNetClass {
        /// Net class UUID
        #[arg(long = "net-class")]
        net_class: Uuid,
    },
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
    /// Inspect a scoped replacement manifest artifact without consulting live board/library inputs
    InspectScopedReplacementManifestArtifact {
        /// Manifest path
        path: PathBuf,
    },
    /// Validate a scoped replacement manifest for drift/missing inputs
    ValidateScopedReplacementManifest {
        /// Manifest path(s)
        paths: Vec<PathBuf>,
    },
    /// Validate a scoped replacement manifest artifact against the supported schema/version and current artifact encoding
    ValidateScopedReplacementManifestArtifact {
        /// Manifest path
        path: PathBuf,
    },
    /// Compare one scoped replacement manifest artifact against another artifact semantically after normalization
    CompareScopedReplacementManifestArtifact {
        /// Reference manifest path
        path: PathBuf,
        /// Artifact path to compare
        #[arg(long = "artifact")]
        artifact: PathBuf,
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
