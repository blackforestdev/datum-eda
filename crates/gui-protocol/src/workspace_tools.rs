//! Shared workspace tool and gesture state.
use crate::PointNm;
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
    pub tool: WorkspaceTool,
    pub anchor: Option<PointNm>,
    pub preview: Option<PointNm>,
    pub target_object_id: Option<String>,
}

impl AuthoringGestureState {
    pub fn idle(tool: WorkspaceTool) -> Self {
        Self {
            tool,
            anchor: None,
            preview: None,
            target_object_id: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.anchor.is_some() || self.preview.is_some() || self.target_object_id.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoringToolState {
    pub snap: AuthoringSnapState,
    pub gesture: AuthoringGestureState,
}
