//! Selected-object edits share native selection, guarded source ownership and the writer.
use super::*;
#[path = "runtime_schematic_custom_field.rs"]
mod custom_field;
#[path = "runtime_schematic_delete_review.rs"]
mod delete_review;
#[path = "runtime_schematic_selected_label.rs"]
mod label_edit;
use datum_gui_protocol::schematic_authoring::{
    BusEditDraft, FieldsEditDraft,
    selected::{self, DeleteKind, SymbolEditDraft},
};

impl Runtime {
    pub(crate) fn selected_symbol_edit_snapshot(
        &self,
    ) -> Option<(FieldsEditDraft, String, String)> {
        let e = &self.workspace().schematic_authoring;
        Some((e.editing.clone()?, e.reference.clone(), e.value.clone()))
    }

    pub(crate) fn restore_selected_symbol_edit(
        &mut self,
        (mut draft, reference, value): (FieldsEditDraft, String, String),
        reason: &str,
    ) {
        let placement_canceled = match &mut draft {
            FieldsEditDraft::Custom(draft) => draft.return_to_form(),
            _ => false,
        };
        let basis = draft.basis().clone();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.bus = matches!(draft, FieldsEditDraft::Bus(_));
        e.editing = Some(draft);
        e.reference = reference;
        e.value = value;
        e.reference_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.reference);
        e.value_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.value);
        e.picker = true;
        e.refusal = Some(if placement_canceled {
            format!("{reason}; placement canceled. Choose Show or Place text to retry")
        } else {
            reason.into()
        });
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

    pub(super) fn selected_schematic_idle(&self) -> Result<(), String> {
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.check_work.pending()
            || self.native_edit.preview.is_some()
            || self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker
            || self.workspace().schematic_authoring.creation.is_some()
            || self.workspace().schematic_authoring.delete_review.is_some()
            || self.workspace().library.draft.is_some()
            || self.workspace().library.acquisition.is_some()
        {
            return Err("Finish or cancel the current edit before changing the selection".into());
        }
        Ok(())
    }

    pub(crate) fn begin_selected_symbol_edit(&mut self) -> bool {
        self.begin_selected_fields_edit(false)
    }
    pub(crate) fn begin_selected_bus_edit(&mut self) -> bool {
        self.begin_selected_fields_edit(true)
    }
    fn begin_selected_fields_edit(&mut self, bus: bool) -> bool {
        let prepare = (|| {
            self.selected_schematic_idle()?;
            if bus {
                selected::bus_admission(self.workspace())?;
            } else {
                selected::admission(self.workspace(), true)?;
            }
            let (basis, target) = selected::target(self.workspace())?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok());
            let sheet = self
                .workspace()
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&basis.source))
                .ok_or("Selected source unavailable")?;
            let draft = if bus {
                FieldsEditDraft::Bus(BusEditDraft {
                    entries: {
                        let mut entries: Vec<_> = sheet
                            .bus_entries
                            .values()
                            .filter(|e| e.bus == target.id)
                            .cloned()
                            .collect();
                        entries.sort_by_key(|e| e.uuid);
                        entries
                    },
                    original: sheet
                        .buses
                        .get(&target.id)
                        .cloned()
                        .ok_or("Selected Bus unavailable")?,
                    target,
                    basis,
                    tip,
                })
            } else {
                let symbol = sheet
                    .symbols
                    .get(&target.id)
                    .ok_or("Selected Symbol unavailable")?;
                FieldsEditDraft::Symbol(SymbolEditDraft {
                    original_reference: symbol.reference.clone(),
                    original_value: symbol.value.clone(),
                    target,
                    basis,
                    tip,
                })
            };
            Ok::<_, String>(draft)
        })();
        let draft = match prepare {
            Ok(draft) => draft,
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                return true;
            }
        };
        self.cancel_native_schematic();
        let basis = draft.basis().clone();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        (e.reference, e.value) = draft.fields();
        e.reference_editor =
            datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.reference);
        e.value_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.value);
        e.field = PlacementField::Reference;
        e.bus = bus;
        e.picker = true;
        e.refusal = None;
        e.editing = Some(draft);
        self.session.workspace_mut().authoring.gesture.view_basis = Some(basis);
        self.invalidate_frame();
        true
    }

    pub(super) fn apply_selected_symbol_fields(&mut self) -> bool {
        self.apply_selected_fields_action(false)
    }
    pub(super) fn delete_selected_bus_entry(&mut self) -> bool {
        self.apply_selected_fields_action(true)
    }
    fn apply_selected_fields_action(&mut self, delete_entry: bool) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let request = (|| {
            let e = &self.workspace().schematic_authoring;
            let draft = e
                .editing
                .as_ref()
                .ok_or("No selected Symbol edit is open")?;
            self.guard_selected_fields_edit(draft)?;
            if delete_entry {
                let FieldsEditDraft::Bus(bus) = draft else {
                    return Err("Select a Bus entry in the Bus form".into());
                };
                Ok(Some(bus.delete_entry(e)?))
            } else {
                draft.request(e)
            }
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
        if matches!(&self.workspace().selection,datum_gui_protocol::SelectionTarget::Native(view) if view.members.len()>1)
        {
            return self.begin_schematic_delete_review();
        }
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
