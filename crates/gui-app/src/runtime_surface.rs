//! Native surface dimensions, configuration and resize invalidation ownership.
//! Preserve immediate surface/input/terminal updates; size-independent authored
//! geometry survives physical resizing while prepared layout remains fresh.

use super::*;

impl Runtime {
    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.apply_resize(width.max(1), height.max(1));
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
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_scene();
    }

    fn apply_resize(&mut self, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }
        append_gui_diagnostic_line(format!(
            "resize apply {}x{} -> {width}x{height}",
            self.config.width, self.config.height
        ));
        self.config.width = width;
        self.config.height = height;
        append_gui_diagnostic_line("surface configure begin");
        self.surface.configure(&self.device, &self.config);
        append_gui_diagnostic_line("surface configure end");
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_surface_size();
    }

    pub(super) fn run_resize_torture_smoke(&mut self) -> Result<()> {
        let restored = (1344_u32, 806_u32);
        let maximized = (1920_u32, 1051_u32);
        append_gui_verbose_diagnostic_line("resize torture begin");
        for (index, (width, height)) in [
            maximized, restored, maximized, restored, maximized, restored,
        ]
        .into_iter()
        .enumerate()
        {
            append_gui_verbose_diagnostic_line(format!(
                "resize torture step {index} target {width}x{height}"
            ));
            let retained_before = self
                .retained_scene
                .as_ref()
                .filter(|scene| scene.can_reuse_for_surface_resize())
                .map(|scene| scene.world_vertices().as_ptr());
            // Exercise a burst and verify retained ownership survives both axes.
            self.resize(width + 11, height);
            self.resize(width, height + 17);
            self.resize(width, height);
            anyhow::ensure!(
                (self.config.width, self.config.height) == (width, height),
                "resize input geometry did not reach the latest dimensions"
            );
            self.render()
                .with_context(|| format!("resize torture render step {index} {width}x{height}"))?;
            if let Some(vertices) = retained_before {
                anyhow::ensure!(
                    self.retained_scene
                        .as_ref()
                        .is_some_and(|scene| scene.world_vertices().as_ptr() == vertices),
                    "physical resize rebuilt independent world geometry"
                );
            }
        }
        // DPI is a separate dependency and must still discard retained data.
        let scale = self.scale_factor;
        self.set_scale_factor(f64::from(scale + 0.25));
        anyhow::ensure!(self.retained_scene.is_none() && self.schematic_retained_scene.is_none());
        self.set_scale_factor(f64::from(scale));
        self.render().context("resize smoke restored DPI")?;
        append_gui_verbose_diagnostic_line("resize torture end");
        Ok(())
    }
}
