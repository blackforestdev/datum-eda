//! Cursor feedback uses the same live gesture as tool chrome, only over canvas.
use crate::*;
use winit::window::CursorIcon;

impl Runtime {
    pub(super) fn editor_cursor_icon(&self, pointer: (f32, f32)) -> CursorIcon {
        let state = self.workspace();
        let panes = self.current_layout().viewport_panes(&state.ui.layout);
        let canvas = panes.panes.iter().any(|pane| {
            pane.id == panes.focused
                && matches!(pane.content, PaneContent::Board | PaneContent::Schematic)
                && pane.rect.scene.contains(pointer.0, pointer.1)
        });
        cursor_for_state(
            state,
            canvas,
            self.pan_gesture.is_active() || self.native_edit.preview.is_some(),
        )
    }
}

fn cursor_for_state(
    state: &datum_gui_protocol::ReviewWorkspaceState,
    canvas: bool,
    dragging: bool,
) -> CursorIcon {
    if !canvas
        || state.ui.active_menu.is_some()
        || state.ui.marking_menu.is_some()
        || state.schematic_authoring.picker
        || state.schematic_authoring.editing.is_some()
    {
        CursorIcon::Default
    } else if dragging {
        CursorIcon::Grabbing
    } else if state.ui.layout.focused_content() == PaneContent::Schematic
        && state.schematic_authoring.active()
    {
        CursorIcon::Crosshair
    } else if state.tool == WorkspaceTool::Move {
        CursorIcon::Grab
    } else {
        CursorIcon::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface_port_cursor_yields_to_form_then_tracks_preview_and_cancel() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
        state.ui.layout.focused = datum_gui_protocol::PaneId(1);
        state.schematic_authoring.label =
            Some(datum_gui_protocol::schematic_authoring::LabelDraft::port(
                eda_engine::schematic::PortDirection::Input,
            ));
        state.schematic_authoring.picker = true;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.schematic_authoring.picker = false;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Crosshair);
        assert_eq!(cursor_for_state(&state, false, false), CursorIcon::Default);
        state.ui.active_menu = Some("Place".into());
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.ui.active_menu = None;
        state.schematic_authoring.cancel();
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
    }
    #[test]
    fn cursor_feedback_follows_live_gesture_and_yields_to_chrome() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.layout = datum_gui_protocol::WorkspaceLayout::board_schematic();
        state.ui.layout.focused = datum_gui_protocol::PaneId(1);
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.tool = WorkspaceTool::Move;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Grab);
        assert_eq!(cursor_for_state(&state, true, true), CursorIcon::Grabbing);
        assert_eq!(cursor_for_state(&state, false, true), CursorIcon::Default);
        state.tool = WorkspaceTool::Select;
        state.schematic_authoring.wire = true;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Crosshair);
        state.ui.active_menu = Some("Place".into());
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.ui.active_menu = None;
        state.schematic_authoring.cancel();
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.schematic_authoring.chosen = Some(0);
        state.schematic_authoring.picker = true;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Default);
        state.schematic_authoring.picker = false;
        assert_eq!(cursor_for_state(&state, true, false), CursorIcon::Crosshair);
    }
}
