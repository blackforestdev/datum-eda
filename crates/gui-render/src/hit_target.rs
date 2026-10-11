//! Shared typed pointer targets; source acquisition stays with the consumer.
use crate::RectPx;
use datum_gui_protocol::WorkspaceTool;
pub use datum_gui_viewport::CameraState;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitTarget {
    Library(datum_gui_protocol::library_browser::LibraryControl),
    SchematicPicker,
    SchematicLibraryPage(bool),
    SchematicLibraryChoice(usize),
    SchematicReference,
    SchematicValue,
    SchematicPlace,
    SchematicCancel,
    SchematicFinishWire,
    NewProjectModal,
    NewProjectName,
    NewProjectDestination,
    NewProjectUnitsChoice(datum_gui_protocol::NewProjectUnitsChoice),
    NewProjectUnitsSummary,
    NewProjectRetryGlobal,
    NewProjectCancel,
    NewProjectCreate,
    GlobalPreferencesModal,
    GlobalPreferencesSection(String),
    GlobalPreferencesSearch,
    GlobalPreferencesSettingName(String),
    GlobalPreferencesControl(String),
    GlobalPreferencesChoice {
        key: String,
        value: String,
    },
    GlobalPreferencesReset(String),
    GlobalPreferencesExplanationClose,
    CloseRevisionSurface,
    ToggleRevisionIssuanceArm,
    OpenRevisionWitness(String),
    ProposalAction(String),
    ReviewAction(String),
    AuthoredObject(String),
    SelectionScope(Option<datum_gui_protocol::AuthoredSelectionClass>),
    SelectionInventoryToggle,
    SelectionFiltersToggle,
    SelectionClassToggle(datum_gui_protocol::AuthoredSelectionClass),
    SelectionInspectorScroll {
        total: usize,
        visible: usize,
    },
    FitBoard,
    FitReviewTarget,
    SetWorkspaceTool(WorkspaceTool),
    ReviewPrev,
    ReviewNext,
    ToggleShowAuthored,
    ToggleShowProposed,
    ToggleShowUnrouted,
    ToggleDimUnrelated,
    ToggleLayer(String),
    LayerScrollRegion,
    ToggleSelectedBoardTextMirrored,
    ToggleSelectedBoardTextKeepUpright,
    ToggleSelectedBoardTextBold,
    CycleSelectedBoardTextRenderIntent,
    CycleSelectedBoardTextFamily,
    CycleSelectedBoardTextHAlign,
    CycleSelectedBoardTextVAlign,
    EditSelectedBoardTextRenderIntent,
    EditSelectedBoardTextFamily,
    EditSelectedBoardTextAlignment,
    DecreaseSelectedBoardTextHeight,
    IncreaseSelectedBoardTextHeight,
    RotateSelectedBoardTextCounterClockwise90,
    RotateSelectedBoardTextClockwise90,
    DecreaseSelectedBoardTextLineSpacing,
    IncreaseSelectedBoardTextLineSpacing,
    EditSelectedBoardTextContent,
    EditSelectedBoardTextHeight,
    EditSelectedBoardTextRotation,
    EditSelectedBoardTextLineSpacing,
    TerminalTab,
    TerminalSessionTab(String),
    TerminalSessionClose(String),
    TerminalSessionNew,
    TerminalSessionTerminateActive,
    TerminalSessionForceKillActive,
    TerminalSessionRetryTermination,
    TerminalShutdownCancel,
    TerminalClipboardCopy,
    TerminalClipboardPaste,
    TerminalProfileNext,
    TerminalThemeNext,
    TerminalLinkCopy,
    TerminalLinkOpen,
    TerminalLinkConfirmOpen,
    TerminalLinkCancel,
    TerminalClipboardConfirmWrite,
    TerminalClipboardCancelWrite,
    /// The exact visible terminal cell rectangle (T0-C02): clicking SHELL
    /// CONTENT is deliberate terminal keyboard entry. Cell coordinates for a
    /// click are derived from the same shared geometry that produced the
    /// region rect (`datum_gui_viewport::terminal_screen_geometry`).
    TerminalScreen,
    /// An inactive visible split leaf. Clicking it promotes that exact owned
    /// session to the tab's focused input/render authority.
    TerminalPaneScreen(String),
    TerminalSplitDivider(Vec<datum_gui_protocol::TerminalSplitChild>),
    CheckFinding(String),
    ProductionArtifact(String),
    ProductionArtifactFile(String),
    ProductionOutputJobRun(datum_gui_protocol::TerminalCommandHandoff),
    ProductionTerminalCommand(datum_gui_protocol::TerminalCommandHandoff),
    ArtifactPreviewZoomIn,
    ArtifactPreviewZoomOut,
    ArtifactPreviewReset,
    ArtifactPreviewViewport,
    ToggleArtifactPreviewGeometry,
    ToggleArtifactPreviewDrills,
    ConsoleHistoryToggle,
    ConsoleHistoryFilter(datum_gui_protocol::ConsoleHistoryFilter),
    MenuTitle(String),
    MenuItem {
        menu: String,
        label: String,
    },
    MarkingMenuItem {
        menu_key: String,
        slot: String,
        label: String,
    },
    DockResizeHandle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitRegion {
    pub target: HitTarget,
    pub rect: RectPx,
}
/// Which pane's scene a resolved screen point / world hit belongs to (UVT-004,
/// the CoordinateHit keystone). `world_point_at_screen` resolves the FOCUSED-vs-
/// containing pane and reports the surface so callers route the follow-up world
/// hit-test to the matching retained scene (the board `RetainedScene` vs the
/// companion schematic one) in that surface's own camera/space. Board is the
/// only interactive surface pre-S3; Schematic is the new one this slice unblocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneSurface {
    Board,
    Schematic,
}
