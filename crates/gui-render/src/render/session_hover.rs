//! Material and text hover dependencies that predate the immediate hover ring.
use super::*;
use datum_gui_protocol::{HoverTarget, PaneContent, SelectionTarget};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BoardHoverDependency {
    Disabled,
    None,
    Pad(usize),
    Unknown,
}

impl BoardHoverDependency {
    pub(super) fn capture(state: &ReviewWorkspaceState, retained: &RetainedScene) -> Self {
        if !matches!(state.selection, SelectionTarget::None) {
            Self::Disabled
        } else {
            Self::resolve(state.ui.hovered_object.as_ref(), retained)
        }
    }

    fn resolve(hover: Option<&HoverTarget>, retained: &RetainedScene) -> Self {
        if !retained.hover_membership.available() {
            return Self::Unknown;
        }
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
            .map_or(Self::Unknown, |i| match retained.hover_is_pad(i) {
                Some(true) => Self::Pad(i),
                Some(false) => Self::None,
                None => Self::Unknown,
            })
    }

    pub(super) fn permits_pointer(
        self,
        hover: Option<&HoverTarget>,
        retained: &RetainedScene,
    ) -> bool {
        self == Self::Disabled || (self != Self::Unknown && self == Self::resolve(hover, retained))
    }
}
