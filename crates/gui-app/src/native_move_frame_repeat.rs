//! Repeated real-board edit: publish B atomically, then accept the next action.
use super::*;
use datum_gui_protocol::native_edit_client::NativeEditAction;

impl Fixture {
    pub(super) fn repeat_move(&mut self, events: &ActiveEventLoop) {
        let runtime = self.app.runtime.as_mut().unwrap();
        match self.stage {
            28 => {
                assert!(workspace_keyboard::apply(
                    runtime,
                    workspace_keyboard::character_action("m", false).unwrap()
                ));
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 30.0, y + 20.0, false);
            }
            29 => {
                if runtime.native_move_frame_needed()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                let preview = runtime.native_edit.preview.as_ref().unwrap();
                assert_eq!(
                    Some(
                        &preview
                            .footprint()
                            .expect("Footprint preview")
                            .component
                            .component_uuid
                    ),
                    self.post_move_component.as_ref()
                );
                let base_position = preview
                    .base()
                    .components
                    .iter()
                    .find(|c| Some(&c.component_uuid) == self.post_move_component.as_ref())
                    .unwrap()
                    .position;
                let rect=runtime.presented_hits.regions().iter().find(|r|matches!(&r.target,HitTarget::AuthoredObject(id) if id==&preview.footprint().expect("Footprint preview").component.object_id)).unwrap().rect;
                let (x, y) = (
                    (rect.x + rect.width * 0.5) as f64,
                    (rect.y + rect.height * 0.5) as f64,
                );
                let world = runtime
                    .prepared_scene()
                    .unwrap()
                    .world_point_at_screen(x as f32, y as f32)
                    .unwrap()
                    .0;
                let destination = runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .unwrap()
                    .destination(world, runtime.workspace().authoring.snap)
                    .unwrap();
                assert_ne!(destination, base_position);
                self.committing_position = Some(destination);
                self.commit_started = Some(Instant::now());
                self.mouse(events, x, y, true);
                assert!(self.app.runtime.as_ref().unwrap().native_edit.pending());
            }
            30 => {
                let expected = self.committing_position.unwrap();
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.post_move_component.as_ref())
                        .unwrap()
                        .position,
                    expected,
                    "B placement retained while waiting and after qualification"
                );
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    self.first_committed_position.unwrap(),
                    "B commit preserves A"
                );
                let after = std::fs::read(&self.board_path).unwrap();
                assert_ne!(Some(&after), self.committed.as_ref());
                self.committed = Some(after);
                eprintln!(
                    "move-frame second_click_to_confirm_ms={} project_ready_revision={}",
                    self.commit_started.unwrap().elapsed().as_millis(),
                    runtime.workspace().scene.source_revision
                );
                let next_started = Instant::now();
                assert!(workspace_keyboard::apply(
                    runtime,
                    workspace_keyboard::character_action("m", false).unwrap()
                ));
                assert_eq!(
                    Some(
                        &runtime
                            .native_edit
                            .preview
                            .as_ref()
                            .unwrap()
                            .footprint()
                            .expect("Footprint preview")
                            .component
                            .component_uuid
                    ),
                    self.post_move_component.as_ref(),
                    "next action acquires the fully published B"
                );
                eprintln!(
                    "move-frame second_commit_next_M_us={}",
                    next_started.elapsed().as_micros()
                );
                runtime.dispatch_session_command(SessionCommand::CancelAuthoringGesture);
                self.app.request_redraw_if_needed();
            }
            31 => {
                if runtime.native_move_frame_needed()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(runtime.native_edit.preview.is_none());
                assert!(runtime.queue_native_edit(NativeEditAction::Save));
            }
            32 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    self.committed.clone().unwrap()
                );
            }
            _ => {
                runtime.begin_application_terminal_shutdown();
                self.stage = 34;
                return;
            }
        }
        self.stage += 1;
    }
}
