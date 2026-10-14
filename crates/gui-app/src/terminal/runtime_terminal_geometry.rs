//! Shared terminal viewport geometry, cell hit testing, and PTY resize seam.

use super::*;

impl Runtime {
    pub(super) fn begin_dock_resize_drag(&mut self, pointer: (f32, f32)) -> bool {
        if self.workspace().ui.active_dock_tab.is_none() {
            return false;
        }
        let strip = self.current_layout().bottom_strip;
        let handle = datum_gui_render::RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 6.0,
        };
        if !handle.contains(pointer.0, pointer.1) {
            return false;
        }
        self.dock_drag_active = true;
        true
    }

    pub(super) fn toggle_terminal_maximized(&mut self) -> bool {
        if !crate::runtime_terminal_dock::toggle_terminal_maximized(
            &mut self.session.workspace_mut().ui,
        ) {
            return false;
        }
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }

    pub(super) fn cursor_in_dock(&self) -> bool {
        let Some((_, y)) = self.last_cursor_pos else {
            return false;
        };
        y >= self.current_layout().bottom_strip.y
    }

    pub(super) fn handle_dock_resize_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let window_height = self.config.height as f32;
        let new_height_physical =
            (window_height - next_cursor_pos.1).clamp(32.0, window_height * 0.6);
        let new_height_logical = (new_height_physical / self.scale_factor.max(0.01)) as u32;
        if self.workspace().ui.dock_height_px == new_height_logical
            && !self.workspace().ui.terminal_maximized
        {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        ui.terminal_maximized = false;
        ui.dock_height_px = new_height_logical;
        self.invalidate_frame();
        true
    }

    pub(super) fn finish_dock_resize_drag(&mut self) -> Option<winit::window::CursorIcon> {
        if !std::mem::take(&mut self.dock_drag_active) {
            return None;
        }
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        Some(
            self.last_cursor_pos
                .and_then(|pointer| self.dock_resize_cursor_icon(pointer))
                .unwrap_or(winit::window::CursorIcon::Default),
        )
    }

    pub(super) fn terminal_screen_geometry(&self) -> datum_gui_viewport::TerminalScreenGeometry {
        let layout =
            self.shell_layout_for_dock(Some(self.workspace().ui.effective_dock_height_px()));
        datum_gui_viewport::terminal_screen_geometry_with_chrome_scale(
            layout.bottom_strip.into(),
            self.workspace().ui.terminal.font_scale_millis,
            self.scale_factor,
        )
    }

    pub(super) fn terminal_screen_cell_at(&self, x: f32, y: f32) -> Option<(u16, u16)> {
        self.terminal_screen_geometry().cell_at(x, y)
    }

    pub(super) fn click_terminal_screen(&mut self) -> bool {
        if let Some((column, row)) = self
            .last_cursor_pos
            .and_then(|(x, y)| self.terminal_screen_cell_at(x, y))
        {
            self.trace_click(format!("terminal screen cell ({column}, {row})"));
        }
        true
    }

    /// PTY rows/columns derive from the exact visible cell rectangle used by
    /// the renderer (T0-C02), never from a separate chrome estimate.
    pub(super) fn resize_terminal_to_dock(&mut self) {
        let root_geometry = self.terminal_screen_geometry();
        let active_layout = self
            .workspace()
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
            .cloned();
        let panes = active_layout
            .as_ref()
            .map(|tab| datum_gui_viewport::terminal_split_geometries(root_geometry, tab))
            .unwrap_or_default();
        let focused = panes
            .iter()
            .find(|pane| pane.focused)
            .map(|pane| pane.geometry)
            .unwrap_or(root_geometry);
        let (cols, rows) = (focused.columns, focused.rows);
        append_gui_verbose_diagnostic_line(|| format!("terminal resize begin {cols}x{rows}"));
        let result = if panes.is_empty() {
            self.terminal_sessions.resize_active_surface(
                cols,
                rows,
                focused.screen.width.round() as u32,
                focused.screen.height.round() as u32,
            )
        } else {
            self.terminal_sessions.resize_active_tab_surfaces(&panes)
        };
        match result {
            Ok(()) => {
                let terminal = &mut self.session.workspace_mut().ui.terminal;
                terminal.columns = cols;
                terminal.rows = rows;
                append_gui_verbose_diagnostic_line(|| "terminal resize end");
            }
            Err(err) => {
                append_gui_diagnostic_line(format!("terminal resize failed: {err}"));
                self.log_terminal_event(format!("terminal resize failed: {err}"));
            }
        }
    }
}
