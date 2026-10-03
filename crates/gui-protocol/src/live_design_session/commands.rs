//! Shared workspace command/event vocabulary; no private editor dispatch.
use crate::{PointNm, SelectionTarget, WorkspaceTool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    SetTool(WorkspaceTool),
    BeginAuthoringGesture {
        world: PointNm,
        target_object_id: Option<String>,
    },
    PreviewAuthoringGesture {
        world: PointNm,
        target_object_id: Option<String>,
    },
    CancelAuthoringGesture,
    SelectReviewAction(String),
    SelectAuthoredObject(String),
    SelectCheckFinding(String),
    ClearSelection,
    SelectPreviousReviewAction,
    SelectNextReviewAction,
    ToggleShowAuthored,
    ToggleShowProposed,
    ToggleShowUnrouted,
    ToggleDimUnrelated,
    ToggleLayerVisibility(String),
    FocusProductionArtifact(String),
    FocusProductionArtifactFile(String),
    ZoomArtifactPreviewIn,
    ZoomArtifactPreviewOut,
    PanArtifactPreview {
        delta_x_ppm: i32,
        delta_y_ppm: i32,
    },
    ResetArtifactPreviewViewport,
    ToggleArtifactPreviewGeometry,
    ToggleArtifactPreviewDrills,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    SelectionChanged(SelectionTarget),
    SceneChanged,
    FrameChanged,
    ToolChanged(WorkspaceTool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommandResult {
    pub handled: bool,
    pub events: Vec<SessionEvent>,
}
