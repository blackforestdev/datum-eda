//! Minimal immutable inputs for the shared, complete selection view builder.
use super::*;
use datum_gui_protocol::{BoardReviewSceneV1, ReviewWorkspaceState};
use std::collections::BTreeMap;

pub(crate) struct ProjectionInputs {
    board: BoardReviewSceneV1,
    schematic: Option<BoardReviewSceneV1>,
    show_authored: bool,
    layers: BTreeMap<String, bool>,
}
impl ProjectionInputs {
    pub(crate) fn capture(state: &ReviewWorkspaceState) -> Self {
        let _probe =
            crate::gui_runtime_support::phase_probe::Probe::start("run_projection_capture");
        Self {
            board: state.scene.clone(),
            schematic: state.schematic_scene.clone(),
            show_authored: state.ui.filters.show_authored,
            layers: state.ui.filters.layer_visibility.clone(),
        }
    }
    pub(super) fn same_projection(&self, other: &Self) -> bool {
        self.board == other.board
            && self.schematic == other.schematic
            && self.show_authored == other.show_authored
            && self.layers == other.layers
    }
    pub(super) fn matches_workspace(&self, state: &ReviewWorkspaceState) -> bool {
        self.board == state.scene
            && self.schematic == state.schematic_scene
            && self.show_authored == state.ui.filters.show_authored
            && self.layers == state.ui.filters.layer_visibility
    }
    pub(super) fn build(
        &self,
        model: &DesignModel,
        authority: &NativeSelectionResolution,
        subject: SelectionSubject,
    ) -> Result<
        datum_gui_protocol::selection_view::SelectionView,
        datum_gui_protocol::selection_resolution::SelectionResolutionError,
    > {
        let _probe = crate::gui_runtime_support::phase_probe::Probe::start("run_projection_build");
        datum_gui_protocol::selection_view::build_selection_view(
            model,
            authority,
            subject,
            &self.board,
            self.schematic.as_ref(),
            |layer| self.show_authored && self.layers.get(layer).copied().unwrap_or(true),
        )
    }
}
