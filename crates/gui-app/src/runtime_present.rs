//! Surface acquisition, frame composition, and window-system presentation.
//! Camera and pointer frames share the same compositor notification boundary.

use super::*;

impl Runtime {
    pub(super) fn render(&mut self) -> Result<bool> {
        let render_started = std::time::Instant::now();
        let acquire_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(format!(
            "render begin {}x{}",
            self.config.width, self.config.height
        ));
        append_gui_verbose_diagnostic_line("render acquire begin");
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                append_gui_diagnostic_line(format!(
                    "surface acquire recovered by reconfigure at {}x{}",
                    self.config.width, self.config.height
                ));
                self.surface.configure(&self.device, &self.config);
                self.invalidate_frame();
                return Ok(false);
            }
            Err(wgpu::SurfaceError::Timeout) => {
                append_gui_diagnostic_line("surface acquire timeout; frame skipped");
                self.invalidate_frame();
                return Ok(false);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                anyhow::bail!("surface out of memory");
            }
            Err(err) => {
                anyhow::bail!("acquire next surface texture: {err}");
            }
        };
        let acquire_elapsed = acquire_started.elapsed();
        append_gui_verbose_diagnostic_line("render acquire end");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scene_started = std::time::Instant::now();
        let retained_was_cached = self.retained_scene.is_some();
        let prepared_was_cached = self.prepared_scene.is_some();
        let mut retained_build_ms = 0;
        let mut prepared_build_ms = 0;
        if self.prepared_scene.is_none() {
            append_gui_verbose_diagnostic_line(format!(
                "render scene prepare begin retained_cached={retained_was_cached}"
            ));
            self.scene_dirty = false;
            if self.retained_scene.is_none() {
                let retained_started = std::time::Instant::now();
                append_gui_verbose_diagnostic_line("retained scene build begin");
                self.retained_scene = Some(RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                ));
                retained_build_ms = retained_started.elapsed().as_millis();
                append_gui_verbose_diagnostic_line(format!(
                    "retained scene build end {retained_build_ms}ms"
                ));
            }
            let prepared_started = std::time::Instant::now();
            append_gui_verbose_diagnostic_line("prepared scene build begin");
            self.prepared_scene = Some(self.build_terminal_prepared_scene()?);
            prepared_build_ms = prepared_started.elapsed().as_millis();
            append_gui_verbose_diagnostic_line(format!(
                "prepared scene build end {prepared_build_ms}ms"
            ));
        }
        let scene_elapsed = scene_started.elapsed();
        // P2.2a: resolve the companion schematic world buffer lazily (cleared on
        // every scene/frame invalidation, so this stays fresh). `None` when the
        // workspace has no companion schematic / Schematic pane — second pass off.
        if self.schematic_retained_scene.is_none() {
            self.schematic_retained_scene = RetainedScene::from_workspace_schematic_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before render")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before render")?;
        let schematic_retained = self.schematic_retained_scene.as_ref();
        let renderer_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("renderer render begin");
        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            prepared,
            retained,
            schematic_retained,
            self.config.width,
            self.config.height,
        )?;
        let renderer_elapsed = renderer_started.elapsed();
        append_gui_verbose_diagnostic_line(format!(
            "renderer render end {}ms",
            renderer_elapsed.as_millis()
        ));
        let present_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("frame present begin");
        // Wayland frame callbacks pace subsequent redraws; notify immediately
        // before presentation, after the rendering commands have been submitted.
        self.window.pre_present_notify();
        frame.present();
        let present_elapsed = present_started.elapsed();
        append_gui_verbose_diagnostic_line(format!(
            "frame present end {}ms total={}ms",
            present_elapsed.as_millis(),
            render_started.elapsed().as_millis()
        ));
        self.trace_timing(format!(
            "runtime render total={}ms acquire={}ms scene={}ms retained_build={}ms prepared_build={}ms renderer={}ms present={}ms retained_was_cached={} prepared_was_cached={}",
            render_started.elapsed().as_millis(),
            acquire_elapsed.as_millis(),
            scene_elapsed.as_millis(),
            retained_build_ms,
            prepared_build_ms,
            renderer_elapsed.as_millis(),
            present_elapsed.as_millis(),
            retained_was_cached,
            prepared_was_cached
        ));
        Ok(true)
    }
}
