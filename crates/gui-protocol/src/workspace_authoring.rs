//! Workspace authoring gesture and handoff ownership.
use crate::*;

impl ReviewWorkspaceState {
    pub fn set_tool(&mut self, tool: WorkspaceTool) -> bool {
        if self.tool == tool {
            return false;
        }
        self.tool = tool;
        self.authoring.gesture = AuthoringGestureState::idle(tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "set_tool".to_string(),
            detail: format!("tool {}", tool.label()),
        });
        true
    }

    pub fn snap_authoring_point(&self, point: PointNm) -> PointNm {
        let snap = self.authoring.snap;
        if !snap.enabled || snap.grid_nm <= 0 {
            return point;
        }
        PointNm {
            x: snap_nm(point.x, snap.grid_nm),
            y: snap_nm(point.y, snap.grid_nm),
        }
    }

    pub fn begin_authoring_gesture(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if self.tool == WorkspaceTool::Select {
            return false;
        }
        let point = self.snap_authoring_point(world);
        self.authoring.gesture = AuthoringGestureState {
            tool: self.tool,
            anchor: Some(point),
            preview: Some(point),
            target_object_id,
        };
        self.last_command_status = Some(EditorCommandStatus {
            action: "begin_authoring_gesture".to_string(),
            detail: format!("{} @ {},{}", self.tool.label(), point.x, point.y),
        });
        true
    }

    pub fn preview_authoring_gesture(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if !self.authoring.gesture.is_active() {
            return false;
        }
        let point = self.snap_authoring_point(world);
        if self.authoring.gesture.preview == Some(point)
            && self.authoring.gesture.target_object_id == target_object_id
        {
            return false;
        }
        self.authoring.gesture.preview = Some(point);
        if self.tool != WorkspaceTool::Move {
            self.authoring.gesture.target_object_id = target_object_id;
        }
        true
    }

    pub fn cancel_authoring_gesture(&mut self) -> bool {
        if !self.authoring.gesture.is_active() {
            return false;
        }
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "cancel_authoring_gesture".to_string(),
            detail: format!("cancelled {}", self.tool.label()),
        });
        true
    }

    pub fn finish_draw_board_track_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::DrawBoardTrack {
            return None;
        }
        let from = self.authoring.gesture.anchor?;
        let to = self.snap_authoring_point(world);
        if from == to {
            self.authoring.gesture.preview = Some(to);
            self.last_command_status = Some(EditorCommandStatus {
                action: "draw_board_track".to_string(),
                detail: "track end must differ from start".to_string(),
            });
            return None;
        }
        let handoff = self.draw_board_track_handoff(from, to)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "draw_board_track".to_string(),
            detail: format!("queued track {},{} -> {},{}", from.x, from.y, to.x, to.y),
        });
        Some(handoff)
    }

    pub fn finish_place_board_via_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::PlaceBoardVia {
            return None;
        }
        let point = self.snap_authoring_point(world);
        let handoff = self.place_board_via_handoff(point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "place_board_via".to_string(),
            detail: format!("queued via @ {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn finish_place_board_text_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::PlaceBoardText {
            return None;
        }
        let point = self.snap_authoring_point(world);
        let handoff = self.place_board_text_handoff(point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "place_board_text".to_string(),
            detail: format!("queued board text @ {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn finish_move_component_handoff(
        &mut self,
        world: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        if self.tool != WorkspaceTool::Move {
            return None;
        }
        let target = self.authoring.gesture.target_object_id.clone()?;
        let point = self.snap_authoring_point(world);
        let handoff = self.move_component_handoff(&target, point)?;
        self.authoring.gesture = AuthoringGestureState::idle(self.tool);
        self.last_command_status = Some(EditorCommandStatus {
            action: "move_board_component".to_string(),
            detail: format!("queued component move {target} -> {},{}", point.x, point.y),
        });
        Some(handoff)
    }

    pub fn draw_board_track_handoff(
        &self,
        from: PointNm,
        to: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let net_uuid = self.review.net_uuid.as_deref().or_else(|| {
            self.scene
                .net_display
                .first()
                .map(|net| net.net_uuid.as_str())
        })?;
        let layer = self
            .review
            .proposal_actions
            .first()
            .map(|action| action.layer)
            .unwrap_or(1);
        let width_nm = self
            .review
            .proposal_actions
            .first()
            .map(|action| action.width_nm)
            .unwrap_or(200_000);
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("draw-board-track"),
            shell_quote_arg(&root),
            shell_quote_arg("--net"),
            shell_quote_arg(net_uuid),
            shell_quote_arg("--from-x-nm"),
            shell_quote_arg(&from.x.to_string()),
            shell_quote_arg("--from-y-nm"),
            shell_quote_arg(&from.y.to_string()),
            shell_quote_arg("--to-x-nm"),
            shell_quote_arg(&to.x.to_string()),
            shell_quote_arg("--to-y-nm"),
            shell_quote_arg(&to.y.to_string()),
            shell_quote_arg("--width-nm"),
            shell_quote_arg(&width_nm.to_string()),
            shell_quote_arg("--layer"),
            shell_quote_arg(&layer.to_string()),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.draw_board_track".to_string(),
            mcp_alias: Some("datum.pcb.draw_board_track".to_string()),
            command,
        })
    }

    pub fn place_board_via_handoff(&self, at: PointNm) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let net_uuid = self.default_authoring_net_uuid()?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("place-board-via"),
            shell_quote_arg(&root),
            shell_quote_arg("--net"),
            shell_quote_arg(net_uuid),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&at.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&at.y.to_string()),
            shell_quote_arg("--drill-nm"),
            shell_quote_arg("300000"),
            shell_quote_arg("--diameter-nm"),
            shell_quote_arg("600000"),
            shell_quote_arg("--from-layer"),
            shell_quote_arg("1"),
            shell_quote_arg("--to-layer"),
            shell_quote_arg("16"),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.place_board_via".to_string(),
            mcp_alias: Some("datum.pcb.place_board_via".to_string()),
            command,
        })
    }

    pub fn place_board_text_handoff(&self, at: PointNm) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("place-board-text"),
            shell_quote_arg(&root),
            shell_quote_arg("--text"),
            shell_quote_arg("TEXT"),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&at.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&at.y.to_string()),
            shell_quote_arg("--render-intent"),
            shell_quote_arg("annotation"),
            shell_quote_arg("--h-align"),
            shell_quote_arg("center"),
            shell_quote_arg("--v-align"),
            shell_quote_arg("center"),
            shell_quote_arg("--layer"),
            shell_quote_arg("21"),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.place_board_text".to_string(),
            mcp_alias: Some("datum.pcb.place_board_text".to_string()),
            command,
        })
    }

    pub fn delete_authored_object_handoff(
        &self,
        target_object_id: &str,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let root = backing.request.project_root.to_string_lossy();
        let (command_id, verb, flag, uuid) = delete_command_parts_from_object_id(target_object_id)?;
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg(verb),
            shell_quote_arg(&root),
            shell_quote_arg(flag),
            shell_quote_arg(uuid),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: command_id.to_string(),
            mcp_alias: Some(command_id.to_string()),
            command,
        })
    }

    pub fn move_component_handoff(
        &self,
        target_object_id: &str,
        to: PointNm,
    ) -> Option<TerminalCommandHandoff> {
        let backing = self.backing.as_ref()?;
        let component = self.component_from_target_object_id(target_object_id)?;
        let root = backing.request.project_root.to_string_lossy();
        let command = [
            shell_quote_arg("datum-eda"),
            shell_quote_arg("project"),
            shell_quote_arg("move-board-component"),
            shell_quote_arg(&root),
            shell_quote_arg("--component"),
            shell_quote_arg(&component.component_uuid),
            shell_quote_arg("--x-nm"),
            shell_quote_arg(&to.x.to_string()),
            shell_quote_arg("--y-nm"),
            shell_quote_arg(&to.y.to_string()),
        ]
        .join(" ");
        Some(TerminalCommandHandoff {
            command_id: "datum.pcb.move_board_component".to_string(),
            mcp_alias: Some("datum.pcb.move_board_component".to_string()),
            command,
        })
    }

    fn default_authoring_net_uuid(&self) -> Option<&str> {
        self.review.net_uuid.as_deref().or_else(|| {
            self.scene
                .net_display
                .first()
                .map(|net| net.net_uuid.as_str())
        })
    }

    fn component_from_target_object_id(&self, target_object_id: &str) -> Option<&ComponentBounds> {
        let normalized = target_object_id
            .strip_prefix("component:")
            .and_then(|component_uuid| {
                self.scene
                    .components
                    .iter()
                    .find(|component| component.component_uuid == component_uuid)
                    .map(|component| component.object_id.as_str())
            })
            .unwrap_or(target_object_id);
        self.scene
            .components
            .iter()
            .find(|component| component.object_id == normalized)
    }
}
