//! Shared native Move acquisition and complete selected-target ownership.
use super::*;
#[path = "symbol_pose.rs"]
mod symbol_pose;
impl Runtime {
    pub(crate) fn pick_schematic_move(&mut self, world: PointNm, screen: (f32, f32)) {
        use datum_gui_protocol::{SelectionSubject, SelectionTarget};
        if matches!(&self.workspace().selection, SelectionTarget::Native(view)
            if !view.stale && matches!(&view.subject, SelectionSubject::Compound(_)))
        {
            self.begin_selected_native_move_at(Some(world));
            return;
        }
        let Ok(hit) = self.selection_world_hit(
            world,
            SceneSurface::Schematic,
            self.pane_at_screen(screen.0, screen.1),
        ) else {
            return;
        };
        let (Some(HitTarget::AuthoredObject(key)), Some(basis)) = (hit.target, hit.basis) else {
            return;
        };
        let Some(expected) = basis
            .bindings
            .get(&key)
            .map(|binding| binding.acquired.clone())
        else {
            return;
        };
        // Publishing selection invalidates PreparedScene. Keep this qualified
        // identity instead of re-hitting a scene that awaits the next frame.
        if self
            .acquire_native_key_from_basis(&key, Some(world), &basis)
            .unwrap_or(false)
            && matches!(&self.workspace().selection, SelectionTarget::Native(view)
                if !view.stale && view.subject == SelectionSubject::Object(expected))
        {
            self.begin_selected_native_move_at(Some(world));
        }
    }
    // Every admitted subject is indivisible; acquisition never uses Inspector scope.
    pub(crate) fn begin_selected_native_move(&mut self) {
        // A toolbar/menu command arms Move; acquisition waits for a canvas
        // click. A shortcut over the canvas can acquire the selected object now.
        if self
            .last_cursor_pos
            .and_then(|(x, y)| {
                self.prepared_scene()
                    .and_then(|scene| scene.world_point_at_screen(x, y))
            })
            .is_some()
        {
            self.begin_selected_native_move_at(None);
        }
    }
    pub(crate) fn begin_selected_native_move_at(&mut self, anchor: Option<PointNm>) {
        if self.workspace().schematic_authoring.delete_review.is_some() {
            return;
        }

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
        if matches!(&selected.subject, SelectionSubject::Compound(_)) {
            let subject = selected.subject.clone();
            self.begin_schematic_move(subject, anchor);
            return;
        }
        let SelectionSubject::Object(id) = &selected.subject else {
            return;
        };
        if matches!(
            id.class,
            AuthoredSelectionClass::Symbol
                | AuthoredSelectionClass::Label
                | AuthoredSelectionClass::SchematicText
        ) {
            let id = id.clone();
            self.begin_schematic_move(SelectionSubject::Object(id), anchor);
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
        self.native_move_click(anchor.unwrap_or(world), Some(target));
    }
    pub(crate) fn native_move_click(&mut self, world: PointNm, target: Option<String>) -> bool {
        if self.workspace().schematic_authoring.delete_review.is_some() {
            return true;
        }

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
            let valid = {
                matches!(&self.workspace().selection,
                datum_gui_protocol::SelectionTarget::Native(view)
                    if !view.stale && view.project.to_string() == preview.base().project_uuid
                        && view.revision.0 == preview.base().source_revision
                        && preview.matches_subject(&view.subject))
            };
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
    fn begin_schematic_move(
        &mut self,
        subject: datum_gui_protocol::SelectionSubject,
        anchor: Option<PointNm>,
    ) {
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
                _ => anyhow::bail!("Select a current native schematic object before moving"),
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
                "Schematic selection changed; select it again"
            );
            let (x, y) = self
                .last_cursor_pos
                .context("Move requires a cursor in the Schematic pane")?;
            // A captured press/menu pickup survives selection publication, which
            // invalidates PreparedScene. Never re-hit or substitute the menu leaf.
            let (world,pane)=if let Some(world)=anchor {
                (world,Some(self.workspace().ui.layout.focused))
            } else {
                let (world,surface)=self.prepared_scene().and_then(|s|s.world_point_at_screen(x,y))
                    .context("Move requires a cursor in the Schematic pane")?;
                anyhow::ensure!(surface==SceneSurface::Schematic,"Move requires a cursor in the Schematic pane");
                (world,self.pane_at_screen(x,y))
            };
            let view_basis=self.capture_native_authoring_view(pane).map_err(anyhow::Error::msg)?;
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
            let preview = match &subject {
                datum_gui_protocol::SelectionSubject::Object(id) => {
                    let begin = if id.class == datum_gui_protocol::AuthoredSelectionClass::SchematicText {NativeMovePreview::text}
                    else if id.class == datum_gui_protocol::AuthoredSelectionClass::Label {
                        NativeMovePreview::label
                    } else { NativeMovePreview::symbol };
                    begin(&base,&self.workspace().schematic_authoring,id,world,self.workspace().authoring.snap,root)?
                }
                _ => {
                    use datum_gui_protocol::footprint_move_preview::native::group::{symbol_members,SymbolGroupPreview};
                    let members=symbol_members(&subject)?;
                    let native=self.native_selection.as_ref().context("Native selection authority unavailable")?;
                    anyhow::ensure!(native.model.project.project_id.to_string()==base.project_uuid
                        && native.model.model_revision.0==base.source_revision,"Move authority changed; select the complete set again");
                    let authority=&native.authority;
                    let basis=eda_engine::api::native_write::schematic_symbol_move::SymbolMoveSetBasis::capture(
                        authority.snapshot(), &members.iter().map(|id|id.id).collect::<Vec<_>>(), &members[0].instance_path)?;
                    NativeMovePreview::Symbols(SymbolGroupPreview::begin(&base,&self.workspace().schematic_authoring,&subject,basis,world,self.workspace().authoring.snap,root)?)
                }
            };
            Ok((preview, world, view_basis))
        })();
        match result {
            Ok((preview, world, view_basis)) => {
                let (world, target) = match &preview {
                    NativeMovePreview::Symbols(group) => (
                        group.reference,
                        format!(
                            "schematic-symbols:{}",
                            match &subject {
                                datum_gui_protocol::SelectionSubject::Compound(c) =>
                                    c.members.len(),
                                _ => 0,
                            }
                        ),
                    ),
                    NativeMovePreview::Text(text) => {
                        (world, format!("schematic-text:{}", text.id()))
                    }
                    NativeMovePreview::Label { basis, .. } => {
                        (world, format!("schematic-label:{}", basis.label.uuid))
                    }
                    NativeMovePreview::Symbol { basis, .. } => {
                        (world, format!("schematic-symbol:{}", basis.symbol.uuid))
                    }
                    _ => unreachable!(),
                };
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
                self.native_edit.preview = Some(preview);
                self.invalidate_frame();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
    }
}
