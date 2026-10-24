//! Minimal immutable inputs for the shared, complete selection view builder.
use super::*;
use datum_gui_protocol::{BoardReviewSceneV1, ReviewWorkspaceState};
use std::collections::BTreeMap;

pub(crate) struct ProjectionInputs {
    board: BoardReviewSceneV1,
    schematics: Vec<BoardReviewSceneV1>,
    show_authored: bool,
    layers: BTreeMap<String, bool>,
}
impl ProjectionInputs {
    fn schematics(state: &ReviewWorkspaceState) -> Vec<BoardReviewSceneV1> {
        datum_gui_protocol::schematic_documents::visible_scenes(state)
            .into_iter()
            .cloned()
            .collect()
    }
    pub(crate) fn capture(state: &ReviewWorkspaceState) -> Self {
        let _probe =
            crate::gui_runtime_support::phase_probe::Probe::start("run_projection_capture");
        Self {
            board: state.scene.clone(),
            schematics: Self::schematics(state),
            show_authored: state.ui.filters.show_authored,
            layers: state.ui.filters.layer_visibility.clone(),
        }
    }
    pub(super) fn same_projection(&self, other: &Self) -> bool {
        self.board == other.board
            && self.schematics == other.schematics
            && self.show_authored == other.show_authored
            && self.layers == other.layers
    }
    pub(super) fn matches_workspace(&self, state: &ReviewWorkspaceState) -> bool {
        self.board == state.scene
            && self.schematics == Self::schematics(state)
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
        datum_gui_protocol::selection_view::build_selection_view_for_documents(
            model,
            authority,
            subject,
            &self.board,
            &self.schematics.iter().collect::<Vec<_>>(),
            |layer| self.show_authored && self.layers.get(layer).copied().unwrap_or(true),
        )
    }
}
