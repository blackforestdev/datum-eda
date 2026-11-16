//! Retained assignment context delegates to existing Library and Proposal owners.
use super::*;
use datum_gui_protocol::{
    library_authoring::LibraryDraft,
    library_browser::{
        LibraryControl as C,
        component_resolution::{ComponentResolution, ResolutionAction as A},
    },
    schematic_authoring::{assignment::AssignmentEvidence, selected},
};
impl Runtime {
    pub(crate) fn component_resolution_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        let Some(context) = self
            .workspace()
            .library
            .resolution
            .as_ref()
            .filter(|r| !r.in_library)
        else {
            return false;
        };
        if self.native_edit.pending()
            || self.check_work.pending()
            || event.repeat
            || self.modifiers.control_key()
            || self.modifiers.alt_key()
            || self.modifiers.super_key()
        {
            return true;
        }
        let actions = context.actions(&self.workspace().library);
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                return self.handle_component_resolution(A::Cancel);
            }
            Key::Named(NamedKey::Enter) if !self.modifiers.shift_key() => {
                if let Some(action) = context.focus.filter(|a| actions.contains(a)) {
                    return self.handle_component_resolution(action);
                }
            }
            Key::Named(NamedKey::Tab) => {
                let n = actions.len();
                let index = context
                    .focus
                    .and_then(|f| actions.iter().position(|a| *a == f));
                let next = if self.modifiers.shift_key() {
                    index.map_or(n - 1, |i| (i + n - 1) % n)
                } else {
                    index.map_or(0, |i| (i + 1) % n)
                };
                self.session
                    .workspace_mut()
                    .library
                    .resolution
                    .as_mut()
                    .unwrap()
                    .focus = Some(actions[next]);
                self.invalidate_frame();
            }
            _ => {}
        }
        true
    }
    pub(crate) fn begin_empty_component_resolution(&mut self) -> Result<bool, String> {
        let (_, target) = selected::target(self.workspace())?;
        let evidence = AssignmentEvidence::capture_resolution(self.workspace(), &target)?;
        if !evidence.options.choices.is_empty() {
            return Ok(false);
        }
        self.begin_component_resolution(target, false)?;
        Ok(true)
    }
    pub(crate) fn begin_component_resolution(
        &mut self,
        target: datum_gui_protocol::AuthoredSelectionIdentity,
        transfer: bool,
    ) -> Result<(), String> {
        let state = self.workspace();
        if state.library.draft.is_some()
            || state.library.acquisition.is_some()
            || state.library.resolution.is_some()
        {
            return Err(
                "Finish or cancel current Library work before resolving another component".into(),
            );
        }
        let context = ComponentResolution::capture(state, &target, transfer)?;
        self.session.workspace_mut().library.resolution = Some(context);
        self.session.workspace_mut().ui.board_transfer = None;
        Ok(())
    }
    pub(crate) fn resolution_library_return(&mut self, control: &C) -> bool {
        let b = &self.workspace().library;
        if *control != C::Cancel
            || b.resolution.is_none()
            || b.draft.is_some()
            || b.acquisition.is_some()
        {
            return false;
        }
        self.handle_component_resolution(A::Return)
    }
    pub(crate) fn handle_component_resolution(&mut self, action: A) -> bool {
        let Some(mut context) = self.workspace().library.resolution.clone() else {
            return false;
        };
        if self.native_edit.pending() || self.check_work.pending() {
            return true;
        }
        if !matches!(action, A::Return | A::Cancel)
            && !context.actions(&self.workspace().library).contains(&action)
        {
            return true;
        }
        if action != A::Return {
            context.focus = Some(action);
        }
        let result = (|| -> Result<(), String> {
            if self.workspace().library.draft.is_some()
                || self.workspace().library.acquisition.is_some()
            {
                return Err("Finish or cancel the Library draft or source preview first".into());
            }
            if let Err(reason) = context.verify(self.workspace()) {
                if action == A::Cancel {
                    self.session.workspace_mut().library.resolution = None;
                    return Ok(());
                }
                return Err(reason);
            }
            self.session.workspace_mut().ui.layout.focused = context.basis.pane;
            self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                context.basis.pane,
            ));
            if action != A::Cancel {
                AssignmentEvidence::admit_target(self.workspace(), &context.evidence.target)?;
            }
            match action {
                A::Cancel => {
                    self.session.workspace_mut().library.resolution = None;
                    if context.transfer {
                        self.begin_board_transfer();
                    }
                }
                A::Return => {
                    context.in_library = false;
                    context.evidence = AssignmentEvidence::capture_resolution(
                        self.workspace(),
                        &context.evidence.target,
                    )?;
                    context.refusal = None;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                }
                A::Assign => {
                    let review = datum_gui_protocol::schematic_authoring::delete_review::DeleteReview::capture_assignment_target(self.workspace(), &context.evidence.target)?;
                    if !self.prepare_native_circuit(false) {
                        return Ok(());
                    }
                    self.session.workspace_mut().library.resolution = None;
                    let editor = &mut self.session.workspace_mut().schematic_authoring;
                    editor.chosen = None;
                    editor.label = Some(datum_gui_protocol::schematic_authoring::LabelDraft::new(
                        eda_engine::schematic::LabelKind::Local,
                    ));
                    editor.assignment_draft = Some(review);
                    editor.refusal = None;
                }
                A::Footprint => {
                    context.in_library = true;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                    self.open_library();
                    let pane = self.workspace().ui.layout.focused;
                    self.session
                        .workspace_mut()
                        .library
                        .resolution
                        .as_mut()
                        .unwrap()
                        .library_pane = Some(pane);
                    self.session.workspace_mut().library.kind = 5;
                    self.activate_library(C::Acquire);
                }
                A::Entity | A::Part | A::Map => {
                    let kind = match action {
                        A::Entity => "entities",
                        A::Part => "parts",
                        _ => "pin_pad_maps",
                    };
                    let model = self
                        .workspace()
                        .library
                        .basis
                        .as_ref()
                        .ok_or("Library source unavailable")?;
                    let mut draft = LibraryDraft::begin(
                        &self.workspace().library,
                        None,
                        kind,
                        model.model_revision.0.clone(),
                        eda_engine::revision::transaction_tip(&model.journal),
                    )?;
                    if action == A::Entity {
                        let symbol: eda_engine::substrate::ObjectId = context
                            .evidence
                            .options
                            .original
                            .lib_id
                            .as_deref()
                            .ok_or("Saved Symbol missing")?
                            .parse()
                            .map_err(|_| "Saved Symbol invalid")?;
                        let raw = self
                            .workspace()
                            .library
                            .entries
                            .iter()
                            .find(|e| e.id == symbol && e.kind == "symbols" && e.warning.is_none())
                            .ok_or("Saved Symbol unavailable")?;
                        let unit = raw
                            .raw
                            .get("unit")
                            .and_then(serde_json::Value::as_str)
                            .and_then(|s| s.parse().ok())
                            .ok_or("Saved Symbol Unit unavailable")?;
                        draft.retain_entity_symbol(symbol, unit)?;
                    }
                    context.authored.push((kind.into(), draft.id));
                    context.in_library = true;
                    self.session.workspace_mut().library.resolution = Some(context.clone());
                    self.open_library();
                    let pane = self.workspace().ui.layout.focused;
                    self.session
                        .workspace_mut()
                        .library
                        .resolution
                        .as_mut()
                        .unwrap()
                        .library_pane = Some(pane);
                    let b = &mut self.session.workspace_mut().library;
                    b.draft = Some(draft);
                    b.focus = Some(C::Field(0));
                    b.refusal = None;
                }
            }
            Ok(())
        })();
        if let Err(reason) = result {
            if let Some(c) = &mut self.session.workspace_mut().library.resolution {
                c.refusal = Some(reason.clone());
            }
            self.session.workspace_mut().library.refusal = Some(reason);
        }
        self.invalidate_frame();
        true
    }
}
