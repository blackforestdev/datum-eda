use anyhow::{Context, Result};
use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use clap::Parser;
use datum_gui_protocol::{
    BoardTextAlignmentField, BoardTextBooleanField, BoardTextCycleField, BoardTextHeightStep,
    BoardTextLineSpacingStep, BoardTextRotationStep, DockTab, LiveDesignSession, LiveReviewRequest,
    MarkingMenuState, PointNm, RectNm, SceneBounds, SessionCommand, SessionEvent,
    TerminalCommandHandoff, WorkspaceTool, ensure_known_good_demo_request,
    load_board_editor_workspace_state, load_kicad_schematic_workspace_state,
    load_live_workspace_state, materialize_kicad_board_request,
};
#[cfg(feature = "visual")]
use datum_gui_render::visual_capture::OffscreenRenderer;
use datum_gui_render::{
    CameraState, HitTarget, PreparedScene, Renderer, RetainedScene, ShellLayout,
};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(feature = "visual")]
use std::sync::mpsc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

mod app_bootstrap;
mod app_shell;
mod artifact_preview_controls;
mod board_text_terminal_commands;
mod gui_runtime_support;
mod pane_cameras;
mod production_status_refresh;
mod retained_scene_cache_key;
mod runtime_terminal_context;
mod terminal_active_context;
mod terminal_activity_snapshot;
mod terminal_check_context;
mod terminal_context;
mod terminal_context_contract;
mod terminal_context_io;
mod terminal_input;
mod terminal_journal_context;
mod terminal_proposal_context;
mod terminal_screen;
mod terminal_session;
mod terminal_session_context;
mod terminal_session_controls;
mod terminal_session_events;
use app_bootstrap::{GuiArgs, LaunchState};
use app_shell::App;
use board_text_terminal_commands::{
    BoardTextEditTerminalField, BoardTextQuickEditTerminalAction, board_text_edit_terminal_command,
    board_text_quick_edit_terminal_command,
};
pub(crate) use gui_runtime_support::*;
use pane_cameras::PaneCameras;
use retained_scene_cache_key::retained_selection_cache_key;
#[cfg(feature = "visual")]
use std::fs;
use terminal_input::{
    TerminalKeyAction, terminal_focus_event_sequence, terminal_key_action,
    terminal_sgr_mouse_button_sequence, terminal_sgr_mouse_motion_sequence,
    terminal_sgr_mouse_wheel_sequence, terminal_urxvt_mouse_button_sequence,
    terminal_urxvt_mouse_motion_sequence, terminal_urxvt_mouse_wheel_sequence,
    terminal_utf8_mouse_button_sequence, terminal_utf8_mouse_motion_sequence,
    terminal_utf8_mouse_wheel_sequence, terminal_x10_mouse_button_sequence,
    terminal_x10_mouse_motion_sequence, terminal_x10_mouse_wheel_sequence,
};
use terminal_screen::terminal_scrollback_copy_text;
use terminal_session::{
    TerminalLaunchContext, TerminalSessionRegistry, refresh_terminal_session_context_from_state,
    terminal_launch_context_from_state,
};
use terminal_session_events::{
    prepare_terminal_command_execution, record_manual_terminal_command_handoff,
};

#[cfg(feature = "visual")]
const COPY_BYTES_PER_PIXEL: u32 = 4;
#[cfg(feature = "visual")]
const WGPU_COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;
const ASSISTANT_ACTIVITY_COMMAND: &str =
    "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20";

const RETAINED_SCENE_CACHE_LIMIT: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetainedSceneCacheKey {
    scene_id: String,
    source_revision: String,
    width: u32,
    height: u32,
    scale_bits: u32,
    dock_height_px: u32,
    show_authored: bool,
    show_proposed: bool,
    show_unrouted: bool,
    dim_unrelated: bool,
    layer_visibility: BTreeMap<String, bool>,
    selection: String,
}

fn main() -> Result<()> {
    install_gui_panic_hook();
    reset_gui_diagnostic_log();
    let args = GuiArgs::parse();
    append_gui_diagnostic_line(format!("startup args={args:?}"));
    if args.visual_test && args.exit_after_screenshot && !args.window_visual_test {
        return run_offscreen_visual_test(&args);
    }
    let event_loop = EventLoop::new().context("failed to create event loop")?;
    let mut app = App::new(args);
    event_loop.run_app(&mut app).context("failed to run app")
}

#[cfg(feature = "visual")]
fn run_offscreen_visual_test(args: &GuiArgs) -> Result<()> {
    args.validate_visual_args()?;
    append_gui_diagnostic_line("offscreen visual test begin");
    let request = args
        .resolve_request()
        .context("resolve offscreen visual-test review context")?;
    let workspace_include_review = !args.wants_plain_project_board_view();
    let mut state = if let Some(schematic_file) = &args.schematic_file {
        load_kicad_schematic_workspace_state(schematic_file)
            .context("load schematic offscreen workspace state")?
    } else if args.wants_plain_project_board_view() {
        load_board_editor_workspace_state(&request)
            .context("load board editor offscreen workspace state")?
    } else {
        load_live_workspace_state(&request).context("load live offscreen workspace state")?
    };
    // Preset a component selection when requested, mirroring the on-screen launch
    // path in app_bootstrap. `--select` accepts a reference designator (e.g. R1)
    // resolved against the loaded scene, or a raw object_id; an unknown selector
    // leaves the inspector empty rather than crashing, so the parity capture fails
    // loudly on a bad selector.
    if let Some(sel) = &args.select {
        let object_id = state
            .scene
            .components
            .iter()
            .find(|c| c.reference == *sel)
            .map(|c| c.object_id.clone())
            .unwrap_or_else(|| sel.clone());
        state.select_authored_object(&object_id);
    }
    // Capture/test affordance (decision 021): seed the pane tree if
    // --initial-layout was set so this offscreen path renders that shape; a no-op
    // otherwise, so the default parity capture is untouched.
    args.apply_initial_layout(&mut state.ui.layout);
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let (width, height) = args.visual_window_size()?;
    let scale_factor = args.visual_scale_factor.unwrap_or(1.0);
    let screenshot_out = args
        .screenshot_out
        .as_ref()
        .context("--screenshot-out is required for --visual-test")?;
    let mut renderer =
        OffscreenRenderer::new(width, height).context("create offscreen renderer")?;
    renderer
        .warm_workspace_for_surface_scale(&state, Some(camera), scale_factor)
        .context("warm offscreen visual-test renderer")?;
    let image = renderer
        .render_workspace_for_surface_scale(&state, Some(camera), scale_factor)
        .context("render offscreen visual-test workspace")?;
    if let Some(parent) = screenshot_out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create screenshot directory {}", parent.display()))?;
    }
    image.save(screenshot_out).with_context(|| {
        format!(
            "write offscreen visual-test screenshot {}",
            screenshot_out.display()
        )
    })?;
    append_gui_diagnostic_line(format!(
        "offscreen visual test end path={} include_review={workspace_include_review}",
        screenshot_out.display()
    ));
    Ok(())
}

#[cfg(not(feature = "visual"))]
fn run_offscreen_visual_test(_args: &GuiArgs) -> Result<()> {
    anyhow::bail!("datum-gui --visual-test requires the datum-gui-app visual feature")
}

fn fatal_gui_error(event_loop: &ActiveEventLoop, context: &str, err: impl std::fmt::Display) -> ! {
    append_gui_diagnostic_line(format!("fatal {context}: {err}"));
    eprintln!("datum-gui error: {context}: {err}");
    event_loop.exit();
    std::process::exit(1);
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        append_gui_diagnostic_line("resumed event");
        if self.window.is_some() {
            append_gui_diagnostic_line("resumed ignored; window already exists");
            return;
        }
        // Block the event loop until there is work to do. Winit 0.30 defaults
        // to ControlFlow::Poll, which busy-loops the main thread and burns
        // CPU while the GUI is idle. M7 review is an event-driven surface;
        // Wait is correct. Redraws are explicitly requested via
        // `request_redraw_if_needed()` when state changes.
        event_loop.set_control_flow(ControlFlow::Wait);
        self.args
            .validate_visual_args()
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "visual launch args invalid", err));
        append_gui_diagnostic_line("launch state load begin");
        let launch_state = self
            .args
            .load_launch_state()
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
        window.set_ime_allowed(true);
        let window_ref: &'static Window = Box::leak(Box::new(window));
        append_gui_diagnostic_line("runtime creation begin");
        let runtime = pollster::block_on(Runtime::new(
            window_ref,
            launch_state,
            self.args.visual_scale_factor,
        ))
        .unwrap_or_else(|err| fatal_gui_error(event_loop, "runtime creation failed", err));
        append_gui_diagnostic_line("runtime creation end");
        self.runtime = Some(runtime);
        self.window = Some(window_ref);
        window_ref.set_visible(true);
        append_gui_diagnostic_line("window visible");
        self.request_redraw_if_needed();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self.window {
            if window.id() != window_id {
                return;
            }
        }
        if let Some(label) = window_event_diagnostic_label(&event) {
            append_gui_verbose_diagnostic_line(format!("window event {label}"));
        }
        if matches!(event, WindowEvent::CloseRequested) {
            append_gui_diagnostic_line("close requested");
            event_loop.exit();
            return;
        }
        if let Some(runtime) = &mut self.runtime
            && runtime.poll_terminal_output()
        {
            self.request_redraw_if_needed();
        }
        match event {
            WindowEvent::Ime(winit::event::Ime::Commit(text))
                if self
                    .runtime
                    .as_ref()
                    .is_some_and(|runtime| runtime.dock_accepts_text_input()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.append_dock_text(&text)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.resize(size.width, size.height);
                    self.request_redraw_if_needed();
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
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.report_terminal_focus_event(focused);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let next_pos = (position.x as f32, position.y as f32);
                    runtime.last_cursor_pos = Some(next_pos);
                    if runtime.report_terminal_mouse_motion() {
                        self.request_redraw_if_needed();
                        return;
                    }
                    let mut changed = false;
                    if runtime.dock_drag_active {
                        changed = runtime.handle_dock_resize_drag(next_pos);
                    } else if runtime.marking_menu_active() {
                        changed = runtime.update_marking_menu_preview(next_pos);
                    } else if runtime.middle_drag_active || runtime.right_drag_active {
                        changed = runtime.handle_pan_drag(next_pos);
                    }
                    // Update hover state
                    if !runtime.dock_drag_active
                        && !runtime.middle_drag_active
                        && !runtime.right_drag_active
                        && !runtime.marking_menu_active()
                    {
                        changed = runtime.handle_authoring_pointer_move(next_pos) || changed;
                        changed = runtime.update_hover(next_pos) || changed;
                    }
                    if changed {
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let scroll_lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 20.0,
                    };
                    if runtime.report_terminal_mouse_wheel(scroll_lines) {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if runtime.cursor_in_dock() && scroll_lines.abs() > 0.01 {
                        if runtime.handle_dock_scroll(scroll_lines) {
                            self.request_redraw_if_needed();
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
                            self.request_redraw_if_needed();
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Middle,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.report_terminal_mouse_button(MouseButton::Middle, state) {
                        return;
                    }
                    runtime.middle_drag_active = state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.report_terminal_mouse_button(MouseButton::Right, state) {
                        return;
                    }
                    match state {
                        ElementState::Pressed => {
                            if runtime.open_marking_menu_at_cursor() {
                                self.request_redraw_if_needed();
                            } else {
                                runtime.right_drag_active = true;
                            }
                        }
                        ElementState::Released => {
                            runtime.right_drag_active = false;
                            if runtime.dismiss_marking_menu() {
                                self.request_redraw_if_needed();
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if runtime
                        .report_terminal_mouse_button(MouseButton::Left, ElementState::Pressed)
                    {
                        return;
                    }
                    // Check if clicking dock resize handle
                    if let Some((x, y)) = runtime.last_cursor_pos {
                        let prepared = runtime.prepared_scene();
                        if let Some(HitTarget::DockResizeHandle) = prepared.hit_test(x, y) {
                            runtime.dock_drag_active = true;
                            self.request_redraw_if_needed();
                            return;
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.dock_drag_active = false;
                    if runtime
                        .report_terminal_mouse_button(MouseButton::Left, ElementState::Released)
                    {
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
            WindowEvent::KeyboardInput { event, .. }
                if self.runtime.as_ref().is_some_and(|runtime| {
                    terminal_raw_input_should_handle(
                        runtime.terminal_accepts_raw_input(),
                        runtime.is_paste_shortcut(&event),
                        runtime.is_copy_shortcut(&event),
                    )
                }) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.handle_terminal_key_input(&event)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.runtime.as_ref().is_some_and(|runtime| {
                    runtime.workspace().ui.active_dock_tab.is_some()
                        && runtime.is_paste_shortcut(&event)
                }) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.paste_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.runtime.as_ref().is_some_and(|runtime| {
                    runtime.workspace().ui.active_dock_tab.is_some()
                        && runtime.is_copy_shortcut(&event)
                }) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.copy_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if self.runtime.as_ref().is_some_and(|runtime| {
                    runtime.workspace().ui.active_dock_tab.is_some()
                        && runtime.is_cut_shortcut(&event)
                }) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.cut_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } if self.runtime.as_ref().is_some_and(|runtime| {
                runtime.dock_accepts_text_input() && !runtime.modifiers.control_key()
            }) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.append_dock_text(text)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.dock_accepts_text_input()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.append_dock_text(" ")
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Backspace),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.backspace_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Enter),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.submit_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.marking_menu_active()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.dismiss_marking_menu()
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.cancel_terminal_rename() {
                        self.request_redraw_if_needed();
                        return;
                    }
                    // Clear input first; only close dock if input is already empty.
                    let input_was_empty =
                        runtime.current_dock_input().map_or(true, |s| s.is_empty());
                    if input_was_empty {
                        if runtime.close_active_dock() {
                            self.request_redraw_if_needed();
                        }
                    } else {
                        let ui = &mut runtime.session.workspace_mut().ui;
                        match ui.active_dock_tab {
                            Some(DockTab::Terminal) => {
                                ui.terminal.input.clear();
                                ui.terminal.cursor = 0;
                            }
                            None => {}
                        }
                        runtime.invalidate_frame();
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::ArrowLeft),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.move_dock_cursor(-1)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::ArrowRight),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.move_dock_cursor(1)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Home),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.move_dock_cursor_to_edge(true)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::End),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.move_dock_cursor_to_edge(false)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Tab),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime
                    && runtime.complete_dock_input()
                {
                    self.request_redraw_if_needed();
                }
            }
            // Pane focus cycling (decision 021): Tab -> next leaf, Shift+Tab ->
            // previous leaf, when the dock does not own the keyboard. Reuses the
            // FEEL warm-camera focus swap; workspace view state, never journaled.
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Tab),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if self
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_none()) =>
            {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.modifiers.shift_key() {
                        runtime.pane_focus_prev();
                    } else {
                        runtime.pane_focus_next();
                    }
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("s") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::Select)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("b") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::PlaceBoardText)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("v") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::PlaceBoardVia)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("m") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::Move)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("x") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::Delete)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("r") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.set_workspace_tool(WorkspaceTool::DrawBoardTrack)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("f") => {
                if let Some(runtime) = &mut self.runtime {
                    if !runtime.workspace().ui.active_dock_tab.is_some() {
                        runtime.fit_camera();
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("t") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.fit_review_target()
                {
                    self.request_redraw_if_needed();
                }
            }
            // Maximize / restore the focused pane (decision 021 zoom). `Z` is free
            // of the tool keys (s/b/v/m/x/r), fit (f), and review-nav ([ ]); gated
            // to no-active-dock so it never eats terminal input. Workspace view
            // state (transient zoom over the tile tree), never journaled.
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text.eq_ignore_ascii_case("z") => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                {
                    runtime.pane_toggle_zoom();
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text == "[" => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.dispatch_session_command(SessionCommand::SelectPreviousReviewAction)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(ref text),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } if text == "]" => {
                if let Some(runtime) = &mut self.runtime
                    && !runtime.workspace().ui.active_dock_tab.is_some()
                    && runtime.dispatch_session_command(SessionCommand::SelectNextReviewAction)
                {
                    self.request_redraw_if_needed();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.dispatch_session_command(SessionCommand::CancelAuthoringGesture) {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if !matches!(
                        runtime.workspace().selection,
                        datum_gui_protocol::SelectionTarget::None
                    ) && runtime.dispatch_session_command(SessionCommand::ClearSelection)
                    {
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(runtime) = &mut self.runtime {
                    append_gui_verbose_diagnostic_line("redraw handler begin");
                    runtime.redraw_pending = false;
                    let render_started = std::time::Instant::now();
                    if let Err(err) = runtime.render() {
                        fatal_gui_error(event_loop, "render failed", err);
                    }
                    runtime.trace_timing(format!(
                        "redraw render {}ms",
                        render_started.elapsed().as_millis()
                    ));
                }
                if self.advance_kwin_lifecycle_smoke(event_loop) {
                    return;
                }
                if let Some(runtime) = &mut self.runtime {
                    if self.args.interaction_smoke
                        && let Err(err) = runtime.run_interaction_smoke()
                    {
                        fatal_gui_error(event_loop, "interaction smoke failed", err);
                    }
                    if self.args.resize_torture_smoke
                        && let Err(err) = runtime.run_resize_torture_smoke()
                    {
                        fatal_gui_error(event_loop, "resize torture smoke failed", err);
                    }
                    if self.args.visual_test {
                        let screenshot_out =
                            self.args.screenshot_out.as_ref().unwrap_or_else(|| {
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
                    append_gui_verbose_diagnostic_line("redraw handler end");
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.poll_background_work(event_loop);
    }
}

struct Runtime {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    scale_factor: f32,
    renderer: Renderer,
    session: LiveDesignSession,
    /// The ACTIVE camera: the focused leaf's view. Pan/zoom/fit read and write it
    /// and the renderer projects the single live scene through it. The other
    /// leaves' cameras rest warm in `pane_cameras`; on a focus change this is
    /// stashed there and replaced with the newly-focused leaf's warm camera
    /// (P2.1b — no refit lag on focus-switch).
    camera: CameraState,
    /// Warm per-leaf view cameras keyed by `PaneId` (decision 021, P2.1b).
    pane_cameras: PaneCameras,
    last_cursor_pos: Option<(f32, f32)>,
    middle_drag_active: bool,
    right_drag_active: bool,
    dock_drag_active: bool,
    terminal_mouse_button: Option<MouseButton>,
    modifiers: ModifiersState,
    redraw_pending: bool,
    retained_scene: Option<RetainedScene>,
    retained_scene_cache: Vec<(RetainedSceneCacheKey, RetainedScene)>,
    prepared_scene: Option<PreparedScene>,
    scene_dirty: bool,
    terminal_sessions: TerminalSessionRegistry,
    terminal_disconnected_reported: bool,
    terminal_launch_context: TerminalLaunchContext,
    workspace_include_review: bool,
    terminal_production_refresh_pending: bool,
    terminal_workspace_refresh_pending: bool,
    terminal_production_refresh_due: Option<std::time::Instant>,
    terminal_production_refresh_attempts: u8,
    terminal_rename_session_id: Option<String>,
    clipboard: Option<Clipboard>,
}

fn terminal_scrollback_page_step(workspace: &datum_gui_protocol::ReviewWorkspaceState) -> usize {
    let visible_hint = workspace.ui.terminal.lines.len().min(24);
    visible_hint.saturating_sub(1).max(1)
}

impl Runtime {
    fn workspace(&self) -> &datum_gui_protocol::ReviewWorkspaceState {
        self.session.workspace()
    }

    async fn new(
        window: &'static Window,
        launch_state: LaunchState,
        scale_factor_override: Option<f32>,
    ) -> Result<Self> {
        let runtime_started = std::time::Instant::now();
        let LaunchState {
            request: _request,
            state,
            camera,
            terminal_launch_context,
            terminal_sessions,
            workspace_include_review,
        } = launch_state;
        // The initially-focused leaf seeds the warm per-leaf camera store; its
        // camera is the fit camera the launch path already computed.
        let initial_focus = state.ui.layout.focused;
        let wgpu_started = std::time::Instant::now();
        append_gui_diagnostic_line("wgpu instance create begin");
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).context("create surface")?;
        append_gui_diagnostic_line("wgpu request adapter begin");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("request adapter")?;
        append_gui_diagnostic_line("wgpu request device begin");
        let adapter_format_features =
            wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES & adapter.features();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("datum-m7-spike-device"),
                required_features: adapter_format_features,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .context("request device")?;
        append_gui_diagnostic_line("wgpu request device end");
        trace_startup_timing(format!(
            "wgpu init {}ms",
            wgpu_started.elapsed().as_millis()
        ));
        let size = window.inner_size();
        let scale_factor = scale_factor_override.unwrap_or_else(|| window.scale_factor() as f32);
        let caps = surface.get_capabilities(&adapter);
        // Force an sRGB surface so the renderer's sRGB->linear vertex conversion
        // round-trips correctly (near-black tokens must render near-black, not the
        // washed-out grey a linear surface produced from raw sRGB values).
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let msaa_samples = select_msaa_samples(&adapter, format);
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(caps.present_modes[0]);
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
        append_gui_diagnostic_line(format!(
            "initial surface configure begin {}x{} format={:?} present={:?} msaa={}",
            config.width, config.height, config.format, config.present_mode, msaa_samples
        ));
        surface.configure(&device, &config);
        append_gui_diagnostic_line("initial surface configure end");
        let renderer_started = std::time::Instant::now();
        append_gui_diagnostic_line("renderer init begin");
        let renderer = Renderer::new(&device, &queue, config.format, msaa_samples);
        append_gui_diagnostic_line("renderer init end");
        trace_startup_timing(format!(
            "renderer init {}ms",
            renderer_started.elapsed().as_millis()
        ));
        let mut runtime = Self {
            surface,
            device,
            queue,
            config,
            scale_factor,
            renderer,
            session: LiveDesignSession::new(state),
            camera,
            pane_cameras: PaneCameras::new(initial_focus, camera),
            last_cursor_pos: None,
            middle_drag_active: false,
            right_drag_active: false,
            dock_drag_active: false,
            terminal_mouse_button: None,
            modifiers: ModifiersState::empty(),
            redraw_pending: false,
            retained_scene: None,
            retained_scene_cache: Vec::new(),
            prepared_scene: None,
            scene_dirty: true,
            terminal_sessions,
            terminal_disconnected_reported: false,
            terminal_launch_context,
            workspace_include_review,
            terminal_production_refresh_pending: false,
            terminal_workspace_refresh_pending: false,
            terminal_production_refresh_due: None,
            terminal_production_refresh_attempts: 0,
            terminal_rename_session_id: None,
            clipboard: Clipboard::new().ok(),
        };
        runtime.sync_terminal_tabs();
        runtime.resize_terminal_to_dock();
        trace_startup_timing(format!(
            "runtime total {}ms",
            runtime_started.elapsed().as_millis()
        ));
        Ok(runtime)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.apply_resize(width.max(1), height.max(1));
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        let next = (scale_factor as f32).max(0.01);
        if (self.scale_factor - next).abs() <= f32::EPSILON {
            return;
        }
        append_gui_diagnostic_line(format!(
            "scale factor apply {:.4} -> {:.4}",
            self.scale_factor, next
        ));
        self.scale_factor = next;
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_scene();
    }

    fn apply_resize(&mut self, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }
        append_gui_diagnostic_line(format!(
            "resize apply {}x{} -> {width}x{height}",
            self.config.width, self.config.height
        ));
        self.config.width = width;
        self.config.height = height;
        append_gui_diagnostic_line("surface configure begin");
        self.surface.configure(&self.device, &self.config);
        append_gui_diagnostic_line("surface configure end");
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            self.resize_terminal_to_dock();
        }
        self.invalidate_scene();
    }

    fn render(&mut self) -> Result<()> {
        let render_started = std::time::Instant::now();
        let acquire_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(format!(
            "render begin {}x{}",
            self.config.width, self.config.height
        ));
        append_gui_verbose_diagnostic_line("render acquire begin");
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                append_gui_diagnostic_line(format!(
                    "surface acquire recovered by reconfigure at {}x{}",
                    self.config.width, self.config.height
                ));
                self.surface.configure(&self.device, &self.config);
                self.invalidate_frame();
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => {
                append_gui_diagnostic_line("surface acquire timeout; frame skipped");
                self.invalidate_frame();
                return Ok(());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                anyhow::bail!("surface out of memory");
            }
            Err(err) => {
                anyhow::bail!("acquire next surface texture: {err}");
            }
        };
        let acquire_elapsed = acquire_started.elapsed();
        append_gui_verbose_diagnostic_line("render acquire end");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scene_started = std::time::Instant::now();
        let retained_was_cached = self.retained_scene.is_some();
        let prepared_was_cached = self.prepared_scene.is_some();
        let mut retained_build_ms = 0;
        let mut prepared_build_ms = 0;
        if self.prepared_scene.is_none() {
            append_gui_verbose_diagnostic_line(format!(
                "render scene prepare begin retained_cached={retained_was_cached}"
            ));
            self.scene_dirty = false;
            if self.retained_scene.is_none() {
                let retained_started = std::time::Instant::now();
                append_gui_verbose_diagnostic_line("retained scene build begin");
                self.retained_scene = Some(RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                ));
                retained_build_ms = retained_started.elapsed().as_millis();
                append_gui_verbose_diagnostic_line(format!(
                    "retained scene build end {retained_build_ms}ms"
                ));
            }
            let retained = self
                .retained_scene
                .as_ref()
                .context("retained scene should exist before prepared scene rebuild")?;
            let prepared_started = std::time::Instant::now();
            append_gui_verbose_diagnostic_line("prepared scene build begin");
            self.prepared_scene = Some(PreparedScene::from_workspace_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
                self.camera,
                retained,
            ));
            prepared_build_ms = prepared_started.elapsed().as_millis();
            append_gui_verbose_diagnostic_line(format!(
                "prepared scene build end {prepared_build_ms}ms"
            ));
        }
        let scene_elapsed = scene_started.elapsed();
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before render")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before render")?;
        let renderer_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("renderer render begin");
        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            &prepared,
            retained,
            self.config.width,
            self.config.height,
        )?;
        let renderer_elapsed = renderer_started.elapsed();
        append_gui_verbose_diagnostic_line(format!(
            "renderer render end {}ms",
            renderer_elapsed.as_millis()
        ));
        let present_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("frame present begin");
        frame.present();
        let present_elapsed = present_started.elapsed();
        append_gui_verbose_diagnostic_line(format!(
            "frame present end {}ms total={}ms",
            present_elapsed.as_millis(),
            render_started.elapsed().as_millis()
        ));
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
        Ok(())
    }

    fn run_interaction_smoke(&mut self) -> Result<()> {
        let resized_width = self.config.width.saturating_add(137).max(1);
        let resized_height = self.config.height.saturating_add(83).max(1);
        self.resize(resized_width, resized_height);
        self.render().context("interaction smoke resize render")?;

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
        self.render().context("interaction smoke click render")?;
        Ok(())
    }

    fn run_resize_torture_smoke(&mut self) -> Result<()> {
        let restored = (1344_u32, 806_u32);
        let maximized = (1920_u32, 1051_u32);
        append_gui_verbose_diagnostic_line("resize torture begin");
        for (index, (width, height)) in [
            maximized, restored, maximized, restored, maximized, restored,
        ]
        .into_iter()
        .enumerate()
        {
            append_gui_verbose_diagnostic_line(format!(
                "resize torture step {index} target {width}x{height}"
            ));
            self.resize(width, height);
            self.render()
                .with_context(|| format!("resize torture render step {index} {width}x{height}"))?;
        }
        append_gui_verbose_diagnostic_line("resize torture end");
        Ok(())
    }

    #[cfg(feature = "visual")]
    fn write_visual_screenshot(&mut self, path: &Path) -> Result<()> {
        let image = self.capture_visual_screenshot()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create screenshot directory {}", parent.display()))?;
        }
        image
            .save(path)
            .with_context(|| format!("write visual shell screenshot {}", path.display()))
    }

    #[cfg(not(feature = "visual"))]
    fn write_visual_screenshot(&mut self, _path: &Path) -> Result<()> {
        anyhow::bail!("datum-gui visual screenshots require the datum-gui-app visual feature")
    }

    #[cfg(feature = "visual")]
    fn capture_visual_screenshot(&mut self) -> Result<image::RgbaImage> {
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("datum-gui-layer-b-visual-capture-target"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        if self.prepared_scene.is_none() {
            self.scene_dirty = false;
            let retained = self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            self.prepared_scene = Some(PreparedScene::from_workspace_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
                self.camera,
                retained,
            ));
        }
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before visual screenshot")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before visual screenshot")?;
        self.renderer.render(
            &self.device,
            &self.queue,
            &target_view,
            prepared,
            retained,
            self.config.width,
            self.config.height,
        )?;
        self.read_visual_texture(&target)
    }

    #[cfg(feature = "visual")]
    fn read_visual_texture(&self, texture: &wgpu::Texture) -> Result<image::RgbaImage> {
        let width = self.config.width;
        let height = self.config.height;
        let unpadded_bytes_per_row = width * COPY_BYTES_PER_PIXEL;
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, WGPU_COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-gui-layer-b-visual-readback-buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("datum-gui-layer-b-visual-readback-encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("poll device for visual shell readback")?;
        receiver
            .recv()
            .context("wait for visual shell readback mapping")?
            .context("map visual shell readback buffer")?;

        let mapped = buffer_slice.get_mapped_range();
        let mut pixels = vec![0_u8; (width * height * COPY_BYTES_PER_PIXEL) as usize];
        for row in 0..height as usize {
            let source_start = row * padded_bytes_per_row as usize;
            let source_end = source_start + unpadded_bytes_per_row as usize;
            let dest_start = row * unpadded_bytes_per_row as usize;
            let dest_end = dest_start + unpadded_bytes_per_row as usize;
            pixels[dest_start..dest_end].copy_from_slice(&mapped[source_start..source_end]);
        }
        drop(mapped);
        output_buffer.unmap();

        convert_texture_pixels_to_rgba(&mut pixels, self.config.format)?;
        image::RgbaImage::from_raw(width, height, pixels)
            .context("construct visual shell image from readback pixels")
    }

    fn prepared_scene(&mut self) -> &PreparedScene {
        let retained = self.retained_scene.get_or_insert_with(|| {
            RetainedScene::from_workspace_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            )
        });
        self.prepared_scene.get_or_insert_with(|| {
            self.scene_dirty = false;
            PreparedScene::from_workspace_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
                self.camera,
                retained,
            )
        })
    }

    fn retained_scene_cache_key(&self) -> RetainedSceneCacheKey {
        let workspace = self.workspace();
        RetainedSceneCacheKey {
            scene_id: workspace.scene.scene_id.clone(),
            source_revision: workspace.scene.source_revision.clone(),
            width: self.config.width,
            height: self.config.height,
            scale_bits: self.scale_factor.to_bits(),
            dock_height_px: workspace.ui.dock_height_px,
            show_authored: workspace.ui.filters.show_authored,
            show_proposed: workspace.ui.filters.show_proposed,
            show_unrouted: workspace.ui.filters.show_unrouted,
            dim_unrelated: workspace.ui.filters.dim_unrelated,
            layer_visibility: workspace.ui.filters.layer_visibility.clone(),
            selection: retained_selection_cache_key(workspace, &workspace.selection),
        }
    }

    fn cache_retained_scene(&mut self, key: RetainedSceneCacheKey, retained: RetainedScene) {
        if let Some(index) = self
            .retained_scene_cache
            .iter()
            .position(|(cached_key, _)| cached_key == &key)
        {
            self.retained_scene_cache.remove(index);
        }
        self.retained_scene_cache.push((key, retained));
        if self.retained_scene_cache.len() > RETAINED_SCENE_CACHE_LIMIT {
            self.retained_scene_cache.remove(0);
        }
    }

    fn restore_cached_retained_scene(&mut self) -> bool {
        let key = self.retained_scene_cache_key();
        if let Some(index) = self
            .retained_scene_cache
            .iter()
            .position(|(cached_key, _)| cached_key == &key)
        {
            let (_, retained) = self.retained_scene_cache.remove(index);
            self.retained_scene = Some(retained);
            return true;
        }
        false
    }

    fn invalidate_scene_for_session_change(&mut self, previous_key: RetainedSceneCacheKey) {
        if let Some(retained) = self.retained_scene.take() {
            self.cache_retained_scene(previous_key, retained);
        }
        self.prepared_scene = None;
        self.scene_dirty = true;
        self.restore_cached_retained_scene();
    }

    fn invalidate_scene(&mut self) {
        self.retained_scene = None;
        self.retained_scene_cache.clear();
        self.prepared_scene = None;
        self.scene_dirty = true;
    }

    fn invalidate_frame(&mut self) {
        self.prepared_scene = None;
        self.scene_dirty = true;
    }

    fn push_terminal_line(&mut self, line: String) {
        let ui = &mut self.session.workspace_mut().ui;
        ui.terminal.lines.push(line);
        if ui.terminal.lines.len() > 240 {
            ui.terminal.lines.drain(0..ui.terminal.lines.len() - 240);
        }
        ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn set_active_dock(&mut self, tab: DockTab) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        if ui.active_dock_tab == Some(tab) {
            return false;
        }
        let dock_was_open = ui.active_dock_tab.is_some();
        ui.active_dock_tab = Some(tab);
        if dock_was_open {
            self.invalidate_frame();
        } else {
            self.invalidate_scene();
        }
        if matches!(tab, DockTab::Terminal) {
            self.resize_terminal_to_dock();
            self.refresh_terminal_activity_summary();
        }
        true
    }

    fn close_active_dock(&mut self) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        if ui.active_dock_tab.is_none() {
            return false;
        }
        ui.active_dock_tab = None;
        self.invalidate_scene();
        true
    }

    fn dock_accepts_text_input(&self) -> bool {
        self.workspace().ui.active_dock_tab.is_some()
    }

    fn terminal_accepts_raw_input(&self) -> bool {
        matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal))
            && self.terminal_rename_session_id.is_none()
            && self.terminal_sessions.active_attached()
    }

    fn handle_terminal_key_input(&mut self, event: &KeyEvent) -> bool {
        let application_cursor_keys = self.workspace().ui.terminal.application_cursor_keys;
        let application_keypad = self.workspace().ui.terminal.application_keypad;
        match terminal_key_action(
            event,
            self.modifiers,
            application_cursor_keys,
            application_keypad,
        ) {
            TerminalKeyAction::Write(bytes) => self.write_terminal_bytes(&bytes),
            TerminalKeyAction::Interrupt => {
                if !self.terminal_sessions.active_attached() {
                    self.push_terminal_line(
                        "terminal session is detached; activate the tab to reattach".to_string(),
                    );
                    return true;
                }
                if let Err(err) = self.terminal_sessions.active().interrupt() {
                    self.push_terminal_line(format!("terminal interrupt failed: {err}"));
                }
                true
            }
            TerminalKeyAction::TerminateSession => {
                self.terminate_terminal_session();
                true
            }
            TerminalKeyAction::RestartSession => {
                self.restart_terminal_session();
                true
            }
            TerminalKeyAction::ScrollbackPageUp => {
                self.scroll_terminal_scrollback(terminal_scrollback_page_step(self.workspace()));
                true
            }
            TerminalKeyAction::ScrollbackPageDown => {
                self.scroll_terminal_scrollback_down(terminal_scrollback_page_step(
                    self.workspace(),
                ));
                true
            }
            TerminalKeyAction::ScrollbackTop => {
                self.scroll_terminal_scrollback_to_top();
                true
            }
            TerminalKeyAction::ScrollbackBottom => {
                self.scroll_terminal_scrollback_to_bottom();
                true
            }
            TerminalKeyAction::ConsumeRelease => true,
            TerminalKeyAction::LetPasteShortcutHandle
            | TerminalKeyAction::LetCopyShortcutHandle
            | TerminalKeyAction::Ignore => false,
        }
    }

    fn scroll_terminal_scrollback(&mut self, delta: usize) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        let max = terminal.lines.len();
        terminal.scroll_offset = (terminal.scroll_offset + delta).min(max);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_down(&mut self, delta: usize) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = terminal.scroll_offset.saturating_sub(delta);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_top(&mut self) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = terminal.lines.len();
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_bottom(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn write_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        if !self.terminal_sessions.active_attached() {
            self.push_terminal_line(
                "terminal session is detached; activate the tab to reattach".to_string(),
            );
            return true;
        }
        if bytes.iter().any(|byte| matches!(byte, b'\n' | b'\r')) {
            self.mark_terminal_production_refresh_pending();
        }
        if let Err(err) = self.terminal_sessions.active().write_bytes(bytes) {
            self.push_terminal_line(format!("terminal write failed: {err}"));
        }
        true
    }

    fn report_terminal_focus_event(&mut self, focused: bool) {
        if !self.workspace().ui.terminal.focus_event_reporting
            || !self.terminal_sessions.active_attached()
        {
            return;
        }
        let bytes = terminal_focus_event_sequence(focused);
        if let Err(err) = self.terminal_sessions.active().write_bytes(bytes) {
            self.push_terminal_line(format!("terminal focus report failed: {err}"));
        }
    }

    fn report_terminal_mouse_button(&mut self, button: MouseButton, state: ElementState) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let pressed = state == ElementState::Pressed;
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_button_sequence(button, pressed, column, row),
            Some("urxvt") => terminal_urxvt_mouse_button_sequence(button, pressed, column, row),
            Some("utf8") => terminal_utf8_mouse_button_sequence(button, pressed, column, row),
            None => terminal_x10_mouse_button_sequence(button, pressed, column, row),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        self.terminal_mouse_button = if state == ElementState::Pressed {
            Some(button)
        } else {
            None
        };
        true
    }

    fn report_terminal_mouse_motion(&mut self) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let terminal = &self.workspace().ui.terminal;
        let held_button = match terminal.mouse_reporting_mode.as_deref() {
            Some("any_event") => self.terminal_mouse_button,
            Some("button_event") => {
                let Some(button) = self.terminal_mouse_button else {
                    return false;
                };
                Some(button)
            }
            _ => return false,
        };
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_motion_sequence(held_button, column, row),
            Some("urxvt") => held_button
                .and_then(|button| terminal_urxvt_mouse_motion_sequence(button, column, row)),
            Some("utf8") => held_button
                .and_then(|button| terminal_utf8_mouse_motion_sequence(button, column, row)),
            None => held_button
                .and_then(|button| terminal_x10_mouse_motion_sequence(button, column, row)),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        true
    }

    fn report_terminal_mouse_wheel(&mut self, scroll_lines: f32) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_wheel_sequence(scroll_lines, column, row),
            Some("urxvt") => terminal_urxvt_mouse_wheel_sequence(scroll_lines, column, row),
            Some("utf8") => terminal_utf8_mouse_wheel_sequence(scroll_lines, column, row),
            None => terminal_x10_mouse_wheel_sequence(scroll_lines, column, row),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        true
    }

    fn terminal_mouse_reporting_active(&self) -> bool {
        let terminal = &self.workspace().ui.terminal;
        terminal.mouse_reporting_mode.is_some() && self.terminal_sessions.active_attached()
    }

    fn terminal_mouse_encoding_sequence(
        &self,
        sequence: impl FnOnce(Option<&str>) -> Option<Vec<u8>>,
    ) -> Option<Vec<u8>> {
        sequence(
            self.workspace()
                .ui
                .terminal
                .mouse_coordinate_encoding
                .as_deref(),
        )
    }

    fn terminal_mouse_cell(&self) -> Option<(u16, u16)> {
        let (x, y) = self.last_cursor_pos?;
        let layout = self.current_layout();
        let rect_x = layout.bottom_strip.x + 12.0;
        let rect_y = layout.bottom_strip.y + 44.0;
        let rect_width = layout.bottom_strip.width - 24.0;
        let rect_height = (layout.bottom_strip.height - 56.0).max(0.0);
        if x < rect_x || x > rect_x + rect_width || y < rect_y || y > rect_y + rect_height {
            return None;
        }
        let terminal = &self.workspace().ui.terminal;
        let column =
            (((x - rect_x) / rect_width.max(1.0)) * terminal.columns as f32).floor() as u16;
        let row = (((y - rect_y) / rect_height.max(1.0)) * terminal.rows as f32).floor() as u16;
        Some((
            column.saturating_add(1).min(terminal.columns.max(1)),
            row.saturating_add(1).min(terminal.rows.max(1)),
        ))
    }

    fn write_terminal_mouse_report(&mut self, bytes: &[u8]) {
        if let Err(err) = self.terminal_sessions.active().write_bytes(bytes) {
            self.push_terminal_line(format!("terminal mouse report failed: {err}"));
        }
    }

    fn terminate_terminal_session(&mut self) {
        match self
            .terminal_sessions
            .terminate_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {}
            Err(err) => self.push_terminal_line(format!("terminal terminate failed: {err}")),
        }
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn restart_terminal_session(&mut self) {
        match self.terminal_sessions.restart_active(
            &mut self.session.workspace_mut().ui.terminal,
            &self.terminal_launch_context,
        ) {
            Ok(()) => self.resize_terminal_to_dock(),
            Err(err) => self.push_terminal_line(format!("terminal restart failed: {err}")),
        }
        self.terminal_disconnected_reported = false;
        self.terminal_production_refresh_pending = false;
        self.terminal_workspace_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn activate_terminal_session(&mut self, session_id: &str) -> bool {
        if let Err(err) = self.terminal_sessions.activate(session_id) {
            self.push_terminal_line(format!("terminal session activate failed: {err}"));
            return true;
        }
        let _ = refresh_terminal_session_context_from_state(
            self.terminal_sessions.active(),
            &self.terminal_launch_context,
            self.workspace(),
            self.last_cursor_pos,
        );
        self.set_active_dock(DockTab::Terminal);
        self.refresh_terminal_activity_summary();
        self.sync_terminal_tabs();
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        true
    }

    fn is_paste_shortcut(&self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Released {
            return false;
        }
        (self.modifiers.control_key()
            && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyV)))
            || (self.modifiers.shift_key()
                && matches!(event.logical_key, Key::Named(NamedKey::Insert)))
    }

    fn is_copy_shortcut(&self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Released
            || !self.modifiers.control_key()
            || !matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyC))
        {
            return false;
        }
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) => self.modifiers.shift_key(),
            None => false,
        }
    }

    fn is_cut_shortcut(&self, event: &KeyEvent) -> bool {
        event.state == ElementState::Released
            && self.modifiers.control_key()
            && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyX))
    }

    fn append_dock_text(&mut self, text: &str) -> bool {
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        if !self.dock_tab_accepts_edit(active) {
            return false;
        }
        if text.chars().any(|ch| ch.is_control()) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = match active {
            DockTab::Terminal => (&mut ui.terminal.input, &mut ui.terminal.cursor),
        };
        let byte_pos = char_to_byte_pos(input, *cursor);
        input.insert_str(byte_pos, text);
        *cursor += text.chars().count();
        self.invalidate_frame();
        true
    }

    fn dock_tab_accepts_edit(&self, active: DockTab) -> bool {
        matches!(active, DockTab::Terminal) && self.terminal_rename_session_id.is_some()
    }

    fn current_dock_input(&self) -> Option<&str> {
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) if self.terminal_rename_session_id.is_some() => {
                Some(self.workspace().ui.terminal.input.as_str())
            }
            _ => None,
        }
    }

    fn current_dock_input_mut(&mut self) -> Option<&mut String> {
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) if self.terminal_rename_session_id.is_some() => {
                Some(&mut self.session.workspace_mut().ui.terminal.input)
            }
            _ => None,
        }
    }

    fn copy_dock_input(&mut self) -> bool {
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            let Some(text) = terminal_scrollback_copy_text(&self.workspace().ui.terminal) else {
                return false;
            };
            if self.write_clipboard_text(&text).is_err() {
                self.push_terminal_line("clipboard copy failed".to_string());
                return true;
            }
            self.push_terminal_line("terminal scrollback copied".to_string());
            return true;
        }
        let Some(input) = self
            .workspace()
            .ui
            .active_dock_tab
            .and_then(|_| self.current_dock_input_mut().map(|s| s.clone()))
        else {
            return false;
        };
        if self.write_clipboard_text(&input).is_err() {
            self.push_terminal_line("clipboard copy failed".to_string());
        }
        false
    }

    fn cut_dock_input(&mut self) -> bool {
        let Some(text) = self
            .workspace()
            .ui
            .active_dock_tab
            .and_then(|_| self.current_dock_input_mut().map(|s| s.clone()))
        else {
            return false;
        };
        if self.write_clipboard_text(&text).is_err() {
            self.push_terminal_line("clipboard cut failed".to_string());
            return true;
        }
        if let Some(input) = self.current_dock_input_mut() {
            input.clear();
        }
        self.invalidate_frame();
        true
    }

    fn paste_dock_input(&mut self) -> bool {
        let Ok(text) = self.read_clipboard_text() else {
            self.push_terminal_line("clipboard paste failed".to_string());
            return false;
        };
        if text.is_empty() {
            return false;
        }
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            let bytes = terminal_paste_bytes(
                &text,
                self.terminal_sessions.active_bracketed_paste_enabled(),
            );
            return self.write_terminal_bytes(&bytes);
        }
        self.append_dock_text(&text)
    }

    fn read_clipboard_text(&mut self) -> Result<String> {
        if let Some(clipboard) = &mut self.clipboard
            && let Ok(text) = clipboard
                .get()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text()
            && !text.is_empty()
        {
            return Ok(text);
        }
        self.read_clipboard_text_fallback()
    }

    fn write_clipboard_text(&mut self, text: &str) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard
            && clipboard
                .set()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text(text.to_string())
                .is_ok()
        {
            return Ok(());
        }
        self.write_clipboard_text_fallback(text)
    }

    fn read_clipboard_text_fallback(&self) -> Result<String> {
        let output = Command::new("/usr/bin/xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .context("read clipboard with xclip")?;
        if !output.status.success() {
            anyhow::bail!("xclip clipboard read failed");
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn write_clipboard_text_fallback(&self, text: &str) -> Result<()> {
        let mut child = Command::new("/usr/bin/xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .context("spawn xclip for clipboard write")?;
        let mut stdin = child.stdin.take().context("take xclip stdin")?;
        stdin
            .write_all(text.as_bytes())
            .context("write clipboard text to xclip")?;
        drop(stdin);
        let status = child.wait().context("wait for xclip clipboard write")?;
        if !status.success() {
            anyhow::bail!("xclip clipboard write failed");
        }
        Ok(())
    }

    fn backspace_dock_input(&mut self) -> bool {
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        if !self.dock_tab_accepts_edit(active) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = match active {
            DockTab::Terminal => (&mut ui.terminal.input, &mut ui.terminal.cursor),
        };
        if *cursor > 0 {
            let byte_pos = char_to_byte_pos(input, *cursor - 1);
            let byte_end = char_to_byte_pos(input, *cursor);
            input.drain(byte_pos..byte_end);
            *cursor -= 1;
            self.invalidate_frame();
            return true;
        }
        false
    }

    fn move_dock_cursor(&mut self, delta: i32) -> bool {
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        if !self.dock_tab_accepts_edit(active) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = match active {
            DockTab::Terminal => (&ui.terminal.input, &mut ui.terminal.cursor),
        };
        let char_count = input.chars().count();
        let new_pos = (*cursor as i32 + delta).clamp(0, char_count as i32) as usize;
        if new_pos != *cursor {
            *cursor = new_pos;
            self.invalidate_frame();
            return true;
        }
        false
    }

    fn move_dock_cursor_to_edge(&mut self, home: bool) -> bool {
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        if !self.dock_tab_accepts_edit(active) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = match active {
            DockTab::Terminal => (&ui.terminal.input, &mut ui.terminal.cursor),
        };
        let target = if home { 0 } else { input.chars().count() };
        if target != *cursor {
            *cursor = target;
            self.invalidate_frame();
            return true;
        }
        false
    }

    fn complete_dock_input(&mut self) -> bool {
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) => false,
            None => false,
        }
    }

    fn submit_dock_input(&mut self) -> bool {
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) => self.submit_terminal_rename_input(),
            None => false,
        }
    }

    fn log_review_event(&mut self, message: impl Into<String>) {
        // GUI-action narration is the AutoCAD/Eagle command-echo: it belongs in
        // the (not-yet-built) editor command console, never in the real PTY
        // terminal. Route it to the invisible console sink. No repaint is forced
        // here — the sink has no visible surface, and every narrating action
        // already invalidates the frame independently.
        self.session
            .workspace_mut()
            .ui
            .push_console_line(message.into());
    }

    fn apply_session_result(
        &mut self,
        result: datum_gui_protocol::SessionCommandResult,
        previous_retained_key: Option<RetainedSceneCacheKey>,
    ) -> bool {
        if !result.handled {
            return false;
        }
        for event in result.events {
            match event {
                SessionEvent::SceneChanged => {
                    if let Some(key) = previous_retained_key.clone() {
                        self.invalidate_scene_for_session_change(key);
                    } else {
                        self.invalidate_scene();
                    }
                }
                // Text and outline selection feedback is drawn as a lightweight
                // screen overlay. Do not rebuild retained board geometry when
                // only that overlay target changes.
                SessionEvent::SelectionChanged(selection) => {
                    let next_selection_key =
                        retained_selection_cache_key(self.workspace(), &selection);
                    if previous_retained_key
                        .as_ref()
                        .is_some_and(|key| key.selection == next_selection_key)
                    {
                        self.invalidate_frame();
                    } else if let Some(key) = previous_retained_key.clone() {
                        self.invalidate_scene_for_session_change(key);
                    } else {
                        self.invalidate_scene();
                    }
                }
                SessionEvent::FrameChanged => self.invalidate_frame(),
                SessionEvent::ToolChanged(_) => self.invalidate_frame(),
            }
        }
        true
    }

    fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
        let previous_retained_key = self.retained_scene_cache_key();
        let result = self.session.apply(command);
        self.apply_session_result(result, Some(previous_retained_key))
    }

    fn set_workspace_tool(&mut self, tool: WorkspaceTool) -> bool {
        if !matches!(tool, WorkspaceTool::Select) {
            self.push_terminal_line(format!(
                "{} is disabled in the Phase 1 read-only GUI",
                tool.label()
            ));
            self.invalidate_frame();
            return true;
        }
        let handled = self.dispatch_session_command(SessionCommand::SetTool(tool));
        if handled {
            self.log_review_event(format!("tool {}", tool.label()));
        }
        handled
    }

    fn active_tool_is_authoring(&self) -> bool {
        false
    }

    fn handle_authoring_pointer_move(&mut self, screen_pos: (f32, f32)) -> bool {
        if !self.active_tool_is_authoring() || !self.workspace().authoring.gesture.is_active() {
            return false;
        }
        let prepared = self.prepared_scene();
        let Some(world) = prepared.world_point_at_screen(screen_pos.0, screen_pos.1) else {
            return false;
        };
        let target_object_id = self.authoring_target_object_id(world);
        self.dispatch_session_command(SessionCommand::PreviewAuthoringGesture {
            world,
            target_object_id,
        })
    }

    fn handle_authoring_canvas_click(
        &mut self,
        world: PointNm,
        target_object_id: Option<String>,
    ) -> bool {
        if !self.active_tool_is_authoring() {
            return false;
        }
        match self.workspace().tool {
            WorkspaceTool::DrawBoardTrack
                if self.workspace().authoring.gesture.anchor.is_some() =>
            {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_draw_board_track_handoff(world)
                else {
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "draw-board-track");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardVia => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_via_handoff(world)
                else {
                    self.push_terminal_line("place via requires a board net context".to_string());
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-via");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::PlaceBoardText => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_place_board_text_handoff(world)
                else {
                    self.push_terminal_line("place text requires project backing".to_string());
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "place-board-text");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Move if self.workspace().authoring.gesture.anchor.is_some() => {
                let Some(handoff) = self
                    .session
                    .workspace_mut()
                    .finish_move_component_handoff(world)
                else {
                    self.push_terminal_line(
                        "move requires a selected component target".to_string(),
                    );
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "move-board-component");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Move => {
                let Some(target) = target_object_id.clone() else {
                    self.push_terminal_line("move requires clicking a component first".to_string());
                    return true;
                };
                if !target.starts_with("component:") {
                    self.push_terminal_line("move currently supports components only".to_string());
                    return true;
                }
            }
            WorkspaceTool::Delete => {
                let Some(target) = target_object_id else {
                    self.push_terminal_line(
                        "delete requires an authored object target".to_string(),
                    );
                    return true;
                };
                let Some(handoff) = self.workspace().delete_authored_object_handoff(&target) else {
                    self.push_terminal_line(format!("delete unsupported target {target}"));
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "delete-authored-object");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Select | WorkspaceTool::DrawBoardTrack => {}
        }
        self.dispatch_session_command(SessionCommand::BeginAuthoringGesture {
            world,
            target_object_id,
        })
    }

    fn authoring_target_object_id(&mut self, world: PointNm) -> Option<String> {
        let target = {
            let retained = self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            retained
                .hit_test_authored_world(world, self.session.workspace())
                .cloned()
        };
        match target {
            Some(HitTarget::AuthoredObject(id)) => Some(id),
            Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        }
    }

    fn queue_authoring_terminal_handoff(
        &mut self,
        handoff: TerminalCommandHandoff,
        event_label: &str,
    ) {
        if self
            .workspace()
            .backing
            .as_ref()
            .is_some_and(|backing| backing.request.board_file.is_some())
        {
            self.set_active_dock(DockTab::Terminal);
            self.push_terminal_line(
                "authoring tools require a native Datum project; open with --project-root instead of --board <kicad_pcb>"
                    .to_string(),
            );
            return;
        }
        self.set_active_dock(DockTab::Terminal);
        self.mark_terminal_workspace_refresh_pending();
        let command = prepare_terminal_command_execution(
            self.terminal_sessions.active(),
            "authoring_tool_command",
            &handoff,
        )
        .unwrap_or_else(|err| {
            self.push_terminal_line(format!("terminal handoff prepare failed: {err}"));
            handoff.command.clone()
        });
        let mut bytes = command.into_bytes();
        bytes.push(b'\r');
        self.write_terminal_bytes(&bytes);
        self.log_review_event(format!("queued authoring command {event_label}"));
    }

    fn handle_primary_click(&mut self) -> bool {
        if self.dismiss_marking_menu() {
            return true;
        }
        let Some((x, y)) = self.last_cursor_pos else {
            self.trace_click("primary click ignored: no cursor position".to_string());
            return false;
        };
        // Click-to-focus (decision 021): a press landing in a NON-focused pane
        // swaps focus to that pane (an instant warm-camera swap via the FEEL
        // focus path) instead of running board interaction in the wrong pane. A
        // click in the already-focused pane (or outside every pane) falls through
        // to today's select/pan behavior below. This is gui_local workspace view
        // state — never a verb/commit/journal path.
        if let Some(pane_id) = self.pane_at_screen(x, y)
            && pane_id != self.workspace().ui.layout.focused
        {
            self.swap_pane_focus(|layout| layout.focused = pane_id);
            self.log_review_event(format!("click-to-focus pane {}", pane_id.0));
            self.trace_click(format!("primary click ({x:.1}, {y:.1}) focus-swapped to pane {}", pane_id.0));
            return true;
        }
        let prepared_started = std::time::Instant::now();
        let (prepared_target, world_point) = {
            let prepared = self.prepared_scene();
            (
                prepared.hit_test(x, y).cloned(),
                prepared.world_point_at_screen(x, y),
            )
        };
        let prepared_elapsed = prepared_started.elapsed();
        if let Some(target) = prepared_target {
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) prepared target {target:?}; prepare {}ms; dock {:?}",
                prepared_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return self.select_hit_target(&target);
        }
        if let Some(world_point) = world_point {
            let retained_started = std::time::Instant::now();
            let retained_target = {
                let retained = self.retained_scene.get_or_insert_with(|| {
                    RetainedScene::from_workspace_for_surface(
                        self.session.workspace(),
                        self.config.width,
                        self.config.height,
                        self.scale_factor,
                    )
                });
                retained
                    .hit_test_authored_world(world_point, self.session.workspace())
                    .cloned()
            };
            let retained_elapsed = retained_started.elapsed();
            let target_object_id = match &retained_target {
                Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => {
                    Some(id.clone())
                }
                _ => None,
            };
            if self.handle_authoring_canvas_click(world_point, target_object_id) {
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) world ({}, {}) handled by authoring tool {}; prepare {}ms; retained {}ms",
                    world_point.x,
                    world_point.y,
                    self.workspace().tool.label(),
                    prepared_elapsed.as_millis(),
                    retained_elapsed.as_millis()
                ));
                return true;
            }
            if let Some(target) = retained_target {
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) world ({}, {}) retained target {target:?}; prepare {}ms; retained {}ms; dock {:?}",
                    world_point.x,
                    world_point.y,
                    prepared_elapsed.as_millis(),
                    retained_elapsed.as_millis(),
                    self.workspace().ui.active_dock_tab
                ));
                return self.select_hit_target(&target);
            }
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) world ({}, {}) no retained target; prepare {}ms; retained {}ms; dock {:?}",
                world_point.x,
                world_point.y,
                prepared_elapsed.as_millis(),
                retained_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return false;
        }
        self.trace_click(format!(
            "primary click ({x:.1}, {y:.1}) no prepared or viewport target; prepare {}ms; dock {:?}",
            prepared_elapsed.as_millis(),
            self.workspace().ui.active_dock_tab
        ));
        false
    }

    fn trace_click(&self, message: String) {
        if std::env::var_os("DATUM_TRACE_CLICKS").is_some() {
            eprintln!("[datum-click] {message}");
        }
    }

    fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        let started = std::time::Instant::now();
        let handled = self.select_hit_target_inner(target);
        self.trace_timing(format!(
            "select target {target:?} handled={handled} {}ms",
            started.elapsed().as_millis()
        ));
        handled
    }

    fn select_hit_target_inner(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::ReviewAction(action_id) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectReviewAction(
                    action_id.clone(),
                ));
                if handled {
                    self.log_review_event(format!("selected review action {action_id}"));
                }
                handled
            }
            HitTarget::AuthoredObject(object_id) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectAuthoredObject(
                    object_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.hovered_object_id = None;
                    self.log_review_event(format!("selected authored object {object_id}"));
                }
                handled
            }
            HitTarget::CheckFinding(fingerprint) => {
                let handled = self.dispatch_session_command(SessionCommand::SelectCheckFinding(
                    fingerprint.clone(),
                ));
                if handled {
                    let target = self
                        .session
                        .workspace()
                        .checks
                        .findings
                        .iter()
                        .find(|finding| finding.fingerprint == *fingerprint)
                        .and_then(|finding| {
                            datum_gui_protocol::check_finding_scene_target_object_id(
                                &self.session.workspace().scene,
                                finding,
                            )
                        });
                    self.session.workspace_mut().ui.hovered_object_id = target.clone();
                    if let Some(target) = target {
                        let fit = self.fit_scene_object(&target);
                        self.log_review_event(format!(
                            "selected check finding {fingerprint}; target {target}{}",
                            if fit { "; fit" } else { "" }
                        ));
                    } else {
                        self.log_review_event(format!("selected check finding {fingerprint}"));
                    }
                }
                handled
            }
            HitTarget::FitBoard => {
                self.fit_camera();
                self.log_review_event("fit board".to_string());
                true
            }
            HitTarget::FitReviewTarget => {
                let handled = self.fit_review_target();
                if handled {
                    self.log_review_event("fit active review target".to_string());
                }
                handled
            }
            HitTarget::SetWorkspaceTool(tool) => self.set_workspace_tool(*tool),
            HitTarget::ReviewPrev => {
                let handled =
                    self.dispatch_session_command(SessionCommand::SelectPreviousReviewAction);
                if handled {
                    self.log_review_event("selected previous review action".to_string());
                }
                handled
            }
            HitTarget::ReviewNext => {
                let handled = self.dispatch_session_command(SessionCommand::SelectNextReviewAction);
                if handled {
                    self.log_review_event("selected next review action".to_string());
                }
                handled
            }
            HitTarget::ToggleShowAuthored => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowAuthored);
                if handled {
                    let state = if self.workspace().ui.filters.show_authored {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_review_event(format!("authored visibility {state}"));
                }
                handled
            }
            HitTarget::ToggleShowProposed => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowProposed);
                if handled {
                    let state = if self.workspace().ui.filters.show_proposed {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_review_event(format!("proposal visibility {state}"));
                }
                handled
            }
            HitTarget::ToggleShowUnrouted => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleShowUnrouted);
                if handled {
                    let state = if self.workspace().ui.filters.show_unrouted {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_review_event(format!("unrouted visibility {state}"));
                }
                handled
            }
            HitTarget::ToggleDimUnrelated => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleDimUnrelated);
                if handled {
                    let state = if self.workspace().ui.filters.dim_unrelated {
                        "on"
                    } else {
                        "off"
                    };
                    self.log_review_event(format!("dim unrelated {state}"));
                }
                handled
            }
            HitTarget::ToggleLayer(layer_id) => {
                let handled = self.dispatch_session_command(SessionCommand::ToggleLayerVisibility(
                    layer_id.clone(),
                ));
                if handled {
                    self.session.workspace_mut().ui.filters.active_layer_id =
                        Some(layer_id.clone());
                    let visible = self
                        .workspace()
                        .ui
                        .filters
                        .layer_visibility
                        .get(layer_id)
                        .copied()
                        .unwrap_or(true);
                    let state = if visible { "visible" } else { "hidden" };
                    self.log_review_event(format!("layer {layer_id} {state}"));
                    self.invalidate_scene();
                }
                handled
            }
            HitTarget::ToggleSelectedBoardTextMirrored => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Mirrored)
            }
            HitTarget::ToggleSelectedBoardTextKeepUpright => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::KeepUpright)
            }
            HitTarget::ToggleSelectedBoardTextBold => {
                self.toggle_selected_board_text_boolean(BoardTextBooleanField::Bold)
            }
            HitTarget::CycleSelectedBoardTextRenderIntent => {
                self.cycle_selected_board_text_field(BoardTextCycleField::RenderIntent)
            }
            HitTarget::CycleSelectedBoardTextFamily => {
                self.cycle_selected_board_text_field(BoardTextCycleField::Family)
            }
            HitTarget::CycleSelectedBoardTextHAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Horizontal)
            }
            HitTarget::CycleSelectedBoardTextVAlign => {
                self.cycle_selected_board_text_alignment(BoardTextAlignmentField::Vertical)
            }
            HitTarget::DecreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextHeight => {
                self.step_selected_board_text_height(BoardTextHeightStep::Increase)
            }
            HitTarget::RotateSelectedBoardTextCounterClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::CounterClockwise90)
            }
            HitTarget::RotateSelectedBoardTextClockwise90 => {
                self.step_selected_board_text_rotation(BoardTextRotationStep::Clockwise90)
            }
            HitTarget::DecreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Decrease)
            }
            HitTarget::IncreaseSelectedBoardTextLineSpacing => {
                self.step_selected_board_text_line_spacing(BoardTextLineSpacingStep::Increase)
            }
            HitTarget::EditSelectedBoardTextContent => {
                self.begin_selected_board_text_content_edit()
            }
            HitTarget::EditSelectedBoardTextHeight => self.begin_selected_board_text_height_edit(),
            HitTarget::EditSelectedBoardTextRotation => {
                self.begin_selected_board_text_rotation_edit()
            }
            HitTarget::EditSelectedBoardTextLineSpacing => {
                self.begin_selected_board_text_line_spacing_edit()
            }
            HitTarget::EditSelectedBoardTextRenderIntent => {
                self.begin_selected_board_text_render_intent_edit()
            }
            HitTarget::EditSelectedBoardTextFamily => self.begin_selected_board_text_family_edit(),
            HitTarget::EditSelectedBoardTextAlignment => {
                self.begin_selected_board_text_alignment_edit()
            }
            HitTarget::TerminalTab => self.set_active_dock(DockTab::Terminal),
            HitTarget::TerminalSessionTab(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalSessionNew => self.spawn_terminal_session_tab(),
            HitTarget::TerminalSessionRenameActive => self.rename_active_terminal_session(),
            HitTarget::TerminalSessionRestartActive => {
                self.restart_terminal_session();
                true
            }
            HitTarget::TerminalSessionDetachActive => self.detach_active_terminal_session(),
            HitTarget::TerminalSessionCloseActive => self.close_active_terminal_session(),
            HitTarget::TerminalActivitySummary(summary) => {
                self.set_active_dock(DockTab::Terminal);
                self.push_terminal_line(format!("selected terminal activity span: {summary}"));
                self.log_review_event("selected terminal activity span".to_string());
                true
            }
            HitTarget::ProductionArtifact(artifact_id) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifact(artifact_id.clone()),
                );
                if handled {
                    self.log_review_event(format!("focused production artifact {artifact_id}"));
                }
                handled
            }
            HitTarget::ProductionArtifactFile(path) => {
                let handled = self.dispatch_session_command(
                    SessionCommand::FocusProductionArtifactFile(path.clone()),
                );
                if handled {
                    self.log_review_event(format!("focused production artifact file {path}"));
                }
                handled
            }
            HitTarget::ProductionOutputJobRun(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_output_job_run",
                    &handoff,
                )
                .unwrap_or_else(|err| {
                    self.push_terminal_line(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                self.write_terminal_bytes(&bytes);
                self.log_review_event(format!("ran production output command {}", handoff.command));
                true
            }
            HitTarget::ProductionTerminalCommand(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_terminal_command",
                    &handoff,
                )
                .unwrap_or_else(|err| {
                    self.push_terminal_line(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                self.write_terminal_bytes(&bytes);
                self.log_review_event(format!(
                    "ran production terminal command {}",
                    handoff.command
                ));
                true
            }
            HitTarget::ArtifactPreviewZoomIn
            | HitTarget::ArtifactPreviewZoomOut
            | HitTarget::ArtifactPreviewReset
            | HitTarget::ToggleArtifactPreviewGeometry
            | HitTarget::ToggleArtifactPreviewDrills => self
                .select_artifact_preview_hit_target(target)
                .unwrap_or(false),
            HitTarget::ArtifactPreviewViewport => false,
            HitTarget::MenuTitle(menu) => self.toggle_menu(menu),
            HitTarget::MenuItem { menu, label } => self.activate_menu_item(menu, label),
            HitTarget::MarkingMenuItem { .. } => self.dismiss_marking_menu(),
            HitTarget::DockResizeHandle => false, // handled in mouse press
        }
    }

    fn marking_menu_active(&self) -> bool {
        self.workspace().ui.marking_menu.is_some()
    }

    fn open_marking_menu_at_cursor(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        if self.cursor_in_dock() {
            return false;
        }
        let world_point = {
            let prepared = self.prepared_scene();
            prepared.world_point_at_screen(x, y)
        };
        let Some(world_point) = world_point else {
            return false;
        };
        let retained_target = {
            let retained = self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            retained
                .hit_test_authored_world(world_point, self.session.workspace())
                .cloned()
        };
        let target_object_id = match retained_target {
            Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => Some(id),
            _ => None,
        };
        let menu_key = marking_menu_key_for_target(target_object_id.as_deref());
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.marking_menu = Some(MarkingMenuState {
            menu_key,
            target_object_id,
            anchor_x_px: x.round() as i32,
            anchor_y_px: y.round() as i32,
            preview_slot: None,
            gesture_dx_px: 0,
            gesture_dy_px: 0,
        });
        self.invalidate_frame();
        true
    }

    fn update_marking_menu_preview(&mut self, pos: (f32, f32)) -> bool {
        let Some(menu) = self.session.workspace_mut().ui.marking_menu.as_mut() else {
            return false;
        };
        let dx = (pos.0 - menu.anchor_x_px as f32).round() as i32;
        let dy = (pos.1 - menu.anchor_y_px as f32).round() as i32;
        let next_slot = marking_slot_for_delta(dx, dy);
        if menu.gesture_dx_px == dx && menu.gesture_dy_px == dy && menu.preview_slot == next_slot {
            return false;
        }
        menu.gesture_dx_px = dx;
        menu.gesture_dy_px = dy;
        menu.preview_slot = next_slot;
        self.invalidate_frame();
        true
    }

    fn dismiss_marking_menu(&mut self) -> bool {
        if self.session.workspace().ui.marking_menu.is_none() {
            return false;
        }
        self.session.workspace_mut().ui.marking_menu = None;
        self.invalidate_frame();
        true
    }

    fn toggle_menu(&mut self, menu: &str) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = if ui.active_menu.as_deref() == Some(menu) {
            None
        } else {
            Some(menu.to_string())
        };
        self.invalidate_frame();
        true
    }

    fn activate_menu_item(&mut self, menu_name: &str, label: &str) -> bool {
        let item = datum_gui_protocol::load_default_gui_menu_model()
            .ok()
            .and_then(|model| {
                model
                    .menubar
                    .into_iter()
                    .find(|menu| menu.menu == menu_name)
                    .and_then(|menu| menu.items.into_iter().find(|item| item.label == label))
            });
        self.session.workspace_mut().ui.active_menu = None;
        let Some(item) = item else {
            self.push_terminal_line(format!("menu item {menu_name}/{label} unavailable"));
            self.invalidate_frame();
            return true;
        };
        if let Some(action) = item.gui_local.as_deref() {
            return self.activate_gui_local_menu_action(action);
        }
        let reason = item
            .not_built
            .as_deref()
            .or(item.verb.as_deref())
            .or(item.submenu.as_deref())
            .unwrap_or("disabled in Phase 1");
        self.push_terminal_line(format!("{menu_name}/{label} disabled: {reason}"));
        self.invalidate_frame();
        true
    }

    fn activate_gui_local_menu_action(&mut self, action: &str) -> bool {
        match action {
            "view.fit" => {
                self.fit_camera();
                self.log_review_event("menu view.fit".to_string());
                true
            }
            "view.zoom_in" => {
                self.zoom_view_from_menu(1.2);
                self.log_review_event("menu view.zoom_in".to_string());
                true
            }
            "view.zoom_out" => {
                self.zoom_view_from_menu(1.0 / 1.2);
                self.log_review_event("menu view.zoom_out".to_string());
                true
            }
            "terminal.toggle" => {
                if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
                    self.close_active_dock()
                } else {
                    self.set_active_dock(DockTab::Terminal)
                }
            }
            // Workspace pane ops (decision 021). These reach the same warm pane-op
            // path the FEEL breakpoint proves is zero-re-resolve. The menu manifest
            // does not emit these ids yet (that is the later bindings pass); wiring
            // them here keeps the ops reachable through the one action dispatch.
            "view.split_vertical" => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Vertical);
                self.log_review_event("menu view.split_vertical".to_string());
                true
            }
            "view.split_horizontal" => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
                self.log_review_event("menu view.split_horizontal".to_string());
                true
            }
            "view.close_pane" => {
                self.pane_close_focused();
                self.log_review_event("menu view.close_pane".to_string());
                true
            }
            "view.focus_next" => {
                self.pane_focus_next();
                self.log_review_event("menu view.focus_next".to_string());
                true
            }
            "view.focus_prev" => {
                self.pane_focus_prev();
                self.log_review_event("menu view.focus_prev".to_string());
                true
            }
            "view.maximize_pane" => {
                self.pane_toggle_zoom();
                self.log_review_event("menu view.maximize_pane".to_string());
                true
            }
            "view.preset_single" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::Single);
                self.log_review_event("menu view.preset_single".to_string());
                true
            }
            "view.preset_board_schematic" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::BoardSchematic);
                self.log_review_event("menu view.preset_board_schematic".to_string());
                true
            }
            "view.fill_board" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Board);
                self.log_review_event("menu view.fill_board".to_string());
                true
            }
            "view.fill_schematic" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Schematic);
                self.log_review_event("menu view.fill_schematic".to_string());
                true
            }
            _ => {
                self.push_terminal_line(format!("menu action {action} is view-local but unwired"));
                self.invalidate_frame();
                true
            }
        }
    }

    fn zoom_view_from_menu(&mut self, zoom_delta: f32) {
        let prepared = self.prepared_scene();
        let scene_viewport = prepared.scene_viewport;
        let bounds = self.workspace().scene.bounds.clone();
        self.camera.zoom_about_screen_point(
            scene_viewport,
            &bounds,
            scene_viewport.x + scene_viewport.width * 0.5,
            scene_viewport.y + scene_viewport.height * 0.5,
            zoom_delta,
        );
        self.invalidate_scene();
    }

    fn trace_timing(&self, message: String) {
        if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
            eprintln!("[datum-timing] {message}");
        }
    }

    fn selected_board_text(&self) -> Option<&datum_gui_protocol::BoardTextPrimitive> {
        let datum_gui_protocol::SelectionTarget::AuthoredObject(object_id) =
            &self.workspace().selection
        else {
            return None;
        };
        self.workspace()
            .scene
            .board_texts
            .iter()
            .find(|text| &text.object_id == object_id)
    }

    fn begin_selected_board_text_content_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Content)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text content")
    }

    fn begin_selected_board_text_height_edit(&mut self) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_edit_terminal_command(text, BoardTextEditTerminalField::Height))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text height")
    }

    fn begin_selected_board_text_rotation_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Rotation)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text rotation")
    }

    fn begin_selected_board_text_line_spacing_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::LineSpacing)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text line spacing",
        )
    }

    fn begin_selected_board_text_render_intent_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::RenderIntent)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text render intent",
        )
    }

    fn begin_selected_board_text_family_edit(&mut self) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_edit_terminal_command(text, BoardTextEditTerminalField::Family))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, "editing selected board text font")
    }

    fn begin_selected_board_text_alignment_edit(&mut self) -> bool {
        let Some(command) = self.selected_board_text().map(|text| {
            board_text_edit_terminal_command(text, BoardTextEditTerminalField::Alignment)
        }) else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(
            command,
            "editing selected board text alignment",
        )
    }

    fn begin_selected_board_text_command_edit(
        &mut self,
        command: String,
        event: impl Into<String>,
    ) -> bool {
        self.set_active_dock(DockTab::Terminal);
        if let Err(err) = record_manual_terminal_command_handoff(
            self.terminal_sessions.active(),
            "board_text_terminal_command",
            "datum.gui.board_text.edit_prefill",
            "prefill",
            &command,
        ) {
            self.push_terminal_line(format!("terminal handoff event write failed: {err}"));
        }
        self.write_terminal_bytes(command.as_bytes());
        self.invalidate_frame();
        self.log_review_event(event.into());
        true
    }

    fn toggle_selected_board_text_boolean(&mut self, field: BoardTextBooleanField) -> bool {
        let field_label = match field {
            BoardTextBooleanField::Mirrored => "mirrored",
            BoardTextBooleanField::KeepUpright => "keep-upright",
            BoardTextBooleanField::Bold => "bold",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::ToggleBoolean(field),
            format!("editing selected board text {field_label}"),
        )
    }

    fn cycle_selected_board_text_field(&mut self, field: BoardTextCycleField) -> bool {
        let field_label = match field {
            BoardTextCycleField::RenderIntent => "render intent",
            BoardTextCycleField::Family => "font family",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::CycleField(field),
            format!("editing selected board text {field_label}"),
        )
    }

    fn cycle_selected_board_text_alignment(&mut self, field: BoardTextAlignmentField) -> bool {
        let field_label = match field {
            BoardTextAlignmentField::Horizontal => "horizontal align",
            BoardTextAlignmentField::Vertical => "vertical align",
        };
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::CycleAlignment(field),
            format!("editing selected board text {field_label}"),
        )
    }

    fn step_selected_board_text_line_spacing(&mut self, step: BoardTextLineSpacingStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepLineSpacing(step),
            "editing selected board text line spacing".to_string(),
        )
    }

    fn step_selected_board_text_height(&mut self, step: BoardTextHeightStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepHeight(step),
            "editing selected board text height".to_string(),
        )
    }

    fn step_selected_board_text_rotation(&mut self, step: BoardTextRotationStep) -> bool {
        self.begin_selected_board_text_quick_edit(
            BoardTextQuickEditTerminalAction::StepRotation(step),
            "editing selected board text rotation".to_string(),
        )
    }

    fn begin_selected_board_text_quick_edit(
        &mut self,
        action: BoardTextQuickEditTerminalAction,
        event: String,
    ) -> bool {
        let Some(command) = self
            .selected_board_text()
            .map(|text| board_text_quick_edit_terminal_command(text, action))
        else {
            self.log_review_event("no board text selected".to_string());
            return false;
        };
        self.begin_selected_board_text_command_edit(command, event)
    }

    fn fit_camera(&mut self) {
        self.camera = CameraState::fit_to_bounds(&self.workspace().scene.bounds);
        self.invalidate_frame();
    }

    fn fit_review_target(&mut self) -> bool {
        let Some(bounds) = self.active_review_bounds() else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    fn fit_scene_object(&mut self, object_id: &str) -> bool {
        let Some(bounds) = self.scene_object_bounds(object_id) else {
            return false;
        };
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.invalidate_frame();
        true
    }

    fn scene_object_bounds(&self, object_id: &str) -> Option<SceneBounds> {
        let scene = &self.workspace().scene;
        if let Some(component) = scene
            .components
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return Some(padded_rect_bounds(component.bounds, 1_500_000));
        }
        if let Some(pad) = scene.pads.iter().find(|item| item.object_id == object_id) {
            return Some(padded_rect_bounds(pad.bounds, 500_000));
        }
        if let Some(track) = scene.tracks.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(track.path.iter().copied(), 750_000);
        }
        if let Some(via) = scene.vias.iter().find(|item| item.object_id == object_id) {
            let radius = (via.diameter_nm / 2).max(250_000);
            return bounds_from_points([via.position].into_iter(), radius + 500_000);
        }
        if let Some(zone) = scene.zones.iter().find(|item| item.object_id == object_id) {
            return bounds_from_points(zone.polygon.iter().copied(), 750_000);
        }
        if let Some(text) = scene
            .board_texts
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points([text.position].into_iter(), text.height_nm.max(500_000));
        }
        if let Some(graphic) = scene
            .board_graphics
            .iter()
            .find(|item| item.object_id == object_id)
        {
            return bounds_from_points(graphic.path.iter().copied(), 750_000);
        }
        scene
            .outline
            .iter()
            .find(|item| item.object_id == object_id)
            .and_then(|outline| bounds_from_points(outline.path.iter().copied(), 750_000))
    }

    fn active_review_bounds(&self) -> Option<SceneBounds> {
        let action_id = &self.workspace().active_review_target_id;
        let mut points = Vec::<PointNm>::new();

        for overlay in self
            .workspace()
            .scene
            .proposal_overlay_primitives
            .iter()
            .filter(|overlay| &overlay.proposal_action_id == action_id)
        {
            points.extend(overlay.path.iter().copied());
        }

        if let Some(evidence_key) = self
            .workspace()
            .selected_review_action()
            .map(|action| format!("segment:{}", action.selected_path_segment_index))
        {
            for review in self
                .workspace()
                .scene
                .review_primitives
                .iter()
                .filter(|review| review.evidence_key.as_deref() == Some(evidence_key.as_str()))
            {
                points.extend(review.path.iter().copied());
            }
        }

        let action = self.workspace().selected_review_action()?;
        for pad in self.workspace().scene.pads.iter().filter(|pad| {
            pad.pad_uuid == action.from_anchor_pad_uuid || pad.pad_uuid == action.to_anchor_pad_uuid
        }) {
            points.push(pad.center);
        }

        if points.is_empty() {
            return None;
        }

        let (min_x, max_x) = points
            .iter()
            .map(|point| point.x)
            .fold((i64::MAX, i64::MIN), |(min_x, max_x), x| {
                (min_x.min(x), max_x.max(x))
            });
        let (min_y, max_y) = points
            .iter()
            .map(|point| point.y)
            .fold((i64::MAX, i64::MIN), |(min_y, max_y), y| {
                (min_y.min(y), max_y.max(y))
            });
        let padding_nm = 1_500_000_i64;
        Some(SceneBounds {
            min_x: min_x.saturating_sub(padding_nm),
            min_y: min_y.saturating_sub(padding_nm),
            max_x: max_x.saturating_add(padding_nm),
            max_y: max_y.saturating_add(padding_nm),
        })
    }

    fn handle_pan_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let Some(previous) = self.last_cursor_pos else {
            return false;
        };
        let prepared = self.prepared_scene().clone();
        if self.handle_artifact_preview_pan_drag(&prepared, previous, next_cursor_pos) {
            return true;
        }
        let scene_viewport = self.scene_viewport();
        let bounds = self.workspace().scene.bounds.clone();
        self.camera.pan_pixels(
            scene_viewport,
            &bounds,
            next_cursor_pos.0 - previous.0,
            next_cursor_pos.1 - previous.1,
        );
        self.invalidate_frame();
        true
    }

    fn handle_zoom(&mut self, delta: f32) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        let scene_viewport = self.scene_viewport();
        if !scene_viewport.contains(x, y) {
            return false;
        }
        let bounds = self.workspace().scene.bounds.clone();
        self.camera
            .zoom_about_screen_point(scene_viewport, &bounds, x, y, delta);
        self.invalidate_frame();
        true
    }

    // ---------------------------------------------------------------------
    // Workspace pane ops (decision 021, P2.1b). Every one is a PURE view-state
    // mutation of the pane tree: it swaps the active camera to the target leaf's
    // warm camera (never a refit) and calls `invalidate_frame`, which rebuilds
    // only the cheap prepared chrome/scene and KEEPS the resolved world scene —
    // so a pane op costs zero world-scene re-resolve (the "no noticeable lag"
    // gate). Only the focused leaf renders live (single-live-scene); non-focused
    // Board leaves render as today (idle real-content snapshot lands with P2.2
    // multi-scene). These are never journaled — they are workspace state.
    // ---------------------------------------------------------------------

    /// Shared focus-change core: stash the outgoing leaf's live camera, mutate
    /// focus via `f`, then activate the incoming leaf's warm camera (fit only if
    /// that leaf has never been focused).
    fn swap_pane_focus(&mut self, f: impl FnOnce(&mut datum_gui_protocol::WorkspaceLayout)) {
        let outgoing = self.workspace().ui.layout.focused;
        f(&mut self.session.workspace_mut().ui.layout);
        let incoming = self.workspace().ui.layout.focused;
        if incoming != outgoing {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(outgoing, self.camera, incoming, || {
                CameraState::fit_to_bounds(&bounds)
            });
        }
        self.invalidate_frame();
    }

    fn pane_focus_next(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_next());
    }

    fn pane_focus_prev(&mut self) {
        self.swap_pane_focus(|layout| layout.focus_prev());
    }

    fn pane_split_focused(&mut self, orientation: datum_gui_protocol::SplitOrientation) {
        // Focus is unchanged by a split; the fresh child inherits the focused
        // sibling's warm framing so it opens looking like the pane it split from.
        let before: std::collections::BTreeSet<_> =
            self.workspace().ui.layout.leaves().into_iter().collect();
        self.session
            .workspace_mut()
            .ui
            .layout
            .split_focused(orientation);
        let inherited = self.camera;
        for id in self.workspace().ui.layout.leaves() {
            if !before.contains(&id) {
                self.pane_cameras.inherit(id, inherited);
            }
        }
        self.invalidate_frame();
    }

    fn pane_close_focused(&mut self) {
        let outgoing = self.workspace().ui.layout.focused;
        self.session.workspace_mut().ui.layout.close_focused();
        let incoming = self.workspace().ui.layout.focused;
        let live = self.workspace().ui.layout.leaves();
        self.pane_cameras.retain_live(&live);
        if incoming != outgoing {
            let bounds = self.workspace().scene.bounds.clone();
            self.camera = self.pane_cameras.focus_to(outgoing, self.camera, incoming, || {
                CameraState::fit_to_bounds(&bounds)
            });
        }
        self.invalidate_frame();
    }

    fn pane_toggle_zoom(&mut self) {
        // Transient maximize of the focused leaf; focus and cameras are untouched.
        self.session.workspace_mut().ui.layout.toggle_zoom();
        self.invalidate_frame();
    }

    fn pane_apply_preset(&mut self, preset: datum_gui_protocol::WorkspacePreset) {
        self.session.workspace_mut().ui.layout.apply_preset(preset);
        // A preset rebuilds the tree with fresh ids; reset the warm store to the
        // new focused leaf and fit it (a preset is a deliberate layout reset).
        let focused = self.workspace().ui.layout.focused;
        let bounds = self.workspace().scene.bounds.clone();
        self.camera = CameraState::fit_to_bounds(&bounds);
        self.pane_cameras.reset(focused, self.camera);
        self.invalidate_frame();
    }

    fn pane_set_focused_content(&mut self, content: datum_gui_protocol::PaneContent) {
        self.session
            .workspace_mut()
            .ui
            .layout
            .set_focused_content(content);
        self.invalidate_frame();
    }

    fn current_layout(&self) -> ShellLayout {
        ShellLayout::for_surface(
            self.config.width,
            self.config.height,
            self.scale_factor,
            if self.workspace().ui.active_dock_tab.is_some() {
                Some(self.workspace().ui.dock_height_px)
            } else {
                None
            },
        )
    }

    fn scene_viewport(&self) -> datum_gui_render::RectPx {
        self.current_layout()
            .scene_viewport(&self.workspace().ui.layout)
    }

    /// The workspace leaf pane whose frame contains screen point `(x, y)`, tiling
    /// the current shell exactly as the renderer does. Backs click-to-focus
    /// (decision 021): a press outside every pane (sidebars/dock/menu) returns
    /// `None` and the click falls through to normal board behavior.
    fn pane_at_screen(&self, x: f32, y: f32) -> Option<datum_gui_protocol::PaneId> {
        self.current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .leaf_at(x, y)
    }

    fn update_hover(&mut self, pos: (f32, f32)) -> bool {
        let prepared = self.prepared_scene();
        let new_hover = match prepared.hit_test(pos.0, pos.1) {
            Some(HitTarget::AuthoredObject(id)) => Some(id.clone()),
            Some(HitTarget::ReviewAction(id)) => Some(id.clone()),
            _ => None,
        };
        let current = &self.session.workspace().ui.hovered_object_id;
        if &new_hover != current {
            self.session.workspace_mut().ui.hovered_object_id = new_hover;
            self.invalidate_scene();
            return true;
        }
        false
    }

    fn cursor_in_dock(&self) -> bool {
        let Some((_, y)) = self.last_cursor_pos else {
            return false;
        };
        let layout = self.current_layout();
        y >= layout.bottom_strip.y
    }

    fn handle_dock_resize_drag(&mut self, next_cursor_pos: (f32, f32)) -> bool {
        let window_height = self.config.height as f32;
        let new_height_physical =
            (window_height - next_cursor_pos.1).clamp(32.0, window_height * 0.6);
        let new_height_logical = new_height_physical / self.scale_factor.max(0.01);
        let new_height_logical = new_height_logical as u32;
        if self.workspace().ui.dock_height_px == new_height_logical {
            return false;
        }
        self.session.workspace_mut().ui.dock_height_px = new_height_logical;
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }

    fn resize_terminal_to_dock(&mut self) {
        let bottom_height = self.current_layout().bottom_strip.height;
        let cols = ((self.config.width as f32 - 24.0) / 7.5).floor().max(20.0) as u16;
        let rows = ((bottom_height - 76.0) / 16.0).floor().max(4.0) as u16;
        append_gui_verbose_diagnostic_line(format!("terminal resize begin {cols}x{rows}"));
        match self.terminal_sessions.resize_active(cols, rows) {
            Ok(()) => {
                let terminal = &mut self.session.workspace_mut().ui.terminal;
                terminal.columns = cols;
                terminal.rows = rows;
                append_gui_verbose_diagnostic_line("terminal resize end");
            }
            Err(err) => {
                append_gui_diagnostic_line(format!("terminal resize failed: {err}"));
                self.push_terminal_line(format!("terminal resize failed: {err}"));
            }
        }
    }
}

fn padded_rect_bounds(rect: RectNm, padding_nm: i64) -> SceneBounds {
    SceneBounds {
        min_x: rect.min_x.saturating_sub(padding_nm),
        min_y: rect.min_y.saturating_sub(padding_nm),
        max_x: rect.max_x.saturating_add(padding_nm),
        max_y: rect.max_y.saturating_add(padding_nm),
    }
}

fn bounds_from_points(
    points: impl IntoIterator<Item = PointNm>,
    padding_nm: i64,
) -> Option<SceneBounds> {
    let mut iter = points.into_iter();
    let first = iter.next()?;
    let mut min_x = first.x;
    let mut max_x = first.x;
    let mut min_y = first.y;
    let mut max_y = first.y;
    for point in iter {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    Some(SceneBounds {
        min_x: min_x.saturating_sub(padding_nm),
        min_y: min_y.saturating_sub(padding_nm),
        max_x: max_x.saturating_add(padding_nm),
        max_y: max_y.saturating_add(padding_nm),
    })
}

fn marking_menu_key_for_target(target_object_id: Option<&str>) -> String {
    let key = match target_object_id {
        Some(id) if id.starts_with("component:") => "pcb.component",
        Some(id) if id.starts_with("pad:") => "pcb.pad",
        Some(id) if id.starts_with("track:") => "pcb.track",
        Some(id) if id.starts_with("via:") => "pcb.via",
        Some(id) if id.starts_with("zone:") => "pcb.zone",
        Some(id) if id.starts_with("net:") => "pcb.net",
        _ => "pcb.empty",
    };
    key.to_string()
}

fn marking_slot_for_delta(dx: i32, dy: i32) -> Option<String> {
    let dx = dx as f32;
    let dy = dy as f32;
    if (dx * dx + dy * dy).sqrt() < 18.0 {
        return None;
    }
    let angle = dy.atan2(dx).to_degrees();
    let slot = if (-22.5..22.5).contains(&angle) {
        "E"
    } else if (22.5..67.5).contains(&angle) {
        "SE"
    } else if (67.5..112.5).contains(&angle) {
        "S"
    } else if (112.5..157.5).contains(&angle) {
        "SW"
    } else if angle >= 157.5 || angle < -157.5 {
        "W"
    } else if (-157.5..-112.5).contains(&angle) {
        "NW"
    } else if (-112.5..-67.5).contains(&angle) {
        "N"
    } else {
        "NE"
    };
    Some(slot.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_bootstrap::parse_window_size;

    #[test]
    fn parses_visual_window_size() {
        assert_eq!(parse_window_size("1280x768").unwrap(), (1280, 768));
    }

    #[test]
    fn bounds_from_points_applies_padding() {
        let bounds = bounds_from_points([PointNm { x: 10, y: 20 }, PointNm { x: 30, y: -10 }], 5)
            .expect("bounds should exist");
        assert_eq!(
            bounds,
            SceneBounds {
                min_x: 5,
                min_y: -15,
                max_x: 35,
                max_y: 25
            }
        );
    }

    #[test]
    fn rejects_invalid_visual_window_size() {
        assert!(parse_window_size("1280").is_err());
        assert!(parse_window_size("0x768").is_err());
        assert!(parse_window_size("1280x0").is_err());
    }

    #[test]
    fn terminal_raw_input_defers_paste_and_copy_shortcuts() {
        assert!(terminal_raw_input_should_handle(true, false, false));
        assert!(!terminal_raw_input_should_handle(true, true, false));
        assert!(!terminal_raw_input_should_handle(true, false, true));
        assert!(!terminal_raw_input_should_handle(false, false, false));
    }

    #[test]
    fn marking_menu_key_maps_phase_one_board_objects() {
        assert_eq!(
            marking_menu_key_for_target(Some("component:U1")),
            "pcb.component"
        );
        assert_eq!(marking_menu_key_for_target(Some("pad:P1")), "pcb.pad");
        assert_eq!(marking_menu_key_for_target(Some("track:T1")), "pcb.track");
        assert_eq!(marking_menu_key_for_target(Some("via:V1")), "pcb.via");
        assert_eq!(marking_menu_key_for_target(Some("zone:Z1")), "pcb.zone");
        assert_eq!(marking_menu_key_for_target(None), "pcb.empty");
    }

    #[test]
    fn marking_slot_for_delta_uses_screen_direction_wheel() {
        assert_eq!(marking_slot_for_delta(0, -40).as_deref(), Some("N"));
        assert_eq!(marking_slot_for_delta(40, 0).as_deref(), Some("E"));
        assert_eq!(marking_slot_for_delta(0, 40).as_deref(), Some("S"));
        assert_eq!(marking_slot_for_delta(-40, 0).as_deref(), Some("W"));
        assert_eq!(marking_slot_for_delta(30, -30).as_deref(), Some("NE"));
        assert_eq!(marking_slot_for_delta(3, 3), None);
    }

    #[test]
    fn terminal_paste_bytes_wraps_when_bracketed_paste_is_enabled() {
        assert_eq!(terminal_paste_bytes("alpha\nbeta", false), b"alpha\nbeta");
        assert_eq!(
            terminal_paste_bytes("alpha\nbeta", true),
            b"\x1b[200~alpha\nbeta\x1b[201~"
        );
    }

    #[test]
    fn assistant_activity_command_is_session_scoped() {
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("context session-activity"));
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("$DATUM_SESSION_ID"));
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("--limit 20"));
        assert_eq!(
            ASSISTANT_ACTIVITY_COMMAND,
            "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20"
        );
    }

    #[cfg(feature = "visual")]
    #[test]
    fn converts_bgra_readback_to_rgba() {
        let mut pixels = vec![1, 2, 3, 255, 10, 20, 30, 255];
        convert_texture_pixels_to_rgba(&mut pixels, wgpu::TextureFormat::Bgra8UnormSrgb).unwrap();
        assert_eq!(pixels, vec![3, 2, 1, 255, 30, 20, 10, 255]);
    }
}

fn char_to_byte_pos(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn terminal_paste_bytes(text: &str, bracketed_paste: bool) -> Vec<u8> {
    if !bracketed_paste {
        return text.as_bytes().to_vec();
    }
    let mut bytes = Vec::with_capacity(text.len() + 12);
    bytes.extend_from_slice(b"\x1b[200~");
    bytes.extend_from_slice(text.as_bytes());
    bytes.extend_from_slice(b"\x1b[201~");
    bytes
}
