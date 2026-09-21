//! Native owned-window host for the Project genesis form.

use super::*;
use crate::global_preferences_window::GlobalPreferencesWindowSurface;

const DEFAULT_NEW_PROJECT_SIZE: LogicalSize<f64> = LogicalSize::new(760.0, 720.0);
const MIN_NEW_PROJECT_SIZE: LogicalSize<f64> = LogicalSize::new(620.0, 620.0);

impl App {
    pub(super) fn sync_owned_product_windows(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        self.sync_global_preferences_window(event_loop)?;
        self.sync_project_preferences_window(event_loop)?;
        self.sync_new_project_window(event_loop)
    }

    pub(super) fn dispatch_owned_product_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) -> Option<WindowEvent> {
        if self
            .new_project_window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
        {
            self.new_project_window_event(event_loop, event);
            return None;
        }
        if self
            .global_preferences_window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
        {
            self.global_preferences_window_event(event_loop, event);
            return None;
        }
        if self
            .project_preferences_window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
        {
            self.project_preferences_window_event(event_loop, event);
            return None;
        }
        Some(event)
    }

    pub(super) fn sync_new_project_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let open = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.workspace().ui.new_project.open);
        if !open {
            let had_window = self.new_project_window.is_some();
            if let Some(window) = self.new_project_window.take() {
                window.set_visible(false);
            }
            if let Some(surface) = &self.new_project_surface {
                self.frames.close(surface.window_id());
            }
            self.new_project_surface = None;
            if had_window && let Some(window) = self.window {
                window.focus_window();
            }
            return Ok(());
        }
        if self.new_project_window.is_none() {
            let window = std::sync::Arc::new(
                event_loop.create_window(
                    WindowAttributes::default()
                        .with_title("New Project — Datum")
                        .with_inner_size(DEFAULT_NEW_PROJECT_SIZE)
                        .with_min_inner_size(MIN_NEW_PROJECT_SIZE)
                        .with_resizable(true)
                        .with_visible(false),
                )?,
            );
            owned_window_policy::establish_native_owner(
                self.window
                    .context("main window must exist before New Project")?,
                &window,
            )
            .context("establish native New Project owner")?;
            window.set_ime_allowed(false);
            let surface = GlobalPreferencesWindowSurface::new(
                self.runtime
                    .as_ref()
                    .context("runtime must exist before New Project window")?,
                window.clone(),
                self.args.visual_scale_factor,
            )?;
            self.frames.register(
                window.id(),
                surface.measurements.epoch(),
                window.inner_size(),
            );
            self.new_project_surface = Some(surface);
            self.new_project_window = Some(window.clone());
            owned_window_policy::show_owned_window(&window, &mut self.frames);
        }
        Ok(())
    }

    pub(super) fn new_project_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.close_new_project();
                }
                if let Err(error) = self.sync_new_project_window(event_loop) {
                    fatal_gui_error(event_loop, "close New Project window", error);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::Resized(size) => {
                if let (Some(runtime), Some(surface)) =
                    (&self.runtime, &mut self.new_project_surface)
                {
                    surface.resize(runtime, size.width, size.height);
                }
                if let Some(window) = &self.new_project_window {
                    self.frames.invalidate(window);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let scale_factor = self
                    .args
                    .visual_scale_factor
                    .map(f64::from)
                    .unwrap_or(scale_factor);
                if let Some(surface) = &mut self.new_project_surface {
                    surface.set_scale_factor(scale_factor);
                }
                if let Some(window) = &self.new_project_window {
                    self.frames.invalidate(window);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = &mut self.new_project_surface {
                    surface.set_cursor_position(
                        Some((position.x as f32, position.y as f32)),
                        &mut self.frames,
                    );
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = &mut self.new_project_surface {
                    surface.set_cursor_position(None, &mut self.frames);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                let target = self
                    .new_project_surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target);
                if let (Some(runtime), Some(target)) = (&mut self.runtime, target.as_ref()) {
                    runtime.activate_new_project_hit(target);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.modifiers = modifiers.state();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.handle_new_project_key(&event);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::RedrawRequested => {
                self.redraw_owned_window(event_loop, native_frame_adapters::OwnedHost::New)
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_window_minimum_fits_the_complete_genesis_form() {
        assert_eq!(DEFAULT_NEW_PROJECT_SIZE, LogicalSize::new(760.0, 720.0));
        assert_eq!(MIN_NEW_PROJECT_SIZE, LogicalSize::new(620.0, 620.0));
    }
}
