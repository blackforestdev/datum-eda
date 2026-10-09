//! Finite ordinary first-edit script, including post-commit selection identity.
use super::*;

impl Fixture {
    pub(super) fn advance(&mut self, events: &ActiveEventLoop) {
        let runtime = self.app.runtime.as_mut().unwrap();
        if self.logged_stage != Some(self.stage) {
            eprintln!(
                "move-frame stage={} redraw_gap_ms={} selection={:?} preview={}",
                self.stage,
                self.stage_started.elapsed().as_millis(),
                match &runtime.workspace().selection {
                    datum_gui_protocol::SelectionTarget::Native(v) => Some(&v.subject),
                    _ => None,
                },
                runtime.native_edit.preview.is_some()
            );
            self.logged_stage = Some(self.stage);
        }
        self.stage_started = Instant::now();
        match self.stage {
            0 => {
                let component = runtime
                    .workspace()
                    .scene
                    .components
                    .iter()
                    .find(|c| c.component_uuid == "ca340e78-04a5-5307-a024-9c5720a807d2")
                    .or_else(|| {
                        runtime
                            .workspace()
                            .scene
                            .components
                            .iter()
                            .find(|c| !c.locked)
                    })
                    .unwrap()
                    .clone();
                // Use an actual Pad hit on this Footprint, through production mouse dispatch.
                let point = runtime
                    .workspace()
                    .scene
                    .pads
                    .iter()
                    .find(|p| p.component_uuid == component.component_uuid)
                    .and_then(|p| p.center.integer_point())
                    .unwrap_or(component.position);
                let prepared = runtime.prepared_scene().unwrap();
                let v = prepared.scene_viewport;
                let a = (v.x + 20.0, v.y + 20.0);
                let wa = prepared.world_point_at_screen(a.0, a.1).unwrap().0;
                let wb = prepared
                    .world_point_at_screen(a.0 + 100.0, a.1 + 100.0)
                    .unwrap()
                    .0;
                let x = a.0 as f64 + (point.x - wa.x) as f64 * 100.0 / (wb.x - wa.x) as f64;
                let y = a.1 as f64 + (point.y - wa.y) as f64 * 100.0 / (wb.y - wa.y) as f64;
                self.origin = Some((x, y));
                self.mouse(events, x, y, true);
            }
            1 => {
                let datum_gui_protocol::SelectionTarget::Native(view) =
                    &runtime.workspace().selection
                else {
                    panic!("native Object selection required");
                };
                let datum_gui_protocol::SelectionSubject::Object(id) = &view.subject else {
                    panic!("Footprint Object required");
                };
                assert_eq!(
                    id.class,
                    datum_gui_protocol::AuthoredSelectionClass::Footprint
                );
                self.selected_component = Some(id.id.to_string());
                self.original_label = runtime.presented_hits.regions().iter().find_map(|r| {
                    matches!(&r.target, HitTarget::AuthoredObject(target) if target == &runtime.workspace().scene.components.iter().find(|c| c.component_uuid == id.id.to_string()).unwrap().object_id).then_some(r.rect)
                });
                assert!(
                    self.original_label.is_some(),
                    "selected Footprint label must be presented before M"
                );
                self.original_position = Some(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| c.component_uuid == id.id.to_string())
                        .unwrap()
                        .position,
                );
                eprintln!(
                    "move-frame M cursor={:?} origin={:?} tool={:?} components={:?}",
                    runtime.last_cursor_pos,
                    self.origin,
                    runtime.workspace().tool,
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| c.component_uuid == "ca340e78-04a5-5307-a024-9c5720a807d2")
                        .map(|c| (&c.component_uuid, &c.object_id, c.locked))
                );
                let started = Instant::now();
                let action = workspace_keyboard::character_action("m", false).unwrap();
                workspace_keyboard::apply(runtime, action);
                eprintln!(
                    "move-frame M_us={} immediate_preview={}",
                    started.elapsed().as_micros(),
                    runtime.native_edit.preview.is_some()
                );
                self.app.request_redraw_if_needed();
            }
            2 => {
                // Record the owner sequence before accommodating the old two-click adapter.
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 1.0, y + 1.0, false);
            }
            3 => {
                assert!(
                    runtime.native_edit.preview.is_some(),
                    "M must pick up the selected Footprint without another click"
                );
                self.app.request_redraw_if_needed();
            }
            4..=15 => {
                let (x, y) = self.origin.unwrap();
                if self.stage == 4 && std::env::var_os("DATUM_FIRST_EDIT_POINTER_BURST").is_some() {
                    // Reproduce accumulated native motion in one dispatch batch,
                    // before the next ordinary frame/background round. Finite input,
                    // not a responsiveness budget or OS injection mechanism.
                    let started = Instant::now();
                    for step in 0..192 {
                        let delta = 2.0 + (step % 12) as f64;
                        self.mouse(events, x + delta, y + delta, false);
                    }
                    eprintln!(
                        "move-frame accumulated_pointer_batch_ms={}",
                        started.elapsed().as_millis()
                    );
                }
                let delta = (self.stage - 3) as f64 * 2.0;
                self.mouse(events, x + delta, y + delta, false);
            }
            16 => {
                let preview = runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .expect("moving preview remains active");
                let component = runtime
                    .workspace()
                    .scene
                    .components
                    .iter()
                    .find(|c| c.component_uuid == preview.component.component_uuid)
                    .unwrap();
                assert_ne!(
                    component.position, preview.component.position,
                    "pointer movement must translate the selected Footprint"
                );
                let label = runtime.presented_hits.regions().iter().find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &preview.component.object_id)).expect("moved Footprint label must be presented");
                let original = self.original_label.unwrap();
                eprintln!(
                    "move-frame presented label original={original:?} moved={:?} pending_frame={}",
                    label.rect,
                    runtime.renderer.render_session().has_pending_frame()
                );
                assert!(
                    (label.rect.x - original.x).abs() > 2.0
                        || (label.rect.y - original.y).abs() > 2.0,
                    "prepared translation alone is insufficient: the actually presented Footprint must move"
                );
                // Cancel another queued motion before any frame can materialize it.
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 30.0, y + 30.0, false);
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .dispatch_session_command(SessionCommand::CancelAuthoringGesture);
                self.app.request_redraw_if_needed();
            }
            17 => {
                if runtime.renderer.render_session().has_pending_frame() {
                    return;
                }
                let original = self.original_label.unwrap();
                let actual = self.selected_label();
                assert!(
                    (actual.x - original.x).abs() < 1.0 && (actual.y - original.y).abs() < 1.0,
                    "Cancel must restore the presented Footprint"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert_eq!(std::fs::read(&self.board_path).unwrap(), self.preimage);
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    self.original_position.unwrap(),
                    "cancelled queued motion must not resurrect on the next frame"
                );
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x, y, false);
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(workspace_keyboard::apply(
                    runtime,
                    workspace_keyboard::character_action("m", false).unwrap()
                ));
                assert!(
                    runtime.native_edit.preview.is_some(),
                    "M restarts after cancel"
                );
            }
            18 => {
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 60.0, y + 30.0, false);
            }
            19 => {
                if runtime.native_move_frame_needed()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                // The published Footprint label is an AuthoredObject screen hit.
                // Before the correction it reselected instead of committing Move.
                let target = &runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .unwrap()
                    .component
                    .object_id;
                let region = runtime
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == target))
                    .expect("rendered Footprint must expose its authored label");
                let x = (region.rect.x + region.rect.width * 0.5) as f64;
                let y = (region.rect.y + region.rect.height * 0.5) as f64;
                let click_world = runtime
                    .prepared_scene()
                    .unwrap()
                    .world_point_at_screen(x as f32, y as f32)
                    .unwrap()
                    .0;
                let runtime = self.app.runtime.as_ref().unwrap();
                let destination = runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .unwrap()
                    .destination(click_world, runtime.workspace().authoring.snap)
                    .unwrap();
                self.committing_position = Some(destination);
                self.commit_started = Some(Instant::now());
                self.mouse(events, x, y, true);
                assert_eq!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    destination,
                    "Placement must retain the clicked pose while the canonical edit is pending"
                );
                assert!(
                    self.app.runtime.as_ref().unwrap().native_edit.pending(),
                    "destination click must commit rather than reselect"
                );
            }
            20 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(
                    !runtime.native_edit.blocked,
                    "committed refresh must be qualified"
                );
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    self.committing_position.unwrap(),
                    "qualified canonical placement must match the final click"
                );
                eprintln!(
                    "move-frame pending_placement_presented={} click_to_confirm_ms={}",
                    self.pending_placement_presented,
                    self.commit_started.unwrap().elapsed().as_millis()
                );
                let after = std::fs::read(&self.board_path).unwrap();
                assert_ne!(after, self.preimage);
                self.committed = Some(after);
                self.committed_label = Some(self.selected_label());
                assert_ne!(
                    self.committed_label, self.original_label,
                    "Commit must present the translated Footprint, not just change bytes"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                eprintln!(
                    "move-frame committed presented_revision={} status={:?}",
                    runtime.workspace().scene.source_revision,
                    runtime.workspace().last_command_status
                );
                assert!(runtime.queue_native_history(false));
            }
            21 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    self.preimage,
                    "Undo restores preimage"
                );
                assert_eq!(
                    self.selected_label(),
                    self.original_label.unwrap(),
                    "Undo must be presented"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(runtime.queue_native_history(true));
            }
            22 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    *self.committed.as_ref().unwrap(),
                    "Redo restores committed board"
                );
                assert_eq!(
                    self.selected_label(),
                    self.committed_label.unwrap(),
                    "Redo must be presented"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(runtime.queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::Save
                ));
            }
            23 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    *self.committed.as_ref().unwrap()
                );
            }
            24 => {
                let other = runtime.workspace().scene.components.iter()
                    .find(|c| !c.locked && Some(&c.component_uuid) != self.selected_component.as_ref()
                        && runtime.presented_hits.regions().iter().any(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &c.object_id)))
                    .expect("real board supplies another presented unlocked Footprint");
                let rect = runtime.presented_hits.regions().iter()
                    .find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &other.object_id)).unwrap().rect;
                self.post_move_component = Some(other.component_uuid.clone());
                // No Escape or new M: an idle Move mode must select normally.
                self.mouse(
                    events,
                    (rect.x + rect.width * 0.5) as f64,
                    (rect.y + rect.height * 0.5) as f64,
                    true,
                );
            }
            25 => {
                let expected = self.post_move_component.as_ref().unwrap();
                assert!(
                    runtime.native_edit.preview.is_none(),
                    "idle Move must select the next object, not grab an unselected Footprint"
                );
                assert!(
                    !runtime.native_edit.pending(),
                    "selection cannot start another canonical edit"
                );
                let datum_gui_protocol::SelectionTarget::Native(view) =
                    &runtime.workspace().selection
                else {
                    panic!("complete native selection required");
                };
                assert!(
                    matches!(&view.subject, datum_gui_protocol::SelectionSubject::Object(id)
                    if id.class == datum_gui_protocol::AuthoredSelectionClass::Footprint && &id.id.to_string() == expected)
                );
                assert!(!view.stale);
                assert_eq!(view.revision.0, runtime.workspace().scene.source_revision);
                assert!(workspace_keyboard::apply(
                    runtime,
                    workspace_keyboard::character_action("m", false).unwrap()
                ));
                assert_eq!(
                    &runtime
                        .native_edit
                        .preview
                        .as_ref()
                        .unwrap()
                        .component
                        .component_uuid,
                    expected,
                    "M must move exactly the newly selected Footprint"
                );
                let rect = self.selected_label();
                self.mouse(
                    events,
                    (rect.x + rect.width * 0.5) as f64,
                    (rect.y + rect.height * 0.5) as f64,
                    false,
                );
            }
            26 => {
                if runtime.native_move_frame_needed()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                let preview = runtime.native_edit.preview.as_ref().unwrap();
                assert_eq!(
                    Some(&preview.component.component_uuid),
                    self.post_move_component.as_ref(),
                    "hovering another Footprint cannot replace acquired target"
                );
                let first = self.selected_component.as_ref().unwrap();
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| &c.component_uuid == first)
                        .unwrap()
                        .position,
                    preview
                        .base
                        .components
                        .iter()
                        .find(|c| &c.component_uuid == first)
                        .unwrap()
                        .position,
                    "hovered first Footprint stays unchanged"
                );
                runtime.dispatch_session_command(SessionCommand::CancelAuthoringGesture);
                self.app.request_redraw_if_needed();
            }
            27 => {
                if runtime.renderer.render_session().has_pending_frame() {
                    return;
                }
                assert!(runtime.native_edit.preview.is_none());
                assert!(!runtime.native_edit.pending());
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    self.committed.clone().unwrap(),
                    "next selection and cancelled second preview must not write"
                );
                eprintln!(
                    "move-frame subsequent selection and M retained target={} without implicit edit or writes",
                    self.post_move_component.as_ref().unwrap()
                );
            }
            _ => {
                runtime.begin_application_terminal_shutdown();
                self.stage = 29;
                return;
            }
        }
        self.stage += 1;
    }
    pub(super) fn select_while_commit_pending(&mut self, events: &ActiveEventLoop) {
        let runtime = self.app.runtime.as_ref().unwrap();
        let other = runtime.workspace().scene.components.iter()
            .find(|c| !c.locked && Some(&c.component_uuid) != self.selected_component.as_ref()
                && runtime.presented_hits.regions().iter().any(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &c.object_id)))
            .expect("real board supplies another Footprint during commit");
        let rect = runtime
            .presented_hits
            .regions()
            .iter()
            .find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &other.object_id))
            .unwrap()
            .rect;
        let expected = other.component_uuid.clone();
        self.post_move_component = Some(expected.clone());
        self.mouse(
            events,
            (rect.x + rect.width * 0.5) as f64,
            (rect.y + rect.height * 0.5) as f64,
            true,
        );
        let runtime = self.app.runtime.as_mut().unwrap();
        assert!(runtime.native_edit.pending());
        assert!(runtime.native_edit.preview.is_none());
        assert!(
            matches!(&runtime.workspace().selection, datum_gui_protocol::SelectionTarget::Native(v)
            if matches!(&v.subject, datum_gui_protocol::SelectionSubject::Object(id)
                if Some(&id.id.to_string()) == self.selected_component.as_ref())),
            "pending Move click cannot replace the selection or start another gesture"
        );
        let _ = workspace_keyboard::apply(
            runtime,
            workspace_keyboard::character_action("m", false).unwrap(),
        );
        assert!(
            runtime.native_edit.preview.is_none(),
            "M during pending commit cannot grab another object"
        );
        assert!(
            runtime.native_edit.pending(),
            "original writer remains the only pending action"
        );
        eprintln!("move-frame pending commit click/M for target={expected} refused without retry");
    }
}
