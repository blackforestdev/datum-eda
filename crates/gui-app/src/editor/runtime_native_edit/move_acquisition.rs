//! Shared native Move acquisition and complete selected-target ownership.
use super::*;
impl Runtime {
    // The bounded first-edit capability moves one complete Footprint. Never
    // silently reduce a Run, compound selection or another object to a subset.
    pub(crate) fn begin_selected_native_move(&mut self) {
        use datum_gui_protocol::{AuthoredSelectionClass, SelectionSubject, SelectionTarget};
        let SelectionTarget::Native(selected) = &self.workspace().selection else {
            return;
        };
        if selected.stale {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Select a current Footprint before moving",
            );
            return;
        }
        let SelectionSubject::Object(id) = &selected.subject else {
            return;
        };
        if id.class == AuthoredSelectionClass::Symbol {
            let id = id.clone();
            self.begin_symbol_move(id);
            return;
        }
        if id.class != AuthoredSelectionClass::Footprint {
            return;
        }
        let Some(target) = self
            .workspace()
            .scene
            .components
            .iter()
            .find(|c| c.component_uuid == id.id.to_string())
            .map(|c| c.object_id.clone())
        else {
            return;
        };
        let Some((x, y)) = self.last_cursor_pos else {
            return;
        };
        let Some((world, SceneSurface::Board)) = self
            .prepared_scene()
            .and_then(|s| s.world_point_at_screen(x, y))
        else {
            return;
        };
        self.native_move_click(world, Some(target));
    }
    pub(crate) fn native_move_click(&mut self, world: PointNm, target: Option<String>) -> bool {
        if self.native_edit.pending() || self.check_work.pending() || self.native_edit.blocked {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Wait for the current edit or reopen the Project after an unconfirmed outcome",
            );
            return true;
        }
        if let Some(schematic) = self.native_edit.preview.as_ref().map(|p| p.is_schematic()) {
            let pane = self
                .last_cursor_pos
                .and_then(|(x, y)| self.pane_at_screen(x, y));
            if !self.guard_native_authoring_view(pane) {
                return true;
            }
            let surface = self.last_cursor_pos.and_then(|(x, y)| {
                self.prepared_scene()
                    .and_then(|s| s.world_point_at_screen(x, y))
                    .map(|(_, surface)| surface)
            });
            let expected = if schematic {
                SceneSurface::Schematic
            } else {
                SceneSurface::Board
            };
            if surface != Some(expected) {
                self.log_console_refusal(
                    ConsoleFeedbackSource::Tool,
                    "Finish the Move in its original editor",
                );
                return true;
            }
            let preview = self.native_edit.preview.as_ref().unwrap();
            let valid = preview.identity().is_ok_and(|id| {
                matches!(&self.workspace().selection,
                datum_gui_protocol::SelectionTarget::Native(view)
                    if !view.stale && view.project.to_string() == preview.base().project_uuid
                        && view.revision.0 == preview.base().source_revision
                        && view.subject == datum_gui_protocol::SelectionSubject::Object(id))
            });
            if !valid {
                self.log_console_refusal(
                    ConsoleFeedbackSource::Tool,
                    "Move target changed; cancel and select it again",
                );
                return true;
            }
            let destination = match preview.destination(world, self.workspace().authoring.snap) {
                Ok(p) => p,
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                    return true;
                }
            };
            let action = match preview.action(destination) {
                Ok(action) => action,
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                    return true;
                }
            };
            return self.queue_native_edit(action);
        }
        let Some(target) = target else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Move requires clicking an existing Footprint",
            );
            return true;
        };
        let view_basis = match self.capture_native_authoring_view(
            self.last_cursor_pos
                .and_then(|(x, y)| self.pane_at_screen(x, y)),
        ) {
            Ok(basis) if basis.content == PaneContent::Board => basis,
            Ok(_) => {
                self.refuse_native_authoring_view("Move requires its Board source pane".into());
                return true;
            }
            Err(reason) => {
                self.refuse_native_authoring_view(reason);
                return true;
            }
        };
        let preview = match FootprintMovePreview::begin(
            &self.workspace().scene,
            &target,
            world,
            self.workspace().authoring.snap,
        ) {
            Ok(preview) => preview,
            Err(error) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string());
                return true;
            }
        };
        // The selected source identity is the authoring target. Never move a
        // hovered object while the Inspector still describes another subject.
        let current = matches!(&self.workspace().selection,
            datum_gui_protocol::SelectionTarget::Native(view)
            if !view.stale && view.project.to_string() == preview.base.project_uuid
                && view.revision.0 == preview.base.source_revision
                && matches!(&view.subject, datum_gui_protocol::SelectionSubject::Object(id)
                    if id.class == datum_gui_protocol::AuthoredSelectionClass::Footprint
                        && id.id.to_string() == preview.component.component_uuid));
        if !current {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Select a current Footprint before moving",
            );
            return true;
        }
        let target = preview.component.object_id.clone();
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world,
            target_object_id: Some(target),
        });
        self.session.workspace_mut().authoring.gesture.view_basis = Some(view_basis);
        self.native_edit.preview_tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .clone();
        self.native_edit.preview = Some(NativeMovePreview::Footprint(preview));
        self.invalidate_frame();
        true
    }
    fn begin_symbol_move(&mut self, id: datum_gui_protocol::AuthoredSelectionIdentity) {
        if self.native_edit.pending()
            || self.check_work.pending()
            || self.native_edit.blocked
            || !self.native_edit_available()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Wait for the current edit or reopen the Project after an unconfirmed outcome",
            );
            return;
        }
        let result = (|| -> Result<(NativeMovePreview, PointNm, datum_gui_protocol::AuthoringViewBasis)> {
            let selected = match &self.workspace().selection {
                datum_gui_protocol::SelectionTarget::Native(view) => view,
                _ => anyhow::bail!("Select a current native Symbol before moving"),
            };
            let base = self
                .workspace()
                .schematic_scene
                .as_ref()
                .context("Native schematic scene unavailable")?
                .clone();
            anyhow::ensure!(
                !selected.stale
                    && selected.project.to_string() == base.project_uuid
                    && selected.revision.0 == base.source_revision,
                "Symbol selection changed; select it again"
            );
            let (x, y) = self
                .last_cursor_pos
                .context("Move requires a cursor in the Schematic pane")?;
            let (world, surface) = self
                .prepared_scene()
                .and_then(|s| s.world_point_at_screen(x, y))
                .context("Move requires a cursor in the Schematic pane")?;
            anyhow::ensure!(
                surface == SceneSurface::Schematic,
                "Move requires a cursor in the Schematic pane"
            );
            let view_basis = self.capture_native_authoring_view(self.pane_at_screen(x,y))
                .map_err(anyhow::Error::msg)?;
            anyhow::ensure!(view_basis.content == PaneContent::Schematic,
                "Move requires its schematic source pane");
            let root = self
                .workspace()
                .backing
                .as_ref()
                .context("Open a persistent Datum Project before editing")?
                .request
                .project_root
                .clone();
            let preview = NativeMovePreview::symbol(
                &base,
                &self.workspace().schematic_authoring,
                &id,
                world,
                self.workspace().authoring.snap,
                root,
            )?;
            Ok((preview, world, view_basis))
        })();
        match result {
            Ok((preview, world, view_basis)) => {
                self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
                    world,
                    target_object_id: Some(format!("schematic-symbol:{}", id.id)),
                });
                self.session.workspace_mut().authoring.gesture.view_basis = Some(view_basis);
                self.native_edit.preview_tip = self
                    .workspace()
                    .supervision
                    .journal
                    .accepted_transaction_tip
                    .clone();
                self.native_edit.preview = Some(preview);
                self.invalidate_frame();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
    }
}
