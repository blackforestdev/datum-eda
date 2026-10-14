//! Shared tool switching and native authoring pointer actions.
use crate::*;

impl Runtime {
    pub(crate) fn set_workspace_tool(&mut self, tool: WorkspaceTool) -> bool {
        if !matches!(tool, WorkspaceTool::Select)
            && !(tool == WorkspaceTool::Move && self.native_edit_available())
        {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                format!("{} is unavailable in this build", tool.label()),
            );
            self.invalidate_frame();
            return true;
        }
        self.cancel_native_move();
        self.cancel_native_schematic();
        if self.workspace().tool != tool {
            self.cancel_native_selection_gesture();
        }
        let handled = self.dispatch_session_command(SessionCommand::SetTool(tool));
        if handled {
            self.log_console_echo(
                ConsoleFeedbackSource::Tool,
                format!("{} tool active", tool.label()),
            );
        }
        if tool == WorkspaceTool::Move {
            self.begin_selected_native_move();
        }
        handled || self.native_edit.preview.is_some()
    }

    pub(crate) fn active_tool_is_authoring(&self) -> bool {
        self.workspace().tool == WorkspaceTool::Move && self.native_edit_available()
    }

    pub(crate) fn handle_authoring_pointer_move(&mut self, screen_pos: (f32, f32)) -> bool {
        if self.native_schematic_pointer(screen_pos) {
            return true;
        }
        if !self.active_tool_is_authoring() || !self.workspace().authoring.gesture.is_active() {
            return false;
        }
        let Some(prepared) = self.prepared_scene() else {
            return false;
        };
        // Authoring is a board-scene gesture; only the board surface drives it.
        let Some((world, SceneSurface::Board)) =
            prepared.world_point_at_screen(screen_pos.0, screen_pos.1)
        else {
            return false;
        };
        if self.workspace().tool == WorkspaceTool::Move {
            return self.native_move_pointer(world);
        }
        let target_object_id = self.authoring_target_object_id(world);
        self.dispatch_session_command(SessionCommand::PreviewAuthoringGesture {
            world,
            target_object_id,
        })
    }

    pub(crate) fn handle_authoring_canvas_click(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if !self.active_tool_is_authoring() {
            return false;
        }
        if self.workspace().tool == WorkspaceTool::Move {
            return self.native_move_click(world, target_object_id);
        }
        match self.workspace().tool {
            WorkspaceTool::DrawBoardTrack
                if self.workspace().authoring.gesture.anchor.is_some() =>
            {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_draw_board_track_handoff(world)
                else {
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "draw-board-track");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardVia => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_via_handoff(world)
                else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "place via requires a board net context".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-via");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardText => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_text_handoff(world)
                else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "place text requires project backing".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-text");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Delete => {
                let Some(target) = target_object_id else {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "delete requires an authored object target".to_string(),
                    );
                    return true;
                };
                let Some(handoff) = self.workspace().delete_authored_object_handoff(&target) else {
                    self.log_console_refusal_for_target(
                        ConsoleFeedbackSource::Tool,
                        target,
                        "Delete is unavailable for this object type",
                    );
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "delete-authored-object");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Select | WorkspaceTool::DrawBoardTrack | WorkspaceTool::Move => {}
        }
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world,
            target_object_id,
        })
    }

    pub(crate) fn authoring_target_object_id(&mut self, world: PointNm) -> Option<String> {
        let target = {
            if !self.ensure_retained_scene() {
                return None;
            }
            let retained = self
                .renderer
                .render_session()
                .board()
                .expect("retained scene initialized");
            retained.hit_test_authored_world(world, self.session.workspace())
        };
        match target {
            Some(HitTarget::AuthoredObject(id)) => Some(id),
            Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        }
    }

    pub(crate) fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        let started = std::time::Instant::now();
        let handled = self.select_hit_target_inner(target);
        let next_focus =
            keyboard_focus::focus_after_hit_target(self.application_focus(), handled, target);
        if next_focus != self.application_focus() {
            self.set_application_focus(next_focus);
        }
        self.trace_timing(|| {
            format!(
                "select target {target:?} handled={handled} {}ms",
                started.elapsed().as_millis()
            )
        });
        handled
    }
}
