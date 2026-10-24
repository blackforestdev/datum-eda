//! Immutable selection projection consumed by the existing shared AT-SPI bridge.
use datum_gui_protocol::selection_view::SelectionView;
use std::sync::Arc;
#[path = "selection_accessibility/workspace.rs"]
pub(crate) mod workspace;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectionAccessibilitySnapshot {
    pub view: Option<Arc<SelectionView>>,
    pub inspection: Option<Arc<workspace::WorkspaceInspection>>,
    pub focused: bool,
    pub evaluating: Option<(datum_gui_protocol::PaneId, usize)>,
    pub visual_lod: Option<datum_gui_render::SelectionVisualLod>,
    pub pane: Option<datum_gui_protocol::PaneId>,
}
impl crate::Runtime {
    pub(super) fn refresh_inspector_activity(&mut self) {
        use datum_gui_protocol::inspector_context::InspectorActivity as A;
        let activity = if self.native_edit.blocked {
            A::Unconfirmed
        } else if self.native_edit.pending()
            || self.project_open.pending()
            || self.check_work.pending()
        {
            A::Pending
        } else if self.native_edit.preview.is_some()
            || self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker
            || self.workspace().library.draft.is_some()
            || self.workspace().authoring.gesture.is_active()
        {
            A::Preview
        } else {
            A::Quiet
        };
        self.session.workspace_mut().inspector_context.activity = Some(activity);
    }
    pub(super) fn refresh_selection_accessibility(&mut self) {
        self.selection_run.cancel();
        self.refresh_selection_accessibility_lod(None);
        self.refresh_terminal_selection_context_if_changed();
    }
    pub(super) fn refresh_prepared_selection_accessibility(&mut self) {
        let lod = self
            .renderer
            .render_session()
            .prepared()
            .and_then(|scene| scene.selection_visual_lod());
        self.refresh_selection_accessibility_lod(lod);
        self.refresh_terminal_selection_context_if_changed();
    }
    fn refresh_selection_accessibility_lod(
        &mut self,
        lod: Option<datum_gui_render::SelectionVisualLod>,
    ) {
        self.refresh_inspector_activity();
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
        self.terminal_accessibility.update_workspace_selection(
            view,
            pane,
            evaluating,
            lod,
            workspace::WorkspaceInspection::capture(self.session.workspace()),
        );
    }
}
