//! Shared workspace tool and gesture state.
use crate::{PaneContent, PaneId, PointNm, ReviewWorkspaceState};
#[path = "workspace_tools/feedback.rs"]
mod feedback;
pub use feedback::EditorTool;
#[path = "coordinate_entry.rs"]
pub mod coordinate_entry;
pub use coordinate_entry::{CoordinateEntry, CoordinateEntryMode};
#[path = "move_handle_feedback.rs"]
mod move_handle_feedback;
pub use move_handle_feedback::{MoveHandleFeedback, MoveHandleKind};
#[path = "board_routing.rs"]
pub mod board_routing;
#[path = "coordinate_entry_form.rs"]
mod coordinate_entry_form;
pub use coordinate_entry_form::{CoordinateEntryControl, CoordinateEntryForm};

/// One shared consumer token for native authoring previews. It grants no write
/// authority; the canonical client still checks its own immutable intent basis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringViewBasis {
    pub project: uuid::Uuid,
    pub revision: String,
    pub pane: PaneId,
    pub content: PaneContent,
    pub source: uuid::Uuid,
    pub document: Option<crate::schematic_documents::SchematicDocumentKey>,
}

impl AuthoringViewBasis {
    pub fn capture(state: &ReviewWorkspaceState, pane: PaneId) -> Result<Self, String> {
        let basis = Self::capture_root_document(state, pane)?;
        if basis.content == PaneContent::Schematic
            && state.schematic_authoring.sheet != Some(basis.source)
        {
            return Err(
                "This document is read-only; native authoring requires its supported root view"
                    .into(),
            );
        }
        Ok(basis)
    }

    /// Project-wide operations use a qualified root view without selecting a new
    /// sheet-local editing target. Their engine-owned census supplies write scope.
    pub(crate) fn capture_root_document(
        state: &ReviewWorkspaceState,
        pane: PaneId,
    ) -> Result<Self, String> {
        Self::capture_document(state, pane, false)
    }
    /// Explicit unique-occurrence basis; callers retain their tool admission limits.
    pub fn capture_qualified_schematic(
        state: &ReviewWorkspaceState,
        pane: PaneId,
    ) -> Result<Self, String> {
        let basis = Self::capture_document(state, pane, true)?;
        if basis.content != PaneContent::Schematic
            || state
                .schematic_documents
                .rows
                .iter()
                .filter(|row| row.key.source_sheet == basis.source)
                .count()
                != 1
        {
            return Err("Choose one unrepeated schematic parent occurrence".into());
        }
        Ok(basis)
    }
    /// Sheet-local root tools can address any unique root without granting child editing.
    pub fn capture_unique_root_schematic(
        state: &ReviewWorkspaceState,
        pane: PaneId,
    ) -> Result<Self, String> {
        let basis = Self::capture_qualified_schematic(state, pane)?;
        if basis
            .document
            .as_ref()
            .is_none_or(|document| !document.instance_path.is_empty())
        {
            return Err("This tool requires an unrepeated root schematic sheet".into());
        }
        Ok(basis)
    }
    fn capture_document(
        state: &ReviewWorkspaceState,
        pane: PaneId,
        qualified: bool,
    ) -> Result<Self, String> {
        if state.project_unopened || state.ui.layout.focused != pane {
            return Err("Focus the supported native source pane before authoring".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native source basis unavailable")?;
        if state.scene.project_uuid != model.project.project_id.to_string()
            || state.scene.source_revision != model.model_revision.0
        {
            return Err("Native source view is stale".into());
        }
        let content = state
            .ui
            .layout
            .content_for(pane)
            .ok_or("Source pane disappeared")?;
        let (source, document) = match content {
            PaneContent::Board => (
                state
                    .scene
                    .board_uuid
                    .parse()
                    .map_err(|_| "Board source unavailable")?,
                None,
            ),
            PaneContent::Schematic => {
                let document = state
                    .schematic_documents
                    .panes
                    .get(&pane)
                    .ok_or("Choose a qualified schematic document before authoring")?;
                if state.schematic_documents.project != Some(model.project.project_id)
                    || state
                        .schematic_documents
                        .revision
                        .as_ref()
                        .is_none_or(|r| r.0 != model.model_revision.0)
                    || state.schematic_documents.row(document).is_none()
                    || document.project != model.project.project_id
                    || (!qualified && !document.instance_path.is_empty())
                    || state
                        .schematic_authoring
                        .source
                        .as_ref()
                        .is_none_or(|s| s.uuid != document.schematic)
                    || crate::camera_scene_for_pane(state, pane).is_none()
                {
                    return Err("This document is read-only; native authoring requires its supported root view".into());
                }
                (document.source_sheet, Some(document.clone()))
            }
            _ => return Err("This pane has no native authoring target".into()),
        };
        Ok(Self {
            project: model.project.project_id,
            revision: model.model_revision.0.clone(),
            pane,
            content,
            source,
            document,
        })
    }

    pub fn is_current(&self, state: &ReviewWorkspaceState, actual_pane: PaneId) -> bool {
        Self::capture(state, actual_pane).is_ok_and(|current| current == *self)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTool {
    Select,
    DrawBoardTrack,
    PlaceBoardVia,
    PlaceBoardText,
    Move,
    Delete,
}

impl WorkspaceTool {
    pub fn label(self) -> &'static str {
        match self {
            WorkspaceTool::Select => "select",
            WorkspaceTool::DrawBoardTrack => "draw-board-track",
            WorkspaceTool::PlaceBoardVia => "place-board-via",
            WorkspaceTool::PlaceBoardText => "place-board-text",
            WorkspaceTool::Move => "move",
            WorkspaceTool::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthoringSnapState {
    pub enabled: bool,
    pub grid_nm: i64,
}

impl Default for AuthoringSnapState {
    fn default() -> Self {
        Self {
            enabled: true,
            grid_nm: 100_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringGestureState {
    pub preset: Option<crate::schematic_alignment::MenuActionPreset>,
    pub tool: WorkspaceTool,
    pub anchor: Option<PointNm>,
    pub preview: Option<PointNm>,
    pub target_object_id: Option<String>,
    pub view_basis: Option<AuthoringViewBasis>,
}

impl AuthoringGestureState {
    pub fn idle(tool: WorkspaceTool) -> Self {
        Self {
            preset: None,
            tool,
            anchor: None,
            preview: None,
            target_object_id: None,
            view_basis: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some() || self.preview.is_some() || self.target_object_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringToolState {
    pub board_route: Option<board_routing::BoardRouteDraft>,
    pub handle_feedback: Option<MoveHandleFeedback>,
    pub coordinates: Option<CoordinateEntryForm>,
    pub snap: AuthoringSnapState,
    pub gesture: AuthoringGestureState,
}
