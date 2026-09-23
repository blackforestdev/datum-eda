//! Native-window host for Global Preferences.
//!
//! The preference service and interaction state remain in `Runtime`; this type
//! owns only the second native surface, its renderer, and window-local pointer
//! projection. The Design workspace never becomes a backdrop for Preferences.

use super::*;
#[path = "global_preferences_damage.rs"]
mod damage;
#[path = "owned_dialog_pointer.rs"]
mod pointer;
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
    // Dialog hosts never contain world geometry; retain this immutable envelope.
    empty_scene: RetainedScene,
    prepared: Option<PreparedScene>,
    // Input keeps targeting the last presented geometry while damage coalesces.
    presented_hits: gui_runtime_support::presented_hit_regions::PresentedHitRegions,
    cursor_position: Option<(f32, f32)>,
    scroll: datum_gui_viewport::scroll::ScrollViewport,
    new_project_focus: Option<(
        datum_gui_protocol::NewProjectFocus,
        datum_gui_protocol::NewProjectUnitsChoice,
    )>,
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
        Self::new_with_device(
            runtime.native_device_view(),
            window,
            scale_factor_override,
            None,
        )
    }

    fn new_with_device(
        gpu: native_gpu::DeviceView<'_>,
        window: std::sync::Arc<Window>,
        scale_factor_override: Option<f32>,
        previous: Option<&Renderer>,
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
        let samples = select_msaa_samples(gpu.adapter, format);
        let mut renderer = match previous {
            Some(renderer) => {
                renderer.recreate_for_device(gpu.device, gpu.queue, format, samples)?
            }
            None => Renderer::new(gpu.device, gpu.queue, format, samples)?,
        };
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
            empty_scene: RetainedScene::empty(),
            prepared: None,
            presented_hits,
            cursor_position: None,
            scroll: Default::default(),
            new_project_focus: None,
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
        let mut replacement = Self::new_with_device(
            gpu,
            self.window.clone(),
            Some(self.scale_factor),
            Some(&self.renderer),
        )?;
        replacement.cursor_position = self.cursor_position;
        replacement.scroll = self.scroll.clone();
        replacement.scroll.release();
        replacement.new_project_focus = self.new_project_focus;
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
        self.prepared = None;
        self.presented_hits.mark_pending();
    }

    pub(super) fn resize(&mut self, _runtime: &Runtime, width: u32, height: u32) {
        self.surface_transaction.resize(width, height);
        if self.config.width != width || self.config.height != height {
            // Keep scrollbar_pressed until release to consume the old gesture,
            // but stop applying its grab before the new layout is presented.
            self.scroll.release();
        }
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
        // The same focused control may be outside the reflowed viewport.
        self.scroll_focus = None;
        self.presented_hits.clear();
        self.invalidate();
    }

    pub(super) fn set_scale_factor(&mut self, scale_factor: f64) {
        let next = (scale_factor as f32).max(0.01);
        if (self.scale_factor - next).abs() > f32::EPSILON {
            self.scale_factor = next;
            self.scroll.release();
            self.scroll_focus = None;
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
            && self.scroll.drag(y / self.scale_factor)
        {
            self.invalidate();
            frames.invalidate(&self.window);
        }
    }

    pub(super) fn scroll_wheel(
        &mut self,
        delta: MouseScrollDelta,
        frames: &mut native_frame_coordinator::NativeFrameCoordinator,
    ) {
        let before = self.scroll.offset();
        let changed = scroll_wheel(
            &mut self.scroll,
            self.cursor_position,
            self.scale_factor,
            delta,
        );
        append_gui_verbose_diagnostic_line(|| {
            format!(
                "native scroll window={:?} delta={delta:?} scale={} before={before:?} after={:?} maximum={:?} changed={changed}",
                self.window.id(),
                self.scale_factor,
                self.scroll.offset(),
                self.scroll.maximum()
            )
        });
        if changed {
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
        let outcome = self
            .scroll
            .press(x / self.scale_factor, y / self.scale_factor);
        self.scrollbar_pressed = outcome.consumed;
        if outcome.changed {
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
                let dialog = &runtime.workspace().ui.new_project;
                let focus = (dialog.focus, dialog.units_choice);
                let reveal_focus = self.new_project_focus != Some(focus);
                self.new_project_focus = Some(focus);
                self.prepared = Some(self.renderer.prepare_native_new_project_scrolled(
                    &runtime.workspace().ui.new_project,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                    &mut self.scroll,
                    reveal_focus,
                ));
            } else {
                let dialog = if project_preferences {
                    &runtime.workspace().ui.project_preferences
                } else {
                    &runtime.workspace().ui.global_preferences
                };
                let identity_changed =
                    self.scroll_identity
                        .as_ref()
                        .is_none_or(|(section, query, row)| {
                            section != &dialog.section_id
                                || query != &dialog.search_query
                                || *row != dialog.scroll_row
                        });
                let expanded_changed = self.scroll_expanded.0 != dialog.open_choice_key
                    || self.scroll_expanded.1 != dialog.explanation_key;
                let focus_changed = self.scroll_focus.as_ref() != Some(&dialog.focus);
                let mut reveal_row = identity_changed.then_some(dialog.scroll_row);
                if focus_changed || expanded_changed {
                    use datum_gui_protocol::GlobalPreferencesFocus;
                    let key = match &dialog.focus {
                        GlobalPreferencesFocus::SettingName(key)
                        | GlobalPreferencesFocus::Control(key)
                        | GlobalPreferencesFocus::Reset(key) => Some(key.as_str()),
                        GlobalPreferencesFocus::ExplanationClose => {
                            dialog.explanation_key.as_deref()
                        }
                        _ => None,
                    };
                    if let Some(key) = key {
                        reveal_row = dialog.visible_rows().position(|row| row.key == key);
                    }
                }
                if identity_changed || expanded_changed {
                    self.scroll.release();
                }
                // Scrolling rebuilds geometry, but unchanged reveal keys stay owned here.
                if focus_changed {
                    self.scroll_focus = Some(dialog.focus.clone());
                }
                if expanded_changed {
                    self.scroll_expanded.0.clone_from(&dialog.open_choice_key);
                    self.scroll_expanded.1.clone_from(&dialog.explanation_key);
                }
                if identity_changed {
                    self.scroll_identity = Some((
                        dialog.section_id.clone(),
                        dialog.search_query.clone(),
                        dialog.scroll_row,
                    ));
                }
                self.prepared = Some(self.renderer.prepare_native_preferences_scrolled(
                    dialog,
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                    &mut self.scroll,
                    reveal_row,
                ));
            }
            append_gui_verbose_diagnostic_line(|| {
                format!(
                    "native control_meshes window={:?} builds={}",
                    self.window.id(),
                    self.renderer.control_mesh_build_count()
                )
            });
        }
        let view = frame.view();
        let rendered = self.renderer.render_with_submission(
            &runtime.device,
            &runtime.queue,
            &view,
            self.prepared
                .as_ref()
                .context("Global Preferences prepared scene must exist")?,
            &self.empty_scene,
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
        if !rendered? {
            self.surface_transaction.defer_upload_continuation();
            return Ok(false);
        }
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
        if self.handle_owned_dialog_pointer(native_frame_adapters::OwnedHost::Global, &event) {
            return;
        }
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
    let (x, y) = (x / scale, y / scale);
    let viewport = scroll.viewport;
    if x < viewport.x
        || x > viewport.x + viewport.width
        || y < viewport.y
        || y > viewport.y + viewport.height
    {
        return false;
    }
    let Some(wheel) =
        crate::app_native_events::scroll_input::VerticalWheel::from_native(delta, scale)
    else {
        return false;
    };
    scroll.wheel(wheel.pixels(40.0))
}

#[cfg(test)]
#[path = "native_dialog_hit_tests.rs"]
mod native_dialog_hit_tests;

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
        for scale in [1.0, 1.5, 2.0] {
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
                assert!(scroll_wheel(
                    &mut scroll,
                    Some((400.0 * scale, 200.0 * scale)),
                    scale,
                    tiny
                ));
            }
            assert!((scroll.offset() - 25.0 / scale).abs() < 0.001);
            assert!(!scroll_wheel(
                &mut scroll,
                Some((50.0 * scale, 200.0 * scale)),
                scale,
                tiny
            ));
            assert!(!scroll_wheel(
                &mut scroll,
                Some((400.0 * scale, 50.0 * scale)),
                scale,
                tiny
            ));
            assert!(!scroll_wheel(&mut scroll, None, scale, tiny));
            assert!(scroll_wheel(
                &mut scroll,
                Some((400.0 * scale, 200.0 * scale)),
                scale,
                MouseScrollDelta::LineDelta(0.0, -0.5)
            ));
            assert!((scroll.offset() - (25.0 / scale + 20.0)).abs() < 0.001);
            scroll.set_offset(scroll.maximum());
            assert!(!scroll_wheel(
                &mut scroll,
                Some((400.0 * scale, 200.0 * scale)),
                scale,
                tiny
            ));
            assert!(scroll_wheel(
                &mut scroll,
                Some((400.0 * scale, 200.0 * scale)),
                scale,
                MouseScrollDelta::LineDelta(0.0, 0.5)
            ));
        }
    }

    #[test]
    fn native_window_defaults_fit_the_approved_preferences_target() {
        assert_eq!(DEFAULT_PREFERENCES_SIZE, LogicalSize::new(960.0, 720.0));
        assert_eq!(MIN_PREFERENCES_SIZE, LogicalSize::new(700.0, 540.0));
    }
}
