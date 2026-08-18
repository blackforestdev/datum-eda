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

impl Runtime {
    pub(super) fn terminal_tab_cursor_icon(
        &mut self,
        pointer: (f32, f32),
    ) -> Option<winit::window::CursorIcon> {
        if self.terminal_tab_drag.is_some() {
            return Some(winit::window::CursorIcon::Grabbing);
        }
        matches!(
            self.prepared_scene().hit_test(pointer.0, pointer.1),
            Some(HitTarget::TerminalSessionTab(_))
        )
        .then_some(winit::window::CursorIcon::Grab)
    }

    pub(super) fn begin_terminal_tab_drag(&mut self) -> bool {
        let Some(pointer) = self.last_cursor_pos else {
            return false;
        };
        let prepared = self.prepared_scene();
        let session_id = terminal_tab_drag_start(prepared.hit_test(pointer.0, pointer.1));
        let Some(session_id) = session_id else {
            return false;
        };
        let Some(tab_x) = prepared.hit_regions.iter().find_map(|region| {
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
        let target_id = terminal_tab_session(self.prepared_scene().hit_test(pointer.0, pointer.1))
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
            self.log_review_event(format!("terminal tab reorder failed: {err}"));
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
        let next =
            hovered_terminal_close_session(self.prepared_scene().hit_test(pointer.0, pointer.1));
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
        ui.hovered_terminal_close_session_id = None;
        ui.terminal_tab_drag = None;
        ui.terminal_clipboard_menu = None;
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

    pub(super) fn cursor_in_dock(&self) -> bool {
        let Some((_, y)) = self.last_cursor_pos else {
            return false;
        };
        let layout = self.current_layout();
        y >= layout.bottom_strip.y
    }

    pub(super) fn handle_dock_resize_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let window_height = self.config.height as f32;
        let new_height_physical =
            (window_height - next_cursor_pos.1).clamp(32.0, window_height * 0.6);
        let new_height_logical = new_height_physical / self.scale_factor.max(0.01);
        let new_height_logical = new_height_logical as u32;
        if self.workspace().ui.dock_height_px == new_height_logical {
            return false;
        }
        self.session.workspace_mut().ui.dock_height_px = new_height_logical;
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }

    /// The terminal lane geometry for the current surface — the ONE shared
    /// solver (`datum_gui_viewport::terminal_screen_geometry`) the renderer
    /// also draws with, so drawn rows/columns always equal PTY rows/columns
    /// (T0-C02, DATUM_NATIVE_TERMINAL_SPEC.md §2.3). The dock height
    /// preference is applied even while the dock is closed: the PTY keeps the
    /// size the terminal will have when shown.
    pub(super) fn terminal_screen_geometry(&self) -> datum_gui_viewport::TerminalScreenGeometry {
        let layout = ShellLayout::for_surface(
            self.config.width,
            self.config.height,
            self.scale_factor,
            Some(self.workspace().ui.dock_height_px),
        );
        datum_gui_viewport::terminal_screen_geometry(layout.bottom_strip.into())
    }

    /// The terminal cell under a screen point as `(column, row)`, or `None`
    /// outside the visible cell rectangle — the coordinate seam the later
    /// text-selection phase anchors on (T0-C02: the screen hit target carries
    /// cell coordinates).
    pub(super) fn terminal_screen_cell_at(&self, x: f32, y: f32) -> Option<(u16, u16)> {
        self.terminal_screen_geometry().cell_at(x, y)
    }

    /// A primary click on the terminal SCREEN (the `TerminalScreen` hit
    /// target). Focus entry itself is applied by `select_hit_target` through
    /// `hit_target_is_terminal_entry`; here the click resolves its cell
    /// coordinates through the shared geometry.
    pub(super) fn click_terminal_screen(&mut self) -> bool {
        if let Some((column, row)) = self
            .last_cursor_pos
            .and_then(|(x, y)| self.terminal_screen_cell_at(x, y))
        {
            self.trace_click(format!("terminal screen cell ({column}, {row})"));
        }
        true
    }

    /// T0-C02: PTY rows/columns are derived from the exact visible cell
    /// rectangle via the shared geometry — never from a separate chrome
    /// estimate (the retired 76px budget drift).
    pub(super) fn resize_terminal_to_dock(&mut self) {
        let geometry = self.terminal_screen_geometry();
        let (cols, rows) = (geometry.columns, geometry.rows);
        append_gui_verbose_diagnostic_line(format!("terminal resize begin {cols}x{rows}"));
        match self.terminal_sessions.resize_active(cols, rows) {
            Ok(()) => {
                let terminal = &mut self.session.workspace_mut().ui.terminal;
                terminal.columns = cols;
                terminal.rows = rows;
                append_gui_verbose_diagnostic_line("terminal resize end");
            }
            Err(err) => {
                append_gui_diagnostic_line(format!("terminal resize failed: {err}"));
                self.log_review_event(format!("terminal resize failed: {err}"));
            }
        }
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[test]
    fn only_the_per_tab_close_target_owns_close_hover() {
        let close = HitTarget::TerminalSessionClose("terminal-2".to_string());
        assert_eq!(
            hovered_terminal_close_session(Some(&close)).as_deref(),
            Some("terminal-2")
        );
        assert_eq!(
            hovered_terminal_close_session(Some(&HitTarget::TerminalSessionNew)),
            None
        );
        assert_eq!(hovered_terminal_close_session(None), None);
    }

    #[test]
    fn tab_body_starts_reorder_but_close_control_remains_exclusive() {
        let tab = HitTarget::TerminalSessionTab("terminal-2".to_string());
        let close = HitTarget::TerminalSessionClose("terminal-2".to_string());
        assert_eq!(
            terminal_tab_drag_start(Some(&tab)).as_deref(),
            Some("terminal-2")
        );
        assert_eq!(terminal_tab_drag_start(Some(&close)), None);
        assert_eq!(terminal_tab_session(Some(&close)), Some("terminal-2"));
    }
}
