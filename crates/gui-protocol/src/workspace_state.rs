//! Shared consumer workspace state; per-editor projections remain configured data.
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewWorkspaceState {
    pub library: library_browser::LibraryBrowser,
    pub schematic_authoring: schematic_authoring::SchematicAuthoring,
    pub scene: BoardReviewSceneV1,
    /// Companion projection; never a separate design authority.
    pub schematic_scene: Option<BoardReviewSceneV1>,
    pub review: RouteProposalReviewPayload,
    pub production: ProductionStatus,
    pub source_shards: SourceShardStatusSummary,
    pub checks: CheckRunReviewState,
    pub supervision: GuiSupervisionSnapshot,
    pub selection: SelectionTarget,
    pub active_review_target_id: String,
    pub tool: WorkspaceTool,
    pub authoring: AuthoringToolState,
    pub backing: Option<WorkspaceBacking>,
    pub last_command_status: Option<EditorCommandStatus>,
    pub ui: WorkspaceUiState,
}
