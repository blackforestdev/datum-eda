//! Custom metadata configures the existing selected Symbol form and commit path.
use super::*;
use datum_gui_protocol::schematic_authoring::{CustomFieldControl, CustomFieldDraft};

impl Runtime {
    pub(super) fn guard_selected_fields_edit(&self, draft: &FieldsEditDraft) -> Result<(), String> {
        let (basis, target) = selected::target(self.workspace())?;
        let tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .as_ref()
            .and_then(|tip| tip.parse().ok());
        if &basis != draft.basis() || &target != draft.target() || tip != draft.tip() {
            return Err("Selection or Project history changed; cancel and reopen the edit".into());
        }
        Ok(())
    }
    pub(in crate::runtime_schematic) fn handle_custom_field_control(
        &mut self,
        control: CustomFieldControl,
    ) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        let result = (|| {
            let e = &self.workspace().schematic_authoring;
            let draft = e
                .editing
                .as_ref()
                .ok_or("Open the selected Symbol fields first")?;
            self.guard_selected_fields_edit(draft)?;
            if let Some(reason) = datum_gui_protocol::gui_menu_model::native_edit_unavailable_reason(
                self.workspace(),
                "datum.schematic.set_symbol_field",
            ) {
                return Err(reason.into());
            }
            match draft {
                FieldsEditDraft::Symbol(symbol) if control == CustomFieldControl::Open => {
                    if e.reference != symbol.original_reference || e.value != symbol.original_value
                    {
                        return Err(
                            "Apply or cancel Symbol field changes before opening custom metadata"
                                .into(),
                        );
                    }
                    let original = e
                        .source
                        .as_ref()
                        .and_then(|s| s.sheets.get(&symbol.basis.source))
                        .and_then(|s| s.symbols.get(&symbol.target.id))
                        .cloned()
                        .ok_or("Selected Symbol source unavailable")?;
                    if original.reference != symbol.original_reference
                        || original.value != symbol.original_value
                    {
                        return Err("Symbol source changed; cancel and reopen its fields".into());
                    }
                    let field_id = original.fields.first().map(|f| f.uuid);
                    let custom = CustomFieldDraft {
                        symbol: symbol.clone(),
                        original,
                        field_id,
                        remove: false,
                    };
                    custom.check_source(e)?;
                    Ok(FieldsEditDraft::Custom(custom))
                }
                FieldsEditDraft::Custom(custom) => {
                    custom.check_source(e)?;
                    if control == CustomFieldControl::Symbol {
                        custom.require_clean(&e.reference, &e.value)?;
                        return Ok(FieldsEditDraft::Symbol(custom.symbol.clone()));
                    }
                    let mut next = custom.clone();
                    next.navigate(control, &e.reference, &e.value)?;
                    next.check_source(e)?;
                    Ok(FieldsEditDraft::Custom(next))
                }
                _ => Err("Open the selected Symbol's custom metadata first".into()),
            }
        })();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        match result {
            Ok(draft) => {
                (e.reference, e.value) = draft.fields();
                e.reference_editor =
                    datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.reference);
                e.value_editor =
                    datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&e.value);
                e.field = PlacementField::Reference;
                e.editing = Some(draft);
                e.refusal = None;
            }
            Err(reason) => e.refusal = Some(reason),
        }
        self.invalidate_frame();
        true
    }
}
