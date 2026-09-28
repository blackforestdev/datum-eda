//! Surface acquisition, frame composition, and window-system presentation.
//! Camera and pointer frames share the same compositor notification boundary.

use super::*;

impl Runtime {
    pub(super) fn trace_timing(&self, message: impl FnOnce() -> String) {
        if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
            eprintln!("[datum-timing] {}", message());
        }
    }

    pub(super) fn render(&mut self) -> Result<bool> {
        if self.device_health.failed() {
            return Ok(false);
        }
        self.renderer.render_session_mut().retry_content();
        let render_started = std::time::Instant::now();
        if !self.surface_transaction.begin_frame(
            &self.surface,
            &self.device,
            &self.config,
            &self.device_health,
        )? {
            return Ok(false);
        }
        let probe = gui_runtime_support::phase_probe::Probe::start("prepare");
        if gui_runtime_support::native_frame_probe::clear_only() {
            let acquire_started = std::time::Instant::now();
            let Some(mut frame) = self.surface_transaction.acquire(
                &self.surface,
                &self.device,
                &self.config,
                &self.device_health,
            )?
            else {
                return Ok(false);
            };
            let acquire_elapsed = acquire_started.elapsed();
            let view = frame.view();
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
            self.surface_transaction.present(frame, &self.window)?;
            self.trace_timing(|| {
                format!(
                    "runtime render diagnostic_clear=true total={}ms acquire={}ms renderer=0ms",
                    render_started.elapsed().as_millis(),
                    acquire_elapsed.as_millis()
                )
            });
            return Ok(true);
        }
        self.renderer.prepare_surface_attachment(
            &self.device,
            self.config.width,
            self.config.height,
            || !self.device_health.failed(),
        )?;
        let scene_started = std::time::Instant::now();
        let retained_was_cached = self.renderer.render_session().board().is_some();
        let prepared_was_cached = self.renderer.render_session().prepared().is_some();
        let mut retained_build_ms = 0;
        let mut prepared_build_ms = 0;
        if self.renderer.render_session().prepared().is_none() {
            append_gui_verbose_diagnostic_line(|| {
                format!("render scene prepare begin retained_cached={retained_was_cached}")
            });
            if self.renderer.render_session().board().is_none() {
                let retained_started = std::time::Instant::now();
                append_gui_verbose_diagnostic_line(|| "retained scene build begin");
                self.ensure_retained_scene();
                self.renderer.render_session_mut().check_content_budget()?;
                retained_build_ms = retained_started.elapsed().as_millis();
                append_gui_verbose_diagnostic_line(|| {
                    format!("retained scene build end {retained_build_ms}ms")
                });
            }
            let prepared_started = std::time::Instant::now();
            append_gui_verbose_diagnostic_line(|| "prepared scene build begin");
            self.build_terminal_prepared_scene()?;
            prepared_build_ms = prepared_started.elapsed().as_millis();
            append_gui_verbose_diagnostic_line(|| {
                format!("prepared scene build end {prepared_build_ms}ms")
            });
        }
        self.renderer.render_session_mut().check_content_budget()?;
        let scene_elapsed = scene_started.elapsed();
        // P2.2a: resolve the companion schematic world buffer lazily (cleared on
        // every scene/frame invalidation, so this stays fresh). `None` when the
        // workspace has no companion schematic / Schematic pane — second pass off.
        self.ensure_schematic_retained_scene();
        self.renderer.render_session_mut().check_content_budget()?;
        drop(probe);
        let probe = gui_runtime_support::phase_probe::Probe::start("renderer");
        let renderer_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "renderer render begin");
        use gui_runtime_support::native_surface_transaction::NativeRenderTarget;
        let plan = self.surface_transaction.prepare_render_plan(
            &mut self.renderer,
            self.config.width,
            self.config.height,
        )?;
        let mut target = NativeRenderTarget::new(
            &mut self.surface_transaction,
            &self.surface,
            &self.device,
            &self.queue,
            &self.config,
            &self.device_health,
        );
        let rendered = self.renderer.encode_frame(
            plan,
            &self.device,
            &self.queue,
            &mut target,
            &mut NativeRenderTarget::acquire,
            &mut NativeRenderTarget::submitted,
        );
        let (frame, acquire_elapsed) = target.finish(&self.renderer);
        let Some(submitted) = rendered? else {
            return Ok(false);
        };
        let frame = frame.context("rendered native frame must own an acquisition")?;
        let renderer_elapsed = renderer_started.elapsed();
        append_gui_verbose_diagnostic_line(|| {
            format!("renderer render end {}ms", renderer_elapsed.as_millis())
        });
        drop(probe);
        if self.device_health.failed() {
            return Ok(false);
        }
        let present_elapsed = self.present_native_frame(frame, submitted)?;
        append_gui_verbose_diagnostic_line(|| {
            format!(
                "frame present end {}ms total={}ms",
                present_elapsed.as_millis(),
                render_started.elapsed().as_millis()
            )
        });
        self.trace_timing(|| format!(
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
        submitted: datum_gui_render::SubmittedFrame,
    ) -> Result<std::time::Duration> {
        let probe = gui_runtime_support::phase_probe::Probe::start("present");
        let started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "frame present begin");
        let first_device_frame = !self.surface_transaction.has_presented();
        self.surface_transaction.present_rendered(
            frame,
            &self.window,
            &mut self.renderer,
            submitted,
        )?;
        if let Some((hits, console)) = self.renderer.render_session_mut().take_published_frame() {
            self.presented_console_layout = console;
            self.presented_hits.replace(hits);
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
