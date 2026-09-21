//! Surface acquisition, frame composition, and window-system presentation.
//! Camera and pointer frames share the same compositor notification boundary.

use super::*;

impl Runtime {
    pub(super) fn render(&mut self) -> Result<bool> {
        if self.device_health.failed() {
            return Ok(false);
        }
        let render_started = std::time::Instant::now();
        let acquire_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| {
            format!("render begin {}x{}", self.config.width, self.config.height)
        });
        append_gui_verbose_diagnostic_line(|| "render acquire begin");
        let Some(mut frame) = self.surface_transaction.acquire(
            &self.surface,
            &self.device,
            &self.config,
            &self.device_health,
        )?
        else {
            return Ok(false);
        };
        let probe = gui_runtime_support::phase_probe::Probe::start("prepare");
        let acquire_elapsed = acquire_started.elapsed();
        append_gui_verbose_diagnostic_line(|| "render acquire end");
        let view = frame.view();
        if gui_runtime_support::native_frame_probe::clear_only() {
            let submission = gui_runtime_support::native_frame_probe::submit_clear(
                &self.device,
                &self.queue,
                &view,
            );
            self.surface_transaction
                .submitted(&mut frame, &self.queue, submission);
            drop(probe);
            if self.device_health.failed() {
                return Ok(false);
            }
            self.present_native_frame(frame)?;
            self.trace_timing(format!(
                "runtime render diagnostic_clear=true total={}ms acquire={}ms renderer=0ms",
                render_started.elapsed().as_millis(),
                acquire_elapsed.as_millis()
            ));
            return Ok(true);
        }
        self.renderer.prepare_surface_attachment(
            &self.device,
            self.config.width,
            self.config.height,
            || !self.device_health.failed(),
        )?;
        let scene_started = std::time::Instant::now();
        let retained_was_cached = self.retained_scene.is_some();
        let prepared_was_cached = self.prepared_scene.is_some();
        let mut retained_build_ms = 0;
        let mut prepared_build_ms = 0;
        if self.prepared_scene.is_none() {
            append_gui_verbose_diagnostic_line(|| {
                format!("render scene prepare begin retained_cached={retained_was_cached}")
            });
            self.scene_dirty = false;
            if self.retained_scene.is_none() {
                let retained_started = std::time::Instant::now();
                append_gui_verbose_diagnostic_line(|| "retained scene build begin");
                self.retained_scene = Some(RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                ));
                retained_build_ms = retained_started.elapsed().as_millis();
                append_gui_verbose_diagnostic_line(|| {
                    format!("retained scene build end {retained_build_ms}ms")
                });
            }
            let prepared_started = std::time::Instant::now();
            append_gui_verbose_diagnostic_line(|| "prepared scene build begin");
            self.prepared_scene = Some(self.build_terminal_prepared_scene()?);
            prepared_build_ms = prepared_started.elapsed().as_millis();
            append_gui_verbose_diagnostic_line(|| {
                format!("prepared scene build end {prepared_build_ms}ms")
            });
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
        drop(probe);
        let probe = gui_runtime_support::phase_probe::Probe::start("renderer");
        let renderer_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "renderer render begin");
        let rendered = self.renderer.render_with_submission(
            &self.device,
            &self.queue,
            &view,
            prepared,
            retained,
            schematic_retained,
            self.config.width,
            self.config.height,
            &mut |submission| {
                self.surface_transaction
                    .submitted(&mut frame, &self.queue, submission)
            },
        );
        self.surface_transaction
            .observe_attachment(&self.renderer, &frame);
        rendered?;
        let renderer_elapsed = renderer_started.elapsed();
        append_gui_verbose_diagnostic_line(|| {
            format!("renderer render end {}ms", renderer_elapsed.as_millis())
        });
        drop(probe);
        if self.device_health.failed() {
            return Ok(false);
        }
        let present_elapsed = self.present_native_frame(frame)?;
        append_gui_verbose_diagnostic_line(|| {
            format!(
                "frame present end {}ms total={}ms",
                present_elapsed.as_millis(),
                render_started.elapsed().as_millis()
            )
        });
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

    fn present_native_frame(
        &mut self,
        frame: gui_runtime_support::native_surface_transaction::NativeSurfaceFrame,
    ) -> Result<std::time::Duration> {
        let probe = gui_runtime_support::phase_probe::Probe::start("present");
        let started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "frame present begin");
        let first_device_frame = !self.surface_transaction.has_presented();
        self.surface_transaction.present(frame, self.window)?;
        self.presented_console_layout = self
            .prepared_scene
            .as_ref()
            .and_then(PreparedScene::console_overlay_layout);
        if let Some(prepared) = self.prepared_scene.as_mut() {
            self.presented_hits.present(prepared);
        }
        if first_device_frame && self.terminal_owns_input() {
            let (x, y, width, height) = self.terminal_ime_cursor_rect();
            self.window.set_ime_cursor_area(
                winit::dpi::PhysicalPosition::new(x, y),
                winit::dpi::PhysicalSize::new(width, height),
            );
        }
        drop(probe);
        Ok(started.elapsed())
    }
}
