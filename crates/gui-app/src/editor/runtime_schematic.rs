//! Native schematic gestures reuse the existing asynchronous canonical worker.
#[path = "runtime_schematic/keys.rs"]
mod keys;
use super::*;
#[path = "runtime_design_clipboard.rs"]
mod design_clipboard;
#[path = "runtime_schematic_snap.rs"]
mod snap;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use datum_gui_protocol::schematic_authoring::{
    PlacementField, PlacementTransform, SchematicEditRequest, bus_fields,
};
#[path = "runtime_schematic_bus_entry.rs"]
mod bus_entry;
#[path = "runtime_schematic_interface_binding.rs"]
mod interface_binding;
#[path = "runtime_schematic_junction.rs"]
mod junction;
#[path = "runtime_schematic_label.rs"]
mod label;
#[path = "runtime_schematic_library.rs"]
mod library;
#[path = "runtime_schematic_marker.rs"]
mod marker;
#[path = "runtime_schematic_selected.rs"]
mod selected;
impl Runtime {
    pub(super) fn begin_native_schematic(&mut self, wire: bool) -> bool {
        if self.workspace().library.acquisition.is_some()
            || self.workspace().library.draft.is_some()
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Finish or cancel the current library action first",
            );
            return true;
        }
        if !self.prepare_native_circuit(wire) {
            return true;
        }
        if !wire {
            self.session
                .workspace_mut()
                .schematic_authoring
                .begin_component_placement();
            self.open_component_picker();
            self.invalidate_frame();
        }
        true
    }
    pub(super) fn prepare_native_circuit(&mut self, wire: bool) -> bool {
        if !self.prepare_native_schematic_in(wire, true) {
            return false;
        }
        let basis = self.workspace().authoring.gesture.view_basis.clone();
        self.session
            .workspace_mut()
            .schematic_authoring
            .circuit_basis = basis;
        true
    }
    pub(crate) fn begin_native_bus(&mut self) -> bool {
        if !self.prepare_native_circuit(false) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.bus = true;
        e.chosen = None;
        e.reference.clear();
        e.value.clear();
        e.reference_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end("");
        e.value_editor = datum_gui_protocol::single_line_editor::SingleLineEditor::at_end("");
        self.invalidate_frame();
        true
    }
    pub(super) fn prepare_native_schematic(&mut self, wire: bool) -> bool {
        self.prepare_native_schematic_in(wire, false)
    }
    pub(super) fn prepare_native_schematic_in(&mut self, wire: bool, qualified: bool) -> bool {
        if self.workspace().schematic_authoring.delete_review.is_some() {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Accept or dismiss the Proposal first",
            );
            return false;
        }
        if !self.native_edit_available()
            || self.native_edit.pending()
            || self.check_work.pending()
            || self.workspace().ui.layout.focused_content() != PaneContent::Schematic
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Focus a native schematic sheet with no pending edit",
            );
            return false;
        }
        let view_basis = match if qualified {
            datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(
                self.workspace(),
                self.workspace().ui.layout.focused,
            )
        } else {
            self.capture_native_authoring_view(Some(self.workspace().ui.layout.focused))
        } {
            Ok(basis) => basis,
            Err(reason) => {
                self.refuse_native_authoring_view(reason);
                return false;
            }
        };
        self.cancel_native_schematic();
        self.cancel_native_move();
        let Some(base) = (if qualified {
            datum_gui_protocol::camera_scene_for_pane(self.workspace(), view_basis.pane).cloned()
        } else {
            self.workspace().schematic_scene.clone()
        }) else {
            return false;
        };
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        if editor.sheet.is_none() {
            return false;
        }
        editor.base = Some(base);
        editor.wire = wire;
        editor.picker = !wire;
        if !wire && !editor.choices.is_empty() {
            editor.select(0);
        }
        self.session.workspace_mut().authoring.gesture.view_basis = Some(view_basis);
        self.invalidate_frame();
        true
    }
    pub(super) fn cancel_native_schematic(&mut self) -> bool {
        if self.native_edit.pending() {
            return false;
        }
        let child = self.workspace().schematic_authoring.child_draft.is_some()
            || self.workspace().schematic_authoring.circuit_basis.is_some();
        self.session.workspace_mut().schematic_authoring.creation = None;
        let was_active = self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker
            || child;
        let base = self.session.workspace_mut().schematic_authoring.cancel();
        if was_active {
            self.session.workspace_mut().authoring.gesture.view_basis = None;
        }
        if let Some(base) = base {
            if !child {
                self.session.workspace_mut().schematic_scene = Some(base);
            }
            self.invalidate_scene();
            true
        } else {
            false
        }
    }
    fn start_symbol_preview(&mut self) -> bool {
        if !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused)) {
            return true;
        }
        let editor = &mut self.session.workspace_mut().schematic_authoring;
        if editor.bus {
            match bus_fields(&editor.reference, &editor.value) {
                Ok(_) => {
                    editor.picker = false;
                    editor.wire = true;
                    editor.preview_dirty = true;
                    editor.refusal = None;
                }
                Err(reason) => editor.refusal = Some(reason),
            }
        } else if let Some(result) = editor.start_gate_preview() {
            if let Err(reason) = result {
                editor.refusal = Some(reason);
            }
        } else if let Some(reason) = editor.component_placement_refusal() {
            editor.refusal = Some(reason);
        } else if editor.chosen.is_none()
            || editor.reference.trim().is_empty()
            || editor.taken_references.contains(&editor.reference)
        {
            editor.refusal = Some("Choose an available symbol and a unique reference".into());
        } else {
            editor.picker = false;
            editor.preview_dirty = true;
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_schematic_chrome(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::MoveCoordinates => return self.open_move_coordinates(),
            HitTarget::CoordinateEntry(control) => return self.coordinate_field_click(*control),
            _ => {}
        }
        if self.workspace().schematic_authoring.creation.is_some() {
            return self.handle_schematic_creation_hit(target);
        }
        if (self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker)
            && !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused))
        {
            return true;
        }
        if self.workspace().schematic_authoring.label.is_some() {
            return self.handle_label_chrome(target);
        }
        if self.component_picker_open()
            && let HitTarget::Library(control) = target
        {
            return self.activate_library_pointer(control.clone());
        }
        match target {
            HitTarget::SchematicCustomField(control) => {
                return self.handle_custom_field_control(*control);
            }
            HitTarget::SchematicBusEntryChoice => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                if let Some(datum_gui_protocol::schematic_authoring::FieldsEditDraft::Bus(draft)) =
                    &e.editing
                {
                    if !draft.entries.is_empty() {
                        e.bus_entry_choice = (e.bus_entry_choice + 1) % draft.entries.len();
                    }
                }
            }
            HitTarget::SchematicBusEntryDelete => return self.delete_selected_bus_entry(),
            HitTarget::SchematicBrowseLibraries => {
                if self.workspace().schematic_authoring.gate_draft.is_some() {
                    return true;
                }
                return self.browse_schematic_libraries();
            }
            HitTarget::SchematicLibraryChoice(index) => {
                if self.choose_schematic_gate(*index) {
                    return true;
                }
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .select(*index);
            }
            HitTarget::SchematicLibraryPage(forward) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                if *forward && e.choice_offset + 4 < e.picker_choices().len() {
                    e.choice_offset += 4;
                }
                if !*forward {
                    e.choice_offset = e.choice_offset.saturating_sub(4);
                }
            }
            HitTarget::SchematicReference | HitTarget::SchematicValue => {
                if *target == HitTarget::SchematicValue
                    && self.workspace().schematic_authoring.components.active
                {
                    return true;
                }
                self.session.workspace_mut().schematic_authoring.field =
                    if *target == HitTarget::SchematicReference {
                        PlacementField::Reference
                    } else {
                        PlacementField::Value
                    };
                if let Some(region) = self
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| &r.target == target)
                    .cloned()
                    && let Some((x, _)) = self.last_cursor_pos
                {
                    self.start_dialog_editor_pointer(region, x, self.scale_factor);
                    self.finish_dialog_editor_pointer();
                }
            }
            HitTarget::SchematicPlace => {
                if self.component_picker_open() {
                    return self.confirm_component_picker();
                }
                return self.start_symbol_preview();
            }
            HitTarget::SchematicEditApply => return self.apply_selected_symbol_fields(),
            HitTarget::SchematicCancel => {
                self.cancel_native_schematic();
            }
            HitTarget::SchematicFinishWire => return self.finish_native_wire(),
            HitTarget::SchematicPicker => {}
            _ => return false,
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn native_schematic_pointer(&mut self, screen: (f32, f32)) -> bool {
        if self.workspace().schematic_authoring.named_edit.is_some() {
            return true;
        }
        if self
            .native_edit
            .preview
            .as_ref()
            .is_some_and(|p| p.is_schematic())
        {
            if let Some((world, SceneSurface::Schematic)) = self
                .prepared_scene()
                .and_then(|scene| scene.world_point_at_screen(screen.0, screen.1))
            {
                return self.native_move_pointer(world);
            }
            // Never feed Board coordinates into a Schematic Move.
            return true;
        }
        if !self.workspace().schematic_authoring.active() || self.native_edit.pending() {
            return false;
        }
        let resolved = self
            .prepared_scene()
            .and_then(|scene| scene.world_point_at_screen(screen.0, screen.1));
        if self
            .workspace()
            .schematic_authoring
            .design_placement
            .is_some()
        {
            crate::append_gui_verbose_diagnostic_line(|| {
                format!("clipboard pointer screen={screen:?} resolved={resolved:?}")
            });
        }
        let Some((world, SceneSurface::Schematic)) = resolved else {
            return false;
        };
        if !self.guard_native_authoring_view(self.pane_at_screen(screen.0, screen.1)) {
            return true;
        }
        let snapped = self.schematic_command_point(world, screen);
        if self
            .workspace()
            .schematic_authoring
            .design_placement
            .is_some()
        {
            crate::append_gui_verbose_diagnostic_line(|| {
                format!(
                    "clipboard snap={snapped:?} refusal={:?}",
                    self.workspace().schematic_authoring.refusal
                )
            });
        }
        if let Some(snapped) = snapped {
            let e = &mut self.session.workspace_mut().schematic_authoring;
            e.preview_dirty |= e.position != Some(snapped);
            e.position = Some(snapped);
            self.invalidate_frame();
        }
        true
    }
    pub(crate) fn publish_schematic_preview_scene(
        &mut self,
        scene: datum_gui_protocol::BoardReviewSceneV1,
    ) {
        if let Some(child) = &mut self.session.workspace_mut().schematic_authoring.child_draft {
            child.scene = Some(scene);
        } else if self.workspace().schematic_authoring.circuit_basis.is_some() {
            self.session
                .workspace_mut()
                .schematic_authoring
                .circuit_preview = Some(scene);
        } else {
            self.session.workspace_mut().schematic_scene = Some(scene);
        }
        self.session
            .workspace_mut()
            .schematic_authoring
            .preview_dirty = false;
        self.invalidate_scene();
    }
    pub(super) fn prepare_native_schematic_frame(&mut self) {
        if !self.native_edit.pending()
            && (self.workspace().schematic_authoring.active()
                || self.workspace().schematic_authoring.picker)
            && !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused))
        {
            return;
        }
        let e = &self.workspace().schematic_authoring;
        if e.design_placement.is_some() {
            crate::append_gui_verbose_diagnostic_line(|| {
                format!(
                    "clipboard frame position={:?} dirty={} picker={} refusal={:?}",
                    e.position, e.preview_dirty, e.picker, e.refusal
                )
            });
        }
        if !e.active() || !e.preview_dirty || self.native_edit.pending() {
            return;
        }
        let (Some(base), Some(backing)) = (e.base.as_ref(), self.workspace().backing.as_ref())
        else {
            return;
        };
        match e.preview_scene(base, &backing.request.project_root) {
            Ok(scene) => self.publish_schematic_preview_scene(scene),
            Err(error) => {
                let recoverable = self
                    .workspace()
                    .schematic_authoring
                    .design_placement
                    .is_some();
                if recoverable {
                    self.refuse_design_placement_pose(error.to_string());
                } else {
                    self.session.workspace_mut().schematic_authoring.refusal =
                        Some(error.to_string());
                    self.cancel_native_schematic();
                }
            }
        }
    }
    pub(super) fn native_schematic_click(&mut self, world: PointNm, screen: (f32, f32)) -> bool {
        if self.native_edit.pending() && self.workspace().tool == WorkspaceTool::Move {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "Wait for the current Move to finish",
            );
            return true;
        }
        if self
            .native_edit
            .preview
            .as_ref()
            .is_some_and(|p| p.is_schematic())
        {
            return self.native_move_click(world, None);
        }
        if self.workspace().tool == WorkspaceTool::Move
            && !self.workspace().schematic_authoring.active()
            && !self.modifiers.shift_key()
            && !self.modifiers.control_key()
            && !self.modifiers.alt_key()
            && !self.modifiers.super_key()
        {
            self.pick_schematic_move(world, screen);
            return true;
        }
        if !self.workspace().schematic_authoring.active() {
            return false;
        }
        if !self.guard_native_authoring_view(self.pane_at_screen(screen.0, screen.1)) {
            return true;
        }
        if self.native_edit.pending() {
            return true;
        }
        let Some(at) = self.schematic_command_point(world, screen) else {
            return true;
        };
        if self
            .workspace()
            .schematic_authoring
            .design_placement
            .is_some()
        {
            return self.place_design_clipboard(at);
        }
        if self
            .workspace()
            .schematic_authoring
            .field_placement()
            .is_some()
        {
            return self.place_custom_field(at);
        }
        if self.workspace().schematic_authoring.child_draft.is_some() {
            return self.review_child_sheet(at);
        }
        if self.workspace().schematic_authoring.gate_draft.is_some() {
            return self.review_schematic_gate(at);
        }
        if self.workspace().schematic_authoring.marker.is_some() {
            return self.place_native_marker(at);
        }
        if self.workspace().schematic_authoring.bus_entry.is_some() {
            return self.place_native_bus_entry(at);
        }
        if self.workspace().schematic_authoring.label.is_some() {
            return self.place_native_label(at);
        }
        if self.workspace().schematic_authoring.wire {
            let now = std::time::Instant::now();
            // Reuse the native selection repeat-click interval; a later click
            // at the same vertex is not a completion gesture.
            let repeated = self
                .workspace()
                .schematic_authoring
                .last_wire_click
                .is_some_and(|(point, previous)| {
                    point == at
                        && now.duration_since(previous) < std::time::Duration::from_millis(500)
                });
            self.session
                .workspace_mut()
                .schematic_authoring
                .last_wire_click = Some((at, now));
            if repeated {
                return self.finish_native_wire();
            }
            if self.workspace().schematic_authoring.vertices.last() == Some(&at) {
                return true;
            }
            let e = &mut self.session.workspace_mut().schematic_authoring;
            if let Err(reason) = e.append_wire_vertex(at) {
                e.refusal = Some(reason);
            }
            e.position = Some(at);
            e.preview_dirty = true;
            self.invalidate_frame();
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(index) = e.chosen else { return true };
        e.position = Some(at);
        e.preview_dirty = true;
        e.pending_request = Some(SchematicEditRequest::PlaceSymbol {
            sheet: e.placement_sheet().unwrap(),
            instance_path: e.placement_path(),
            library_symbol: e.picker_choices()[index].id,
            explicit_part: e.picker_choices()[index].explicit_part.clone(),
            component: e.picker_choices()[index].component.clone(),
            reference: e.reference.clone(),
            value: e.value.clone(),
            x_nm: at.x,
            y_nm: at.y,
            rotation_deg: e.rotation_deg,
            mirrored: e.mirrored,
        });
        self.prepare_native_schematic_frame();
        self.queue_native_edit(NativeEditAction::Schematic)
    }
    fn finish_native_wire(&mut self) -> bool {
        if !self.guard_native_authoring_view(Some(self.workspace().ui.layout.focused)) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        if !e.wire || e.vertices.len() < 2 {
            return true;
        }
        e.position = e.vertices.last().copied();
        e.preview_dirty = true;
        match e.polyline_request() {
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
