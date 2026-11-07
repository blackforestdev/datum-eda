//! Direct manipulation reuses native Move; this owner only qualifies pointer lifetime.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass, SelectionSubject, SelectionTarget};

pub(crate) struct SelectionMoveDrag {
    subject: SelectionSubject,
    selection_at_press: Option<(
        SelectionSubject,
        datum_gui_protocol::SelectionProjectId,
        String,
    )>,
    acquisition: Option<(String, std::sync::Arc<runtime_selection_hit::BindingBasis>)>,
    wire_edit: Option<eda_engine::api::native_write::schematic_wire_edit_adapter::WireEditParams>,
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
        let selection_at_press = match &self.workspace().selection {
            SelectionTarget::Native(view) if !view.stale => {
                Some((view.subject.clone(), view.project, view.revision.0.clone()))
            }
            SelectionTarget::None => None,
            _ => return None,
        };
        let hit = self.selection_world_hit(world, surface, Some(pane)).ok()?;
        let HitTarget::AuthoredObject(key) = hit.target? else {
            return None;
        };
        let basis = hit.basis?;
        let key = if surface == SceneSurface::Schematic {
            self.wire_handle_pickup_key(key, &basis, world, pane)
        } else {
            key
        };
        let binding = basis.bindings.get(&key)?;
        let current = selection_at_press.as_ref().map(|(subject, _, _)| subject);
        let subject = drag_subject(current, &binding.acquired)?;
        if binding.acquired.class == AuthoredSelectionClass::Wire {
            let scene = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane)?;
            let allow_segment = current == Some(&subject);
            self.wire_pickup_handle(scene, &binding.acquired, world, allow_segment)
                .ok()?;
        }
        // Retain the typed press handle: once the drag crosses its threshold,
        // the cursor is no longer over that grip and selection publication may
        // have invalidated its prepared scene. Never re-hit the motion point.
        let wire_edit = match &subject {
            SelectionSubject::Object(id)
                if matches!(
                    id.class,
                    AuthoredSelectionClass::Wire | AuthoredSelectionClass::Junction
                ) =>
            {
                let scene = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane)?;
                self.wire_edit_configuration(scene, id, world).ok()?
            }
            _ => None,
        };
        crate::append_gui_verbose_diagnostic_line(|| {
            format!("selection drag captured subject={subject:?} anchor={world:?}")
        });
        let acquisition = (current != Some(&subject)).then_some((key, basis));
        Some(SelectionMoveDrag {
            subject,
            selection_at_press,
            acquisition,
            wire_edit,
            anchor: world,
            started: false,
            previous_tool: self.workspace().tool,
        })
    }

    pub(super) fn advance_selection_move(&mut self, world: PointNm) -> bool {
        let Some(mut drag) = self.selection_move.take() else {
            return true;
        };
        let current = if drag.started {
            matches!(&self.workspace().selection, SelectionTarget::Native(view)
                if !view.stale && view.subject == drag.subject)
        } else {
            match (&self.workspace().selection, &drag.selection_at_press) {
                (SelectionTarget::None, None) => true,
                (SelectionTarget::Native(view), Some((subject, project, revision))) => {
                    !view.stale
                        && &view.subject == subject
                        && &view.project == project
                        && &view.revision.0 == revision
                }
                _ => false,
            }
        };
        crate::append_gui_verbose_diagnostic_line(|| {
            format!(
                "selection drag advance started={} current={current} pending={}",
                drag.started,
                self.check_work.pending()
            )
        });
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
                crate::append_gui_verbose_diagnostic_line(|| "selection drag source scene changed");
                return true;
            }
            if let Some((key, captured)) = drag.acquisition.take() {
                // Revalidate the exact occurrence and filters before publishing.
                // A drag picks one authored object, never a repeated-click Run.
                let pane = gesture.pane;
                let Ok(Some(current)) =
                    self.native_binding_basis(SceneSurface::Schematic, Some(pane))
                else {
                    return true;
                };
                if !std::sync::Arc::ptr_eq(&captured, &current)
                    || !captured
                        .bindings
                        .get(&key)
                        .is_some_and(|binding| binding.eligible(&self.workspace().ui.filters))
                {
                    crate::append_gui_verbose_diagnostic_line(
                        || "selection drag binding basis changed",
                    );
                    return true;
                }
                let SelectionSubject::Object(id) = &drag.subject else {
                    return true;
                };
                self.native_selection.as_mut().unwrap().reset_click_depth();
                self.publish_native_acquisition(
                    [id.clone()].into_iter().collect(),
                    AcquisitionMode::Replace,
                    AcquisitionEntry::Individual,
                    None,
                );
                if !matches!(&self.workspace().selection, SelectionTarget::Native(view)
                    if !view.stale && view.subject == drag.subject)
                {
                    return true;
                }
            }
            // Acquisition uses the press point, so crossing the threshold cannot
            // jump the object or discard the first segment of pointer travel.
            self.dispatch_session_command(SessionCommand::SetTool(WorkspaceTool::Move));
            if drag.wire_edit.is_some() {
                self.begin_schematic_move(drag.subject.clone(), Some(drag.anchor), drag.wire_edit);
            } else {
                self.begin_selected_native_move_at(Some(drag.anchor));
            }
            drag.started = self.native_edit.preview.is_some();
            crate::append_gui_verbose_diagnostic_line(|| {
                format!("selection drag move begun={}", drag.started)
            });
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

/// Preserve an admitted selected subject; connectivity pickup is an explicit
/// singleton even when the preceding point click selected a derived Run.
pub(super) fn drag_subject(
    current: Option<&SelectionSubject>,
    hit: &datum_gui_protocol::AuthoredSelectionIdentity,
) -> Option<SelectionSubject> {
    let admitted = current.filter(|subject| {
        matches!(
            subject,
            SelectionSubject::Compound(_)
                | SelectionSubject::Object(datum_gui_protocol::AuthoredSelectionIdentity {
                    class: AuthoredSelectionClass::Symbol
                        | AuthoredSelectionClass::Footprint
                        | AuthoredSelectionClass::Label
                        | AuthoredSelectionClass::SchematicText
                        | AuthoredSelectionClass::Wire
                        | AuthoredSelectionClass::Junction
                        | AuthoredSelectionClass::Port,
                    ..
                })
        )
    });
    if let Some(subject) = admitted.filter(|subject| {
        datum_gui_protocol::footprint_move_preview::native::group::subject_contains(subject, hit)
    }) {
        return Some(subject.clone());
    }
    matches!(
        hit.class,
        AuthoredSelectionClass::Wire
            | AuthoredSelectionClass::Junction
            | AuthoredSelectionClass::Port
    )
    .then(|| SelectionSubject::Object(hit.clone()))
}
