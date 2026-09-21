//! Native owned-window host for the Units-only Project Preferences surface.

use super::*;
use crate::global_preferences_window::{
    DEFAULT_PREFERENCES_SIZE, GlobalPreferencesWindowSurface, MIN_PREFERENCES_SIZE,
};

impl App {
    pub(super) fn sync_project_preferences_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        let open = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.workspace().ui.project_preferences.open);
        if !open {
            let had_window = self.project_preferences_window.is_some();
            if let Some(window) = self.project_preferences_window.take() {
                window.set_visible(false);
            }
            if let Some(surface) = &self.project_preferences_surface {
                self.frames.close(surface.window_id());
            }
            self.project_preferences_surface = None;
            if had_window && let Some(window) = self.window {
                window.focus_window();
            }
            return Ok(());
        }

        if self.project_preferences_window.is_none() {
            let window = std::sync::Arc::new(
                event_loop.create_window(
                    WindowAttributes::default()
                        .with_title("Project Preferences — Datum")
                        .with_inner_size(DEFAULT_PREFERENCES_SIZE)
                        .with_min_inner_size(MIN_PREFERENCES_SIZE)
                        .with_resizable(true)
                        .with_visible(false),
                )?,
            );
            owned_window_policy::establish_native_owner(
                self.window
                    .context("main window must exist before Project Preferences")?,
                &window,
            )
            .context("establish native Project Preferences owner")?;
            window.set_ime_allowed(false);
            let surface = GlobalPreferencesWindowSurface::new(
                self.runtime
                    .as_ref()
                    .context("runtime must exist before Project Preferences window")?,
                window.clone(),
                self.args.visual_scale_factor,
            )?;
            self.frames
                .register(window.id(), surface.measurements.epoch());
            self.project_preferences_surface = Some(surface);
            self.project_preferences_window = Some(window.clone());
            owned_window_policy::show_owned_window(&window, &mut self.frames);
        }

        let raise = self
            .runtime
            .as_mut()
            .is_some_and(Runtime::take_project_preferences_raise_request);
        if raise && let Some(window) = &self.project_preferences_window {
            owned_window_policy::raise_owned_window(window, &mut self.frames);
        }
        Ok(())
    }

    pub(super) fn project_preferences_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.close_project_preferences();
                }
                if let Err(error) = self.sync_project_preferences_window(event_loop) {
                    fatal_gui_error(event_loop, "close Project Preferences window", error);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::Resized(size) => {
                if let (Some(runtime), Some(surface)) =
                    (&self.runtime, &mut self.project_preferences_surface)
                {
                    surface.resize(runtime, size.width, size.height);
                }
                if let Some(window) = &self.project_preferences_window {
                    self.frames.invalidate(window);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let scale_factor = self
                    .args
                    .visual_scale_factor
                    .map(f64::from)
                    .unwrap_or(scale_factor);
                if let Some(surface) = &mut self.project_preferences_surface {
                    surface.set_scale_factor(scale_factor);
                }
                if let Some(window) = &self.project_preferences_window {
                    self.frames.invalidate(window);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = &mut self.project_preferences_surface {
                    surface.set_cursor_position(
                        Some((position.x as f32, position.y as f32)),
                        &mut self.frames,
                    );
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(surface) = &mut self.project_preferences_surface {
                    surface.scrollbar_input(ElementState::Released, &mut self.frames);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = &mut self.project_preferences_surface {
                    surface.set_cursor_position(None, &mut self.frames);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if self
                    .project_preferences_surface
                    .as_mut()
                    .is_some_and(|surface| surface.scrollbar_input(state, &mut self.frames))
                    || state != ElementState::Released
                {
                    return;
                }
                let target = self
                    .project_preferences_surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target);
                if let (Some(runtime), Some(target)) = (&mut self.runtime, target.as_ref()) {
                    let _ = runtime.activate_project_preferences_hit_target(target);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll_preferences_window(delta, true);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.modifiers = modifiers.state();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.handle_project_preferences_key(&event);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::RedrawRequested => {
                self.redraw_owned_window(event_loop, native_frame_adapters::OwnedHost::Project)
            }
            _ => {}
        }
    }
}
