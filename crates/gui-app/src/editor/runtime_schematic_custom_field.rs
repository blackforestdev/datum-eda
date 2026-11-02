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
                        visibility: None,
                        placing: false,
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
        let base = self.workspace().schematic_scene.clone();
        let e = &mut self.session.workspace_mut().schematic_authoring;
        match result {
            Ok(draft) => {
                if let FieldsEditDraft::Custom(custom) = &draft
                    && custom.placing
                {
                    e.base = base;
                    e.picker = false;
                    e.position = custom
                        .selected()
                        .and_then(|f| f.position)
                        .map(|p| PointNm { x: p.x, y: p.y });
                    e.preview_dirty = true;
                }
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

impl Runtime {
    pub(in crate::runtime_schematic) fn place_custom_field(&mut self, at: PointNm) -> bool {
        let request = (|| {
            let e = &self.workspace().schematic_authoring;
            let fields = e.editing.as_ref().ok_or("Field edit unavailable")?;
            self.guard_selected_fields_edit(fields)?;
            let draft = e.field_placement().ok_or("Field placement unavailable")?;
            draft.check_source(e)?;
            let field = draft.selected().ok_or("Field unavailable")?;
            if field.position == Some(eda_engine::ir::geometry::Point::new(at.x, at.y))
                && draft
                    .visibility
                    .is_none_or(|visible| visible == field.visible)
            {
                return Ok(None);
            }
            draft.point_request(at).map(Some)
        })();
        match request {
            Ok(None) => {
                self.cancel_native_schematic();
                self.invalidate_frame();
                true
            }
            Ok(Some(request)) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.position = Some(at);
                e.preview_dirty = true;
                e.pending_request = Some(request);
                self.prepare_native_schematic_frame();
                self.queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::Schematic,
                )
            }
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
                self.invalidate_frame();
                true
            }
        }
    }
    pub(in crate::runtime_schematic) fn return_custom_field_form(&mut self) -> bool {
        let e = &mut self.session.workspace_mut().schematic_authoring;
        if let Some(FieldsEditDraft::Custom(draft)) = &mut e.editing {
            draft.return_to_form();
        }
        e.picker = true;
        e.position = None;
        e.preview_dirty = false;
        let base = e.base.take();
        if let Some(base) = base {
            self.session.workspace_mut().schematic_scene = Some(base);
            self.invalidate_scene();
        }
        self.invalidate_frame();
        true
    }
}
