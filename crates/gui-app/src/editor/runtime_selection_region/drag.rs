//! Direct manipulation reuses native Move; this owner only qualifies pointer lifetime.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass, SelectionSubject, SelectionTarget};

pub(crate) struct SelectionMoveDrag {
    subject: SelectionSubject,
    anchor: PointNm,
    started: bool,
    previous_tool: WorkspaceTool,
}

impl Runtime {
    pub(crate) fn capture_selection_move(
        &mut self,
        world: PointNm,
        surface: SceneSurface,
        pane: datum_gui_protocol::PaneId,
    ) -> Option<SelectionMoveDrag> {
        if self.modifiers.alt_key() || self.modifiers.super_key() {
            return None;
        }
        let SelectionTarget::Native(view) = &self.workspace().selection else {
            return None;
        };
        if view.stale {
            return None;
        }
        let subject = view.subject.clone();
        if !matches!(
            &subject,
            SelectionSubject::Compound(_)
                | SelectionSubject::Object(datum_gui_protocol::AuthoredSelectionIdentity {
                    class: AuthoredSelectionClass::Symbol
                        | AuthoredSelectionClass::Footprint
                        | AuthoredSelectionClass::Label,
                    ..
                })
        ) {
            return None;
        }
        let hit = self.selection_world_hit(world, surface, Some(pane)).ok()?;
        let HitTarget::AuthoredObject(key) = hit.target? else {
            return None;
        };
        let basis = hit.basis?;
        if !datum_gui_protocol::footprint_move_preview::native::group::subject_contains(
            &subject,
            &basis.bindings.get(&key)?.acquired,
        ) {
            return None;
        }
        Some(SelectionMoveDrag {
            subject,
            anchor: world,
            started: false,
            previous_tool: self.workspace().tool,
        })
    }

    pub(super) fn advance_selection_move(&mut self, world: PointNm) -> bool {
        let Some(mut drag) = self.selection_move.take() else {
            return true;
        };
        let current = matches!(&self.workspace().selection, SelectionTarget::Native(view)
            if !view.stale && view.subject == drag.subject);
        if !current || self.check_work.pending() {
            if drag.started {
                self.cancel_native_move();
            }
            self.dispatch_session_command(SessionCommand::SetTool(drag.previous_tool));
            return true;
        }
        if !drag.started {
            let Some(gesture) = &self.selection_gesture else {
                return true;
            };
            let scene = datum_gui_protocol::camera_scene_for_pane(self.workspace(), gesture.pane);
            if scene.is_none_or(|s| {
                format!("{}@{}", s.scene_id, s.source_revision) != gesture.source_revision
            }) {
                return true;
            }
            // Acquisition uses the press point, so crossing the threshold cannot
            // jump the object or discard the first segment of pointer travel.
            self.dispatch_session_command(SessionCommand::SetTool(WorkspaceTool::Move));
            self.begin_selected_native_move_at(Some(drag.anchor));
            drag.started = self.native_edit.preview.is_some();
            if !drag.started {
                self.dispatch_session_command(SessionCommand::SetTool(drag.previous_tool));
                return true;
            }
        }
        self.native_move_pointer(world);
        if self.native_edit.preview.is_some() {
            self.selection_move = Some(drag);
        } else {
            self.dispatch_session_command(SessionCommand::SetTool(drag.previous_tool));
        }
        true
    }

    pub(super) fn cancel_selection_move(&mut self) -> bool {
        let Some(drag) = self.selection_move.take() else {
            return false;
        };
        if drag.started {
            self.cancel_native_move();
        }
        self.dispatch_session_command(SessionCommand::SetTool(drag.previous_tool));
        true
    }

    pub(super) fn finish_selection_move(
        &mut self,
        drag: Option<SelectionMoveDrag>,
        gesture: &SelectionGesture,
    ) {
        let Some(drag) = drag.filter(|drag| drag.started) else {
            return;
        };
        let destination = self.last_cursor_pos.and_then(|(x, y)| {
            if self.pane_at_screen(x, y) != Some(gesture.pane) {
                return None;
            }
            self.prepared_scene()
                .and_then(|s| s.world_point_at_screen(x, y))
                .map(|(p, _)| p)
        });
        if let Some(world) = destination {
            self.native_move_click(world, None);
        }
        // A refused release ends this drag; it must never become a later click edit.
        if !self.native_edit.pending() {
            self.cancel_native_move();
        }
        self.dispatch_session_command(SessionCommand::SetTool(drag.previous_tool));
    }
}
