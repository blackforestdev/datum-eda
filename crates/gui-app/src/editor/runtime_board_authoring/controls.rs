//! PCB chrome actions use the existing native queue and shared editor focus.
use super::*;
impl Runtime {
    pub(crate) fn handle_board_control(&mut self, control: BoardControl) -> bool {
        match control {
            BoardControl::Outline => return self.begin_native_board_tool(BoardToolKind::Outline),
            BoardControl::Route => return self.begin_native_board_tool(BoardToolKind::Route),
            BoardControl::Zone => return self.begin_native_board_tool(BoardToolKind::Zone),
            BoardControl::Fill => {
                if self.workspace().authoring.board_controls.is_some() {
                    return self.board_authoring_error(
                        "Finish or cancel the boundary before filling Zones",
                    );
                }
                return self.queue_native_edit(NativeEditAction::BoardZone);
            }
            _ => {}
        }
        if self.native_edit.pending() {
            return true;
        }
        if control == BoardControl::Cancel {
            self.session.workspace_mut().cancel_authoring_gesture();
            self.session.workspace_mut().set_tool(WorkspaceTool::Select);
            self.invalidate_scene();
            return true;
        }
        if !self.board_authoring_current() {
            return self
                .board_authoring_error("Board source, pane or layer changed; cancel and restart");
        }
        self.set_application_focus(keyboard_focus::focus_after_canvas_click(
            self.workspace().ui.layout.focused,
        ));
        self.session
            .workspace_mut()
            .authoring
            .board_controls
            .as_mut()
            .unwrap()
            .focus = Some(control);
        match control {
            BoardControl::Field(field) => {
                self.session
                    .workspace_mut()
                    .authoring
                    .board_controls
                    .as_mut()
                    .unwrap()
                    .focus = Some(BoardControl::Field(field));
                if let Some(region) = self
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| r.target == HitTarget::BoardAuthoring(control))
                    .cloned()
                    && let Some((x, _)) = self.last_cursor_pos
                {
                    self.start_dialog_editor_pointer(region, x, self.scale_factor);
                    self.finish_dialog_editor_pointer();
                }
            }
            BoardControl::Thermal => {
                let setup = self
                    .session
                    .workspace_mut()
                    .authoring
                    .board_controls
                    .as_mut()
                    .unwrap();
                setup.focus = Some(BoardControl::Thermal);
                setup.thermal = Some(match setup.thermal {
                    None => false,
                    Some(v) => !v,
                });
            }
            BoardControl::Posture => {
                use eda_engine::api::native_write::schematic_wire_edit::{
                    WireCornerOrder, WireRouting,
                };
                if let Some(route) = &mut self.session.workspace_mut().authoring.board_route {
                    let current = route.posture();
                    let routing = match current.routing {
                        WireRouting::Free => WireRouting::Orthogonal,
                        WireRouting::Orthogonal => WireRouting::Diagonal45,
                        WireRouting::Diagonal45 => WireRouting::Free,
                    };
                    if let Err(error) = route.set_posture(WireRouteConfig {
                        routing,
                        corner: WireCornerOrder::Start,
                    }) {
                        return self.board_authoring_error(error);
                    }
                }
            }
            BoardControl::ApplyRule => {
                return self.queue_native_edit(NativeEditAction::CreateBoardClearance);
            }
            BoardControl::Finish => {
                let kind = self
                    .workspace()
                    .authoring
                    .board_controls
                    .as_ref()
                    .unwrap()
                    .kind;
                return self.queue_native_edit(match kind {
                    BoardToolKind::Route => NativeEditAction::BoardRoute,
                    BoardToolKind::Zone => NativeEditAction::BoardZone,
                    BoardToolKind::Outline => NativeEditAction::BoardOutline,
                });
            }
            _ => {}
        }
        self.refresh_board_route_stop();
        self.invalidate_frame();
        true
    }
}

impl Runtime {
    pub(crate) fn mark_board_authoring_pending(&mut self, action: NativeEditAction) {
        if matches!(
            action,
            NativeEditAction::BoardRoute
                | NativeEditAction::BoardOutline
                | NativeEditAction::BoardZone
                | NativeEditAction::CreateBoardClearance
        ) && let Some(setup) = &mut self.session.workspace_mut().authoring.board_controls
        {
            setup.pending = true;
        }
    }
    pub(crate) fn restore_board_authoring(
        &mut self,
        mut previous: datum_gui_protocol::AuthoringToolState,
        intent: &NativeEditIntent,
        result: &Result<String, String>,
    ) {
        if !matches!(
            intent.action,
            NativeEditAction::BoardRoute
                | NativeEditAction::BoardOutline
                | NativeEditAction::BoardZone
                | NativeEditAction::CreateBoardClearance
        ) {
            return;
        }
        if let Some(route) = &mut previous.board_route {
            route.release_pending();
        }
        if let Some(outline) = &mut previous.board_outline {
            outline.release_pending();
        }
        if let Some(zone) = &mut previous.board_zone {
            zone.release_pending();
        }
        if let Err(error) = result {
            if let Some(setup) = &mut previous.board_controls {
                setup.pending = false;
                setup.refusal = Some(error.clone());
            }
            self.session.workspace_mut().authoring = previous;
        } else if matches!(intent.action, NativeEditAction::CreateBoardClearance) {
            if let Some(old) = previous.board_controls {
                let mut setup = match BoardAuthoringControls::capture(self.workspace(), old.kind) {
                    Ok(setup) => setup,
                    Err(error) => {
                        self.board_authoring_error(error);
                        return;
                    }
                };
                setup.fields = old.fields;
                setup.thermal = old.thermal;
                let route = previous.board_route;
                self.session.workspace_mut().cancel_authoring_gesture();
                self.session.workspace_mut().set_tool(match setup.kind {
                    BoardToolKind::Route => WorkspaceTool::DrawBoardTrack,
                    BoardToolKind::Zone => WorkspaceTool::DrawBoardZone,
                    BoardToolKind::Outline => WorkspaceTool::DrawBoardOutline,
                });
                if let Some(old_route) = route {
                    if let Err(error) = self.session.workspace_mut().begin_board_route(
                        old_route.basis().start_pad,
                        setup.layer,
                        old_route.posture(),
                    ) {
                        setup.refusal = Some(error);
                    }
                }
                self.session.workspace_mut().authoring.board_controls = Some(setup);
                self.refresh_board_zone_readback();
            }
        } else {
            self.session.workspace_mut().cancel_authoring_gesture();
            self.session.workspace_mut().set_tool(WorkspaceTool::Select);
        }
        self.invalidate_scene();
    }
}

impl Runtime {
    pub(crate) fn board_authoring_admits(&mut self, action: NativeEditAction) -> bool {
        if self.workspace().authoring.board_controls.is_some()
            && !matches!(
                action,
                NativeEditAction::BoardRoute
                    | NativeEditAction::BoardOutline
                    | NativeEditAction::BoardZone
                    | NativeEditAction::CreateBoardClearance
            )
        {
            self.board_authoring_error("Finish or cancel the PCB draft first");
            return false;
        }
        true
    }
}

impl Runtime {
    pub(crate) fn release_board_authoring_pending(&mut self) {
        let state = self.session.workspace_mut();
        if let Some(route) = &mut state.authoring.board_route {
            route.release_pending();
        }
        if let Some(outline) = &mut state.authoring.board_outline {
            outline.release_pending();
        }
        if let Some(zone) = &mut state.authoring.board_zone {
            zone.release_pending();
        }
        if let Some(setup) = &mut state.authoring.board_controls {
            setup.pending = false;
        }
    }
}

impl Runtime {
    pub(crate) fn refresh_board_zone_readback(&mut self) {
        if let Some(mut setup) = self.session.workspace_mut().authoring.board_controls.take() {
            setup.refresh_zone_projection(self.workspace());
            self.session.workspace_mut().authoring.board_controls = Some(setup);
        }
    }
}
