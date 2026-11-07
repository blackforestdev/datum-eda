//! Exact coordinate entry and grid displacement configure the existing Move.
use super::*;
use datum_gui_protocol::{
    AuthoringViewBasis, CoordinateEntry, CoordinateEntryControl as Control, CoordinateEntryForm,
    CoordinateEntryMode,
};
use eda_engine::ir::units::LengthQuantity;

impl Runtime {
    pub(crate) fn coordinate_input_owner_available(&self) -> bool {
        coordinate_owner_available(self.workspace())
    }
    pub(crate) fn current_exact_move_position(&self) -> Option<PointNm> {
        self.native_edit
            .exact_position
            .or(self.native_edit.pending_position.filter(|point| {
                self.native_edit
                    .preview
                    .as_ref()
                    .is_some_and(|preview| preview.scene_at(*point).is_ok())
            }))
            .or(self.native_edit.current_position)
            .or_else(|| {
                self.native_edit
                    .preview
                    .as_ref()
                    .map(NativeMovePreview::reference_position)
            })
    }
    fn coordinate_basis_current(&self, basis: &AuthoringViewBasis) -> bool {
        let captured = if basis.content == PaneContent::Schematic {
            AuthoringViewBasis::capture_qualified_schematic(self.workspace(), basis.pane)
        } else {
            AuthoringViewBasis::capture(self.workspace(), basis.pane)
        };
        captured.is_ok_and(|current| current == *basis)
            && self.workspace().ui.layout.focused == basis.pane
            && self.native_edit.preview.as_ref().is_some_and(|p| {
                p.base().project_uuid == basis.project.to_string() && p.base().source_revision == basis.revision
                    && matches!(&self.workspace().selection, datum_gui_protocol::SelectionTarget::Native(view)
                        if !view.stale && p.matches_subject(&view.subject))
            })
    }
    pub(crate) fn open_move_coordinates(&mut self) -> bool {
        // Presented hits may predate a keyboard-opened menu/modal. Current
        // ownership is authoritative before focus, buffers, preview or writes.
        if !self.coordinate_input_owner_available() {
            return true;
        }

        let result = (|| -> Result<CoordinateEntryForm> {
            anyhow::ensure!(
                self.workspace().tool == WorkspaceTool::Move
                    && !self.native_edit.pending()
                    && !self.native_edit.blocked,
                "Start a current Move before entering coordinates"
            );
            let preview = self
                .native_edit
                .preview
                .as_ref()
                .context("Choose the Move target first")?;
            let basis = self
                .workspace()
                .authoring
                .gesture
                .view_basis
                .clone()
                .context("Move source basis unavailable")?;
            anyhow::ensure!(
                self.coordinate_basis_current(&basis),
                "Move source changed; cancel and select it again"
            );
            let model = self
                .native_selection
                .as_ref()
                .context("Native project unavailable")?;
            let units =
                eda_engine::api::native_write::project::project_display_units(&model.model)?
                    .resolve()
                    .map_err(|error| anyhow::anyhow!("Project units unavailable: {error:?}"))?;
            let position = self
                .current_exact_move_position()
                .context("Move position unavailable")?;
            let quantity = if preview.is_schematic() {
                LengthQuantity::SchematicGeometry
            } else {
                LengthQuantity::BoardLayout
            };
            let entry = CoordinateEntry::new(
                basis.project,
                quantity,
                units,
                preview.reference_position(),
                position,
                CoordinateEntryMode::Absolute,
            )
            .map_err(anyhow::Error::msg)?;
            Ok(CoordinateEntryForm {
                entry,
                focus: Control::X,
                label: "Position".into(),
                refusal: None,
                restore_position: position,
                basis,
                pending: false,
                stale: false,
            })
        })();
        match result {
            Ok(form) => {
                self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                    form.basis.pane,
                ));
                self.session.workspace_mut().authoring.coordinates = Some(form);
                self.invalidate_frame();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        true
    }
    /// Preflight before changing pending/current pose. A refused displacement
    /// leaves the prior valid preview and pointer latch untouched.
    fn install_exact_move_position(&mut self, position: PointNm) -> Result<()> {
        let preview = self
            .native_edit
            .preview
            .as_ref()
            .context("Move preview ended")?;
        preview.scene_at(position)?;
        self.native_edit.current_position = Some(position);
        self.native_edit.pending_position = Some(position);
        self.native_edit.exact_position = Some(position);
        self.native_edit.exact_cursor = self.last_cursor_pos;
        self.session.workspace_mut().authoring.gesture.preview = Some(position);
        self.invalidate_frame();
        Ok(())
    }
    pub(crate) fn coordinate_refusal(&mut self, error: String, stale: bool) {
        let changed = self
            .workspace()
            .authoring
            .coordinates
            .as_ref()
            .is_some_and(|form| form.refusal.as_ref() != Some(&error));
        if changed {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error.clone());
        }
        if let Some(form) = &mut self.session.workspace_mut().authoring.coordinates {
            form.refusal = Some(error);
            form.stale |= stale;
        }
        self.invalidate_frame();
    }
    /// Reuse the captured parser and native preview preflight on field changes.
    /// Invalid text remains editable; only a valid pose replaces the ghost.
    pub(crate) fn refresh_coordinate_preview(&mut self) {
        if !self.coordinate_input_owner_available() {
            return;
        }
        let Some(form) = self.workspace().authoring.coordinates.as_ref() else {
            return;
        };
        if self.native_edit.pending() || form.pending {
            return;
        }
        if form.stale || !self.coordinate_basis_current(&form.basis) {
            self.coordinate_refusal(
                "Move source changed; cancel and select it again".into(),
                true,
            );
            return;
        }
        let result = (|| -> Result<PointNm> {
            coordinate_preview_position(
                &form.entry,
                self.native_edit
                    .preview
                    .as_ref()
                    .context("Move preview ended")?,
            )
        })();
        match result {
            Ok(position) => match self.install_exact_move_position(position) {
                Ok(()) => {
                    if let Some(form) = &mut self.session.workspace_mut().authoring.coordinates {
                        form.refusal = None;
                    }
                }
                Err(error) => self.coordinate_refusal(error.to_string(), false),
            },
            Err(error) => self.coordinate_refusal(error.to_string(), false),
        }
    }
    pub(crate) fn coordinate_field_click(&mut self, control: Control) -> bool {
        // Presented hits may predate a keyboard-opened menu/modal. Current
        // ownership is authoritative before focus, buffers, preview or writes.
        if !self.coordinate_input_owner_available() {
            return true;
        }

        if let Some(form) = self.workspace().authoring.coordinates.as_ref() {
            self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                form.basis.pane,
            ));
        }
        let handled = self.coordinate_control(control);
        if matches!(control, Control::X | Control::Y) {
            let target = HitTarget::CoordinateEntry(control);
            if let Some(region) = self
                .presented_hits
                .regions()
                .iter()
                .find(|r| r.target == target)
                .cloned()
                && let Some((x, _)) = self.last_cursor_pos
            {
                self.start_dialog_editor_pointer(region, x, self.scale_factor);
                self.finish_dialog_editor_pointer();
            }
        }
        handled
    }
    pub(crate) fn coordinate_control(&mut self, control: Control) -> bool {
        // Presented hits may predate a keyboard-opened menu/modal. Current
        // ownership is authoritative before focus, buffers, preview or writes.
        if !self.coordinate_input_owner_available() {
            return true;
        }

        if self.workspace().authoring.coordinates.is_none() {
            return false;
        }
        if control == Control::Cancel {
            let form = self
                .session
                .workspace_mut()
                .authoring
                .coordinates
                .take()
                .unwrap();
            if !form.stale && self.coordinate_basis_current(&form.basis) {
                if let Err(error) = self.install_exact_move_position(form.restore_position) {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                }
            }
            self.invalidate_frame();
            return true;
        }
        if self.native_edit.pending() {
            return true;
        }
        if matches!(control, Control::Absolute | Control::Delta) {
            let mode = if control == Control::Absolute {
                CoordinateEntryMode::Absolute
            } else {
                CoordinateEntryMode::Delta
            };
            let result = {
                let form = self
                    .session
                    .workspace_mut()
                    .authoring
                    .coordinates
                    .as_mut()
                    .unwrap();
                form.entry.set_mode(mode).map(|()| form.focus = control)
            };
            match result {
                Ok(()) => self.refresh_coordinate_preview(),
                Err(error) => self.coordinate_refusal(error, false),
            }
            self.invalidate_frame();
            return true;
        }
        if matches!(control, Control::X | Control::Y) {
            self.session
                .workspace_mut()
                .authoring
                .coordinates
                .as_mut()
                .unwrap()
                .focus = control;
            self.invalidate_frame();
            return true;
        }
        let form = self.workspace().authoring.coordinates.as_ref().unwrap();
        if form.stale || !self.coordinate_basis_current(&form.basis) {
            self.coordinate_refusal(
                "Move source changed; cancel and select it again".into(),
                true,
            );
            return true;
        }
        let result = (|| -> Result<NativeEditAction> {
            let position = form.entry.destination().map_err(anyhow::Error::msg)?;
            let preview = self
                .native_edit
                .preview
                .as_ref()
                .context("Move preview ended")?;
            preview.scene_at(position)?;
            preview.action(position)
        })();
        match result {
            Ok(action) => {
                self.queue_native_edit(action);
            }
            Err(error) => self.coordinate_refusal(error.to_string(), false),
        }
        true
    }
    pub(crate) fn consume_coordinate_escape_release(
        &mut self,
        event: &winit::event::KeyEvent,
    ) -> bool {
        if event.state == ElementState::Released
            && matches!(event.logical_key, Key::Named(NamedKey::Escape))
            && self.native_edit.coordinate_escape_release
        {
            self.native_edit.coordinate_escape_release = false;
            return true;
        }
        false
    }
    pub(crate) fn handle_move_coordinate_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        if !self.coordinate_input_owner_available() {
            return false;
        }
        if self.workspace().authoring.coordinates.is_some() {
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => {
                    self.native_edit.coordinate_escape_release = true;
                    return self.coordinate_control(Control::Cancel);
                }
                Key::Named(NamedKey::Tab) => {
                    self.session
                        .workspace_mut()
                        .authoring
                        .coordinates
                        .as_mut()
                        .unwrap()
                        .focus_next(self.modifiers.shift_key());
                    self.invalidate_frame();
                    return true;
                }
                Key::Named(NamedKey::Enter) => {
                    let focus = self
                        .workspace()
                        .authoring
                        .coordinates
                        .as_ref()
                        .unwrap()
                        .focus;
                    if !matches!(focus, Control::X | Control::Y) {
                        return self.coordinate_control(focus);
                    }
                }
                _ => {}
            }
            self.handle_shared_editor_key(event);
            return true;
        }
        if !matches!(
            self.workspace().ui.layout.focused_content(),
            PaneContent::Schematic | PaneContent::Board
        ) || self.workspace().tool != WorkspaceTool::Move
            || self.native_edit.preview.is_none()
            || self.native_edit.pending()
            || self.native_edit.blocked
            || self.workspace().ui.active_menu.is_some()
            || self.workspace().ui.new_project.open
            || self.workspace().ui.project_open.open
            || self.workspace().schematic_authoring.picker
            || self.workspace().schematic_authoring.creation.is_some()
            || self.workspace().schematic_authoring.delete_review.is_some()
            || self.pan_gesture.space_is_held()
            || !self.modifiers.is_empty()
        {
            return false;
        }
        let step = self.workspace().authoring.snap.grid_nm;
        if step <= 0 {
            return false;
        }
        let (dx, dy) = match &event.logical_key {
            Key::Named(NamedKey::ArrowLeft) => (-i128::from(step), 0),
            Key::Named(NamedKey::ArrowRight) => (i128::from(step), 0),
            Key::Named(NamedKey::ArrowUp) => (0, -i128::from(step)),
            Key::Named(NamedKey::ArrowDown) => (0, i128::from(step)),
            _ => return false,
        };
        let result = (|| -> Result<()> {
            let basis = self
                .workspace()
                .authoring
                .gesture
                .view_basis
                .as_ref()
                .context("Move source unavailable")?;
            anyhow::ensure!(
                self.coordinate_basis_current(basis),
                "Move source changed; cancel and select it again"
            );
            let current = self
                .current_exact_move_position()
                .context("Move position unavailable")?;
            self.install_exact_move_position(
                checked_move_delta(current, dx, dy).map_err(anyhow::Error::msg)?,
            )
        })();
        if let Err(error) = result {
            self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
        }
        true
    }
}
fn coordinate_owner_available(state: &datum_gui_protocol::ReviewWorkspaceState) -> bool {
    state.ui.active_menu.is_none()
        && !state.ui.new_project.open
        && !state.ui.project_open.open
        && state.library.acquisition.is_none()
        && state.library.draft.is_none()
}
fn coordinate_preview_position(
    entry: &CoordinateEntry,
    preview: &NativeMovePreview,
) -> Result<PointNm> {
    let position = entry.destination().map_err(anyhow::Error::msg)?;
    preview.scene_at(position)?;
    Ok(position)
}
fn checked_move_delta(point: PointNm, dx: i128, dy: i128) -> Result<PointNm, String> {
    Ok(PointNm {
        x: i64::try_from(i128::from(point.x) + dx)
            .map_err(|_| "Coordinate exceeds integer nanometers")?,
        y: i64::try_from(i128::from(point.y) + dy)
            .map_err(|_| "Coordinate exceeds integer nanometers")?,
    })
}
#[cfg(test)]
#[path = "coordinate_tests.rs"]
mod tests;
