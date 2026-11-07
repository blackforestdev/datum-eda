//! Pose keys belong only to the active native Symbol Move, never to field input.
use super::*;
use datum_gui_protocol::schematic_authoring::PlacementTransform;

impl Runtime {
    pub(crate) fn handle_symbol_move_pose_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        if self.workspace().schematic_authoring.picker
            || self.workspace().schematic_authoring.creation.is_some()
            || !self.native_edit.preview.as_ref().is_some_and(|p| {
                matches!(
                    p,
                    NativeMovePreview::Symbol { .. }
                        | NativeMovePreview::Symbols(_)
                        | NativeMovePreview::Text(_)
                        | NativeMovePreview::Wiring(_)
                        | NativeMovePreview::Connectivity { target: datum_gui_protocol::footprint_move_preview::native::ConnectivityMoveTarget::Wiring { .. }, .. }
                )
            })
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let Key::Character(key) = &event.logical_key else {
            return false;
        };
        let transform = match key.to_ascii_lowercase().as_str() {
            "r" => PlacementTransform::Rotate,
            "x" => PlacementTransform::FlipX,
            "y" => PlacementTransform::FlipY,
            _ => return false,
        };
        if self.native_edit.pending() {
            return true;
        }
        let pane = if self.native_edit.exact_position.is_some() {
            Some(self.workspace().ui.layout.focused)
        } else {
            self.last_cursor_pos
                .and_then(|(x, y)| self.pane_at_screen(x, y))
        };
        if !self.guard_native_authoring_view(pane) {
            return true;
        }
        let result = (|| -> Result<PointNm> {
            let position = if let Some(position) = self.native_edit.exact_position {
                position
            } else {
                let (x, y) = self
                    .last_cursor_pos
                    .context("Move requires a cursor in its schematic pane")?;
                let (world, surface) = self
                    .prepared_scene()
                    .and_then(|scene| scene.world_point_at_screen(x, y))
                    .context("Move requires a cursor in its schematic pane")?;
                anyhow::ensure!(
                    surface == SceneSurface::Schematic,
                    "Return to the original schematic pane to transform the selected objects"
                );
                self.native_move_destination(
                    self.native_edit
                        .preview
                        .as_ref()
                        .context("Move preview ended")?,
                    world,
                )?
            };
            let preview = self
                .native_edit
                .preview
                .as_mut()
                .context("Move preview ended")?;
            preview.transform_symbol(transform, position)?;
            Ok(position)
        })();
        match result {
            Ok(position) => {
                if matches!(self.native_edit.preview, Some(NativeMovePreview::Wiring(_))) {
                    self.session
                        .workspace_mut()
                        .authoring
                        .gesture
                        .target_object_id = Some("schematic-wiring-selection".into());
                }
                self.native_edit.pending_position = Some(position);
                self.invalidate_frame();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        true
    }
}
