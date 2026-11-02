//! Junction and intentional-open-pin placement through the shared native gesture.
use super::*;
use datum_gui_protocol::schematic_authoring::MarkerKind;

impl Runtime {
    pub(crate) fn begin_native_marker(&mut self, kind: MarkerKind) -> bool {
        if !self.prepare_native_schematic(false) {
            return true;
        }
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        editor.chosen = None;
        editor.label = None;
        editor.marker = Some(kind);
        editor.picker = false;
        editor.refusal = None;
        editor.preview_dirty = true;
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_marker_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        if event.logical_key == Key::Named(NamedKey::Escape) {
            self.cancel_native_schematic();
            self.set_workspace_tool(WorkspaceTool::Select);
            return true;
        }
        // Shared editor shortcuts retain ownership. Markers have no symbol
        // fields, rotation or picker for Tab/Enter to accidentally activate.
        false
    }

    pub(super) fn place_native_marker(&mut self, at: PointNm) -> bool {
        let request = match self.workspace().schematic_authoring.marker_request(at) {
            Ok(request) => request,
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason.clone());
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                self.invalidate_frame();
                return true;
            }
        };
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        editor.position = Some(at);
        editor.preview_dirty = true;
        editor.refusal = None;
        editor.pending_request = Some(request);
        self.prepare_native_schematic_frame();
        self.queue_native_edit(NativeEditAction::Schematic)
    }
}
