//! Native-window host for Global Preferences.
//!
//! The preference service and interaction state remain in `Runtime`; this type
//! owns only the second native surface, its renderer, and window-local pointer
//! projection. The Design workspace never becomes a backdrop for Preferences.

use super::*;
#[path = "global_preferences_damage.rs"]
mod damage;
pub(crate) use damage::GlobalPreferenceRenderState;
pub(super) const DEFAULT_PREFERENCES_SIZE: LogicalSize<f64> = LogicalSize::new(960.0, 720.0);
pub(super) const MIN_PREFERENCES_SIZE: LogicalSize<f64> = LogicalSize::new(700.0, 540.0);

/// Text-field input differs from button activation: winit represents a space
/// as a named key on native backends, even though it is printable text.
pub(crate) fn dialog_text_input(key: &Key, modifiers: ModifiersState) -> Option<&str> {
    if modifiers.control_key() || modifiers.alt_key() {
        return None;
    }
    match key {
        Key::Named(NamedKey::Space) => Some(" "),
        Key::Character(value) if !value.chars().any(char::is_control) => Some(value.as_str()),
        _ => None,
    }
}

/// Input consumption and rendering damage are independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogInputOutcome {
    Unhandled,
    Consumed,
    Dialog,
    Dependents,
    Workspace,
}

impl DialogInputOutcome {
    /// Opening a choice or explanation changes only the dialog. Boolean and
    /// integer activation writes a preference and retains dependent invalidation.
    pub(crate) fn control_activation(
        dialog: &datum_gui_protocol::GlobalPreferencesDialogState,
        key: &str,
    ) -> Self {
        use datum_gui_protocol::GlobalPreferenceControlUi;
        match dialog
            .rows
            .iter()
            .find(|row| row.key == key)
            .map(|row| &row.control)
        {
            Some(
                GlobalPreferenceControlUi::SingleChoice { .. }
                | GlobalPreferenceControlUi::Identity { .. }
                | GlobalPreferenceControlUi::Structured { .. },
            ) => Self::Dialog,
            Some(
                GlobalPreferenceControlUi::Boolean { .. }
                | GlobalPreferenceControlUi::Integer { .. },
            ) => Self::Dependents,
            None => Self::Consumed,
        }
    }

    pub(crate) fn from_handled(handled: bool, changed: Self) -> Self {
        if handled { changed } else { Self::Unhandled }
    }
}

pub(super) struct GlobalPreferencesWindowSurface {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    // Drop renderer references before recording host attachment retirement.
    pub(super) renderer: Renderer,
    pub(super) surface_transaction:
        gui_runtime_support::native_surface_transaction::SurfaceTransaction,
    scale_factor: f32,
    pub(super) measurements: native_gpu_measurements::Host,
    retained: Option<RetainedScene>,
    prepared: Option<PreparedScene>,
    // Input keeps targeting the last presented geometry while damage coalesces.
    presented_hits: gui_runtime_support::presented_hit_regions::PresentedHitRegions,
    cursor_position: Option<(f32, f32)>,
    scroll: datum_gui_viewport::scroll::ScrollViewport,
    scroll_identity: Option<(String, String, usize)>,
    scrollbar_pressed: bool,
    scroll_focus: Option<datum_gui_protocol::GlobalPreferencesFocus>,
    scroll_expanded: (Option<String>, Option<String>),
    // Retain the native display through renderer/measurement GPU teardown.
    // A wgpu surface alone can drop before the final device/instance reference.
    window: std::sync::Arc<Window>,
}

impl GlobalPreferencesWindowSurface {
    pub(super) fn new(
        runtime: &Runtime,
        window: std::sync::Arc<Window>,
        scale_factor_override: Option<f32>,
    ) -> Result<Self> {
        Self::new_with_device(runtime.native_device_view(), window, scale_factor_override)
    }

    fn new_with_device(
        gpu: native_gpu::DeviceView<'_>,
        window: std::sync::Arc<Window>,
        scale_factor_override: Option<f32>,
    ) -> Result<Self> {
        let surface = gpu
            .instance
            .create_surface(window.clone())
            .context("create Global Preferences native surface")?;
        gui_runtime_support::log_surface_identity(&window, gpu.adapter);
        let caps = surface.get_capabilities(gpu.adapter);
        let config = gui_runtime_support::surface_configuration(
            &caps,
            window.inner_size(),
            Some(gpu.config.format),
        );
        let format = config.format;
        let mut renderer = Renderer::new(
            gpu.device,
            gpu.queue,
            format,
            select_msaa_samples(gpu.adapter, format),
        );
        let measurements = native_gpu_measurements::Host::new(
            &mut renderer,
            gpu.device,
            gpu.queue,
            &window,
            Some(gpu.epoch),
        )?;
        let mut surface_transaction = SurfaceTransaction::new(&window, gpu.health);
        surface_transaction.share_queue_with(gpu.transaction);
        let mut presented_hits =
            gui_runtime_support::presented_hit_regions::PresentedHitRegions::default();
        presented_hits.mark_pending();
        Ok(Self {
            surface,
            surface_transaction,
            config,
            scale_factor: scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32),
            renderer,
            measurements,
            retained: None,
            prepared: None,
            presented_hits,
            cursor_position: None,
            scroll: Default::default(),
            scroll_identity: None,
            scrollbar_pressed: false,
            scroll_focus: None,
            scroll_expanded: (None, None),
            window,
        })
    }

    pub(super) fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub(super) fn replacement(&self, gpu: native_gpu::DeviceView<'_>) -> Result<Self> {
        let mut replacement =
            Self::new_with_device(gpu, self.window.clone(), Some(self.scale_factor))?;
        replacement.cursor_position = self.cursor_position;
        replacement.scroll = self.scroll.clone();
        replacement.scroll.release();
        replacement.scroll_identity = self.scroll_identity.clone();
        replacement.scroll_focus = self.scroll_focus.clone();
        replacement.scroll_expanded = self.scroll_expanded.clone();
        Ok(replacement)
    }

    pub(super) fn cancel_capture(&mut self) {
        self.scroll.release();
        self.scrollbar_pressed = false;
        self.window.set_cursor(winit::window::CursorIcon::Default);
    }

    pub(super) fn invalidate(&mut self) {
        self.retained = None;
        self.prepared = None;
        self.presented_hits.mark_pending();
    }

    pub(super) fn resize(&mut self, _runtime: &Runtime, width: u32, height: u32) {
        self.surface_transaction.resize(width, height);
        if width == 0 || height == 0 {
            return;
        }
        // Backend reconfiguration is owned by SurfaceTransaction. An unchanged
        // logical extent must not discard prepared input/layout while it waits.
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.presented_hits.clear();
        self.invalidate();
    }

    pub(super) fn set_scale_factor(&mut self, scale_factor: f64) {
        let next = (scale_factor as f32).max(0.01);
        if (self.scale_factor - next).abs() > f32::EPSILON {
            self.scale_factor = next;
            self.presented_hits.clear();
            self.invalidate();
        }
    }

    pub(super) fn set_cursor_position(
        &mut self,
        position: Option<(f32, f32)>,
        frames: &mut native_frame_coordinator::NativeFrameCoordinator,
    ) {
        self.cursor_position = position;
        if let Some((_, y)) = position
            && self.scroll.drag(y)
        {
            self.invalidate();
            frames.invalidate(&self.window);
        }
    }

    pub(super) fn scrollbar_input(
        &mut self,
        state: ElementState,
        frames: &mut native_frame_coordinator::NativeFrameCoordinator,
    ) -> bool {
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
            frames.invalidate(&self.window);
        }
        self.scrollbar_pressed
    }

    pub(super) fn hit_target(&self) -> Option<HitTarget> {
        let (x, y) = self.cursor_position?;
        self.presented_hits.hit_test(x, y).cloned()
    }

    pub(super) fn render(
        &mut self,
        runtime: &Runtime,
        project_preferences: bool,
        new_project: bool,
    ) -> Result<bool> {
        if runtime.device_health.failed() {
            return Ok(false);
        }
        let Some(mut frame) = self.surface_transaction.acquire(
            &self.surface,
            &runtime.device,
            &self.config,
            &runtime.device_health,
        )?
        else {
            return Ok(false);
        };
        if self.renderer.prepare_surface_attachment(
            &runtime.device,
            self.config.width,
            self.config.height,
            || !runtime.device_health.failed(),
        )? {
            self.surface_transaction
                .observe_attachment(&self.renderer, &frame);
        }
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
        let view = frame.view();
        let rendered = self.renderer.render_with_submission(
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
            &mut |submission| {
                self.surface_transaction
                    .submitted(&mut frame, &runtime.queue, submission)
            },
        );
        self.surface_transaction
            .observe_attachment(&self.renderer, &frame);
        rendered?;
        if runtime.device_health.failed() {
            return Ok(false);
        }
        self.surface_transaction.present(frame, &self.window)?;
        self.presented_hits
            .present(self.prepared.as_mut().expect("prepared frame presented"));
        Ok(true)
    }
}

impl App {
    pub(super) fn request_dialog_key_redraw(
        &mut self,
        outcome: DialogInputOutcome,
        host: native_frame_adapters::OwnedHost,
    ) -> bool {
        match outcome {
            DialogInputOutcome::Unhandled => return false,
            DialogInputOutcome::Consumed => {}
            DialogInputOutcome::Dialog => self.request_dialog_redraw(host),
            DialogInputOutcome::Dependents => self.request_redraw_if_needed(),
            DialogInputOutcome::Workspace => {
                self.request_workspace_redraw();
                self.request_dialog_redraw(host);
            }
        }
        true
    }

    fn request_dialog_redraw(&mut self, host: native_frame_adapters::OwnedHost) {
        use native_frame_adapters::OwnedHost;
        let surface = match host {
            OwnedHost::Global => &mut self.global_preferences_surface,
            OwnedHost::Project => &mut self.project_preferences_surface,
            OwnedHost::New => &mut self.new_project_surface,
        };
        if let Some(surface) = surface {
            surface.invalidate();
            self.frames.invalidate(&surface.window);
        }
    }
    /// Apply authoritative preference input before selecting damaged hosts.
    pub(super) fn activate_preferences_target(
        &mut self,
        target: Option<&HitTarget>,
        project: bool,
    ) {
        let (Some(target), Some(runtime)) = (target, &mut self.runtime) else {
            return;
        };
        let before = GlobalPreferenceRenderState::capture(&runtime.workspace().ui);
        if project {
            let _ = runtime.activate_project_preferences_hit_target(target);
        } else {
            let _ = runtime.activate_application_overlay_hit_target(target);
        }
        let ui = &runtime.workspace().ui;
        let dialog = if project {
            &ui.project_preferences
        } else {
            &ui.global_preferences
        };
        let outcome = match target {
            HitTarget::GlobalPreferencesModal => DialogInputOutcome::Consumed,
            HitTarget::GlobalPreferencesSection(_)
            | HitTarget::GlobalPreferencesSearch
            | HitTarget::GlobalPreferencesSettingName(_)
            | HitTarget::GlobalPreferencesExplanationClose => DialogInputOutcome::Dialog,
            HitTarget::GlobalPreferencesControl(key) => {
                DialogInputOutcome::control_activation(dialog, key)
            }
            _ => DialogInputOutcome::Dependents,
        };
        let outcome = if project {
            outcome
        } else {
            before.finish(outcome, ui)
        };
        self.request_dialog_key_redraw(
            outcome,
            if project {
                native_frame_adapters::OwnedHost::Project
            } else {
                native_frame_adapters::OwnedHost::Global
            },
        );
    }
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
            self.frames.invalidate(&surface.window);
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
            if let Some(surface) = &self.global_preferences_surface {
                self.frames.close(surface.window_id());
            }
            self.global_preferences_surface = None;
            if had_window && let Some(window) = self.window.as_deref() {
                window.focus_window();
            }
            return Ok(());
        }

        if self.global_preferences_window.is_none() && !self.native_device_available() {
            // Keep the requested logical window state until replacement finishes.
            // The close path above remains available on a failed device.
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
                    .as_deref()
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
            self.frames.register(
                window.id(),
                surface.measurements.epoch(),
                window.inner_size(),
                surface.surface_transaction.recovery.clone(),
            );
            self.global_preferences_surface = Some(surface);
            self.global_preferences_window = Some(window.clone());
            owned_window_policy::show_owned_window(&window, &mut self.frames);
        }

        let raise = self
            .runtime
            .as_mut()
            .is_some_and(Runtime::take_global_preferences_raise_request);
        if raise && let Some(window) = &self.global_preferences_window {
            owned_window_policy::raise_owned_window(window, &mut self.frames);
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
                // The shared extent transition owns the redraw request.
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
                    self.frames.invalidate(window);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_cursor_position(
                        Some((position.x as f32, position.y as f32)),
                        &mut self.frames,
                    );
                }
            }
            WindowEvent::Focused(false) => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.scrollbar_input(ElementState::Released, &mut self.frames);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(surface) = &mut self.global_preferences_surface {
                    surface.set_cursor_position(None, &mut self.frames);
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
                    .is_some_and(|surface| surface.scrollbar_input(state, &mut self.frames))
                    || state != ElementState::Released
                {
                    return;
                }
                let target = self
                    .global_preferences_surface
                    .as_ref()
                    .and_then(GlobalPreferencesWindowSurface::hit_target);
                self.activate_preferences_target(target.as_ref(), false);
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
    fn native_named_space_is_text_only_without_command_modifiers() {
        let none = ModifiersState::empty();
        assert_eq!(
            dialog_text_input(&Key::Named(NamedKey::Space), none),
            Some(" ")
        );
        assert_eq!(
            dialog_text_input(&Key::Character("é".into()), none),
            Some("é")
        );
        assert_eq!(dialog_text_input(&Key::Named(NamedKey::Enter), none), None);
        assert_eq!(dialog_text_input(&Key::Character("\t".into()), none), None);
        for modifier in [ModifiersState::CONTROL, ModifiersState::ALT] {
            assert_eq!(
                dialog_text_input(&Key::Named(NamedKey::Space), modifier),
                None
            );
            assert_eq!(
                dialog_text_input(&Key::Character("x".into()), modifier),
                None
            );
        }
    }

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
