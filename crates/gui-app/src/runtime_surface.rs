//! Native surface dimensions, configuration and resize invalidation ownership.
//! Preserve immediate logical/input/terminal updates; size-independent authored
//! geometry survives physical resizing while prepared layout remains fresh.

use super::*;

impl Runtime {
    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.surface_transaction.resize(width, height);
        if width != 0 && height != 0 {
            self.apply_resize(width, height);
        }
    }

    pub(super) fn set_scale_factor(&mut self, scale_factor: f64) {
        let next = (scale_factor as f32).max(0.01);
        if (self.scale_factor - next).abs() <= f32::EPSILON {
            return;
        }
        append_gui_diagnostic_line(format!(
            "scale factor apply {:.4} -> {:.4}",
            self.scale_factor, next
        ));
        self.scale_factor = next;
        self.presented_console_layout = None;
        self.presented_hits.clear();
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_scene();
    }

    fn apply_resize(&mut self, width: u32, height: u32) {
        // Backend reconfiguration is owned by SurfaceTransaction. An unchanged
        // logical extent must not discard prepared input/layout while it waits.
        if self.config.width == width && self.config.height == height {
            return;
        }
        append_gui_diagnostic_line(format!(
            "resize apply {}x{} -> {width}x{height}",
            self.config.width, self.config.height
        ));
        self.config.width = width;
        self.config.height = height;
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_surface_size();
    }
}
