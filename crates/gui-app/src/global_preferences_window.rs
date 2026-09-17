//! Native-window host for Global Preferences.
//!
//! The preference service and interaction state remain in `Runtime`; this type
//! owns only the second native surface, its renderer, and window-local pointer
//! projection. The Design workspace never becomes a backdrop for Preferences.

use super::*;
pub(super) const DEFAULT_PREFERENCES_SIZE: LogicalSize<f64> = LogicalSize::new(960.0, 720.0);
pub(super) const MIN_PREFERENCES_SIZE: LogicalSize<f64> = LogicalSize::new(700.0, 540.0);

pub(super) struct GlobalPreferencesWindowSurface {
    surface: wgpu::Surface<'static>,
    window: std::sync::Arc<Window>,
    config: wgpu::SurfaceConfiguration,
    scale_factor: f32,
    renderer: Renderer,
    retained: Option<RetainedScene>,
    prepared: Option<PreparedScene>,
    cursor_position: Option<(f32, f32)>,
}

impl GlobalPreferencesWindowSurface {
    pub(super) fn new(
        runtime: &Runtime,
        window: std::sync::Arc<Window>,
        scale_factor_override: Option<f32>,
    ) -> Result<Self> {
        let surface = runtime
            .instance
            .create_surface(window.clone())
            .context("create Global Preferences native surface")?;
        let caps = surface.get_capabilities(&runtime.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| *format == runtime.config.format)
            .or_else(|| {
                caps.formats
                    .iter()
                    .copied()
                    .find(wgpu::TextureFormat::is_srgb)
            })
            .unwrap_or(caps.formats[0]);
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(caps.present_modes[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&runtime.device, &config);
        let renderer = Renderer::new(
            &runtime.device,
            &runtime.queue,
            format,
            select_msaa_samples(&runtime.adapter, format),
        );
        Ok(Self {
            surface,
            config,
            scale_factor: scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32),
            renderer,
            retained: None,
            prepared: None,
            cursor_position: None,
            window,
        })
    }

    pub(super) fn invalidate(&mut self) {
        self.retained = None;
        self.prepared = None;
    }

    pub(super) fn resize(&mut self, runtime: &Runtime, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&runtime.device, &self.config);
        self.invalidate();
    }

    pub(super) fn set_scale_factor(&mut self, scale_factor: f64) {
        let next = (scale_factor as f32).max(0.01);
        if (self.scale_factor - next).abs() > f32::EPSILON {
            self.scale_factor = next;
            self.invalidate();
        }
    }

    pub(super) fn set_cursor_position(&mut self, position: Option<(f32, f32)>) {
        self.cursor_position = position;
    }

    pub(super) fn hit_target(&self) -> Option<HitTarget> {
        let (x, y) = self.cursor_position?;
        self.prepared.as_ref()?.hit_test(x, y).cloned()
    }

    pub(super) fn render(
        &mut self,
        runtime: &Runtime,
        project_preferences: bool,
        new_project: bool,
    ) -> Result<()> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&runtime.device, &self.config);
                self.invalidate();
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                anyhow::bail!("Global Preferences surface out of memory")
            }
            Err(error) => anyhow::bail!("acquire Global Preferences surface texture: {error}"),
        };
        if self.prepared.is_none() {
            if new_project {
                let mut workspace = runtime.workspace().clone();
                workspace.ui.global_preferences.open = false;
                self.retained = Some(RetainedScene::from_workspace_for_surface(
                    &workspace,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                ));
                self.prepared = Some(PreparedScene::from_workspace_with_terminal_renderer(
                    &workspace,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                    runtime.camera,
                    self.retained.as_ref().expect("retained scene initialized"),
                    &[],
                    None,
                    true,
                ));
            } else {
                let dialog = if project_preferences {
                    &runtime.workspace().ui.project_preferences
                } else {
                    &runtime.workspace().ui.global_preferences
                };
                self.retained = Some(RetainedScene::empty());
                self.prepared = Some(PreparedScene::from_native_preferences(
                    dialog,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                ));
            }
        }
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.render(
            &runtime.device,
            &runtime.queue,
            &view,
            self.prepared
                .as_ref()
                .context("Global Preferences prepared scene must exist")?,
            self.retained
                .as_ref()
                .context("Global Preferences retained scene must exist")?,
            None,
            self.config.width,
            self.config.height,
        )?;
        self.window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

impl App {
    /// Scrolling changes only the owned dialog's transient viewport. Coalesced
    /// redraws must not rebuild the main Design window or other owned windows.
    pub(super) fn scroll_preferences_window(&mut self, delta: MouseScrollDelta, project: bool) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        let ui = &mut runtime.session.workspace_mut().ui;
        let (dialog, surface, window) = if project {
            (
                &mut ui.project_preferences,
                &mut self.project_preferences_surface,
                &self.project_preferences_window,
            )
        } else {
            (
                &mut ui.global_preferences,
                &mut self.global_preferences_surface,
                &self.global_preferences_window,
            )
        };
        if scroll_dialog(dialog, delta) {
            if let Some(surface) = surface {
                surface.invalidate();
            }
            if let Some(window) = window {
                window.request_redraw();
            }
        }
    }

    pub(super) fn sync_global_preferences_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        let open = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.workspace().ui.global_preferences.open);
        if !open {
            let had_window = self.global_preferences_window.is_some();
            if let Some(window) = self.global_preferences_window.take() {
                window.set_visible(false);
            }
            self.global_preferences_surface = None;
            if had_window && let Some(window) = self.window {
                window.focus_window();
            }
            return Ok(());
        }

        if self.global_preferences_window.is_none() {
            let window = std::sync::Arc::new(
                event_loop.create_window(
                    WindowAttributes::default()
                        .with_title("Global Preferences — Datum")
                        .with_inner_size(DEFAULT_PREFERENCES_SIZE)
                        .with_min_inner_size(MIN_PREFERENCES_SIZE)
                        .with_resizable(true)
                        .with_visible(false),
                )?,
            );
            owned_window_policy::establish_native_owner(
                self.window
                    .context("main window must exist before Preferences")?,
                &window,
            )
            .context("establish native Preferences owner")?;
            window.set_ime_allowed(false);
            let surface = GlobalPreferencesWindowSurface::new(
                self.runtime
                    .as_ref()
                    .context("runtime must exist before Preferences window")?,
                window.clone(),
                self.args.visual_scale_factor,
            )?;
            self.global_preferences_surface = Some(surface);
            self.global_preferences_window = Some(window.clone());
            owned_window_policy::show_owned_window(&window);
        }

        let raise = self
            .runtime
            .as_mut()
            .is_some_and(Runtime::take_global_preferences_raise_request);
        if raise && let Some(window) = &self.global_preferences_window {
            owned_window_policy::raise_owned_window(window);
        }
        Ok(())
    }

    pub(super) fn global_preferences_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.close_global_preferences();
                }
                if let Err(error) = self.sync_global_preferences_window(event_loop) {
                    fatal_gui_error(event_loop, "close Global Preferences window", error);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::Resized(size) => {
                if let (Some(runtime), Some(surface)) =
                    (&self.runtime, &mut self.global_preferences_surface)
                {
                    surface.resize(runtime, size.width, size.height);
                }
                if let Some(window) = &self.global_preferences_window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let scale_factor = self
                    .args
                    .visual_scale_factor
                    .map(f64::from)
                    .unwrap_or(scale_factor);
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_scale_factor(scale_factor);
                }
                if let Some(window) = &self.global_preferences_window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_cursor_position(Some((position.x as f32, position.y as f32)));
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_cursor_position(None);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                let target = self
                    .global_preferences_surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target);
                if let (Some(runtime), Some(target)) = (&mut self.runtime, target.as_ref()) {
                    let _ = runtime.activate_application_overlay_hit_target(target);
                }
                self.request_redraw_if_needed();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll_preferences_window(delta, false);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.modifiers = modifiers.state();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                keyboard_focus::handle_keyboard_input(self, &event);
            }
            WindowEvent::RedrawRequested => {
                if let (Some(runtime), Some(surface)) =
                    (&self.runtime, &mut self.global_preferences_surface)
                    && let Err(error) = surface.render(runtime, false, false)
                {
                    fatal_gui_error(event_loop, "render Global Preferences window", error);
                }
            }
            _ => {}
        }
    }
}

fn scroll_dialog(
    dialog: &mut datum_gui_protocol::GlobalPreferencesDialogState,
    delta: MouseScrollDelta,
) -> bool {
    let rows = match delta {
        MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
        MouseScrollDelta::PixelDelta(position) => (position.y / 40.0).round() as i32,
    };
    rows != 0 && dialog.scroll_rows(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_events_without_scroll_movement_do_not_request_a_frame() {
        let mut dialog = datum_gui_protocol::GlobalPreferencesDialogState::default();
        let before = dialog.clone();
        for _ in 0..100 {
            for delta in [
                MouseScrollDelta::LineDelta(0.0, 1.0),
                MouseScrollDelta::LineDelta(0.0, -1.0),
                MouseScrollDelta::LineDelta(1.0, 0.0),
                MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, 1.0)),
            ] {
                assert!(!scroll_dialog(&mut dialog, delta));
            }
        }
        assert_eq!(dialog, before);
    }

    #[test]
    fn native_window_defaults_fit_the_approved_preferences_target() {
        assert_eq!(DEFAULT_PREFERENCES_SIZE, LogicalSize::new(960.0, 720.0));
        assert_eq!(MIN_PREFERENCES_SIZE, LogicalSize::new(700.0, 540.0));
    }
}
