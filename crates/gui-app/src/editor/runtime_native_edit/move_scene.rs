//! Native Move preview publication, cancellation and failed-write restoration.
use super::*;

impl Runtime {
    pub(crate) fn cancel_native_move(&mut self) -> bool {
        self.native_edit.pending_position = None;
        self.native_edit.preview_awaiting_presentation = false;
        if let Some(preview) = self.native_edit.preview.take() {
            self.native_edit.preview_tip = None;
            self.set_move_scene(preview.is_schematic(), preview.base().clone());
            // Every schematic Move acquired a qualified occurrence basis;
            // cancellation clears it uniformly for every subject class.
            if preview.is_schematic() {
                let editor = &mut self.session.workspace_mut().schematic_authoring;
                editor.circuit_basis = None;
                editor.circuit_preview = None;
            }
            self.session.workspace_mut().cancel_authoring_gesture();
            self.restore_context_move_tool();
            self.invalidate_scene();
            return true;
        }
        false
    }
    pub(super) fn restore_pending_move_base(&mut self) {
        if let Some((schematic, base)) = self.native_edit.pending_move_base.take()
            && base.project_uuid == self.workspace().scene.project_uuid
        {
            self.set_move_scene(schematic, base);
            self.invalidate_scene();
        }
    }
    pub(super) fn set_move_scene(
        &mut self,
        schematic: bool,
        scene: datum_gui_protocol::BoardReviewSceneV1,
    ) {
        if schematic {
            let editor = &mut self.session.workspace_mut().schematic_authoring;
            if editor.circuit_basis.as_ref().is_some_and(|basis| {
                basis.document == scene.schematic_document
                    && basis.revision == scene.source_revision
                    && basis.project.to_string() == scene.project_uuid
            }) {
                editor.circuit_preview = Some(scene);
            } else if editor.circuit_basis.is_none()
                && self
                    .workspace()
                    .schematic_scene
                    .as_ref()
                    .is_some_and(|primary| primary.schematic_document == scene.schematic_document)
            {
                self.session.workspace_mut().schematic_scene = Some(scene);
            }
        } else {
            self.session.workspace_mut().scene = scene;
        }
    }
}
