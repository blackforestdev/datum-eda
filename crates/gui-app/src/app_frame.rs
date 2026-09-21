//! Main-window redraw completion and explicit capture/smoke orchestration.

use super::*;

impl App {
    pub(super) fn redraw_main_window(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_deref() else {
            return;
        };
        let Some(receipt) = self.frames.begin_frame(window.id()) else {
            return;
        };
        let presented = if let Some(runtime) = &mut self.runtime {
            append_gui_verbose_diagnostic_line(|| "redraw handler begin");
            let started = std::time::Instant::now();
            let presented = match runtime.render() {
                Ok(presented) => presented,
                Err(_) if runtime.device_health.failed() => false,
                Err(error) => {
                    self.frames.render_failed(window.id(), &error);
                    false
                }
            };
            runtime.trace_timing(format!("redraw render {}ms", started.elapsed().as_millis()));
            presented
        } else {
            false
        };
        self.frames.frame_finished(window, receipt, presented);
        if self.advance_kwin_lifecycle_smoke(event_loop) {
            return;
        }
        if self.advance_native_resize_smoke(presented) || !presented {
            return;
        }
        // Explicit interaction setup is one-shot, not another animation loop.
        if presented
            && std::mem::take(&mut self.args.interaction_smoke)
            && let Some(runtime) = &mut self.runtime
            && let Err(err) = runtime.run_interaction_smoke()
        {
            fatal_gui_error(event_loop, "interaction smoke failed", err);
        }
        if let Some(runtime) = &mut self.runtime {
            if self.args.visual_test {
                let screenshot_out = self.args.screenshot_out.as_ref().unwrap_or_else(|| {
                    fatal_gui_error(
                        event_loop,
                        "visual screenshot failed",
                        "--screenshot-out is required",
                    )
                });
                if let Err(err) = runtime.write_visual_screenshot(screenshot_out) {
                    fatal_gui_error(event_loop, "visual screenshot failed", err);
                }
                if self.args.exit_after_screenshot {
                    event_loop.exit();
                }
            }
            append_gui_verbose_diagnostic_line(|| "redraw handler end");
        }
    }
}

impl Runtime {
    pub(super) fn run_interaction_smoke(&mut self) -> Result<()> {
        let size = self.window.inner_size();
        self.resize(size.width, size.height);
        // This explicit synchronous proof runner waits for its previous frame;
        // ordinary event-loop rendering uses nonblocking queue admission.
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(2)),
        })?;
        anyhow::ensure!(
            self.render().context("interaction smoke initial render")?,
            "interaction smoke initial frame was not presented"
        );

        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before interaction smoke click")?;
        let click = (
            prepared.scene_viewport.x + prepared.scene_viewport.width * 0.5,
            prepared.scene_viewport.y + prepared.scene_viewport.height * 0.5,
        );
        self.last_cursor_pos = Some(click);
        let _ = self.update_hover(click);
        let _ = self.handle_primary_click();
        // This explicit synchronous proof runner waits for its previous frame;
        // ordinary event-loop rendering uses nonblocking queue admission.
        self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(2)),
        })?;
        anyhow::ensure!(
            self.render().context("interaction smoke click render")?,
            "interaction smoke click frame was not presented"
        );
        Ok(())
    }
}
