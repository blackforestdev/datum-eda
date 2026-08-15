//! Runtime dock/terminal viewport geometry (T0-C02, decomposed from
//! `runtime_camera_pane.rs` under source-size governance / decision 022):
//! dock tab ownership (open/close, moved from `main.rs` under the same
//! governance), dock cursor/resize-drag handling plus the terminal
//! screen-cell geometry seam — the ONE shared solver both the renderer and the PTY size derive
//! from (`DATUM_NATIVE_TERMINAL_SPEC.md` §2.3; decision 027 FT-001/FT-008).
//! A child module of the crate root, so it sees `Runtime`'s private
//! fields/methods via `use super::*` exactly as the inline impl did.

use super::*;

impl Runtime {
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
        // TF-01: keyboard focus must not outlive the surface that owns it —
        // a closed dock with Terminal focus would swallow keys without a
        // visible recipient. Closing the dock hands ownership back to the editor.
        if self.keyboard_focus == KeyboardFocus::Terminal {
            self.set_keyboard_focus(KeyboardFocus::Editor);
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
