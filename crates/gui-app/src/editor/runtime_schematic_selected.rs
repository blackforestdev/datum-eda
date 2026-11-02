//! Selected-object edits share native selection, guarded source ownership and the writer.
use super::*;
#[path = "runtime_schematic_selected_label.rs"]
mod label_edit;
use datum_gui_protocol::schematic_authoring::selected::{self, DeleteKind, SymbolEditDraft};

impl Runtime {
    pub(crate) fn selected_symbol_edit_snapshot(
        &self,
    ) -> Option<(SymbolEditDraft, String, String)> {
        let e = &self.workspace().schematic_authoring;
        Some((e.editing.clone()?, e.reference.clone(), e.value.clone()))
    }

    pub(crate) fn restore_selected_symbol_edit(
        &mut self,
        (draft, reference, value): (SymbolEditDraft, String, String),
        reason: &str,
    ) {
        let basis = draft.basis.clone();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.editing = Some(draft);
        e.reference = reference;
        e.value = value;
        e.reference_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.reference);
        e.value_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.value);
        e.picker = true;
        e.refusal = Some(reason.into());
        self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
    }

    fn selected_schematic_action(
        &self,
        edit_fields: bool,
    ) -> Result<
        (
            datum_gui_protocol::AuthoringViewBasis,
            datum_gui_protocol::AuthoredSelectionIdentity,
        ),
        String,
    > {
        self.selected_schematic_idle()?;
        selected::admission(self.workspace(), edit_fields)?;
        selected::target(self.workspace())
    }

    fn selected_schematic_idle(&self) -> Result<(), String> {
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.check_work.pending()
            || self.native_edit.preview.is_some()
            || self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker
            || self.workspace().schematic_authoring.creation.is_some()
            || self.workspace().library.draft.is_some()
            || self.workspace().library.acquisition.is_some()
        {
            return Err("Finish or cancel the current edit before changing the selection".into());
        }
        Ok(())
    }

    pub(crate) fn begin_selected_symbol_edit(&mut self) -> bool {
        let (basis, target) = match self.selected_schematic_action(true) {
            Ok(value) => value,
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                return true;
            }
        };
        let symbol = self
            .workspace()
            .schematic_authoring
            .source
            .as_ref()
            .unwrap()
            .sheets[&basis.source]
            .symbols[&target.id]
            .clone();
        let tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .as_ref()
            .and_then(|tip| tip.parse().ok());
        self.cancel_native_schematic();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.reference = symbol.reference.clone();
        e.value = symbol.value.clone();
        e.reference_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.reference);
        e.value_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.value);
        e.field = PlacementField::Reference;
        e.picker = true;
        e.refusal = None;
        e.editing = Some(SymbolEditDraft {
            target,
            basis: basis.clone(),
            tip,
            original_reference: symbol.reference,
            original_value: symbol.value,
        });
        self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
        self.invalidate_frame();
        true
    }

    pub(super) fn apply_selected_symbol_fields(&mut self) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let request = (|| {
            let e = &self.workspace().schematic_authoring;
            let draft = e
                .editing
                .as_ref()
                .ok_or("No selected Symbol edit is open")?;
            let (basis, target) = selected::target(self.workspace())?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|tip| tip.parse().ok());
            if basis != draft.basis || target != draft.target || tip != draft.tip {
                return Err(
                    "Selection or Project history changed; cancel and reopen the edit".into(),
                );
            }
            if e.source.as_ref().is_some_and(|source| {
                source.sheets.values().any(|sheet| {
                    sheet
                        .symbols
                        .values()
                        .any(|s| s.uuid != target.id && s.reference == e.reference)
                })
            }) {
                return Err("That reference is already in use".into());
            }
            draft.request(&e.reference, &e.value)
        })();
        match request {
            Ok(None) => {
                self.cancel_native_schematic();
                self.invalidate_frame();
                true
            }
            Ok(Some(request)) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .pending_request = Some(request);
                self.queue_native_edit(NativeEditAction::Schematic)
            }
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
                self.invalidate_frame();
                true
            }
        }
    }

    pub(crate) fn delete_selected_schematic(&mut self) -> bool {
        let (basis, target) = match self.selected_schematic_action(false) {
            Ok(value) => value,
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                return true;
            }
        };
        let kind = DeleteKind::from_class(target.class).unwrap();
        self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
        self.session
            .workspace_mut()
            .schematic_authoring
            .pending_request = Some(SchematicEditRequest::DeleteObject {
            object: target.id,
            instance_path: target.instance_path,
            kind,
        });
        self.queue_native_edit(NativeEditAction::Schematic)
    }

    pub(crate) fn delete_selected_schematic_verb(&mut self, expected: &str) -> bool {
        if !selected::admission(self.workspace(), false).is_ok_and(|verb| verb == expected) {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Selection changed; choose the object's Delete action again",
            );
            return true;
        }
        self.delete_selected_schematic()
    }
}
