//! Material and text hover dependencies that predate the immediate hover ring.
use super::*;
use datum_gui_protocol::{HoverTarget, PaneContent, SelectionTarget};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BoardHover {
    Disabled,
    None,
    Region(usize),
    Unknown,
}

impl BoardHover {
    pub(super) fn capture(state: &ReviewWorkspaceState, retained: &RetainedScene) -> Self {
        if !matches!(state.selection, SelectionTarget::None) {
            Self::Disabled
        } else {
            Self::resolve(state.ui.hovered_object.as_ref(), retained)
        }
    }

    fn resolve(hover: Option<&HoverTarget>, retained: &RetainedScene) -> Self {
        let Some(hover) = hover.filter(|h| h.surface == PaneContent::Board) else {
            return Self::None;
        };
        retained
            .world_hit_index
            .regions()
            .iter()
            .position(|region| match &region.target {
                crate::HitTarget::AuthoredObject(id) | crate::HitTarget::ReviewAction(id) => {
                    id == &hover.object_id
                }
                _ => false,
            })
            .map_or(Self::Unknown, Self::Region)
    }

    pub(super) fn permits_pointer(
        self,
        hover: Option<&HoverTarget>,
        retained: &RetainedScene,
    ) -> bool {
        self == Self::Disabled || (self != Self::Unknown && self == Self::resolve(hover, retained))
    }
}

impl RenderSession {
    pub(super) fn invalidate_changed_hover(&mut self, state: &ReviewWorkspaceState) -> bool {
        let Some(preparation) = self
            .preparation
            .filter(|p| p.profile == PreparedProfile::Workspace)
        else {
            return false;
        };
        let Some(board) = &self.board else {
            return false;
        };
        let current = BoardHover::capture(state, board);
        if current == preparation.hover && current != BoardHover::Unknown {
            return false;
        }
        self.clear_content();
        true
    }
}
