//! Named schematic creation over the shared form editor and canonical edit worker.
use super::*;
use datum_gui_protocol::{
    native_edit_client::NativeEditAction,
    schematic_authoring::{NewSchematicDraft, SchematicEditRequest},
};
impl Runtime {
    pub(super) fn trace_schematic_creation_event(&self, source: &str, key: Option<&KeyEvent>) {
        if self.workspace().schematic_authoring.creation.is_some()
            && std::env::var_os("DATUM_TRACE_CLICKS").is_some()
        {
            self.trace_click(format!(
                "schematic form {source}: focus={:?} menu={:?} key={:?}",
                self.application_focus(),
                self.workspace().ui.active_menu,
                key.map(|event| (
                    match &event.logical_key {
                        Key::Named(key) => Some(key),
                        _ => None,
                    },
                    event.state
                ))
            ));
        }
    }

    pub(super) fn begin_schematic_creation(&mut self) -> bool {
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.check_work.pending()
            || self.native_edit.blocked
            || self.workspace().library.draft.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish the current edit before creating a schematic",
            );
            return true;
        }
        self.cancel_native_schematic();
        self.cancel_native_move();
        let state = self.session.workspace_mut();
        state.schematic_authoring.creation = Some(NewSchematicDraft {
            name: String::new(),
            editor: Default::default(),
            focus: 0,
            revision: state.scene.source_revision.clone(),
            tip: state
                .supervision
                .journal
                .accepted_transaction_tip
                .as_deref()
                .and_then(|s| s.parse().ok()),
            refusal: None,
        });
        self.trace_schematic_creation_event("begin", None);
        self.invalidate_frame();
        true
    }
    fn submit_schematic_creation(&mut self) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let Some(draft) = &mut self.session.workspace_mut().schematic_authoring.creation else {
            return false;
        };
        let name = draft.name.trim().to_owned();
        if name.is_empty() || name.chars().any(char::is_control) {
            draft.refusal = Some("Enter a nonempty, single-line name.".into());
            self.invalidate_frame();
            return true;
        }
        self.session
            .workspace_mut()
            .schematic_authoring
            .pending_request = Some(SchematicEditRequest::CreateSheet {
            sheet: eda_engine::substrate::ObjectId::new_v4(),
            name,
        });
        self.queue_native_edit(NativeEditAction::Schematic)
    }
    pub(super) fn handle_schematic_creation_hit(&mut self, target: &HitTarget) -> bool {
        if !matches!(
            target,
            HitTarget::SchematicName
                | HitTarget::SchematicCreate
                | HitTarget::SchematicCancel
                | HitTarget::SchematicPicker
        ) {
            return false;
        }
        let pane = self.workspace().ui.layout.focused;
        self.set_application_focus(ApplicationFocus::Editor(pane));
        if self.native_edit.pending() {
            return true;
        }
        match target {
            HitTarget::SchematicCreate => return self.submit_schematic_creation(),
            HitTarget::SchematicCancel => {
                self.cancel_native_schematic();
            }
            HitTarget::SchematicName => {
                if let Some(d) = &mut self.session.workspace_mut().schematic_authoring.creation {
                    d.focus = 0;
                }
                if let Some(region) = self
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| r.target == *target)
                    .cloned()
                    && let Some((x, _)) = self.last_cursor_pos
                {
                    // The shared controller uses the same scale and shaped caret geometry as paint.
                    let scale = self.scale_factor;
                    self.start_dialog_editor_pointer(region, x, scale);
                    self.finish_dialog_editor_pointer();
                }
            }
            HitTarget::SchematicPicker => {}
            _ => return false,
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_schematic_creation_key(&mut self, event: &KeyEvent) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        match event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.cancel_native_schematic();
            }
            Key::Named(NamedKey::Tab) => {
                let back = self.modifiers.shift_key();
                if let Some(d) = &mut self.session.workspace_mut().schematic_authoring.creation {
                    d.focus = (d.focus + if back { 2 } else { 1 }) % 3;
                }
            }
            Key::Named(NamedKey::Enter) => {
                if self
                    .workspace()
                    .schematic_authoring
                    .creation
                    .as_ref()
                    .is_some_and(|d| d.focus == 1)
                {
                    self.cancel_native_schematic();
                } else {
                    return self.submit_schematic_creation();
                }
            }
            _ => {
                self.handle_shared_editor_key(event);
            }
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn open_created_schematic(&mut self, sheet: eda_engine::substrate::ObjectId) {
        let key = self
            .workspace()
            .schematic_documents
            .rows
            .iter()
            .find(|row| row.key.source_sheet == sheet && row.key.instance_path.is_empty())
            .map(|row| row.key.clone());
        if let Some(key) = key {
            let revision = self.workspace().scene.source_revision.clone();
            self.open_schematic_document(&key, &revision);
        } else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Schematic created but its document view is unavailable; reopen the Project",
            );
        }
    }
}
