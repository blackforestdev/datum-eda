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
    pub(super) surface_transaction:
        gui_runtime_support::native_surface_transaction::SurfaceTransaction,
    scale_factor: f32,
    renderer: Renderer,
    retained: Option<RetainedScene>,
    prepared: Option<PreparedScene>,
    cursor_position: Option<(f32, f32)>,
    scroll: datum_gui_viewport::scroll::ScrollViewport,
    scroll_identity: Option<(String, String, usize)>,
    scrollbar_pressed: bool,
    scroll_focus: Option<datum_gui_protocol::GlobalPreferencesFocus>,
    scroll_expanded: (Option<String>, Option<String>),
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
        gui_runtime_support::log_surface_identity(&window, &runtime.adapter);
        let caps = surface.get_capabilities(&runtime.adapter);
        let config = gui_runtime_support::surface_configuration(
            &caps,
            window.inner_size(),
            Some(runtime.config.format),
        );
        let format = config.format;
        surface.configure(&runtime.device, &config);
        let renderer = Renderer::new(
            &runtime.device,
            &runtime.queue,
            format,
            select_msaa_samples(&runtime.adapter, format),
        );
        Ok(Self {
            surface,
            surface_transaction:
                gui_runtime_support::native_surface_transaction::SurfaceTransaction::new(
                    &config,
                    window.inner_size(),
                ),
            config,
            scale_factor: scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32),
            renderer,
            retained: None,
            prepared: None,
            cursor_position: None,
            scroll: Default::default(),
            scroll_identity: None,
            scrollbar_pressed: false,
            scroll_focus: None,
            scroll_expanded: (None, None),
            window,
        })
    }

    pub(super) fn invalidate(&mut self) {
        self.retained = None;
        self.prepared = None;
    }

    pub(super) fn resize(&mut self, runtime: &Runtime, width: u32, height: u32) {
        self.surface_transaction.resize(width, height);
        let width = width.max(1);
        let height = height.max(1);
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface_transaction
            .configure_resize(&self.surface, &runtime.device, &self.config);
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
        if let Some((_, y)) = position
            && self.scroll.drag(y)
        {
            self.invalidate();
            self.window.request_redraw();
        }
    }

    pub(super) fn scrollbar_input(&mut self, state: ElementState) -> bool {
        if state == ElementState::Released {
            self.scroll.release();
            return std::mem::take(&mut self.scrollbar_pressed);
        }
        let Some((x, y)) = self.cursor_position else {
            return false;
        };
        self.scrollbar_pressed = self.scroll.press(x, y);
        if self.scrollbar_pressed {
            self.invalidate();
            self.window.request_redraw();
        }
        self.scrollbar_pressed
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
        if !self
            .surface_transaction
            .begin_frame(&self.surface, &runtime.device, &self.config)?
        {
            return Ok(());
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                if !self.surface_transaction.acquisition_failed(true) {
                    self.surface.configure(&runtime.device, &self.config);
                }
                self.invalidate();
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => {
                self.surface_transaction.acquisition_failed(false);
                return Ok(());
            }
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
                let identity = (
                    dialog.section_id.clone(),
                    dialog.search_query.clone(),
                    dialog.scroll_row,
                );
                let mut reveal_row =
                    (self.scroll_identity.as_ref() != Some(&identity)).then_some(dialog.scroll_row);
                let expanded = (
                    dialog.open_choice_key.clone(),
                    dialog.explanation_key.clone(),
                );
                if self.scroll_focus.as_ref() != Some(&dialog.focus)
                    || self.scroll_expanded != expanded
                {
                    use datum_gui_protocol::GlobalPreferencesFocus;
                    let key = match &dialog.focus {
                        GlobalPreferencesFocus::SettingName(key)
                        | GlobalPreferencesFocus::Control(key)
                        | GlobalPreferencesFocus::Reset(key) => Some(key.as_str()),
                        _ => None,
                    };
                    if let Some(key) = key {
                        reveal_row = dialog.visible_rows().position(|row| row.key == key);
                    }
                }
                self.scroll_focus = Some(dialog.focus.clone());
                self.scroll_expanded = expanded;
                self.scroll_identity = Some(identity);
                self.prepared = Some(PreparedScene::from_native_preferences_scrolled(
                    dialog,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                    &mut self.scroll,
                    reveal_row,
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
        self.surface_transaction.presented(&runtime.queue);
        Ok(())
    }
}

impl App {
    /// Scrolling changes only the owned dialog's transient viewport. Coalesced
    /// redraws must not rebuild the main Design window or other owned windows.
    pub(super) fn scroll_preferences_window(&mut self, delta: MouseScrollDelta, project: bool) {
        let surface = if project {
            &mut self.project_preferences_surface
        } else {
            &mut self.global_preferences_surface
        };
        let Some(surface) = surface else {
            return;
        };
        if scroll_wheel(
            &mut surface.scroll,
            surface.cursor_position,
            surface.scale_factor,
            delta,
        ) {
            surface.invalidate();
            surface.window.request_redraw();
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
            WindowEvent::Focused(false) => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.scrollbar_input(ElementState::Released);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_cursor_position(None);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if self
                    .global_preferences_surface
                    .as_mut()
                    .is_some_and(|surface| surface.scrollbar_input(state))
                    || state != ElementState::Released
                {
                    return;
                }
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

/// Keep native physical-pixel input in the renderer's coordinate space. Line
/// input has a fixed logical step; fractional deltas are never quantized.
fn scroll_wheel(
    scroll: &mut datum_gui_viewport::scroll::ScrollViewport,
    cursor: Option<(f32, f32)>,
    scale: f32,
    delta: MouseScrollDelta,
) -> bool {
    let Some((x, y)) = cursor else {
        return false;
    };
    let viewport = scroll.viewport;
    if x < viewport.x
        || x > viewport.x + viewport.width
        || y < viewport.y
        || y > viewport.y + viewport.height
    {
        return false;
    }
    let pixels = match delta {
        MouseScrollDelta::LineDelta(_, y) => y * 40.0 * scale,
        MouseScrollDelta::PixelDelta(position) => position.y as f32,
    };
    scroll.wheel(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_wheel_preserves_fractional_pixels_and_pane_locality() {
        let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
        scroll.layout(
            datum_gui_viewport::ScreenRectPx {
                x: 210.0,
                y: 100.0,
                width: 750.0,
                height: 440.0,
            },
            800.0,
        );
        let tiny = MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, -0.25));
        for _ in 0..100 {
            assert!(scroll_wheel(&mut scroll, Some((400.0, 200.0)), 1.5, tiny));
        }
        assert_eq!(scroll.offset(), 25.0);
        assert!(!scroll_wheel(&mut scroll, Some((50.0, 200.0)), 1.5, tiny));
        assert!(!scroll_wheel(&mut scroll, Some((400.0, 50.0)), 1.5, tiny));
        assert!(!scroll_wheel(&mut scroll, None, 1.5, tiny));
        assert!(scroll_wheel(
            &mut scroll,
            Some((400.0, 200.0)),
            1.5,
            MouseScrollDelta::LineDelta(0.0, -0.5)
        ));
        assert_eq!(scroll.offset(), 55.0);
        scroll.set_offset(scroll.maximum());
        assert!(!scroll_wheel(&mut scroll, Some((400.0, 200.0)), 1.5, tiny));
        assert!(scroll_wheel(
            &mut scroll,
            Some((400.0, 200.0)),
            1.5,
            MouseScrollDelta::LineDelta(0.0, 0.5)
        ));
    }

    #[test]
    fn native_window_defaults_fit_the_approved_preferences_target() {
        assert_eq!(DEFAULT_PREFERENCES_SIZE, LogicalSize::new(960.0, 720.0));
        assert_eq!(MIN_PREFERENCES_SIZE, LogicalSize::new(700.0, 540.0));
    }
}
