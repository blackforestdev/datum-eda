//! Runtime dock/terminal viewport geometry (T0-C02, decomposed from
//! `runtime_camera_pane.rs` under source-size governance / decision 022):
//! dock tab ownership (open/close, moved from `main.rs` under the same
//! governance), dock cursor/resize-drag handling plus the terminal
//! screen-cell geometry seam — the ONE shared solver both the renderer and the PTY size derive
//! from (`DATUM_NATIVE_TERMINAL_SPEC.md` §2.3; decision 027 FT-001/FT-008).
//! A child module of the crate root, so it sees `Runtime`'s private
//! fields/methods via `use super::*` exactly as the inline impl did.

use super::*;

fn hovered_terminal_close_session(target: Option<&HitTarget>) -> Option<String> {
    match target {
        Some(HitTarget::TerminalSessionClose(session_id)) => Some(session_id.clone()),
        _ => None,
    }
}

fn terminal_tab_session(target: Option<&HitTarget>) -> Option<&str> {
    match target {
        Some(
            HitTarget::TerminalSessionTab(session_id) | HitTarget::TerminalSessionClose(session_id),
        ) => Some(session_id),
        _ => None,
    }
}

fn terminal_tab_drag_start(target: Option<&HitTarget>) -> Option<String> {
    match target {
        Some(HitTarget::TerminalSessionTab(session_id)) if !session_id.is_empty() => {
            Some(session_id.clone())
        }
        _ => None,
    }
}

#[cfg(test)]
fn owns_dock_resize_cursor(target: Option<&HitTarget>, drag_active: bool) -> bool {
    drag_active || matches!(target, Some(HitTarget::DockResizeHandle))
}

pub(super) fn toggle_terminal_maximized(ui: &mut datum_gui_protocol::WorkspaceUiState) -> bool {
    if ui.active_dock_tab != Some(DockTab::Terminal) {
        return false;
    }
    ui.terminal_maximized = !ui.terminal_maximized;
    true
}

impl Runtime {
    pub(super) fn pointer_cursor_icon(&mut self, pointer: (f32, f32)) -> winit::window::CursorIcon {
        if pointer.1 >= self.current_layout().bottom_strip.y
            && let Some(icon) = self
                .dock_resize_cursor_icon(pointer)
                .or_else(|| self.terminal_split_cursor_icon(pointer))
                .or_else(|| self.terminal_tab_cursor_icon(pointer))
                .or_else(|| self.terminal_link_cursor_icon(pointer))
        {
            return icon;
        }
        match self.divider_resize_cursor(pointer.0, pointer.1) {
            Some(datum_gui_protocol::SplitOrientation::Vertical) => {
                winit::window::CursorIcon::EwResize
            }
            Some(datum_gui_protocol::SplitOrientation::Horizontal) => {
                winit::window::CursorIcon::NsResize
            }
            None => winit::window::CursorIcon::Default,
        }
    }

    pub(super) fn terminal_split_cursor_icon(
        &mut self,
        pointer: (f32, f32),
    ) -> Option<winit::window::CursorIcon> {
        let direction = if let Some(drag) = self.terminal_split_drag.as_ref() {
            Some(drag.direction)
        } else {
            let path = match self.presented_hits.hit_test(pointer.0, pointer.1) {
                Some(HitTarget::TerminalSplitDivider(path)) => Some(path.clone()),
                _ => None,
            };
            path.and_then(|path| {
                self.active_terminal_split_dividers()
                    .into_iter()
                    .find(|divider| divider.path == path)
                    .map(|divider| divider.direction)
            })
        }?;
        Some(match direction {
            datum_gui_protocol::TerminalSplitDirection::SideBySide => {
                winit::window::CursorIcon::EwResize
            }
            datum_gui_protocol::TerminalSplitDirection::Stacked => {
                winit::window::CursorIcon::NsResize
            }
        })
    }

    fn active_terminal_split_dividers(
        &self,
    ) -> Vec<datum_gui_viewport::TerminalSplitDividerGeometry> {
        let root = self.terminal_screen_geometry();
        self.workspace()
            .ui
            .terminal
            .active_tab_id
            .as_deref()
            .and_then(|tab_id| {
                self.workspace()
                    .ui
                    .terminal
                    .tab_layouts
                    .iter()
                    .find(|tab| tab.tab_id == tab_id)
            })
            .map(|tab| datum_gui_viewport::terminal_split_dividers(root, tab))
            .unwrap_or_default()
    }

    pub(super) fn begin_terminal_split_drag(&mut self) -> bool {
        let Some(pointer) = self.last_cursor_pos else {
            return false;
        };
        let path = match self.presented_hits.hit_test(pointer.0, pointer.1) {
            Some(HitTarget::TerminalSplitDivider(path)) => path.clone(),
            _ => return false,
        };
        let Some(divider) = self
            .active_terminal_split_dividers()
            .into_iter()
            .find(|divider| divider.path == path)
        else {
            return false;
        };
        self.terminal_split_drag = Some(TerminalSplitDividerDrag {
            path,
            direction: divider.direction,
            split_bounds: divider.split_bounds,
        });
        true
    }

    pub(super) fn advance_terminal_split_drag(&mut self, pointer: (f32, f32)) -> bool {
        let Some(drag) = self.terminal_split_drag.as_ref() else {
            return false;
        };
        let path = drag.path.clone();
        let ratio = drag.ratio_millis_at(pointer);
        if let Err(error) = self.terminal_sessions.set_active_split_ratio(&path, ratio) {
            self.log_terminal_event(format!("terminal split resize failed: {error}"));
            self.terminal_split_drag = None;
            return false;
        }
        self.sync_terminal_tabs();
        self.invalidate_frame();
        true
    }

    pub(super) fn finish_terminal_split_drag(&mut self) -> Option<winit::window::CursorIcon> {
        self.terminal_split_drag.take()?;
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        Some(
            self.last_cursor_pos
                .and_then(|pointer| self.terminal_split_cursor_icon(pointer))
                .unwrap_or(winit::window::CursorIcon::Default),
        )
    }

    pub(super) fn dock_resize_cursor_icon(
        &mut self,
        pointer: (f32, f32),
    ) -> Option<winit::window::CursorIcon> {
        if self.dock_drag_active {
            return Some(winit::window::CursorIcon::NsResize);
        }
        let strip = self.current_layout().bottom_strip;
        let over_handle = self.workspace().ui.active_dock_tab.is_some()
            && pointer.0 >= strip.x
            && pointer.0 <= strip.x + strip.width
            && pointer.1 >= strip.y
            && pointer.1 <= strip.y + 6.0;
        over_handle.then_some(winit::window::CursorIcon::NsResize)
    }

    pub(super) fn terminal_tab_cursor_icon(
        &mut self,
        pointer: (f32, f32),
    ) -> Option<winit::window::CursorIcon> {
        if self.terminal_tab_drag.is_some() {
            return Some(winit::window::CursorIcon::Grabbing);
        }
        matches!(
            self.presented_hits.hit_test(pointer.0, pointer.1),
            Some(HitTarget::TerminalSessionTab(_))
        )
        .then_some(winit::window::CursorIcon::Grab)
    }

    pub(super) fn begin_terminal_tab_drag(&mut self) -> bool {
        let Some(pointer) = self.last_cursor_pos else {
            return false;
        };
        let session_id =
            terminal_tab_drag_start(self.presented_hits.hit_test(pointer.0, pointer.1));
        let Some(session_id) = session_id else {
            return false;
        };
        let Some(tab_x) = self.presented_hits.regions().iter().find_map(|region| {
            (region.target == HitTarget::TerminalSessionTab(session_id.clone()))
                .then_some(region.rect.x)
        }) else {
            return false;
        };
        self.terminal_tab_drag = Some(terminal_tab_drag::TerminalTabDrag::new(
            session_id, pointer, tab_x,
        ));
        self.terminal_tab_drag_release_suppressed = false;
        true
    }

    pub(super) fn advance_terminal_tab_drag(&mut self, pointer: (f32, f32)) -> bool {
        let target_id = terminal_tab_session(self.presented_hits.hit_test(pointer.0, pointer.1))
            .map(str::to_string);
        let Some(drag) = &mut self.terminal_tab_drag else {
            return false;
        };
        let changed = drag.advance(pointer, target_id.as_deref());
        let visual = drag.visual_state(pointer.0);
        let visual_changed = self.workspace().ui.terminal_tab_drag != visual;
        if visual_changed {
            self.session.workspace_mut().ui.terminal_tab_drag = visual;
        }
        if changed || visual_changed {
            self.invalidate_frame();
        }
        changed || visual_changed
    }

    pub(super) fn finish_terminal_tab_drag(&mut self) -> bool {
        if std::mem::take(&mut self.terminal_tab_drag_release_suppressed) {
            return true;
        }
        let Some(drag) = self.terminal_tab_drag.take() else {
            return false;
        };
        self.session.workspace_mut().ui.terminal_tab_drag = None;
        if let Some(target_id) = drag.target_session_id()
            && let Err(err) = self
                .terminal_sessions
                .reorder_session(drag.session_id(), target_id)
        {
            self.log_terminal_event(format!("terminal tab reorder failed: {err}"));
        }
        self.select_hit_target(&HitTarget::TerminalSessionTab(
            drag.session_id().to_string(),
        ))
    }

    pub(super) fn cancel_terminal_tab_drag(&mut self) -> bool {
        let canceled = self.terminal_tab_drag.take().is_some();
        self.terminal_tab_drag_release_suppressed |= canceled;
        if self
            .session
            .workspace_mut()
            .ui
            .terminal_tab_drag
            .take()
            .is_some()
        {
            self.invalidate_frame();
        }
        canceled
    }

    pub(super) fn update_terminal_tab_hover(&mut self, pointer: (f32, f32)) -> bool {
        let next = if pointer.1 >= self.current_layout().bottom_strip.y {
            hovered_terminal_close_session(self.presented_hits.hit_test(pointer.0, pointer.1))
        } else {
            None
        };
        if self.workspace().ui.hovered_terminal_close_session_id == next {
            return false;
        }
        self.session
            .workspace_mut()
            .ui
            .hovered_terminal_close_session_id = next;
        self.invalidate_frame();
        true
    }

    pub(super) fn clear_terminal_tab_hover(&mut self) -> bool {
        if self
            .session
            .workspace_mut()
            .ui
            .hovered_terminal_close_session_id
            .take()
            .is_none()
        {
            return false;
        }
        self.invalidate_frame();
        true
    }

    /// A mouse-aware child may consume terminal pointer events only after the
    /// terminal screen owns keyboard focus. The activation press therefore
    /// establishes focus through the shared cell rectangle before mouse
    /// reporting runs; otherwise the child's report would swallow the only
    /// click that can make Tab and text belong to the PTY.
    pub(super) fn focus_terminal_screen_before_mouse_report(&mut self) -> bool {
        let terminal_visible =
            matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal));
        let child_mouse_reporting = self.workspace().ui.terminal.mouse_reporting_mode.is_some();
        let over_screen = self
            .last_cursor_pos
            .and_then(|(x, y)| self.terminal_screen_cell_at(x, y))
            .is_some();
        let next = keyboard_focus::focus_before_terminal_mouse_press(
            self.application_focus(),
            terminal_visible,
            child_mouse_reporting,
            over_screen,
        );
        if next != self.application_focus() {
            self.set_application_focus(next);
        }
        next == ApplicationFocus::Terminal && over_screen
    }

    pub(super) fn set_active_dock(&mut self, tab: DockTab) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        if ui.active_dock_tab == Some(tab) {
            return false;
        }
        let dock_was_open = ui.active_dock_tab.is_some();
        ui.active_dock_tab = Some(tab);
        if dock_was_open {
            self.invalidate_frame();
        } else {
            self.invalidate_scene();
        }
        if matches!(tab, DockTab::Terminal) {
            self.resize_terminal_to_dock();
            self.refresh_terminal_activity_summary();
        }
        true
    }

    pub(super) fn close_active_dock(&mut self) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        if ui.active_dock_tab.is_none() {
            return false;
        }
        ui.active_dock_tab = None;
        ui.terminal_maximized = false;
        ui.hovered_terminal_close_session_id = None;
        ui.terminal_tab_drag = None;
        ui.terminal_clipboard_menu = None;
        self.terminal_text_selection_drag = None;
        self.terminal_split_drag = None;
        // TF-01: keyboard focus must not outlive the surface that owns it —
        // a closed dock with Terminal focus would swallow keys without a
        // visible recipient. Closing the dock hands ownership back to the editor.
        if self.application_focus() == ApplicationFocus::Terminal {
            let pane = self.workspace().ui.layout.focused;
            self.set_application_focus(ApplicationFocus::Editor(pane));
        }
        self.invalidate_scene();
        true
    }
}

#[cfg(test)]
#[path = "runtime_terminal_dock_tests.rs"]
mod hover_tests;
