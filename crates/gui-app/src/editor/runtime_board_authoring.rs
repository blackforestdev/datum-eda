//! Native PCB tools configure the shared gesture and canonical edit queue.
use crate::*;
use datum_gui_protocol::board_authoring_controls::{
    BoardAuthoringControls, BoardControl, BoardField, BoardToolKind,
};
use datum_gui_protocol::native_edit_client::{NativeEditAction, NativeEditIntent};
use eda_engine::api::native_write::schematic_wire_edit::WireRouteConfig;
#[path = "runtime_board_authoring/controls.rs"]
mod controls;
#[path = "runtime_board_authoring/keys.rs"]
mod keys;

impl Runtime {
    pub(crate) fn begin_native_board_tool(&mut self, kind: BoardToolKind) -> bool {
        if self.native_edit.pending() || self.native_edit.blocked {
            return self.board_authoring_error("Wait for the pending native edit");
        }
        let state = self.workspace();
        if state.schematic_authoring.delete_review.is_some()
            || state.ui.board_transfer.is_some()
            || state.library.draft.is_some()
            || state.library.acquisition.is_some()
        {
            return self.board_authoring_error(
                "Finish or cancel the current review or library edit first",
            );
        }
        let verb = match kind {
            BoardToolKind::Route => "datum.pcb.draw_track",
            BoardToolKind::Zone => "datum.pcb.place_zone",
            BoardToolKind::Outline => "datum.pcb.set_outline",
        };
        if !state
            .backing
            .as_ref()
            .is_some_and(|b| b.native_write_verbs.contains(verb))
        {
            return self.board_authoring_error("Native PCB authoring is unavailable");
        }
        let setup = match BoardAuthoringControls::capture(self.workspace(), kind) {
            Ok(setup) => setup,
            Err(error) => return self.board_authoring_error(error),
        };
        self.cancel_native_selection_gesture();
        self.cancel_native_move();
        self.cancel_native_schematic();
        let workspace = self.session.workspace_mut();
        workspace.cancel_authoring_gesture();
        workspace.set_tool(match kind {
            BoardToolKind::Route => WorkspaceTool::DrawBoardTrack,
            BoardToolKind::Zone => WorkspaceTool::DrawBoardZone,
            BoardToolKind::Outline => WorkspaceTool::DrawBoardOutline,
        });
        workspace.inspector_context.owner =
            datum_gui_protocol::inspector_context::InspectorOwner::Selection;
        workspace.ui.selection_inspector.reset_scroll();
        workspace.authoring.board_controls = Some(setup);
        self.invalidate_scene();
        true
    }
    pub(crate) fn board_authoring_error(&mut self, error: impl Into<String>) -> bool {
        let error = error.into();
        if let Some(controls) = &mut self.session.workspace_mut().authoring.board_controls {
            controls.refusal = Some(error.clone());
        }
        self.log_console_refusal(ConsoleFeedbackSource::Tool, error);
        self.invalidate_frame();
        true
    }
    pub(crate) fn board_authoring_current(&self) -> bool {
        self.workspace()
            .authoring
            .board_controls
            .as_ref()
            .is_some_and(|c| c.current(self.workspace()))
    }
    pub(crate) fn board_authoring_pointer(&mut self, world: PointNm) -> bool {
        if self.workspace().authoring.board_controls.is_none() {
            return false;
        }
        if !self.board_authoring_current() || self.native_edit.pending() {
            return true;
        }
        if self.workspace().authoring.board_route.is_none()
            && self.workspace().authoring.board_zone.is_none()
            && self.workspace().authoring.board_outline.is_none()
        {
            return true;
        }
        let target = self.authoring_target_object_id(world);
        let snapped = if self.workspace().tool == WorkspaceTool::DrawBoardOutline {
            self.workspace().snap_authoring_point(world)
        } else {
            self.board_contact_point(world, target.as_deref())
        };
        let workspace = self.session.workspace_mut();
        // Contacts retain their exact source centre; all other points use the
        // same grid resolver as the rest of the workspace authoring gesture.
        workspace.authoring.gesture.preview = Some(snapped);
        workspace.authoring.gesture.target_object_id = target;
        self.refresh_board_route_stop();
        self.invalidate_scene();
        true
    }
    pub(crate) fn board_contact_point(&self, world: PointNm, target: Option<&str>) -> PointNm {
        if let Some(pad) = target.and_then(|id| {
            self.workspace()
                .scene
                .pads
                .iter()
                .find(|pad| pad.object_id == id)
        }) {
            if let Some(point) = pad.center.integer_point() {
                return point;
            }
        }
        self.workspace().snap_authoring_point(world)
    }
    pub(crate) fn board_authoring_click(&mut self, world: PointNm, target: Option<String>) -> bool {
        let Some(setup) = self.workspace().authoring.board_controls.clone() else {
            return false;
        };
        if self.native_edit.pending() {
            return true;
        }
        if !setup.current(self.workspace()) {
            return self.board_authoring_error(
                "Board source, pane or layer changed; cancel and restart the tool",
            );
        }
        let point = if setup.kind == BoardToolKind::Outline {
            self.workspace().snap_authoring_point(world)
        } else {
            self.board_contact_point(world, target.as_deref())
        };
        let result = match setup.kind {
            BoardToolKind::Outline => {
                if self.workspace().authoring.board_outline.is_none() {
                    if let Err(error) = self.session.workspace_mut().begin_board_outline() {
                        return self.board_authoring_error(error);
                    }
                }
                self.session
                    .workspace_mut()
                    .append_board_outline_vertex(point)
            }
            BoardToolKind::Route => {
                if self.workspace().authoring.board_route.is_none() {
                    let pad = target
                        .as_ref()
                        .and_then(|id| {
                            self.workspace()
                                .scene
                                .pads
                                .iter()
                                .find(|p| &p.object_id == id)
                        })
                        .and_then(|p| p.pad_uuid.parse().ok());
                    let Some(pad) = pad else {
                        return self.board_authoring_error("Pick a connected Pad to start routing");
                    };
                    let result = self.session.workspace_mut().begin_board_route(
                        pad,
                        setup.layer,
                        WireRouteConfig::default(),
                    );
                    if result.is_ok() {
                        let preferred = self
                            .workspace()
                            .authoring
                            .board_route
                            .as_ref()
                            .and_then(|r| r.width_nm());
                        if setup.field(BoardField::Width).text.is_empty() {
                            if let Some(width) = preferred {
                                self.session
                                    .workspace_mut()
                                    .authoring
                                    .board_controls
                                    .as_mut()
                                    .unwrap()
                                    .set_length(BoardField::Width, width);
                            }
                        }
                    }
                    result
                } else {
                    self.apply_board_route_width().and_then(|()| {
                        self.session
                            .workspace_mut()
                            .authoring
                            .board_route
                            .as_mut()
                            .unwrap()
                            .append_vertex(point)
                    })
                }
            }
            BoardToolKind::Zone => {
                let configured = (|| -> Result<(), String> {
                    let net = setup.zone_net(self.workspace())?;
                    let settings = setup.zone_settings()?;
                    if self.workspace().authoring.board_zone.is_none() {
                        self.session
                            .workspace_mut()
                            .begin_board_zone(net, setup.layer)?;
                    } else if self
                        .workspace()
                        .authoring
                        .board_zone
                        .as_ref()
                        .unwrap()
                        .basis()
                        .net
                        != net
                    {
                        return Err("Cancel the boundary before changing its Net".into());
                    }
                    self.session
                        .workspace_mut()
                        .authoring
                        .board_zone
                        .as_mut()
                        .unwrap()
                        .configure(settings)?;
                    self.session.workspace_mut().append_board_zone_vertex(point)
                })();
                configured
            }
        };
        if let Err(error) = result {
            return self.board_authoring_error(error);
        }
        self.session.workspace_mut().authoring.gesture.preview = Some(point);
        self.session
            .workspace_mut()
            .authoring
            .gesture
            .target_object_id = target;
        self.session
            .workspace_mut()
            .authoring
            .board_controls
            .as_mut()
            .unwrap()
            .focus = None;
        if setup.kind == BoardToolKind::Outline {
            self.session
                .workspace_mut()
                .authoring
                .board_controls
                .as_mut()
                .unwrap()
                .refusal = None;
        }
        self.refresh_board_route_stop();
        self.invalidate_scene();
        true
    }
    fn apply_board_route_width(&mut self) -> Result<(), String> {
        let setup = self
            .workspace()
            .authoring
            .board_controls
            .as_ref()
            .ok_or("Start Route first")?;
        let width = setup.length(BoardField::Width)?;
        self.session
            .workspace_mut()
            .authoring
            .board_route
            .as_mut()
            .ok_or("Pick a start Pad")?
            .set_width_nm(width)
    }
    pub(crate) fn refresh_board_route_stop(&mut self) {
        let result = (|| -> Result<(), String> {
            let state = self.workspace();
            let Some(route) = &state.authoring.board_route else {
                return Ok(());
            };
            let width = state
                .authoring
                .board_controls
                .as_ref()
                .ok_or("Route setup unavailable")?
                .length(BoardField::Width)?;
            let model = state
                .library
                .basis
                .as_ref()
                .ok_or("Native source unavailable")?;
            let context = eda_engine::api::native_write::board_routing::stop::StopContext::capture(
                model,
                route.basis(),
            )
            .map_err(|e| e.to_string())?;
            let points = route.preview_points(state.authoring.gesture.preview)?;
            context
                .preview_path(&points, width)
                .map_err(|e| e.to_string())
        })();
        if let Some(setup) = &mut self.session.workspace_mut().authoring.board_controls {
            if setup.kind == BoardToolKind::Route {
                setup.refusal = result.err();
            }
        }
    }
    pub(crate) fn prepare_board_authoring_intent(
        &mut self,
        action: NativeEditAction,
    ) -> Result<Option<NativeEditIntent>, String> {
        if !matches!(
            action,
            NativeEditAction::BoardRoute
                | NativeEditAction::BoardOutline
                | NativeEditAction::BoardZone
                | NativeEditAction::CreateBoardClearance
        ) {
            return Ok(None);
        }
        if matches!(action, NativeEditAction::BoardZone)
            && self.workspace().authoring.board_controls.is_none()
        {
            let zone = match &self.workspace().selection {
                datum_gui_protocol::SelectionTarget::Native(view) if !view.stale => {
                    match &view.subject {
                        datum_gui_protocol::SelectionSubject::Object(id)
                            if id.class == datum_gui_protocol::AuthoredSelectionClass::Zone =>
                        {
                            Some(id.id)
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            return self
                .workspace()
                .prepare_board_zone_fill(zone, None)
                .map(Some);
        }
        if !self.board_authoring_current() {
            return Err("Board tool source changed; cancel and restart".into());
        }
        match action {
            NativeEditAction::BoardRoute => {
                self.apply_board_route_width()?;
                let mut draft = self
                    .session
                    .workspace_mut()
                    .authoring
                    .board_route
                    .take()
                    .ok_or("Pick a start Pad")?;
                // Finish saves anchored vertices only. The live cursor is not
                // another implicit click when the Inspector button is pressed.
                let end = draft.vertices().last().and_then(|point| {
                    self.workspace()
                        .scene
                        .pads
                        .iter()
                        .filter(|pad| {
                            pad.net_uuid.as_deref() == Some(draft.basis().net.to_string().as_str())
                        })
                        .find(|pad| {
                            pad.center.integer_point()
                                == Some(PointNm {
                                    x: point.x,
                                    y: point.y,
                                })
                        })
                        .and_then(|pad| pad.pad_uuid.parse().ok())
                });
                let intent = draft.prepare_intent(self.workspace(), None, end);
                self.session.workspace_mut().authoring.board_route = Some(draft);
                intent.map(Some)
            }
            NativeEditAction::BoardOutline => self
                .session
                .workspace_mut()
                .prepare_board_outline()
                .map(Some),
            NativeEditAction::BoardZone => self
                .session
                .workspace_mut()
                .prepare_configured_board_zone()
                .map(Some),
            NativeEditAction::CreateBoardClearance => {
                let setup = self.workspace().authoring.board_controls.as_ref().unwrap();
                let net = match setup.kind {
                    BoardToolKind::Route => {
                        self.workspace()
                            .authoring
                            .board_route
                            .as_ref()
                            .ok_or("Pick a start Pad")?
                            .basis()
                            .net
                    }
                    BoardToolKind::Zone => setup.zone_net(self.workspace())?,
                    BoardToolKind::Outline => {
                        return Err("Outline has no Net or clearance rule".into());
                    }
                };
                let model = self
                    .workspace()
                    .library
                    .basis
                    .as_ref()
                    .ok_or("Native source unavailable")?;
                let rule = datum_gui_protocol::native_edit_client::board_rule::BoardClearanceRule::capture(model,
                    net, setup.length(BoardField::Clearance)?, setup.rule_priority()?).map_err(|e| e.to_string())?;
                let mut intent = NativeEditIntent::new(
                    self.workspace()
                        .backing
                        .as_ref()
                        .ok_or("Native Project unavailable")?
                        .request
                        .project_root
                        .clone(),
                    rule.project,
                    rule.revision.clone(),
                    rule.tip,
                    action,
                );
                intent.clearance_rule = Some(rule);
                Ok(Some(intent))
            }
            _ => unreachable!(),
        }
    }
}
