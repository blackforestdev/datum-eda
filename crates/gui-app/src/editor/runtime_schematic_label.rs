//! Label and power-port forms configure the shared schematic gesture.
use super::*;
use datum_gui_protocol::schematic_authoring::LabelDraft;
use eda_engine::schematic::LabelKind;

impl Runtime {
    pub(crate) fn begin_native_label(&mut self, kind: LabelKind) -> bool {
        if !self.prepare_native_schematic(false) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.chosen = None;
        e.label = Some(LabelDraft::new(kind));
        e.refusal = None;
        self.invalidate_frame();
        true
    }

    pub(crate) fn begin_native_port(&mut self) -> bool {
        if !self.prepare_native_schematic(false) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.chosen = None;
        e.label = Some(LabelDraft::port(
            eda_engine::schematic::PortDirection::Input,
        ));
        e.refusal = None;
        self.invalidate_frame();
        true
    }

    fn start_label_preview(&mut self) -> bool {
        if self.workspace().schematic_authoring.named_edit.is_some() {
            return self.apply_selected_label_fields();
        }
        if !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused)) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(label) = &mut e.label else {
            return false;
        };
        if label.name.trim().is_empty() || label.name.chars().any(char::is_control) {
            e.refusal = Some("Enter a nonempty single-line name".into());
            label.focus = 0;
        } else {
            label.name = label.name.trim().to_owned();
            label.editor =
                datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&label.name);
            e.picker = false;
            e.preview_dirty = true;
            e.refusal = None;
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_label_chrome(&mut self, target: &HitTarget) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        match target {
            HitTarget::SchematicCancel => {
                self.cancel_native_schematic();
            }
            HitTarget::SchematicLabelPreview | HitTarget::SchematicEditApply => {
                return self.start_label_preview();
            }
            HitTarget::SchematicLabelKind => {
                if self.workspace().schematic_authoring.bus_entry.is_some() {
                    return self.cycle_bus_entry_choice(true);
                }
                let e = &mut self.session.workspace_mut().schematic_authoring;
                if let Some(draft) = &mut e.label {
                    if draft.port_direction.is_some() {
                        draft.cycle_kind();
                        e.preview_dirty |= e.named_edit.is_some();
                    } else if e.named_edit.is_some() {
                        draft.kind = match draft.kind {
                            LabelKind::Local => LabelKind::Global,
                            LabelKind::Global => LabelKind::Hierarchical,
                            LabelKind::Hierarchical => LabelKind::Power,
                            LabelKind::Power => LabelKind::Local,
                        };
                        e.preview_dirty = true;
                    } else {
                        draft.cycle_kind();
                    }
                    draft.focus = 1;
                }
            }
            HitTarget::SchematicLabelName => {
                if self.workspace().schematic_authoring.bus_entry.is_some() {
                    return self.cycle_bus_entry_choice(false);
                }
                if let Some(draft) = &mut self.session.workspace_mut().schematic_authoring.label {
                    draft.focus = 0;
                }
                if let Some(region) = self
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| r.target == *target)
                    .cloned()
                    && let Some((x, _)) = self.last_cursor_pos
                {
                    self.start_dialog_editor_pointer(region, x, self.scale_factor);
                    self.finish_dialog_editor_pointer();
                }
            }
            HitTarget::SchematicPicker => {}
            _ => return false,
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_label_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        let picker = self.workspace().schematic_authoring.picker;
        let focus = self
            .workspace()
            .schematic_authoring
            .label
            .as_ref()
            .unwrap()
            .focus;
        match event.logical_key {
            Key::Named(NamedKey::Escape) => {
                self.cancel_native_schematic();
                self.set_workspace_tool(WorkspaceTool::Select);
            }
            Key::Named(NamedKey::Tab) => {
                let backward = self.modifiers.shift_key();
                let e = &mut self.session.workspace_mut().schematic_authoring;
                if !picker {
                    e.picker = true;
                } else if let Some(draft) = &mut e.label {
                    draft.focus = (draft.focus + if backward { 3 } else { 1 }) % 4;
                }
            }
            Key::Named(NamedKey::Enter) if picker => {
                return match focus {
                    1 => self.handle_label_chrome(&HitTarget::SchematicLabelKind),
                    2 => self.handle_label_chrome(&HitTarget::SchematicCancel),
                    _ => self.start_label_preview(),
                };
            }
            Key::Named(NamedKey::Space | NamedKey::ArrowLeft | NamedKey::ArrowRight)
                if picker && focus == 1 =>
            {
                return self.handle_label_chrome(&HitTarget::SchematicLabelKind);
            }
            Key::Named(NamedKey::Space | NamedKey::ArrowLeft | NamedKey::ArrowRight)
                if picker
                    && focus == 0
                    && self.workspace().schematic_authoring.bus_entry.is_some() =>
            {
                return self.cycle_bus_entry_choice(false);
            }
            _ if picker
                && focus == 0
                && self.workspace().schematic_authoring.bus_entry.is_some() => {}
            _ if picker && focus == 0 => {
                self.handle_shared_editor_key(event);
            }
            _ if !picker => return false,
            _ => {}
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn place_native_label(&mut self, at: PointNm) -> bool {
        if self.workspace().schematic_authoring.named_edit.is_some() {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(label) = &e.label else { return false };
        let Some(sheet) = e.sheet else { return true };
        e.position = Some(at);
        e.preview_dirty = true;
        match label.placement_request(sheet, at) {
            Ok(request) => e.pending_request = Some(request),
            Err(reason) => {
                e.refusal = Some(reason);
                self.invalidate_frame();
                return true;
            }
        }
        self.prepare_native_schematic_frame();
        self.queue_native_edit(NativeEditAction::Schematic)
    }
}
