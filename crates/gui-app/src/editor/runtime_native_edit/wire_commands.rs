//! Context commands configure the existing captured Move and canonical edit path.
use super::*;
use datum_gui_protocol::gui_menu_model::wire_commands::WireCommand;
use datum_gui_protocol::{AuthoredSelectionClass as Class, SelectionSubject, SelectionTarget};
use eda_engine::{
    api::native_write::{
        schematic_wire_edit::{WireEditAction, WireEditBasis, WireHandle},
        schematic_wire_edit_adapter::WireEditParams,
    },
    ir::geometry::Point,
};

impl Runtime {
    fn configure_wire_route(&mut self, command: WireCommand) -> bool {
        let result = (|| -> Result<()> {
            anyhow::ensure!(
                !self.native_edit.pending()
                    && self.native_edit.preview.is_none()
                    && !self.check_work.pending(),
                "Finish the current Move first"
            );
            datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(
                self.workspace(),
                self.workspace().ui.layout.focused,
            )
            .map_err(anyhow::Error::msg)?;
            if let Some(reason) = command.unavailable_reason(self.workspace()) {
                anyhow::bail!(reason);
            }
            let editor = &mut self.session.workspace_mut().schematic_authoring;
            match command {
                WireCommand::SetRouting(routing) => editor.wire_route.routing = routing,
                WireCommand::SwapCorner => {
                    use eda_engine::api::native_write::schematic_wire_edit::WireCornerOrder;
                    editor.wire_route.corner = match editor.wire_route.corner {
                        WireCornerOrder::Start => WireCornerOrder::End,
                        WireCornerOrder::End => WireCornerOrder::Start,
                    };
                }
                WireCommand::RemoveLastVertex => {
                    editor.remove_last_wire_vertex();
                }
                _ => anyhow::bail!("Choose a wire routing option"),
            }
            editor.preview_dirty = true;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.log_console_echo(ConsoleFeedbackSource::Tool,match self.workspace().schematic_authoring.wire_route.routing {
                    eda_engine::api::native_write::schematic_wire_edit::WireRouting::Free=>"Wire routing: free",
                    eda_engine::api::native_write::schematic_wire_edit::WireRouting::Orthogonal=>"Wire routing: 90 degrees",
                    eda_engine::api::native_write::schematic_wire_edit::WireRouting::Diagonal45=>"Wire routing: 45 degrees",
                });
                self.invalidate_frame();
            }
            Err(error) => self.log_console_refusal(ConsoleFeedbackSource::Tool, error.to_string()),
        }
        true
    }

    pub(crate) fn activate_wire_command(
        &mut self,
        command: WireCommand,
        pickup: Option<PointNm>,
    ) -> bool {
        if command.is_configuration() {
            return self.configure_wire_route(command);
        }
        let result = (|| -> Result<_> {
            anyhow::ensure!(
                !self.native_edit.pending()
                    && self.native_edit.preview.is_none()
                    && !self.check_work.pending(),
                "Finish or cancel the current edit first"
            );
            let SelectionTarget::Native(view) = &self.workspace().selection else {
                anyhow::bail!("Select a current wire")
            };
            let SelectionSubject::Object(id) = &view.subject else {
                anyhow::bail!("Choose one wire handle")
            };
            anyhow::ensure!(
                !view.stale && id.class == Class::Wire,
                "Choose one current wire handle"
            );
            let id = id.clone();
            let pane = self.workspace().ui.layout.focused;
            let view_basis = datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(
                self.workspace(),
                pane,
            )
            .map_err(anyhow::Error::msg)?;
            let world = pickup
                .or_else(|| {
                    self.last_cursor_pos.and_then(|(x, y)| {
                        self.prepared_scene()?
                            .world_point_at_screen(x, y)
                            .map(|(p, _)| p)
                    })
                })
                .context("Point at a wire handle in its source pane")?;
            let scene = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane)
                .context("Wire source unavailable")?;
            let mut edit: WireEditParams = self
                .wire_edit_configuration(scene, &id, world)?
                .context("Choose a wire handle")?;
            let sheet = self
                .workspace()
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&view_basis.source))
                .context("Wire source unavailable")?;
            if command == WireCommand::ReconnectEnd {
                edit.handle = self.wire_reconnect_handle(scene, &id, world)?;
            }
            let basis = WireEditBasis::capture(sheet, id.id, edit.edit_seed)?;
            let at = match (command, edit.handle) {
                (WireCommand::ReconnectEnd, WireHandle::Endpoint { .. }) => world,
                (WireCommand::Split, WireHandle::Segment { .. }) => {
                    edit.action = WireEditAction::Split;
                    self.workspace()
                        .schematic_authoring
                        .wire_point_for_sheet(sheet.uuid, &id.id.to_string(), world)
                        .map_err(anyhow::Error::msg)?
                        .context("Choose a point inside the wire span")?
                }
                (WireCommand::RemoveSpan, WireHandle::Segment { .. }) => {
                    edit.action = WireEditAction::RemoveSpan;
                    let anchor = basis.anchor(&edit.handle)?;
                    PointNm {
                        x: anchor.x,
                        y: anchor.y,
                    }
                }
                (WireCommand::RemoveVertex, WireHandle::Vertex { point }) => {
                    edit.action = WireEditAction::RemoveVertex;
                    PointNm {
                        x: point.x,
                        y: point.y,
                    }
                }
                (WireCommand::InsertVertex, WireHandle::Segment { from, to }) => {
                    edit.handle = WireHandle::InsertVertex { from, to };
                    world
                }
                _ => anyhow::bail!(
                    "Point inside a span for Split, Insert Vertex or Remove Segment; point at a bend for Remove Vertex"
                ),
            };
            edit.plan(&basis, Point::new(at.x, at.y))?;
            Ok((id, world, edit, at))
        })();
        let (id, world, edit, at) = match result {
            Ok(value) => value,
            Err(error) => {
                self.log_console_refusal(ConsoleFeedbackSource::Menu, error.to_string());
                return true;
            }
        };
        let previous_tool = self.workspace().tool;
        self.dispatch_session_command(SessionCommand::SetTool(WorkspaceTool::Move));
        self.begin_schematic_move(SelectionSubject::Object(id), Some(world), Some(edit));
        if matches!(
            command,
            WireCommand::InsertVertex | WireCommand::ReconnectEnd
        ) {
            if self.native_edit.preview.is_some() {
                self.native_edit.previous_context_tool = Some(previous_tool);
            } else {
                self.dispatch_session_command(SessionCommand::SetTool(previous_tool));
            }
            return true;
        }
        if let Some(preview) = self.native_edit.preview.as_ref() {
            match preview.action(at) {
                Ok(action) => {
                    self.queue_native_edit(action);
                }
                Err(error) => {
                    self.log_console_refusal(ConsoleFeedbackSource::Menu, error.to_string());
                    self.cancel_native_move();
                }
            }
        }
        self.dispatch_session_command(SessionCommand::SetTool(previous_tool));
        true
    }
}

impl Runtime {
    pub(crate) fn restore_context_move_tool(&mut self) {
        if let Some(tool) = self.native_edit.previous_context_tool.take() {
            self.dispatch_session_command(SessionCommand::SetTool(tool));
        }
    }
}
