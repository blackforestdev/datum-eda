//! Native input/event dispatch; input state is applied before redraw effects.
use super::*;

impl App {
    pub(super) fn resume_native(&mut self, event_loop: &ActiveEventLoop) {
        self.frames.set_suspended(false);
        self.measurement_suspend(false);
        append_gui_diagnostic_line("resumed event");
        if self.window.is_some() {
            append_gui_diagnostic_line("resumed ignored; window already exists");
            return;
        }
        // Sleep until input or an explicitly requested redraw needs service.
        event_loop.set_control_flow(ControlFlow::Wait);
        self.args
            .validate_visual_args()
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "visual launch args invalid", err));
        append_gui_diagnostic_line("launch state load begin");
        let launch_state = self
            .args
            .load_launch_state(Some(self.terminal_event_proxy.clone()))
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "launch state load failed", err));
        append_gui_diagnostic_line("launch state load end");
        let (window_width, window_height) = self
            .args
            .visual_window_size()
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "window size invalid", err));
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Datum EDA")
                    .with_inner_size(LogicalSize::new(window_width as f64, window_height as f64))
                    .with_visible(false),
            )
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "window creation failed", err));
        append_gui_diagnostic_line("window created");
        // Hold chords require raw press/release events; focused rich-text fields
        // may opt into IME explicitly when that ownership model lands.
        window.set_ime_allowed(false);
        let window_ref: &'static Window = Box::leak(Box::new(window));
        append_gui_diagnostic_line("runtime creation begin");
        let runtime = pollster::block_on(Runtime::new(
            window_ref,
            launch_state,
            self.args.visual_scale_factor,
            self.terminal_event_proxy.clone(),
        ))
        .unwrap_or_else(|err| fatal_gui_error(event_loop, "runtime creation failed", err));
        append_gui_diagnostic_line("runtime creation end");
        self.frames.register(
            window_ref.id(),
            runtime.measurements.epoch(),
            window_ref.inner_size(),
            runtime.surface_transaction.recovery.clone(),
        );
        self.runtime = Some(runtime);
        self.window = Some(window_ref);
        window_ref.set_visible(true);
        append_gui_diagnostic_line("window visible");
        self.request_redraw_if_needed();
        if let Err(err) = self.sync_owned_product_windows(event_loop) {
            fatal_gui_error(event_loop, "open owned product window", err);
        }
    }

    pub(super) fn handle_native_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.frames.consume_cancelled_release(window_id, &event) {
            return;
        }
        if matches!(event, WindowEvent::ScaleFactorChanged { .. }) {
            self.cancel_native_host_gestures(window_id);
        }
        if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_some()
            && let Some(label) = window_event_diagnostic_label(&event)
        {
            append_gui_verbose_diagnostic_line(|| format!("window event {window_id:?} {label}"));
        }
        self.frames.window_event(window_id, &event);
        if self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.device_health.failed())
        {
            self.frames.fail_device();
        }
        self.sync_surface_drawability();
        self.measurement_window_event(window_id, &event);
        if matches!(&event, WindowEvent::KeyboardInput { event, .. }
            if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::F5))
            && (self.retry_failed_device() || self.frames.manual_retry(window_id))
        {
            for window in [
                self.window,
                self.global_preferences_window.as_deref(),
                self.project_preferences_window.as_deref(),
                self.new_project_window.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter(|window| window.id() == window_id)
            {
                self.frames.invalidate(window);
            }
            return;
        }

        // A failed renderer cannot show the outcome of editing input. Keep
        // lifecycle/close and modifier-state delivery alive while Retry owns
        // recovery. Dropping a Ctrl/Alt/Shift release would leave stale input
        // state after a successful retry. These updates do not apply edits.
        if self.frames.rendering_failed(window_id)
            && !matches!(
                &event,
                WindowEvent::CloseRequested
                    | WindowEvent::Destroyed
                    | WindowEvent::Resized(_)
                    | WindowEvent::ScaleFactorChanged { .. }
                    | WindowEvent::Focused(_)
                    | WindowEvent::Occluded(_)
                    | WindowEvent::ModifiersChanged(_)
                    | WindowEvent::RedrawRequested
            )
        {
            return;
        }
        if matches!(event, WindowEvent::RedrawRequested) {
            self.frames.redraw_received(window_id);
            return;
        }
        let Some(event) = self.dispatch_owned_product_window_event(event_loop, window_id, event)
        else {
            return;
        };
        if let Some(window) = self.window
            && window.id() != window_id
        {
            return;
        }
        if self.runtime.as_ref().is_some_and(|runtime| {
            runtime.workspace().ui.global_preferences.open
                || runtime.workspace().ui.project_preferences.open
                || runtime.workspace().ui.new_project.open
        }) && !matches!(
            &event,
            WindowEvent::CloseRequested
                | WindowEvent::Resized(_)
                | WindowEvent::ScaleFactorChanged { .. }
                | WindowEvent::Focused(_)
                | WindowEvent::RedrawRequested
        ) {
            return;
        }
        if matches!(event, WindowEvent::CloseRequested) {
            self.request_controlled_close(event_loop);
            return;
        }
        match event {
            WindowEvent::Ime(ime)
                if self
                    .runtime
                    .as_ref()
                    .is_some_and(Runtime::terminal_owns_input) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.handle_terminal_ime(&ime)
                {
                    if let Some(window) = self.window {
                        let (x, y, width, height) = runtime.terminal_ime_cursor_rect();
                        window.set_ime_cursor_area(
                            winit::dpi::PhysicalPosition::new(x, y),
                            winit::dpi::PhysicalSize::new(width, height),
                        );
                    }
                    self.request_workspace_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                if self.args.resize_torture_smoke {
                    self.resize_smoke.note_native_event();
                }
                if let Some(runtime) = &mut self.runtime {
                    runtime.resize(size.width, size.height);
                    // The shared extent transition owns the redraw request.
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let scale_factor = self
                        .args
                        .visual_scale_factor
                        .map(f64::from)
                        .unwrap_or(scale_factor);
                    runtime.set_scale_factor(scale_factor);
                    self.request_workspace_redraw();
                }
            }
            WindowEvent::Focused(focused) => {
                if owned_window_policy::redirect_owner_activation(
                    focused,
                    &mut self.frames,
                    self.project_preferences_window
                        .as_ref()
                        .or(self.global_preferences_window.as_ref()),
                ) {
                    return;
                }
                if let Some(runtime) = &mut self.runtime {
                    runtime.window_focused = focused;
                    let terminal_split_finished =
                        !focused && runtime.finish_terminal_split_drag().is_some();
                    if !focused {
                        runtime.pan_gesture.cancel();
                        runtime.cancel_terminal_tab_drag();
                        runtime.cancel_terminal_text_selection_drag();
                    }
                    if !focused && (runtime.clear_interaction_overlay() || terminal_split_finished)
                    {
                        self.request_workspace_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.last_cursor_pos = None;
                    runtime.pan_gesture.cancel();
                    runtime.cancel_terminal_tab_drag();
                    runtime.cancel_terminal_text_selection_drag();
                    let terminal_split_finished = runtime.finish_terminal_split_drag().is_some();
                    let terminal_hover_cleared = runtime.clear_terminal_tab_hover();
                    if runtime.clear_interaction_overlay()
                        || terminal_hover_cleared
                        || terminal_split_finished
                    {
                        self.request_workspace_redraw();
                    }
                    self.apply_cursor(None);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let next_pos = (position.x as f32, position.y as f32);
                    let previous_pos = runtime.last_cursor_pos;
                    runtime.last_cursor_pos = Some(next_pos);
                    let terminal_hover_changed = runtime.update_terminal_tab_hover(next_pos);
                    if runtime.terminal_tab_drag.is_some() {
                        if runtime.advance_terminal_tab_drag(next_pos) || terminal_hover_changed {
                            self.request_workspace_redraw();
                        }
                        self.apply_cursor_icon(winit::window::CursorIcon::Grabbing);
                        return;
                    }
                    if runtime.terminal_split_drag.is_some() {
                        let changed = runtime.advance_terminal_split_drag(next_pos);
                        let icon = runtime
                            .terminal_split_cursor_icon(next_pos)
                            .unwrap_or(winit::window::CursorIcon::Default);
                        if changed {
                            self.request_workspace_redraw();
                        }
                        self.apply_cursor_icon(icon);
                        return;
                    }
                    if runtime.terminal_clipboard_menu_active() {
                        return;
                    }
                    if runtime.advance_terminal_text_selection(next_pos) {
                        self.apply_cursor_icon(winit::window::CursorIcon::Text);
                        self.request_workspace_redraw();
                        return;
                    }
                    if runtime.report_terminal_mouse_motion() {
                        runtime.clear_interaction_overlay();
                        self.request_workspace_redraw();
                        return;
                    }
                    let mut changed = runtime.update_menu_hover(next_pos) || terminal_hover_changed;
                    if runtime.dock_drag_active {
                        changed = runtime.handle_dock_resize_drag(next_pos);
                    } else if runtime.divider_drag.is_some() {
                        changed = runtime.handle_divider_drag(next_pos);
                    } else if runtime.marking_menu_active() {
                        changed = runtime.update_marking_menu_preview(next_pos);
                    } else if runtime.pan_gesture.is_active() {
                        changed = previous_pos.is_some_and(|previous| {
                            runtime.advance_primary_pan(previous, next_pos)
                        });
                    }
                    if !runtime.dock_drag_active
                        && runtime.divider_drag.is_none()
                        && !runtime.pan_gesture.is_active()
                        && !runtime.marking_menu_active()
                    {
                        changed = runtime.handle_authoring_pointer_move(next_pos) || changed;
                        changed = runtime.update_hover(next_pos) || changed;
                    } else {
                        changed = runtime.clear_interaction_overlay() || changed;
                    }
                    let pointer_cursor = runtime.pointer_cursor_icon(next_pos);
                    if changed {
                        self.request_workspace_redraw();
                    }
                    self.apply_cursor_icon(pointer_cursor);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let scroll_lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 20.0,
                    };
                    if runtime.handle_layer_scroll(scroll_lines) {
                        self.request_workspace_redraw();
                        return;
                    }
                    if runtime.handle_console_history_scroll(scroll_lines) {
                        self.request_workspace_redraw();
                        return;
                    }
                    if runtime.report_terminal_mouse_wheel(scroll_lines) {
                        self.request_workspace_redraw();
                        return;
                    }
                    if runtime.cursor_in_dock() && scroll_lines.abs() > 0.01 {
                        if runtime.handle_dock_scroll(scroll_lines) {
                            self.request_workspace_redraw();
                        }
                    } else {
                        let zoom_delta = if scroll_lines > 0.0 {
                            Some(1.12_f32.powf(scroll_lines.abs().min(3.0)))
                        } else if scroll_lines < 0.0 {
                            Some(0.89_f32.powf(scroll_lines.abs().min(3.0)))
                        } else {
                            None
                        };
                        if let Some(zoom_delta) = zoom_delta
                            && runtime.handle_zoom(zoom_delta)
                        {
                            self.request_workspace_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: button @ (MouseButton::Middle | MouseButton::Right),
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if button == MouseButton::Right
                        && state == ElementState::Pressed
                        && runtime.cursor_in_dock()
                        && runtime.open_terminal_clipboard_menu_at_cursor()
                    {
                        self.request_workspace_redraw();
                        return;
                    }
                    if button == MouseButton::Right && runtime.terminal_clipboard_menu_active() {
                        return;
                    }
                    if runtime.report_terminal_mouse_button(button, state) {
                        return;
                    }
                    if button == MouseButton::Right && runtime.handle_context_menu_button(state) {
                        self.request_workspace_redraw();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.handle_primary_button_press();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if let Some(icon) = runtime.finish_dock_resize_drag() {
                        self.apply_cursor_icon(icon);
                        self.request_workspace_redraw();
                        return;
                    }
                    if let Some(icon) = runtime.finish_terminal_split_drag() {
                        self.apply_cursor_icon(icon);
                        self.request_workspace_redraw();
                        return;
                    }
                    if runtime.finish_terminal_tab_drag() {
                        let icon = runtime
                            .last_cursor_pos
                            .and_then(|pointer| runtime.terminal_tab_cursor_icon(pointer))
                            .unwrap_or(winit::window::CursorIcon::Default);
                        self.apply_cursor_icon(icon);
                        self.request_workspace_redraw();
                        return;
                    }
                    // A completed divider-drag resize ends here; the release must NOT
                    // fall through to click-to-focus / selection.
                    let was_divider_drag = runtime.divider_drag.take().is_some();
                    if runtime.finish_terminal_text_selection() {
                        self.request_workspace_redraw();
                        return;
                    }
                    if !runtime.terminal_clipboard_menu_active()
                        && runtime
                            .report_terminal_mouse_button(MouseButton::Left, ElementState::Released)
                    {
                        return;
                    }
                    if runtime.finish_primary_pan() {
                        self.request_workspace_redraw();
                        return;
                    }
                    if was_divider_drag {
                        self.request_workspace_redraw();
                        return;
                    }
                    let handled = runtime.handle_primary_click();
                    if handled {
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.modifiers = modifiers.state();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && matches!(event.logical_key, Key::Named(NamedKey::Escape))
                    && self
                        .runtime
                        .as_mut()
                        .is_some_and(Runtime::dismiss_terminal_clipboard_menu)
                {
                    self.request_workspace_redraw();
                    return;
                }
                if event.state == ElementState::Pressed
                    && matches!(event.logical_key, Key::Named(NamedKey::Escape))
                    && self
                        .runtime
                        .as_mut()
                        .is_some_and(Runtime::cancel_terminal_tab_drag)
                {
                    self.apply_cursor_icon(winit::window::CursorIcon::Default);
                    self.request_workspace_redraw();
                    return;
                }
                keyboard_focus::handle_keyboard_input(self, &event);
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.resume_native(event_loop);
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_native_window_event(event_loop, window_id, event);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.frames.set_suspended(true);
        self.measurement_suspend(true);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Reconcile close/open input before selecting surviving host tokens.
        if let Err(err) = self.sync_owned_product_windows(event_loop) {
            fatal_gui_error(event_loop, "synchronize owned product window", err);
        }
        // Native tokens, rather than this callback's frequency, drive rendering.
        self.dispatch_native_frame_round(event_loop);
        // Include retries created by this round in the single wait decision.
        self.poll_background_work(event_loop);
        self.poll_resize_smoke_start();
        self.request_restored_native_frames();
        event_loop.set_control_flow(self.frames.take_control_flow());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        if self
            .runtime
            .as_mut()
            .is_some_and(Runtime::handle_terminal_output_wake)
        {
            self.request_workspace_redraw();
        }
    }
}
