//! Immutable selection projection consumed by the existing shared AT-SPI bridge.
use datum_gui_protocol::selection_view::SelectionView;
use std::sync::Arc;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectionAccessibilitySnapshot {
    pub view: Arc<SelectionView>,
    pub focused: bool,
    pub evaluating: Option<(datum_gui_protocol::PaneId, usize)>,
    pub visual_lod: Option<datum_gui_render::SelectionVisualLod>,
    pub pane: Option<datum_gui_protocol::PaneId>,
}
impl crate::Runtime {
    pub(super) fn refresh_selection_accessibility(&mut self) {
        self.refresh_selection_accessibility_lod(None);
    }
    pub(super) fn refresh_prepared_selection_accessibility(&mut self) {
        let lod = self
            .renderer
            .render_session()
            .prepared()
            .and_then(|scene| scene.selection_visual_lod());
        self.refresh_selection_accessibility_lod(lod);
    }
    fn refresh_selection_accessibility_lod(
        &mut self,
        lod: Option<datum_gui_render::SelectionVisualLod>,
    ) {
        let pane = match self.application_focus() {
            datum_gui_protocol::ApplicationFocus::Editor(pane) => Some(pane),
            _ => None,
        };
        let view = match &self.session.workspace().selection {
            datum_gui_protocol::SelectionTarget::Native(view) => Some(view.as_ref()),
            _ => None,
        };
        let evaluating = self
            .workspace()
            .ui
            .selection_region
            .as_ref()
            .and_then(|region| region.evaluating.map(|count| (region.pane, count)));
        self.terminal_accessibility
            .update_selection(view, pane, evaluating, lod);
    }
}
