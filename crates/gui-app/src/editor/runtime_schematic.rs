//! Native schematic gestures reuse the existing asynchronous canonical worker.
use super::*;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use datum_gui_protocol::schematic_authoring::{
    PlacementField, PlacementTransform, SchematicEditRequest,
};
#[path = "runtime_schematic_label.rs"]
mod label;
#[path = "runtime_schematic_library.rs"]
mod library;
#[path = "runtime_schematic_selected.rs"]
mod selected;
impl Runtime {
    pub(super) fn begin_native_schematic(&mut self, wire: bool) -> bool {
        if !self.prepare_native_schematic(wire) {
            return true;
        }
        if !wire {
            return self.browse_schematic_libraries();
        }
        true
    }
    fn prepare_native_schematic(&mut self, wire: bool) -> bool {
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
        let view_basis =
            match self.capture_native_authoring_view(Some(self.workspace().ui.layout.focused)) {
                Ok(basis) => basis,
                Err(reason) => {
                    self.refuse_native_authoring_view(reason);
                    return false;
                }
            };
        self.cancel_native_schematic();
        self.cancel_native_move();
        let Some(base) = self.workspace().schematic_scene.clone() else {
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
        self.session.workspace_mut().schematic_authoring.creation = None;
        let was_active = self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker;
        let base = self.session.workspace_mut().schematic_authoring.cancel();
        if was_active {
            self.session.workspace_mut().authoring.gesture.view_basis = None;
        }
        if let Some(base) = base {
            self.session.workspace_mut().schematic_scene = Some(base);
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
        if editor.chosen.is_none()
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
        match target {
            HitTarget::SchematicBrowseLibraries => return self.browse_schematic_libraries(),
            HitTarget::SchematicLibraryChoice(index) => {
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .select(*index);
            }
            HitTarget::SchematicLibraryPage(forward) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                if *forward && e.choice_offset + 4 < e.choices.len() {
                    e.choice_offset += 4;
                }
                if !*forward {
                    e.choice_offset = e.choice_offset.saturating_sub(4);
                }
            }
            HitTarget::SchematicReference | HitTarget::SchematicValue => {
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
            HitTarget::SchematicPlace => return self.start_symbol_preview(),
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
    pub(super) fn handle_schematic_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        self.trace_schematic_creation_event("schematic key", Some(event));
        if !matches!(
            self.application_focus(),
            datum_gui_protocol::ApplicationFocus::Editor(_)
        ) {
            return false;
        }
        if event.state != ElementState::Pressed {
            return false;
        }
        if self.handle_symbol_move_pose_key(event) {
            return true;
        }
        if self.workspace().schematic_authoring.creation.is_some() {
            return self.handle_schematic_creation_key(event);
        }
        if !self.workspace().schematic_authoring.active()
            && !self.workspace().schematic_authoring.picker
        {
            if self.workspace().ui.layout.focused_content() == PaneContent::Schematic
                && matches!(event.logical_key, Key::Named(NamedKey::Delete))
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
        if self.workspace().schematic_authoring.label.is_some() {
            return self.handle_label_key(event);
        }
        let picker = self.workspace().schematic_authoring.picker;
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
            Key::Named(NamedKey::Escape)
                if !picker
                    && self.workspace().schematic_authoring.wire
                    && self.workspace().schematic_authoring.vertices.len() >= 2 =>
            {
                return self.finish_native_wire();
            }
            Key::Named(NamedKey::Escape) => {
                self.cancel_native_schematic();
            }
            Key::Named(NamedKey::Enter)
                if picker && self.workspace().schematic_authoring.editing.is_some() =>
            {
                return self.apply_selected_symbol_fields();
            }
            Key::Named(NamedKey::Enter)
                if picker && self.workspace().schematic_authoring.choices.is_empty() =>
            {
                return self.browse_schematic_libraries();
            }
            Key::Named(NamedKey::Enter) if picker => return self.start_symbol_preview(),
            Key::Named(NamedKey::Enter) if self.workspace().schematic_authoring.wire => {
                return self.finish_native_wire();
            }
            Key::Named(NamedKey::Tab) if !picker && !self.workspace().schematic_authoring.wire => {
                self.session.workspace_mut().schematic_authoring.picker = true;
            }
            Key::Named(NamedKey::Tab) if picker => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.field = if e.field == PlacementField::Reference {
                    PlacementField::Value
                } else {
                    PlacementField::Reference
                };
            }
            _ => {}
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn native_schematic_pointer(&mut self, screen: (f32, f32)) -> bool {
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
        let Some((world, SceneSurface::Schematic)) = self
            .prepared_scene()
            .and_then(|scene| scene.world_point_at_screen(screen.0, screen.1))
        else {
            return false;
        };
        if !self.guard_native_authoring_view(self.pane_at_screen(screen.0, screen.1)) {
            return true;
        }
        let snapped = self.schematic_snap_point(world, screen);
        if let Some(snapped) = snapped {
            let e = &mut self.session.workspace_mut().schematic_authoring;
            e.preview_dirty |= e.position != Some(snapped);
            e.position = Some(snapped);
            self.invalidate_frame();
        }
        true
    }
    fn schematic_snap_point(&self, world: PointNm, screen: (f32, f32)) -> Option<PointNm> {
        let pane = self.pane_at_screen(screen.0, screen.1)?;
        let scene = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane)?;
        if let Some(HitTarget::AuthoredObject(id)) =
            self.presented_hits.hit_test(screen.0, screen.1)
            && let Some(graphic) = scene.board_graphics.iter().find(|g| &g.object_id == id)
            && let Some(point) = self
                .workspace()
                .schematic_authoring
                .pin_point(&graphic.source_object_uuid)
        {
            return Some(point);
        }
        // Authored world ink is retained, not a screen-chrome hit region. Reuse
        // its existing geometry predicate, admitting only complete committed
        // source pins from the gesture's revision-qualified immutable basis.
        if let Some(retained) = self.renderer.render_session().schematic_for_pane(pane)
            && let Some(HitTarget::AuthoredObject(id)) =
                retained.hit_test_authored_world_eligible(world, self.workspace(), |target| {
                    let HitTarget::AuthoredObject(id) = target else {
                        return false;
                    };
                    scene
                        .board_graphics
                        .iter()
                        .find(|g| &g.object_id == id)
                        .and_then(|graphic| {
                            self.workspace()
                                .schematic_authoring
                                .pin_point(&graphic.source_object_uuid)
                        })
                        .is_some()
                })
            && let Some(point) = scene
                .board_graphics
                .iter()
                .find(|g| g.object_id == id)
                .and_then(|graphic| {
                    self.workspace()
                        .schematic_authoring
                        .pin_point(&graphic.source_object_uuid)
                })
        {
            return Some(point);
        }
        datum_gui_protocol::checked_snap_authoring_point(world, self.workspace().authoring.snap)
            .ok()
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
        if !e.active() || !e.preview_dirty || self.native_edit.pending() {
            return;
        }
        let (Some(base), Some(backing)) = (e.base.as_ref(), self.workspace().backing.as_ref())
        else {
            return;
        };
        match e.preview_scene(base, &backing.request.project_root) {
            Ok(scene) => {
                self.session.workspace_mut().schematic_scene = Some(scene);
                self.session
                    .workspace_mut()
                    .schematic_authoring
                    .preview_dirty = false;
                self.invalidate_scene();
            }
            Err(error) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(error.to_string());
                self.cancel_native_schematic();
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
        if !self.workspace().schematic_authoring.active() {
            return false;
        }
        if !self.guard_native_authoring_view(self.pane_at_screen(screen.0, screen.1)) {
            return true;
        }
        if self.native_edit.pending() {
            return true;
        }
        let Some(at) = self.schematic_snap_point(world, screen) else {
            return true;
        };
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
            e.vertices.push(at);
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
            sheet: e.sheet.unwrap(),
            library_symbol: e.choices[index].id,
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
        e.pending_request = Some(SchematicEditRequest::DrawWire {
            sheet: e.sheet.unwrap(),
            points: e
                .vertices
                .iter()
                .map(|p| eda_engine::ir::geometry::Point { x: p.x, y: p.y })
                .collect(),
        });
        self.prepare_native_schematic_frame();
        self.queue_native_edit(NativeEditAction::Schematic)
    }
}
