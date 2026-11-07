//! Existing native Move preview pipeline, separated from worker ownership.
use super::*;
impl Runtime {
    pub(crate) fn native_move_pointer(&mut self, world: PointNm) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        if self.workspace().authoring.coordinates.is_some() {
            return true;
        }
        // The captured Move prompt is UI, not another canvas destination.
        // Keep an exact keyboard pose when the pointer enters its controls.
        if self.native_edit.preview.is_some()
            && self.last_cursor_pos.is_some_and(|(x, y)| {
                matches!(
                    self.presented_hits.hit_test(x, y),
                    Some(HitTarget::MoveCoordinates | HitTarget::SchematicPicker)
                )
            })
        {
            return true;
        }
        if self.native_edit.exact_position.is_some() {
            if self.native_edit.exact_cursor == self.last_cursor_pos {
                return true;
            }
            self.native_edit.exact_position = None;
            self.native_edit.exact_cursor = None;
        }
        if self.native_edit.preview.is_some() {
            let pane = self
                .last_cursor_pos
                .and_then(|(x, y)| self.pane_at_screen(x, y));
            if !self.guard_native_authoring_view(pane) {
                return true;
            }
        }
        let Some(preview) = &self.native_edit.preview else {
            return false;
        };
        match self.native_move_destination(preview, world) {
            Ok(position) => {
                crate::append_gui_verbose_diagnostic_line(|| {
                    format!("native move intent {position:?}")
                });
                self.native_edit.pending_position = Some(position);
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        // Input captures the latest complete intent. The native frame materializes
        // it once, so accumulated motion does not rebuild copper on every event.
        true
    }
    /// Keep the complete source stable across upload-only native turns. New
    /// pointer intent supersedes only the queued destination, not the source
    /// whose upload/presentation is already in progress.
    pub(crate) fn prepare_native_move_frame(&mut self) {
        if self.native_edit.preview.is_some()
            && !self.native_edit.pending()
            && !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused))
        {
            return;
        }
        if self.native_edit.preview_awaiting_presentation {
            return;
        }
        crate::append_gui_verbose_diagnostic_line(|| {
            format!(
                "native move frame preview={} awaiting={} pending={:?} tool={:?}",
                self.native_edit.preview.is_some(),
                self.native_edit.preview_awaiting_presentation,
                self.native_edit.pending_position,
                self.workspace().tool
            )
        });
        let Some(position) = self.native_edit.pending_position.take() else {
            return;
        };
        let Some(preview) = &self.native_edit.preview else {
            return;
        };
        match preview.scene_at(position) {
            Ok(scene) => {
                crate::append_gui_verbose_diagnostic_line(|| {
                    format!("native move scene ready {position:?}")
                });
                let schematic = preview.is_schematic();
                self.native_edit.current_position = Some(position);
                self.session.workspace_mut().authoring.gesture.preview = Some(position);
                self.publish_move_handle_feedback(position);
                self.set_move_scene(schematic, scene);
                self.native_edit.preview_awaiting_presentation = true;
                self.invalidate_scene();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
    }
    pub(crate) fn native_move_frame_presented(&mut self) {
        self.native_edit.preview_awaiting_presentation = false;
    }
    pub(crate) fn native_move_frame_needed(&self) -> bool {
        self.native_edit.preview.is_some()
            && self.native_edit.pending_position.is_some()
            && !self.native_edit.preview_awaiting_presentation
    }
}
