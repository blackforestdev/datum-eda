//! Existing schematic key ownership, preserving shared field/Move priority.
use super::*;
impl Runtime {
    pub(crate) fn handle_schematic_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        self.trace_schematic_creation_event("schematic key", Some(event));
        if !matches!(
            self.application_focus(),
            datum_gui_protocol::ApplicationFocus::Editor(_)
        ) {
            return false;
        }
        if self.consume_coordinate_escape_release(event) {
            return true;
        }
        if self.component_picker_open() {
            return self.handle_library_key(event);
        }
        if event.state != ElementState::Pressed {
            return false;
        }
        if self.handle_move_coordinate_key(event) {
            return true;
        }
        if self.workspace().schematic_authoring.delete_review.is_some() {
            return self.handle_delete_review_key(event);
        }
        if self.handle_symbol_move_pose_key(event) {
            return true;
        }
        if self.workspace().schematic_authoring.creation.is_some() {
            return self.handle_schematic_creation_key(event);
        }
        if self.workspace().schematic_authoring.child_draft.is_some() {
            return self.child_sheet_key(event);
        }
        if !self.workspace().schematic_authoring.active()
            && !self.workspace().schematic_authoring.picker
        {
            if matches!(
                self.workspace().ui.layout.focused_content(),
                PaneContent::Schematic | PaneContent::Board
            ) && matches!(event.logical_key, Key::Named(NamedKey::Delete))
                && !self.modifiers.control_key()
                && !self.modifiers.alt_key()
                && !self.modifiers.super_key()
            {
                return self.delete_selected_schematic();
            }
            if self.workspace().ui.layout.focused_content() == PaneContent::Schematic
                && !self.modifiers.control_key()
                && matches!(&event.logical_key,Key::Character(key) if key.eq_ignore_ascii_case("w"))
            {
                return self.begin_native_schematic(true);
            }
            return false;
        }
        if self.native_edit.pending() {
            return true;
        }
        if !self.workspace().schematic_authoring.picker
            && !self.workspace().schematic_authoring.wire
            && !self.modifiers.control_key()
            && !self.modifiers.alt_key()
            && !self.modifiers.super_key()
            && matches!(&event.logical_key, Key::Character(key) if key.eq_ignore_ascii_case("w"))
        {
            return self.begin_native_schematic(true);
        }
        if self
            .workspace()
            .schematic_authoring
            .field_placement()
            .is_some()
        {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    return self
                        .workspace()
                        .schematic_authoring
                        .position
                        .map(|at| self.place_custom_field(at))
                        .unwrap_or(true);
                }
                Key::Named(NamedKey::Tab) => return self.return_custom_field_form(),
                Key::Character(key)
                    if ["r", "x", "y"].iter().any(|s| key.eq_ignore_ascii_case(s)) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        if self.workspace().schematic_authoring.marker.is_some() {
            return self.handle_marker_key(event);
        }
        if self.workspace().schematic_authoring.label.is_some() {
            return self.handle_label_key(event);
        }
        let picker = self.workspace().schematic_authoring.picker;
        if picker
            && matches!(
                self.workspace().schematic_authoring.editing,
                Some(datum_gui_protocol::schematic_authoring::FieldsEditDraft::Custom(_))
            )
        {
            use datum_gui_protocol::schematic_authoring::CustomFieldControl;
            match event.logical_key {
                Key::Named(NamedKey::PageUp) => {
                    return self.handle_custom_field_control(CustomFieldControl::Previous);
                }
                Key::Named(NamedKey::PageDown) => {
                    return self.handle_custom_field_control(CustomFieldControl::Next);
                }
                _ => {}
            }
        }
        if picker && self.handle_shared_editor_key(event) {
            return true;
        }
        if !self.modifiers.control_key()
            && !self.modifiers.alt_key()
            && !self.modifiers.super_key()
            && let Key::Character(key) = &event.logical_key
        {
            let transform = match key.to_ascii_lowercase().as_str() {
                "r" => Some(PlacementTransform::Rotate),
                "x" => Some(PlacementTransform::FlipX),
                "y" => Some(PlacementTransform::FlipY),
                _ => None,
            };
            if let Some(transform) = transform
                && self
                    .session
                    .workspace_mut()
                    .schematic_authoring
                    .transform_placement(transform)
            {
                self.invalidate_frame();
                return true;
            }
        }
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.cancel_native_schematic();
                self.set_workspace_tool(WorkspaceTool::Select);
            }
            Key::Named(NamedKey::Enter)
                if picker && self.workspace().schematic_authoring.editing.is_some() =>
            {
                return self.apply_selected_symbol_fields();
            }
            Key::Named(NamedKey::Enter)
                if picker
                    && !self.workspace().schematic_authoring.bus
                    && self.workspace().schematic_authoring.gate_draft.is_none()
                    && self
                        .workspace()
                        .schematic_authoring
                        .picker_choices()
                        .is_empty() =>
            {
                return self.browse_schematic_libraries();
            }
            Key::Named(NamedKey::Enter) if picker => return self.start_symbol_preview(),
            Key::Named(NamedKey::Enter) if self.workspace().schematic_authoring.wire => {
                return self.finish_native_wire();
            }
            Key::Named(NamedKey::Enter)
                if !picker
                    && self.modifiers.is_empty()
                    && self.workspace().schematic_authoring.chosen.is_some()
                    && self
                        .workspace()
                        .schematic_authoring
                        .design_placement
                        .is_none()
                    && self.workspace().schematic_authoring.gate_draft.is_none() =>
            {
                // The accepting menu press is consumed there. Held-key repeats
                // must not turn its new ghost into an unintended placement.
                if event.repeat || self.native_edit.pending() {
                    return true;
                }
                if let Some(screen) = self.last_cursor_pos
                    && let Some((world, SceneSurface::Schematic)) = self
                        .prepared_scene()
                        .and_then(|scene| scene.world_point_at_screen(screen.0, screen.1))
                {
                    return self.native_schematic_click(world, screen);
                }
                return true;
            }
            Key::Named(NamedKey::Tab)
                if !picker
                    && !self.workspace().schematic_authoring.wire
                    && self
                        .workspace()
                        .schematic_authoring
                        .design_placement
                        .is_none() =>
            {
                self.session.workspace_mut().schematic_authoring.picker = true;
            }
            Key::Named(NamedKey::Tab) if picker => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.field = if e.field == PlacementField::Reference && !e.components.active {
                    PlacementField::Value
                } else {
                    PlacementField::Reference
                };
            }
            // Placement owns its editing keys, not the whole keyboard. Let
            // shared tool/navigation shortcuts switch away from a preview.
            _ if !picker => return false,
            _ => {}
        }
        self.invalidate_frame();
        true
    }
}
