use crate::*;

#[derive(Subcommand)]
pub(crate) enum ProjectCommands {
    /// Create a deterministic native project scaffold
    New(ProjectNewArgs),
    /// Inspect a native project scaffold and report resolved file layout
    Inspect(ProjectInspectArgs),
    /// Validate persisted native-project structure, UUID consistency, and non-dangling references
    Validate(ProjectValidateArgs),
    /// Import a KiCad .kicad_mod footprint into a project-local native pool
    ImportKicadFootprint(ProjectImportKiCadFootprintArgs),
    /// Import a KiCad .kicad_pcb board into the native board through journaled operations
    ImportKicadBoard(ProjectImportKiCadBoardArgs),
    /// Import KiCad schematic symbols into a native schematic sheet through journaled operations
    ImportKicadSchematic(ProjectImportKiCadSchematicArgs),
    /// Import an Eagle .lbr library into a project-local native pool through journaled operations
    ImportEagleLibrary(ProjectImportEagleLibraryArgs),
    /// Create a raw native pool-library object through the project journal
    CreatePoolLibraryObject(ProjectCreatePoolLibraryObjectArgs),
    /// Create a typed native pool unit through the project journal
    CreatePoolUnit(ProjectCreatePoolUnitArgs),
    /// Set one typed native pool unit pin through the project journal
    SetPoolUnitPin(ProjectSetPoolUnitPinArgs),
    /// Create a typed native pool symbol through the project journal
    CreatePoolSymbol(ProjectCreatePoolSymbolArgs),
    /// Append one typed native pool symbol line primitive through the project journal
    AddPoolSymbolLine(ProjectAddPoolSymbolLineArgs),
    /// Append one typed native pool symbol polygon/polyline primitive through the project journal
    AddPoolSymbolPolygon(ProjectAddPoolSymbolPolygonArgs),
    /// Append one typed native pool symbol rectangle primitive through the project journal
    AddPoolSymbolRect(ProjectAddPoolSymbolRectArgs),
    /// Append one typed native pool symbol circle primitive through the project journal
    AddPoolSymbolCircle(ProjectAddPoolSymbolCircleArgs),
    /// Append one typed native pool symbol arc primitive through the project journal
    AddPoolSymbolArc(ProjectAddPoolSymbolArcArgs),
    /// Append one typed native pool symbol text primitive through the project journal
    AddPoolSymbolText(ProjectAddPoolSymbolTextArgs),
    /// Set one typed native pool symbol pin anchor through the project journal
    SetPoolSymbolPinAnchor(ProjectSetPoolSymbolPinAnchorArgs),
    /// Create a typed native pool entity through the project journal
    CreatePoolEntity(ProjectCreatePoolEntityArgs),
    /// Create a typed native pool padstack through the project journal
    CreatePoolPadstack(ProjectCreatePoolPadstackArgs),
    /// Create a first-class typed native pool footprint through the project journal
    CreatePoolFootprint(ProjectCreatePoolFootprintArgs),
    /// Generate an IPC-7351B two-terminal chip footprint + padstack through the project journal
    GenerateIpc7351bTwoTerminalChip(ProjectGenerateIpc7351bTwoTerminalChipArgs),
    /// Generate an IPC-7351B SOIC footprint + padstack through the project journal
    GenerateIpc7351bSoic(ProjectGenerateIpc7351bSoicArgs),
    /// Set one first-class typed native pool footprint pad through the project journal
    SetPoolFootprintPad(ProjectSetPoolFootprintPadArgs),
    /// Set first-class typed native pool footprint rectangular courtyard through the project journal
    SetPoolFootprintCourtyardRect(ProjectSetPoolFootprintCourtyardRectArgs),
    /// Set first-class typed native pool footprint polygon courtyard through the project journal
    SetPoolFootprintCourtyardPolygon(ProjectSetPoolFootprintCourtyardPolygonArgs),
    /// Append one first-class typed native pool footprint silkscreen line through the project journal
    AddPoolFootprintSilkscreenLine(ProjectAddPoolFootprintSilkscreenLineArgs),
    /// Append one first-class typed native pool footprint silkscreen rectangle through the project journal
    AddPoolFootprintSilkscreenRect(ProjectAddPoolFootprintSilkscreenRectArgs),
    /// Append one first-class typed native pool footprint silkscreen circle through the project journal
    AddPoolFootprintSilkscreenCircle(ProjectAddPoolFootprintSilkscreenCircleArgs),
    /// Append one first-class typed native pool footprint silkscreen polygon/polyline through the project journal
    AddPoolFootprintSilkscreenPolygon(ProjectAddPoolFootprintSilkscreenPolygonArgs),
    /// Create a typed native pool package through the project journal
    CreatePoolPackage(ProjectCreatePoolPackageArgs),
    /// Set one typed native pool package pad through the project journal
    SetPoolPackagePad(ProjectSetPoolPackagePadArgs),
    /// Set typed native pool package rectangular courtyard through the project journal
    SetPoolPackageCourtyardRect(ProjectSetPoolPackageCourtyardRectArgs),
    /// Set typed native pool package polygon courtyard through the project journal
    SetPoolPackageCourtyardPolygon(ProjectSetPoolPackageCourtyardPolygonArgs),
    /// Append one typed native pool package silkscreen line through the project journal
    AddPoolPackageSilkscreenLine(ProjectAddPoolPackageSilkscreenLineArgs),
    /// Append one typed native pool package silkscreen rectangle through the project journal
    AddPoolPackageSilkscreenRect(ProjectAddPoolPackageSilkscreenRectArgs),
    /// Append one typed native pool package silkscreen circle through the project journal
    AddPoolPackageSilkscreenCircle(ProjectAddPoolPackageSilkscreenCircleArgs),
    /// Append one typed native pool package silkscreen arc through the project journal
    AddPoolPackageSilkscreenArc(ProjectAddPoolPackageSilkscreenArcArgs),
    /// Append one typed native pool package silkscreen polygon/polyline through the project journal
    AddPoolPackageSilkscreenPolygon(ProjectAddPoolPackageSilkscreenPolygonArgs),
    /// Append one typed native pool package silkscreen text primitive through the project journal
    AddPoolPackageSilkscreenText(ProjectAddPoolPackageSilkscreenTextArgs),
    /// Append one typed native pool package 3D model reference through the project journal
    #[command(name = "add-pool-package-model-3d")]
    AddPoolPackageModel3d(ProjectAddPoolPackageModel3dArgs),
    /// Set typed native pool package body-height metadata through the project journal
    SetPoolPackageBodyHeights(ProjectSetPoolPackageBodyHeightsArgs),
    /// Create a typed native pool part through the project journal
    CreatePoolPart(ProjectCreatePoolPartArgs),
    /// Update typed native pool part metadata through the project journal
    SetPoolPartMetadata(ProjectSetPoolPartMetadataArgs),
    /// Update typed native pool part default footprint / pin-pad-map bindings through the project journal
    SetPoolPartBindings(ProjectSetPoolPartBindingsArgs),
    /// Set typed native pool part parametric fields through the project journal
    SetPoolPartParametric(ProjectSetPoolPartParametricArgs),
    /// Set typed native pool part orderable MPNs through the project journal
    SetPoolPartOrderableMpns(ProjectSetPoolPartOrderableMpnsArgs),
    /// Set typed native pool part packaging options through the project journal
    SetPoolPartPackagingOptions(ProjectSetPoolPartPackagingOptionsArgs),
    /// Set typed native pool part behavioural model attachments through the project journal
    SetPoolPartBehaviouralModels(ProjectSetPoolPartBehaviouralModelsArgs),
    /// Copy a behavioural model into pool/models and attach it to a typed native pool part
    AttachPoolPartModel(ProjectAttachPoolPartModelArgs),
    /// Detach a behavioural model reference from a typed native pool part
    DetachPoolPartModel(ProjectDetachPoolPartModelArgs),
    /// Garbage-collect orphaned pool model blobs; dry-run unless --apply is supplied
    GcPoolModels(ProjectGcPoolModelsArgs),
    /// Set typed native pool part thermal metadata through the project journal
    SetPoolPartThermal(ProjectSetPoolPartThermalArgs),
    /// Set typed native pool part supply-chain cache metadata through the project journal
    SetPoolPartSupplyChain(ProjectSetPoolPartSupplyChainArgs),
    /// Set typed native pool part tags through the project journal
    SetPoolPartTags(ProjectSetPoolPartTagsArgs),
    /// Set typed native pool part pad-map entries through the project journal
    SetPoolPartPadMap(ProjectSetPoolPartPadMapArgs),
    /// Set one typed native pool part pad-map entry through the project journal
    SetPoolPartPadMapEntry(ProjectSetPoolPartPadMapEntryArgs),
    /// Create a first-class typed native pool PinPadMap through the project journal
    CreatePoolPinPadMap(ProjectCreatePoolPinPadMapArgs),
    /// Update first-class typed native pool PinPadMap mappings through the project journal
    SetPoolPinPadMap(ProjectSetPoolPinPadMapArgs),
    /// Replace a raw native pool-library object through the project journal
    SetPoolLibraryObject(ProjectSetPoolLibraryObjectArgs),
    /// Delete a raw native pool-library object through the project journal
    DeletePoolLibraryObject(ProjectDeletePoolLibraryObjectArgs),
    /// Replace the native project name
    SetProjectName(ProjectSetProjectNameArgs),
    /// Replace the native project rules payload
    SetProjectRules(ProjectSetProjectRulesArgs),
    /// Append one rule to the native project rules root
    CreateProjectRule(ProjectCreateProjectRuleArgs),
    /// Replace one rule in the native project rules root
    SetProjectRule(ProjectSetProjectRuleArgs),
    /// Delete one rule from the native project rules root
    DeleteProjectRule(ProjectDeleteProjectRuleArgs),
    /// Query native project data from the on-disk scaffold
    Query(ProjectQueryArgs),
    /// Show one persisted proposal record
    ShowProposal(ProjectShowProposalArgs),
    /// Validate one persisted proposal against the current model revision
    ValidateProposal(ProjectValidateProposalArgs),
    /// Defer one draft persisted proposal without applying it
    DeferProposal(ProjectDeferProposalArgs),
    /// Review one persisted proposal record without applying it
    ReviewProposal(ProjectReviewProposalArgs),
    /// Apply one accepted persisted proposal through the generic proposal gateway
    ApplyProposal(ProjectApplyProposalArgs),
    /// Generate draft standards repair proposals from the current check run
    GenerateStandardsRepairProposals(ProjectGenerateStandardsRepairProposalsArgs),
    /// Author a fingerprint-scoped waiver through the native project journal
    WaiveFinding(ProjectWaiveFindingArgs),
    /// Accept a fingerprint-scoped check finding as a deviation through the native project journal
    AcceptDeviation(ProjectAcceptDeviationArgs),
    /// Create or reuse a deterministic Gerber-set OutputJob
    CreateGerberOutputJob(ProjectCreateGerberOutputJobArgs),
    /// Create or reuse a deterministic OutputJob for one artifact include scope
    CreateOutputJob(ProjectCreateOutputJobArgs),
    /// Update an authored OutputJob through the substrate journal path
    UpdateOutputJob(ProjectUpdateOutputJobArgs),
    /// Execute an authored OutputJob using its stored production settings
    RunOutputJob(ProjectRunOutputJobArgs),
    /// Persist a running OutputJobRun evidence record for one OutputJob
    StartOutputJobRun(ProjectStartOutputJobRunArgs),
    /// Mark an existing OutputJobRun evidence record canceled
    CancelOutputJobRun(ProjectCancelOutputJobRunArgs),
    /// Delete an authored OutputJob through the substrate journal path
    DeleteOutputJob(ProjectDeleteOutputJobArgs),
    /// Create or reuse a deterministic manufacturing plan
    CreateManufacturingPlan(ProjectCreateManufacturingPlanArgs),
    /// Update a deterministic manufacturing plan through the substrate journal path
    UpdateManufacturingPlan(ProjectUpdateManufacturingPlanArgs),
    /// Delete a deterministic manufacturing plan through the substrate journal path
    DeleteManufacturingPlan(ProjectDeleteManufacturingPlanArgs),
    /// Create or reuse a deterministic panel projection
    CreatePanelProjection(ProjectCreatePanelProjectionArgs),
    /// Update a deterministic panel projection through the substrate journal path
    UpdatePanelProjection(ProjectUpdatePanelProjectionArgs),
    /// Delete a deterministic panel projection through the substrate journal path
    DeletePanelProjection(ProjectDeletePanelProjectionArgs),
    /// Undo the latest journaled native-project transaction by appending a compensating transaction
    Undo(ProjectUndoArgs),
    /// Redo the latest journaled native-project undo by appending a compensating transaction
    Redo(ProjectRedoArgs),
    /// Export a native project BOM as deterministic CSV from persisted board components
    ExportBom(ExportBomArgs),
    /// Compare a BOM CSV against the current native board-component inventory
    CompareBom(CompareBomArgs),
    /// Validate a BOM CSV byte-for-byte against the current deterministic native inventory renderer
    ValidateBom(ValidateBomArgs),
    /// Inspect a BOM CSV using the deterministic native inventory contract
    InspectBom(InspectBomArgs),
    /// Export a native project pick-and-place file as deterministic CSV from persisted board components
    ExportPnp(ExportPnpArgs),
    /// Compare a PnP CSV against the current native board-component inventory
    ComparePnp(ComparePnpArgs),
    /// Validate a PnP CSV byte-for-byte against the current deterministic native inventory renderer
    ValidatePnp(ValidatePnpArgs),
    /// Inspect a pick-and-place CSV using the deterministic native inventory contract
    InspectPnp(InspectPnpArgs),
    /// Export a native project drill file as deterministic CSV from persisted vias
    ExportDrill(ExportDrillArgs),
    /// Validate a native project drill CSV against the current persisted via inventory
    ValidateDrill(ValidateDrillArgs),
    /// Compare a native project drill CSV semantically against the current persisted via inventory
    CompareDrill(CompareDrillArgs),
    /// Export a native project drill file as narrow Excellon from persisted vias
    ExportExcellonDrill(ExportExcellonDrillArgs),
    /// Inspect a native project drill CSV file
    InspectDrill(InspectDrillArgs),
    InspectExcellonDrill(ProjectInspectExcellonDrillArgs),
    InspectGerber(ProjectInspectGerberArgs),
    /// Compare a narrow Excellon drill file against the current native via inventory
    CompareExcellonDrill(CompareExcellonDrillArgs),
    /// Validate a narrow Excellon drill file against the current native via inventory
    ValidateExcellonDrill(ValidateExcellonDrillArgs),
    ReportDrillHoleClasses(ReportDrillHoleClassesArgs),
    /// Export the native board outline as a narrow RS-274X Gerber file
    ExportGerberOutline(ProjectExportGerberOutlineArgs),
    /// Export one native board copper layer as a narrow RS-274X Gerber file
    ExportGerberCopperLayer(ProjectExportGerberCopperLayerArgs),
    /// Export one native board soldermask layer as a narrow RS-274X Gerber file
    ExportGerberSoldermaskLayer(ProjectExportGerberSoldermaskLayerArgs),
    /// Export one native board silkscreen layer as a narrow RS-274X Gerber file
    ExportGerberSilkscreenLayer(ProjectExportGerberSilkscreenLayerArgs),
    /// Export one native board paste layer as a narrow RS-274X Gerber file
    ExportGerberPasteLayer(ProjectExportGerberPasteLayerArgs),
    /// Export one native board mechanical layer as a narrow RS-274X Gerber file
    ExportGerberMechanicalLayer(ProjectExportGerberMechanicalLayerArgs),
    /// Validate a narrow RS-274X board-outline Gerber against the current native board outline
    ValidateGerberOutline(ProjectValidateGerberOutlineArgs),
    /// Validate a narrow RS-274X copper-layer Gerber against the current native board tracks on one layer
    ValidateGerberCopperLayer(ProjectValidateGerberCopperLayerArgs),
    /// Validate a narrow RS-274X soldermask Gerber against the current native board pads on one source copper layer
    ValidateGerberSoldermaskLayer(ProjectValidateGerberSoldermaskLayerArgs),
    /// Validate a narrow RS-274X silkscreen Gerber against the current native silkscreen geometry on one layer
    ValidateGerberSilkscreenLayer(ProjectValidateGerberSilkscreenLayerArgs),
    /// Validate a narrow RS-274X paste Gerber against the current native board pads on one source copper layer
    ValidateGerberPasteLayer(ProjectValidateGerberPasteLayerArgs),
    /// Validate a narrow RS-274X mechanical-layer Gerber against the current native package mechanical geometry on one layer
    ValidateGerberMechanicalLayer(ProjectValidateGerberMechanicalLayerArgs),
    /// Compare a narrow RS-274X board-outline Gerber against the current native board outline semantics
    CompareGerberOutline(ProjectCompareGerberOutlineArgs),
    /// Compare a narrow RS-274X copper-layer Gerber against the current native board copper semantics on one layer
    CompareGerberCopperLayer(ProjectCompareGerberCopperLayerArgs),
    /// Compare a narrow RS-274X soldermask Gerber against the current native board mask semantics on one layer
    CompareGerberSoldermaskLayer(ProjectCompareGerberSoldermaskLayerArgs),
    /// Compare a narrow RS-274X silkscreen Gerber against the current native silkscreen semantics on one layer
    CompareGerberSilkscreenLayer(ProjectCompareGerberSilkscreenLayerArgs),
    /// Compare a narrow RS-274X paste Gerber against the current native board paste semantics on one layer
    CompareGerberPasteLayer(ProjectCompareGerberPasteLayerArgs),
    /// Compare a narrow RS-274X mechanical-layer Gerber against the current native package mechanical semantics on one layer
    CompareGerberMechanicalLayer(ProjectCompareGerberMechanicalLayerArgs),
    /// Generate the deterministic Gerber export plan for the current native project
    PlanGerberExport(PlanGerberExportArgs),
    /// Export the deterministic Gerber set for the current native project
    ExportGerberSet(ExportGerberSetArgs),
    /// Validate a generated Gerber set against the current deterministic native renderer
    ValidateGerberSet(ValidateGerberSetArgs),
    /// Compare a generated Gerber set artifact semantically against the current deterministic native renderer
    CompareGerberSet(CompareGerberSetArgs),
    /// Compare two Gerber export plans after deterministic normalization
    CompareGerberExportPlan(CompareGerberExportPlanArgs),
    /// Report the current manufacturing set derived from persisted native board state
    ReportManufacturing(ReportManufacturingArgs),
    /// Export the current manufacturing set derived from persisted native board state
    ExportManufacturingSet(ExportManufacturingSetArgs),
    /// Inspect a manufacturing artifact set using the deterministic native renderer contracts
    InspectManufacturingSet(InspectManufacturingSetArgs),
    /// Validate a manufacturing artifact set against the current deterministic native renderer
    ValidateManufacturingSet(ValidateManufacturingSetArgs),
    /// Compare a manufacturing artifact set semantically against the current deterministic native renderer
    CompareManufacturingSet(CompareManufacturingSetArgs),
    /// Emit a deterministic manufacturing manifest from current persisted native state
    ManifestManufacturingSet(ManifestManufacturingSetArgs),
    /// Export the current forward-annotation audit from persisted native schematic+board state
    ExportForwardAnnotationAudit(ProjectExportForwardAnnotationAuditArgs),
    /// Query the current forward-annotation audit from persisted native schematic+board state
    ForwardAnnotationAudit(ProjectForwardAnnotationAuditArgs),
    /// Apply one supported forward-annotation proposal action by stable action ID
    ApplyForwardAnnotationAction(ProjectApplyForwardAnnotationActionArgs),
    /// Apply all currently self-sufficient forward-annotation proposal actions while honoring persisted defer/reject review state
    ApplyForwardAnnotationReviewed(ProjectApplyForwardAnnotationReviewedArgs),
    /// Select one deterministic current route proposal from the accepted candidate family order
    RouteProposal(ProjectRouteProposalArgs),
    /// Report which accepted selector profile should be used for one deterministic routing objective
    RouteStrategyReport(ProjectRouteStrategyReportArgs),
    /// Compare the currently accepted deterministic routing objectives/profiles and recommend one choice
    RouteStrategyCompare(ProjectRouteStrategyCompareArgs),
    /// Report the bounded decision delta between the currently accepted routing objectives/profiles
    RouteStrategyDelta(ProjectRouteStrategyDeltaArgs),
    /// Write one deterministic curated fixture suite plus batch request manifest for repeated route-strategy evidence runs
    WriteRouteStrategyCuratedFixtureSuite(ProjectWriteRouteStrategyCuratedFixtureSuiteArgs),
    /// Materialize the curated fixture suite, evaluate it, and save one reusable batch-result baseline artifact
    CaptureRouteStrategyCuratedBaseline(ProjectCaptureRouteStrategyCuratedBaselineArgs),
    /// Evaluate the current accepted M6 strategy surfaces across a versioned batch request manifest
    RouteStrategyBatchEvaluate(ProjectRouteStrategyBatchEvaluateArgs),
    /// Inspect one saved versioned route-strategy batch result artifact
    InspectRouteStrategyBatchResult(ProjectInspectRouteStrategyBatchResultArgs),
    /// Validate one saved versioned route-strategy batch result artifact for structural and count integrity
    ValidateRouteStrategyBatchResult(ProjectValidateRouteStrategyBatchResultArgs),
    /// Compare two saved versioned route-strategy batch result artifacts by request_id and aggregate summary
    CompareRouteStrategyBatchResult(ProjectCompareRouteStrategyBatchResultArgs),
    /// Evaluate two saved versioned route-strategy batch result artifacts against one deterministic CI gate policy
    GateRouteStrategyBatchResult(ProjectGateRouteStrategyBatchResultArgs),
    /// Summarize many saved versioned route-strategy batch result artifacts from one directory or explicit list
    SummarizeRouteStrategyBatchResults(ProjectSummarizeRouteStrategyBatchResultsArgs),
    /// Review one selected deterministic route proposal or one saved route proposal artifact without mutating project state
    ReviewRouteProposal(ProjectReviewRouteProposalArgs),
    /// Explain how the deterministic current route proposal selector chose or rejected candidate families
    RouteProposalExplain(ProjectRouteProposalExplainArgs),
    /// Export one deterministic route proposal artifact from the currently selected candidate family
    ExportRouteProposal(ProjectExportRouteProposalArgs),
    /// Export one deterministic route proposal artifact from one accepted current route-path candidate family
    ExportRoutePathProposal(ProjectExportRoutePathProposalArgs),
    /// Inspect a versioned route proposal artifact
    InspectRouteProposalArtifact(ProjectInspectRouteProposalArtifactArgs),
    /// Revalidate a versioned route proposal artifact against the current live project state without applying it
    RevalidateRouteProposalArtifact(ProjectRevalidateRouteProposalArtifactArgs),
    /// Apply a versioned route proposal artifact when it still matches the current live project state
    ApplyRouteProposalArtifact(ProjectApplyRouteProposalArtifactArgs),
    /// Apply the currently selected deterministic route proposal through the proposal journal gateway
    RouteApplySelected(ProjectRouteApplySelectedArgs),
    /// Apply one deterministic current route candidate through the proposal journal gateway
    RouteApply(ProjectRouteApplyArgs),
    /// Export the current forward-annotation proposal and review state as a versioned artifact
    ExportForwardAnnotationProposal(ProjectExportForwardAnnotationProposalArgs),
    /// Export a selected subset of the current forward-annotation proposal and matching review state as a versioned artifact
    ExportForwardAnnotationProposalSelection(ProjectExportForwardAnnotationProposalSelectionArgs),
    /// Select a subset of actions from an existing forward-annotation proposal artifact
    SelectForwardAnnotationProposalArtifact(ProjectSelectForwardAnnotationProposalArtifactArgs),
    /// Inspect a versioned forward-annotation proposal artifact
    InspectForwardAnnotationProposalArtifact(ProjectInspectForwardAnnotationProposalArtifactArgs),
    /// Validate a versioned forward-annotation proposal artifact against the supported canonical artifact encoding
    ValidateForwardAnnotationProposalArtifact(ProjectValidateForwardAnnotationProposalArtifactArgs),
    /// Compare a forward-annotation proposal artifact against the current live project proposal state
    CompareForwardAnnotationProposalArtifact(ProjectCompareForwardAnnotationProposalArtifactArgs),
    /// Filter a forward-annotation proposal artifact down to actions still applicable against the current live project proposal
    FilterForwardAnnotationProposalArtifact(ProjectFilterForwardAnnotationProposalArtifactArgs),
    /// Plan artifact-driven forward-annotation apply without mutating project state
    PlanForwardAnnotationProposalArtifactApply(
        ProjectPlanForwardAnnotationProposalArtifactApplyArgs,
    ),
    /// Apply one filtered forward-annotation proposal artifact when all retained actions are still self-sufficient
    ApplyForwardAnnotationProposalArtifact(ProjectApplyForwardAnnotationProposalArtifactArgs),
    /// Import forward-annotation review decisions from an artifact into the current live project state
    ImportForwardAnnotationArtifactReview(ProjectImportForwardAnnotationArtifactReviewArgs),
    /// Replace live forward-annotation review state with validated review decisions from an artifact
    ReplaceForwardAnnotationArtifactReview(ProjectReplaceForwardAnnotationArtifactReviewArgs),
    /// Mark one forward-annotation proposal action as deferred by stable action ID
    DeferForwardAnnotationAction(ProjectDeferForwardAnnotationActionArgs),
    /// Mark one forward-annotation proposal action as rejected by stable action ID
    RejectForwardAnnotationAction(ProjectRejectForwardAnnotationActionArgs),
    /// Clear one persisted forward-annotation review decision by stable action ID
    ClearForwardAnnotationActionReview(ProjectClearForwardAnnotationActionReviewArgs),
    /// Create one native schematic sheet through the substrate journal
    CreateSheet(ProjectCreateSheetArgs),
    /// Delete one native schematic sheet through the substrate journal
    DeleteSheet(ProjectDeleteSheetArgs),
    /// Rename one native schematic sheet through the substrate journal
    RenameSheet(ProjectRenameSheetArgs),
    /// Create one native schematic sheet definition through the substrate journal
    CreateSheetDefinition(ProjectCreateSheetDefinitionArgs),
    /// Create one native schematic sheet instance through the substrate journal
    CreateSheetInstance(ProjectCreateSheetInstanceArgs),
    /// Delete one native schematic sheet instance through the substrate journal
    DeleteSheetInstance(ProjectDeleteSheetInstanceArgs),
    /// Move one native schematic sheet instance through the substrate journal
    MoveSheetInstance(ProjectMoveSheetInstanceArgs),
    /// Bind a parent-sheet hierarchical port to a sheet instance
    BindSheetInstancePort(ProjectBindSheetInstancePortArgs),
    /// Remove a parent-sheet hierarchical port binding from a sheet instance
    UnbindSheetInstancePort(ProjectUnbindSheetInstancePortArgs),
    /// Place one schematic label into an existing native sheet file
    PlaceLabel(ProjectPlaceLabelArgs),
    /// Rename one schematic label in a native sheet file
    RenameLabel(ProjectRenameLabelArgs),
    /// Delete one schematic label from a native sheet file
    DeleteLabel(ProjectDeleteLabelArgs),
    /// Draw one schematic wire into an existing native sheet file
    DrawWire(ProjectDrawWireArgs),
    /// Delete one schematic wire from a native sheet file
    DeleteWire(ProjectDeleteWireArgs),
    /// Place one schematic junction into an existing native sheet file
    PlaceJunction(ProjectPlaceJunctionArgs),
    /// Delete one schematic junction from a native sheet file
    DeleteJunction(ProjectDeleteJunctionArgs),
    /// Place one hierarchical port into an existing native sheet file
    PlacePort(ProjectPlacePortArgs),
    /// Edit one hierarchical port in a native sheet file
    EditPort(ProjectEditPortArgs),
    /// Delete one hierarchical port from a native sheet file
    DeletePort(ProjectDeletePortArgs),
    /// Create one bus in an existing native sheet file
    CreateBus(ProjectCreateBusArgs),
    /// Edit one bus member list in a native sheet file
    EditBusMembers(ProjectEditBusMembersArgs),
    /// Delete one bus from a native sheet file
    DeleteBus(ProjectDeleteBusArgs),
    /// Place one bus entry in an existing native sheet file
    PlaceBusEntry(ProjectPlaceBusEntryArgs),
    /// Delete one bus entry from a native sheet file
    DeleteBusEntry(ProjectDeleteBusEntryArgs),
    /// Place one no-connect marker into an existing native sheet file
    #[command(name = "place-noconnect")]
    PlaceNoConnect(ProjectPlaceNoConnectArgs),
    /// Delete one no-connect marker from a native sheet file
    #[command(name = "delete-noconnect")]
    DeleteNoConnect(ProjectDeleteNoConnectArgs),
    /// Place one schematic symbol into an existing native sheet file
    PlaceSymbol(ProjectPlaceSymbolArgs),
    /// Delete one native schematic symbol
    DeleteSymbol(ProjectDeleteSymbolArgs),
    /// Move one native schematic symbol
    MoveSymbol(ProjectMoveSymbolArgs),
    /// Rotate one native schematic symbol
    RotateSymbol(ProjectRotateSymbolArgs),
    /// Mirror one native schematic symbol
    MirrorSymbol(ProjectMirrorSymbolArgs),
    /// Set one native schematic symbol reference
    SetSymbolReference(ProjectSetSymbolReferenceArgs),
    /// Set one native schematic symbol value
    SetSymbolValue(ProjectSetSymbolValueArgs),
    /// Set one native schematic symbol display mode
    SetSymbolDisplayMode(ProjectSetSymbolDisplayModeArgs),
    /// Set one native schematic symbol hidden-power behavior
    SetSymbolHiddenPowerBehavior(ProjectSetSymbolHiddenPowerBehaviorArgs),
    /// Set one native schematic symbol unit
    SetSymbolUnit(ProjectSetSymbolUnitArgs),
    /// Clear one native schematic symbol unit
    ClearSymbolUnit(ProjectClearSymbolUnitArgs),
    /// Set one native schematic symbol gate
    SetSymbolGate(ProjectSetSymbolGateArgs),
    /// Clear one native schematic symbol gate
    ClearSymbolGate(ProjectClearSymbolGateArgs),
    /// Set one native schematic symbol entity UUID
    SetSymbolEntity(ProjectSetSymbolEntityArgs),
    /// Clear one native schematic symbol entity UUID
    ClearSymbolEntity(ProjectClearSymbolEntityArgs),
    /// Set one native schematic symbol part UUID
    SetSymbolPart(ProjectSetSymbolPartArgs),
    /// Clear one native schematic symbol part UUID
    ClearSymbolPart(ProjectClearSymbolPartArgs),
    /// Set one native schematic symbol lib_id
    SetSymbolLibId(ProjectSetSymbolLibIdArgs),
    /// Clear one native schematic symbol lib_id
    ClearSymbolLibId(ProjectClearSymbolLibIdArgs),
    /// Set one per-pin display override in a native schematic symbol
    SetPinOverride(ProjectSetPinOverrideArgs),
    /// Clear one per-pin display override in a native schematic symbol
    ClearPinOverride(ProjectClearPinOverrideArgs),
    /// Add one native schematic symbol field
    AddSymbolField(ProjectAddSymbolFieldArgs),
    /// Edit one native schematic symbol field
    EditSymbolField(ProjectEditSymbolFieldArgs),
    /// Delete one native schematic symbol field
    DeleteSymbolField(ProjectDeleteSymbolFieldArgs),
    /// Place one native schematic text object
    PlaceText(ProjectPlaceTextArgs),
    /// Edit one native schematic text object
    EditText(ProjectEditTextArgs),
    /// Delete one native schematic text object
    DeleteText(ProjectDeleteTextArgs),
    /// Place one native schematic drawing line
    PlaceDrawingLine(ProjectPlaceDrawingLineArgs),
    /// Place one schematic drawing rectangle into an existing native sheet file
    PlaceDrawingRect(ProjectPlaceDrawingRectArgs),
    /// Place one schematic drawing circle into an existing native sheet file
    PlaceDrawingCircle(ProjectPlaceDrawingCircleArgs),
    /// Place one schematic drawing arc into an existing native sheet file
    PlaceDrawingArc(ProjectPlaceDrawingArcArgs),
    /// Edit one schematic drawing line in a native sheet file
    EditDrawingLine(ProjectEditDrawingLineArgs),
    /// Edit one schematic drawing rectangle in a native sheet file
    EditDrawingRect(ProjectEditDrawingRectArgs),
    /// Edit one schematic drawing circle in a native sheet file
    EditDrawingCircle(ProjectEditDrawingCircleArgs),
    /// Edit one schematic drawing arc in a native sheet file
    EditDrawingArc(ProjectEditDrawingArcArgs),
    /// Delete one native schematic drawing primitive
    DeleteDrawing(ProjectDeleteDrawingArgs),
    /// Place one board text object into the native board file
    PlaceBoardText(ProjectPlaceBoardTextArgs),
    /// Edit one native board text object
    EditBoardText(ProjectEditBoardTextArgs),
    /// Delete one native board text object
    DeleteBoardText(ProjectDeleteBoardTextArgs),
    /// Place one native board keepout polygon
    PlaceBoardKeepout(ProjectPlaceBoardKeepoutArgs),
    /// Edit one native board keepout polygon
    EditBoardKeepout(ProjectEditBoardKeepoutArgs),
    /// Delete one native board keepout polygon
    DeleteBoardKeepout(ProjectDeleteBoardKeepoutArgs),
    /// Replace the native board outline polygon
    SetBoardOutline(ProjectSetBoardOutlineArgs),
    /// Replace the native board stackup
    SetBoardStackup(ProjectSetBoardStackupArgs),
    /// Replace the native board name
    SetBoardName(ProjectSetBoardNameArgs),
    /// Add a default two-layer stackup to the native board
    AddDefaultTopStackup(ProjectAddDefaultTopStackupArgs),
    /// Place one native board net
    PlaceBoardNet(ProjectPlaceBoardNetArgs),
    /// Edit one native board net
    EditBoardNet(ProjectEditBoardNetArgs),
    /// Delete one native board net
    DeleteBoardNet(ProjectDeleteBoardNetArgs),
    /// Draw one native board track
    DrawBoardTrack(ProjectDrawBoardTrackArgs),
    /// Edit one native board track
    EditBoardTrack(ProjectEditBoardTrackArgs),
    /// Delete one native board track
    DeleteBoardTrack(ProjectDeleteBoardTrackArgs),
    /// Place one native board via
    PlaceBoardVia(ProjectPlaceBoardViaArgs),
    /// Edit one native board via
    EditBoardVia(ProjectEditBoardViaArgs),
    /// Delete one native board via
    DeleteBoardVia(ProjectDeleteBoardViaArgs),
    /// Place one native board zone
    PlaceBoardZone(ProjectPlaceBoardZoneArgs),
    /// Edit one native board zone
    EditBoardZone(ProjectEditBoardZoneArgs),
    /// Attempt to fill native board zones as generated evidence without mutating authored board state
    FillZones(ProjectFillZonesArgs),
    /// Delete one native board zone
    DeleteBoardZone(ProjectDeleteBoardZoneArgs),
    /// Set one native board pad net assignment
    SetBoardPadNet(ProjectSetBoardPadNetArgs),
    /// Clear one native board pad net assignment
    ClearBoardPadNet(ProjectClearBoardPadNetArgs),
    /// Edit one native board pad position and/or layer
    EditBoardPad(ProjectEditBoardPadArgs),
    /// Place one native board pad
    PlaceBoardPad(ProjectPlaceBoardPadArgs),
    /// Delete one native board pad
    DeleteBoardPad(ProjectDeleteBoardPadArgs),
    /// Place one native board component/package
    PlaceBoardComponent(ProjectPlaceBoardComponentArgs),
    /// Generate initial native board components from schematic symbols and part package bindings
    GenerateBoardComponents(ProjectGenerateBoardComponentsArgs),
    /// Move one native board component/package
    MoveBoardComponent(ProjectMoveBoardComponentArgs),
    /// Rotate one native board component/package
    RotateBoardComponent(ProjectRotateBoardComponentArgs),
    /// Delete one native board component
    DeleteBoardComponent(ProjectDeleteBoardComponentArgs),
    /// Bind one schematic symbol to one board package as a ComponentInstance
    BindComponentInstance(ProjectBindComponentInstanceArgs),
    /// Replace the refs for one authored ComponentInstance
    SetComponentInstance(ProjectSetComponentInstanceArgs),
    /// Delete one authored ComponentInstance
    DeleteComponentInstance(ProjectDeleteComponentInstanceArgs),
    /// Lock one native board component/package
    SetBoardComponentLocked(ProjectSetBoardComponentLockedArgs),
    /// Clear the locked state on one native board component/package
    ClearBoardComponentLocked(ProjectClearBoardComponentLockedArgs),
    /// Set one native board component reference
    SetBoardComponentReference(SetBoardComponentReferenceArgs),
    /// Set one native board component value
    SetBoardComponentValue(SetBoardComponentValueArgs),
    /// Set one native board component part UUID
    SetBoardComponentPart(SetBoardComponentPartArgs),
    /// Set one native board component package UUID
    SetBoardComponentPackage(SetBoardComponentPackageArgs),
    /// Set one native board component copper side/layer
    SetBoardComponentLayer(SetBoardComponentLayerArgs),
    /// Flip one native board component to a target copper side/layer
    FlipBoardComponent(SetBoardComponentLayerArgs),
    /// Place one native board net class
    PlaceBoardNetClass(ProjectPlaceBoardNetClassArgs),
    /// Edit one native board net class
    EditBoardNetClass(ProjectEditBoardNetClassArgs),
    /// Delete one native board net class
    DeleteBoardNetClass(ProjectDeleteBoardNetClassArgs),
    /// Place one native board dimension
    PlaceBoardDimension(PlaceBoardDimensionArgs),
    /// Edit one native board dimension
    EditBoardDimension(EditBoardDimensionArgs),
    /// Delete one native board dimension
    DeleteBoardDimension(ProjectDeleteBoardDimensionArgs),
}
