//! PCB tools participate in existing editor keyboard ownership.
use super::*;
use winit::keyboard::{Key, NamedKey};
impl Runtime {
    pub(crate) fn handle_board_authoring_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        self.handle_board_authoring_input(&event.logical_key, event.state)
    }
    pub(crate) fn handle_board_authoring_input(&mut self, key: &Key, state: ElementState) -> bool {
        if !self
            .workspace()
            .authoring
            .board_controls
            .as_ref()
            .is_some_and(|c| c.owns_pane(self.workspace()))
            || self.workspace().ui.layout.focused_content() != PaneContent::Board
            || self.workspace().inspector_context.owner
                == datum_gui_protocol::inspector_context::InspectorOwner::Navigator
            || self.application_focus()
                != datum_gui_protocol::ApplicationFocus::Editor(self.workspace().ui.layout.focused)
            || state != ElementState::Pressed
        {
            return false;
        }
        if self.native_edit.pending() {
            return true;
        }
        if matches!(key, Key::Named(NamedKey::Escape)) {
            return self.handle_board_control(BoardControl::Cancel);
        }
        if !self.board_authoring_current() {
            let setup = self.workspace().authoring.board_controls.as_ref().unwrap();
            if setup.kind == BoardToolKind::Outline {
                match key {
                    Key::Named(NamedKey::Tab) => {
                        return self.focus_next_board_control(self.modifiers.shift_key());
                    }
                    Key::Named(NamedKey::Enter | NamedKey::Space) => {
                        let recovery = setup.focus.filter(|control| {
                            matches!(control, BoardControl::Outline | BoardControl::Cancel)
                        });
                        let enabled = recovery.filter(|control| self.presented_hits.regions().iter().any(|hit| {
                            matches!(&hit.target, HitTarget::BoardAuthoringNavigation { basis, kind, offsets, .. }
                                if *basis == setup.basis && *kind == setup.kind && offsets.iter().any(|(c,_,_)| c == control))
                        }));
                        return enabled.is_none_or(|control| self.handle_board_control(control));
                    }
                    _ => {}
                }
            }
            return self.board_authoring_error("Board source changed; cancel and restart");
        }
        if self.handle_shared_editor_input(key, state) {
            return true;
        }
        match key {
            Key::Named(NamedKey::Enter | NamedKey::Space) => {
                let focus = self
                    .workspace()
                    .authoring
                    .board_controls
                    .as_ref()
                    .and_then(|s| s.focus);
                let action = match focus {
                    Some(BoardControl::Field(_)) | None => BoardControl::Finish,
                    Some(control) => control,
                };
                self.handle_board_control(action)
            }
            Key::Named(NamedKey::Tab) => self.focus_next_board_control(self.modifiers.shift_key()),
            Key::Named(NamedKey::Backspace) => {
                let result =
                    if let Some(route) = &mut self.session.workspace_mut().authoring.board_route {
                        route.remove_last_vertex();
                        Ok(())
                    } else if self.workspace().authoring.board_outline.is_some() {
                        self.session
                            .workspace_mut()
                            .remove_board_outline_vertex()
                            .map(|_| ())
                    } else if self.workspace().authoring.board_zone.is_some() {
                        self.session
                            .workspace_mut()
                            .remove_board_zone_vertex()
                            .map(|_| ())
                    } else {
                        Ok(())
                    };
                if let Err(error) = result {
                    return self.board_authoring_error(error);
                }
                if self.workspace().tool == WorkspaceTool::DrawBoardOutline {
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
            _ if self.modifiers.control_key() || self.modifiers.super_key() => false,
            _ => true,
        }
    }
}

impl Runtime {
    fn focus_next_board_control(&mut self, reverse: bool) -> bool {
        let setup = self.workspace().authoring.board_controls.as_ref().unwrap();
        let layout = self
            .presented_hits
            .regions()
            .iter()
            .find_map(|hit| match &hit.target {
                HitTarget::BoardAuthoringNavigation {
                    basis,
                    kind,
                    offsets,
                    total,
                    visible,
                } if *basis == setup.basis && *kind == setup.kind => {
                    Some((offsets.clone(), *total, *visible))
                }
                _ => None,
            });
        let Some((mut controls, total, visible)) = layout else {
            self.invalidate_frame();
            return true;
        };
        // A stale draft can navigate only its rendered, enabled recovery actions.
        if !self.board_authoring_current() {
            controls.retain(|(control, _, _)| {
                matches!(control, BoardControl::Outline | BoardControl::Cancel)
            });
        }
        let focused = setup
            .focus
            .and_then(|focus| controls.iter().position(|(c, _, _)| *c == focus));
        let next = datum_gui_protocol::popup_menu::next_available_index(
            controls.len(),
            focused,
            reverse,
            |_| true,
        );
        if let Some((control, row, span)) = next.and_then(|i| controls.get(i)).copied() {
            let state = self.session.workspace_mut();
            state.authoring.board_controls.as_mut().unwrap().focus = Some(control);
            let navigation = &mut state.ui.selection_inspector;
            // Fields/actions occupy at least two rows; reveal the complete
            // control, using the same bounded viewport service as selection.
            navigation.scroll_offset = datum_gui_viewport::scroll::revealed_row_span_offset(
                navigation.scroll_offset,
                total,
                visible,
                row,
                span,
            );
        }
        self.invalidate_frame();
        true
    }
}
