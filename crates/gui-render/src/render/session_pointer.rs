//! Pointer-only input cannot carry source, camera, pane or chrome replacement.
pub use super::session_sources::{SourceEpoch, SourceRevision};
use super::*;
use datum_gui_protocol::{CrosshairStyle, HoverTarget, PaneContent, ScreenPointPx};

/// Capability for one shared prepared projection, not a caller-supplied revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparationGeneration {
    owner: u64,
    generation: u64,
}

pub struct PointerUpdate<'a> {
    pub generation: Option<PreparationGeneration>,
    pub cursor: Option<ScreenPointPx>,
    pub hover: Option<&'a HoverTarget>,
    pub style: CrosshairStyle,
}

impl RenderSession {
    pub fn pointer_generation(&self) -> Option<PreparationGeneration> {
        (self.prepared.is_some()
            && self
                .preparation
                .is_some_and(|p| p.profile == PreparedProfile::Workspace))
        .then_some(PreparationGeneration {
            owner: self.revisions.owner(),
            generation: self.preparation_generation,
        })
    }

    /// Reject stale/foreign input before changing damage or any projection.
    /// Accepted pointer updates never retire or weaken pending composition work.
    pub fn update_pointer(&mut self, input: PointerUpdate<'_>) -> bool {
        let Some(generation) = input.generation else {
            // Input arrived before preparation or while its snapshot was moved
            // into an attempt. Rebuild from current model input; old completion
            // must not restore a projection that predates this pointer update.
            if self.board.as_ref().is_some_and(|board| {
                self.preparation
                    .is_none_or(|p| !p.hover.permits_pointer(input.hover, board))
            }) {
                self.clear_content();
            } else {
                self.composition_changed();
            }
            return false;
        };
        if self.pointer_generation() != Some(generation) || self.board.is_none() {
            return false;
        }
        if !self
            .preparation
            .expect("validated profile")
            .hover
            .permits_pointer(input.hover, self.board.as_ref().expect("validated board"))
        {
            // Legacy pad material/dimming and hovered net labels belong to the
            // retained/full projection. Invalidate them and their history together;
            // a ring-only update cannot represent this transition correctly.
            self.clear_content();
            return false;
        }
        let bounds = |surface, source: Option<&RetainedScene>| {
            let hover = input.hover.filter(|hover| hover.surface == surface)?;
            crate::interaction_overlay::board_hover_bounds(source?, &hover.object_id)
        };
        let board_hover = bounds(PaneContent::Board, self.board.as_ref());
        let schematic_hover = bounds(PaneContent::Schematic, self.schematic.as_ref());
        self.prepared
            .as_mut()
            .expect("validated preparation")
            .refresh_pointer(input.cursor, input.style, board_hover, schematic_hover);
        self.prepared_revision = self.revisions.update(Change::Interaction);
        true
    }
}

#[cfg(test)]
#[path = "session_pointer_tests.rs"]
mod tests;
