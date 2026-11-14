//! The active-sheet table configures shared text input and Proposal confirmation.
use super::*;
use datum_gui_protocol::schematic_authoring::{
    PlacementField, delete_review::DeleteReview, field_table::FieldTableControl as C,
};
use winit::keyboard::{Key, NamedKey};

impl Runtime {
    pub(crate) fn begin_schematic_field_table(&mut self) -> bool {
        let result = self
            .selected_schematic_idle()
            .and_then(|()| DeleteReview::capture_field_table(self.workspace()));
        match result {
            Ok(review) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .delete_review = Some(review);
                self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(
                    self.workspace().ui.layout.focused,
                ));
            }
            Err(reason) => self.log_console_refusal(ConsoleFeedbackSource::Tool, reason),
        }
        self.invalidate_frame();
        true
    }
    fn field_table_visible_rows(&self) -> usize {
        self.presented_hits
            .regions()
            .iter()
            .filter_map(|r| match r.target {
                HitTarget::SchematicFieldTable(C::Cell(row, _)) => Some(row),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            .max(1)
    }
    pub(crate) fn handle_field_table_scroll(&mut self, lines: f32) -> Option<bool> {
        let table = self
            .workspace()
            .schematic_authoring
            .delete_review
            .as_ref()?
            .fields
            .as_ref()?;
        let (x, y) = self.last_cursor_pos?;
        let within = self.presented_hits.regions().iter().any(|r| {
            r.rect.contains(x, y)
                && matches!(
                    r.target,
                    HitTarget::SchematicPicker | HitTarget::SchematicFieldTable(_)
                )
        });
        if !within {
            return None;
        }
        if self.native_edit.pending() {
            return Some(false);
        }
        let visible = self.field_table_visible_rows();
        let next = datum_gui_viewport::scroll::scrolled_row_offset(
            table.offset,
            table.rows.len(),
            visible,
            lines,
        );
        let changed = next != table.offset;
        if changed {
            self.handle_field_table_control(C::Page(next));
        }
        Some(changed)
    }
    pub(crate) fn handle_field_table_control(&mut self, control: C) -> bool {
        if control == C::Cancel
            && self
                .workspace()
                .schematic_authoring
                .delete_review
                .as_ref()
                .is_some_and(|review| review.is_board_revision())
        {
            return self.confirm_delete_review(false);
        }
        if self.native_edit.pending() {
            return true;
        }
        if control == C::Cancel {
            return self.confirm_delete_review(false);
        }
        if control == C::Accept {
            return self.confirm_delete_review(true);
        }
        if control == C::Review {
            let Some(mut review) = self.workspace().schematic_authoring.delete_review.clone()
            else {
                return false;
            };
            let result = review.stage_field_table(self.workspace());
            match &result {
                Err(reason) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, reason.clone());
                    review.refusal = Some(reason.clone());
                }
                Ok(false) => review.refusal = Some("No changed fields to review".into()),
                _ => {}
            }
            self.session
                .workspace_mut()
                .schematic_authoring
                .delete_review = Some(review);
            if result == Ok(true) {
                return self.queue_native_edit(NativeEditAction::Proposal);
            }
            self.invalidate_frame();
            return true;
        }
        let visible = self.field_table_visible_rows();
        let Some(table) = self
            .session
            .workspace_mut()
            .schematic_authoring
            .delete_review
            .as_mut()
            .and_then(|r| r.fields.as_mut())
        else {
            return false;
        };
        match control {
            C::Cell(row, field) => table.focus(row, field, visible),
            C::Page(first) => {
                table.offset = first.min(table.rows.len().saturating_sub(1));
                table.focus(table.offset, table.field, visible);
            }
            _ => {}
        }
        if matches!(control, C::Cell(..)) {
            if let Some(region) = self
                .presented_hits
                .regions()
                .iter()
                .find(|r| r.target == HitTarget::SchematicFieldTable(control))
                .cloned()
                && let Some((x, _)) = self.last_cursor_pos
            {
                self.start_dialog_editor_pointer(region, x, self.scale_factor);
                self.finish_dialog_editor_pointer();
            }
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_field_table_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        let Some(table) = self
            .workspace()
            .schematic_authoring
            .delete_review
            .as_ref()
            .and_then(|r| r.fields.as_ref())
        else {
            return false;
        };
        let editable = table.editable();
        let (row, field, count) = (table.focus_row, table.field, table.rows.len());
        let visible = self.field_table_visible_rows();
        let cell = row * 2 + usize::from(field == PlacementField::Value);
        let target = match &event.logical_key {
            Key::Named(NamedKey::Escape) => return self.confirm_delete_review(false),
            Key::Named(NamedKey::Enter) if editable => {
                return self.handle_field_table_control(C::Review);
            }
            Key::Named(NamedKey::Tab) if editable => {
                let next = if self.modifiers.shift_key() {
                    (cell + count * 2 - 1) % (count * 2)
                } else {
                    (cell + 1) % (count * 2)
                };
                Some((
                    next / 2,
                    if next % 2 == 0 {
                        PlacementField::Reference
                    } else {
                        PlacementField::Value
                    },
                ))
            }
            Key::Named(NamedKey::Tab) => return false, // Shared explicit Proposal acceptance.
            Key::Named(NamedKey::F6) => Some((
                row,
                if field == PlacementField::Reference {
                    PlacementField::Value
                } else {
                    PlacementField::Reference
                },
            )),
            Key::Named(NamedKey::ArrowUp) => Some((row.saturating_sub(1), field)),
            Key::Named(NamedKey::ArrowDown) => Some(((row + 1).min(count - 1), field)),
            Key::Named(NamedKey::PageUp) => Some((row.saturating_sub(visible), field)),
            Key::Named(NamedKey::PageDown) => Some(((row + visible).min(count - 1), field)),
            _ => None,
        };
        if let Some((row, field)) = target {
            self.session
                .workspace_mut()
                .schematic_authoring
                .delete_review
                .as_mut()
                .unwrap()
                .fields
                .as_mut()
                .unwrap()
                .focus(row, field, visible);
            self.invalidate_frame();
        } else {
            self.handle_shared_editor_key(event);
        }
        true
    }
}
