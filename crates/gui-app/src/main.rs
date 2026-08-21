use anyhow::{Context, Result};
use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use clap::Parser;
use datum_gui_protocol::{
    BoardTextAlignmentField, BoardTextBooleanField, BoardTextCycleField, BoardTextHeightStep,
    BoardTextLineSpacingStep, BoardTextRotationStep, DockTab, HoverTarget, LiveDesignSession,
    LiveReviewRequest, MarkingMenuState, PaneContent, PointNm, RectNm, SceneBounds, SessionCommand,
    SessionEvent, TerminalCommandHandoff, WorkspaceTool, ensure_known_good_demo_request,
    load_board_editor_workspace_state, load_kicad_schematic_workspace_state,
    load_live_workspace_state, materialize_kicad_board_request,
};
#[cfg(feature = "visual")]
use datum_gui_render::visual_capture::OffscreenRenderer;
use datum_gui_render::{
    CameraState, HitTarget, PreparedScene, Renderer, RetainedScene, SceneSurface, ShellLayout,
    TerminalRenderCache,
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
mod application_terminal_shutdown;
mod artifact_preview_controls;
mod board_text_terminal_commands;
mod gui_runtime_support;
mod interaction_refresh;
mod keyboard_focus;
mod pan_gesture;
mod pane_cameras;
mod pane_grid_lod;
mod pane_resize;
mod production_status_refresh;
mod retained_scene_cache_key;
mod runtime_board_text_edit;
mod runtime_camera_fit_targets;
mod runtime_camera_pane;
mod runtime_terminal_clipboard;
mod runtime_terminal_context;
mod runtime_terminal_dock;
mod runtime_terminal_input;
mod runtime_terminal_links;
mod runtime_terminal_pointer;
mod runtime_terminal_render;
mod runtime_terminal_search;
mod runtime_view_actions;
mod terminal_accessibility;
mod terminal_accessibility_bridge;
mod terminal_accessibility_platform;
mod terminal_active_context;
mod terminal_activity_snapshot;
mod terminal_capability;
mod terminal_check_context;
mod terminal_context;
mod terminal_context_contract;
mod terminal_context_io;
#[cfg(test)]
mod terminal_control_input;
mod terminal_core_adapter;
mod terminal_input;
mod terminal_narration;
mod terminal_process;
mod terminal_proposal_context;
mod terminal_session;
mod terminal_session_context;
mod terminal_session_controls;
mod terminal_session_events;
mod terminal_tab_drag;
mod terminal_transport;
mod workspace_keyboard;
use app_bootstrap::{GuiArgs, LaunchState};
use app_shell::{App, fatal_gui_error, terminal_scrollback_page_step};
use board_text_terminal_commands::{
    BoardTextEditTerminalField, BoardTextQuickEditTerminalAction, board_text_edit_terminal_command,
    board_text_quick_edit_terminal_command,
};
use datum_gui_protocol::ApplicationFocus;
pub(crate) use gui_runtime_support::*;
use pan_gesture::PanGestureState;
use pane_cameras::PaneCameras;
use pane_resize::DividerDrag;
use retained_scene_cache_key::retained_selection_cache_key;
#[cfg(feature = "visual")]
use std::fs;
use terminal_input::{TerminalKeyAction, terminal_key_action};
use terminal_session::{
    TerminalLaunchContext, TerminalSessionRegistry, terminal_launch_context_from_state,
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
    let mut app = App::new(args, event_loop.create_proxy());
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
    // Capture/test affordance: focus a named pane, mirroring the windowed path.
    args.apply_focus_pane(&mut state.ui.layout);
    // Capture/test affordance: open a named menu dropdown if --open-menu was set,
    // mirroring the windowed launch path; a no-op otherwise so parity is untouched.
    if let Some(menu) = &args.open_menu {
        state.ui.active_menu = Some(menu.clone());
    }
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
        if let Some(window) = self.window
            && window.id() != window_id
        {
            return;
        }
        if let Some(label) = window_event_diagnostic_label(&event) {
            append_gui_verbose_diagnostic_line(format!("window event {label}"));
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
                    if !focused {
                        runtime.pan_gesture.cancel();
                        runtime.cancel_terminal_tab_drag();
                        runtime.cancel_terminal_text_selection_drag();
                    }
                    if !focused && runtime.clear_interaction_overlay() {
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.last_cursor_pos = None;
                    runtime.pan_gesture.cancel();
                    runtime.cancel_terminal_tab_drag();
                    runtime.cancel_terminal_text_selection_drag();
                    let terminal_hover_cleared = runtime.clear_terminal_tab_hover();
                    if runtime.clear_interaction_overlay() || terminal_hover_cleared {
                        self.request_redraw_if_needed();
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
                            self.request_redraw_if_needed();
                        }
                        self.apply_cursor_icon(winit::window::CursorIcon::Grabbing);
                        return;
                    }
                    if runtime.terminal_clipboard_menu_active() {
                        return;
                    }
                    if runtime.advance_terminal_text_selection(next_pos) {
                        self.apply_cursor_icon(winit::window::CursorIcon::Text);
                        self.request_redraw_if_needed();
                        return;
                    }
                    if runtime.report_terminal_mouse_motion() {
                        runtime.clear_interaction_overlay();
                        self.request_redraw_if_needed();
                        return;
                    }
                    let mut changed = terminal_hover_changed;
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
                        self.request_redraw_if_needed();
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
                button: button @ (MouseButton::Middle | MouseButton::Right),
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if button == MouseButton::Right
                        && state == ElementState::Pressed
                        && runtime.open_terminal_clipboard_menu_at_cursor()
                    {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if button == MouseButton::Right && runtime.terminal_clipboard_menu_active() {
                        return;
                    }
                    if runtime.report_terminal_mouse_button(button, state) {
                        return;
                    }
                    if button == MouseButton::Right && runtime.handle_context_menu_button(state) {
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    if runtime.terminal_clipboard_menu_active() {
                        return;
                    }
                    if runtime.modifiers.control_key() && runtime.arm_terminal_link_at_cursor() {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if runtime.begin_terminal_tab_drag() {
                        self.apply_cursor_icon(winit::window::CursorIcon::Grabbing);
                        return;
                    }
                    runtime.focus_terminal_screen_before_mouse_report();
                    if runtime.begin_terminal_text_selection() {
                        self.apply_cursor_icon(winit::window::CursorIcon::Text);
                        self.request_redraw_if_needed();
                        return;
                    }
                    if runtime
                        .report_terminal_mouse_button(MouseButton::Left, ElementState::Pressed)
                    {
                        return;
                    }
                    if runtime.begin_primary_pan() {
                        if runtime.clear_interaction_overlay() {
                            self.request_redraw_if_needed();
                        }
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
                        // Grab a split divider gutter to resize the split (decision
                        // 021), handled before click-to-focus so grabbing the gutter
                        // resizes instead of focusing a pane.
                        if runtime.begin_divider_drag(x, y) {
                            self.request_redraw_if_needed();
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
                    if let Some(icon) = runtime.finish_dock_resize_drag() {
                        self.apply_cursor_icon(icon);
                        self.request_redraw_if_needed();
                        return;
                    }
                    if runtime.finish_terminal_tab_drag() {
                        let icon = runtime
                            .last_cursor_pos
                            .and_then(|pointer| runtime.terminal_tab_cursor_icon(pointer))
                            .unwrap_or(winit::window::CursorIcon::Default);
                        self.apply_cursor_icon(icon);
                        self.request_redraw_if_needed();
                        return;
                    }
                    // A completed divider-drag resize ends here; the release must NOT
                    // fall through to click-to-focus / selection.
                    let was_divider_drag = runtime.divider_drag.take().is_some();
                    if runtime.finish_terminal_text_selection() {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if !runtime.terminal_clipboard_menu_active()
                        && runtime
                            .report_terminal_mouse_button(MouseButton::Left, ElementState::Released)
                    {
                        return;
                    }
                    if runtime.finish_primary_pan() {
                        self.request_redraw_if_needed();
                        return;
                    }
                    if was_divider_drag {
                        self.request_redraw_if_needed();
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
                    self.request_redraw_if_needed();
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
                    self.request_redraw_if_needed();
                    return;
                }
                keyboard_focus::handle_keyboard_input(self, &event);
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

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        if self
            .runtime
            .as_mut()
            .is_some_and(Runtime::handle_terminal_output_wake)
        {
            self.request_redraw_if_needed();
        }
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
    /// Camera for the renderer's live board leaf. Pointer and focused commands
    /// reach it only when their typed pane route names that leaf; schematic and
    /// additional-pane cameras remain independently warm in `pane_cameras`.
    camera: CameraState,
    /// Warm per-leaf view cameras keyed by `PaneId` (decision 021, P2.1b).
    pane_cameras: PaneCameras,
    pane_grid_lod: pane_grid_lod::PaneGridLod,
    last_cursor_pos: Option<(f32, f32)>,
    pan_gesture: PanGestureState,
    dock_drag_active: bool,
    terminal_tab_drag: Option<terminal_tab_drag::TerminalTabDrag>,
    terminal_tab_drag_release_suppressed: bool,
    terminal_text_selection_drag: Option<runtime_terminal_pointer::TerminalSelectionPoint>,
    /// In-progress split divider-drag resize (decision 021), or `None`. Consumer
    /// view state; never journaled.
    divider_drag: Option<DividerDrag>,
    terminal_mouse_button: Option<MouseButton>,
    modifiers: ModifiersState,
    redraw_pending: bool,
    retained_scene: Option<RetainedScene>,
    retained_scene_cache: Vec<(RetainedSceneCacheKey, RetainedScene)>,
    prepared_scene: Option<PreparedScene>,
    terminal_render_cache: TerminalRenderCache,
    terminal_accessibility: terminal_accessibility_bridge::LinuxTerminalAccessibilityBridge,
    // P2.2a: the static companion schematic world buffer, rendered as the additive
    // second GPU pass into the Schematic pane. Rebuilt lazily whenever it is None;
    // cleared in lockstep with `prepared_scene` on every scene/frame invalidation,
    // so it always reflects the current workspace's `schematic_scene`. `None` when
    // the workspace carries no companion schematic or the layout has no Schematic
    // pane (second pass simply stays off).
    schematic_retained_scene: Option<RetainedScene>,
    scene_dirty: bool,
    terminal_sessions: TerminalSessionRegistry,
    terminal_launch_context: TerminalLaunchContext,
    workspace_include_review: bool,
    terminal_production_refresh_pending: bool,
    terminal_workspace_refresh_pending: bool,
    terminal_production_refresh_due: Option<std::time::Instant>,
    terminal_production_refresh_attempts: u8,
    clipboard: Option<Clipboard>,
    application_shutdown_started: Option<std::time::Instant>,
    application_shutdown_blocked: bool,
}

impl Runtime {
    async fn new(
        window: &'static Window,
        launch_state: LaunchState,
        scale_factor_override: Option<f32>,
    ) -> Result<Self> {
        let runtime_started = std::time::Instant::now();
        let LaunchState {
            request: _request,
            mut state,
            camera,
            terminal_launch_context,
            terminal_sessions,
            workspace_include_review,
        } = launch_state;
        // The initially-focused leaf seeds the warm per-leaf camera store; its
        // camera is the fit camera the launch path already computed.
        let initial_focus = state.ui.layout.focused;
        state.ui.focus = ApplicationFocus::Editor(initial_focus);
        let initial_content = state.ui.layout.focused_content();
        let initial_pane_camera = match initial_content {
            PaneContent::Board => camera,
            PaneContent::Schematic => state
                .schematic_scene
                .as_ref()
                .map(|scene| CameraState::fit_to_bounds(&scene.bounds))
                .unwrap_or(camera),
        };
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
            pane_cameras: PaneCameras::new(initial_focus, initial_content, initial_pane_camera),
            pane_grid_lod: pane_grid_lod::PaneGridLod::default(),
            last_cursor_pos: None,
            pan_gesture: PanGestureState::default(),
            dock_drag_active: false,
            terminal_tab_drag: None,
            terminal_tab_drag_release_suppressed: false,
            terminal_text_selection_drag: None,
            divider_drag: None,
            terminal_mouse_button: None,
            modifiers: ModifiersState::empty(),
            redraw_pending: false,
            retained_scene: None,
            retained_scene_cache: Vec::new(),
            prepared_scene: None,
            terminal_render_cache: TerminalRenderCache::new(),
            terminal_accessibility:
                terminal_accessibility_bridge::LinuxTerminalAccessibilityBridge::default(),
            schematic_retained_scene: None,
            scene_dirty: true,
            terminal_sessions,
            terminal_launch_context,
            workspace_include_review,
            terminal_production_refresh_pending: false,
            terminal_workspace_refresh_pending: false,
            terminal_production_refresh_due: None,
            terminal_production_refresh_attempts: 0,
            clipboard: Clipboard::new().ok(),
            application_shutdown_started: None,
            application_shutdown_blocked: false,
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
            let prepared_started = std::time::Instant::now();
            append_gui_verbose_diagnostic_line("prepared scene build begin");
            self.prepared_scene = Some(self.build_terminal_prepared_scene()?);
            prepared_build_ms = prepared_started.elapsed().as_millis();
            append_gui_verbose_diagnostic_line(format!(
                "prepared scene build end {prepared_build_ms}ms"
            ));
        }
        let scene_elapsed = scene_started.elapsed();
        // P2.2a: resolve the companion schematic world buffer lazily (cleared on
        // every scene/frame invalidation, so this stays fresh). `None` when the
        // workspace has no companion schematic / Schematic pane — second pass off.
        if self.schematic_retained_scene.is_none() {
            self.schematic_retained_scene = RetainedScene::from_workspace_schematic_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before render")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before render")?;
        let schematic_retained = self.schematic_retained_scene.as_ref();
        let renderer_started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("renderer render begin");
        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            prepared,
            retained,
            schematic_retained,
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
            self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            self.prepared_scene = Some(self.build_terminal_prepared_scene()?);
        }
        if self.schematic_retained_scene.is_none() {
            self.schematic_retained_scene = RetainedScene::from_workspace_schematic_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before visual screenshot")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before visual screenshot")?;
        let schematic_retained = self.schematic_retained_scene.as_ref();
        self.renderer.render(
            &self.device,
            &self.queue,
            &target_view,
            prepared,
            retained,
            schematic_retained,
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
        if self.prepared_scene.is_none() {
            self.scene_dirty = false;
            self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace_for_surface(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                    self.scale_factor,
                )
            });
            self.prepared_scene = Some(
                self.build_terminal_prepared_scene()
                    .expect("approved active TerminalCore snapshot fits production limits"),
            );
        }
        self.prepared_scene
            .as_ref()
            .expect("prepared scene initialized above")
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

    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001: there is
    // deliberately NO `push_terminal_line` here. Terminal cells are mutated
    // only by PTY bytes interpreted by the terminal core; Datum notices,
    // diagnostics, and lifecycle messages route through `log_review_event`
    // (console sink) or terminal chrome, never the grid.

    fn handle_terminal_key_input(&mut self, event: &KeyEvent) -> bool {
        if self.handle_terminal_link_confirmation_key(event) {
            return true;
        }
        if self.handle_terminal_search_key(event) {
            return true;
        }
        let application_cursor_keys = self.workspace().ui.terminal.application_cursor_keys;
        let application_keypad = self.workspace().ui.terminal.application_keypad;
        let action = terminal_key_action(
            event,
            self.modifiers,
            application_cursor_keys,
            application_keypad,
        );
        if let Some(handled) = self.handle_close_confirmation_action(&action) {
            return handled;
        }
        match action {
            TerminalKeyAction::CoreKey(input) => {
                match self.terminal_sessions.encode_active_key(&input) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => true,
                    Err(err) => {
                        self.log_review_event(format!("terminal key encoding failed: {err}"));
                        true
                    }
                }
            }
            TerminalKeyAction::NewSession => self.spawn_terminal_session_tab(),
            TerminalKeyAction::TerminateSession => {
                self.terminate_terminal_session();
                true
            }
            TerminalKeyAction::CloseSession => self.close_active_terminal_session(),
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
            TerminalKeyAction::CopyClipboard => self.copy_terminal_scrollback(),
            TerminalKeyAction::PasteClipboard => self.paste_terminal_input(),
            TerminalKeyAction::Search => {
                self.begin_terminal_search();
                true
            }
            TerminalKeyAction::Ignore => false,
        }
    }

    fn scroll_terminal_scrollback(&mut self, delta: usize) {
        let max = self.terminal_sessions.active_render_row_count();
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = (terminal.scroll_offset + delta).min(max);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_down(&mut self, delta: usize) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = terminal.scroll_offset.saturating_sub(delta);
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_top(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset =
            self.terminal_sessions.active_render_row_count();
        self.invalidate_frame();
    }

    fn scroll_terminal_scrollback_to_bottom(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn report_terminal_focus_event(&mut self, focused: bool) {
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let input = if focused {
            datum_terminal_core::FocusInput::Gained
        } else {
            datum_terminal_core::FocusInput::Lost
        };
        match self.terminal_sessions.encode_active_focus(input) {
            Ok(Some(bytes)) => {
                if let Err(err) = self.terminal_sessions.active().write_bytes(&bytes) {
                    self.log_review_event(format!("terminal focus report failed: {err}"));
                }
            }
            Ok(None) => {}
            Err(err) => self.log_review_event(format!("terminal focus encoding failed: {err}")),
        }
    }

    fn terminate_terminal_session(&mut self) {
        match self
            .terminal_sessions
            .terminate_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {}
            Err(err) => self.log_review_event(format!("terminal terminate failed: {err}")),
        }
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn restart_terminal_session(&mut self) {
        match self.terminal_sessions.restart_active(
            &mut self.session.workspace_mut().ui.terminal,
            &self.terminal_launch_context,
        ) {
            Ok(()) => {
                self.log_review_event("terminal restart requested; waiting for verified teardown");
                self.resize_terminal_to_dock();
            }
            Err(err) => self.log_review_event(format!("terminal restart failed: {err}")),
        }
        self.terminal_production_refresh_pending = false;
        self.terminal_workspace_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    fn activate_terminal_session(&mut self, session_id: &str) -> bool {
        if let Err(err) = self
            .terminal_sessions
            .activate_with_lane(session_id, &mut self.session.workspace_mut().ui.terminal)
        {
            self.log_review_event(format!("terminal session activate failed: {err}"));
            return true;
        }
        self.set_active_dock(DockTab::Terminal);
        self.refresh_terminal_activity_summary();
        self.sync_terminal_tabs();
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        true
    }

    fn copy_terminal_scrollback(&mut self) -> bool {
        if !matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            return false;
        }
        let text = match self.terminal_sessions.copy_active_selection() {
            Ok(text) => text,
            Err(_) => return false,
        };
        if self.write_clipboard_text(&text).is_err() {
            self.log_review_event("clipboard copy failed".to_string());
            return true;
        }
        self.log_review_event("terminal text copied".to_string());
        true
    }

    fn paste_terminal_input(&mut self) -> bool {
        let Ok(text) = self.read_clipboard_text() else {
            self.log_review_event("clipboard paste failed".to_string());
            return false;
        };
        if text.is_empty() {
            return false;
        }
        match self.terminal_input_owner() {
            keyboard_focus::TerminalInputOwner::AttachedPty => {
                match self.terminal_sessions.encode_active_paste(&text) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => false,
                    Err(err) => {
                        self.log_review_event(format!("terminal paste encoding failed: {err}"));
                        true
                    }
                }
            }
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
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

    fn log_review_event(&mut self, message: impl Into<String>) {
        // GUI-action narration is the AutoCAD/Eagle command-echo: it belongs in
        // the (not-yet-built) editor command console, never in the real PTY
        // terminal. Route it to the invisible console sink. No repaint is forced
        // here — the sink has no visible surface, and every narrating action
        // already invalidates the frame independently.
        terminal_narration::route_gui_narration(
            &mut self.session.workspace_mut().ui.console,
            message,
        );
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
            self.log_review_event(format!(
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
        // Authoring is a board-scene gesture; only the board surface drives it.
        let Some((world, SceneSurface::Board)) =
            prepared.world_point_at_screen(screen_pos.0, screen_pos.1)
        else {
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
                    self.log_review_event("place via requires a board net context".to_string());
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
                    self.log_review_event("place text requires project backing".to_string());
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
                    self.log_review_event("move requires a selected component target".to_string());
                    self.invalidate_frame();
                    return true;
                };
                self.queue_authoring_terminal_handoff(handoff, "move-board-component");
                self.invalidate_scene();
                return true;
            }
            WorkspaceTool::Move => {
                let Some(target) = target_object_id.clone() else {
                    self.log_review_event("move requires clicking a component first".to_string());
                    return true;
                };
                if !target.starts_with("component:") {
                    self.log_review_event("move currently supports components only".to_string());
                    return true;
                }
            }
            WorkspaceTool::Delete => {
                let Some(target) = target_object_id else {
                    self.log_review_event("delete requires an authored object target".to_string());
                    return true;
                };
                let Some(handoff) = self.workspace().delete_authored_object_handoff(&target) else {
                    self.log_review_event(format!("delete unsupported target {target}"));
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
            self.log_review_event(
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
            self.log_review_event(format!("terminal handoff prepare failed: {err}"));
            handoff.command.clone()
        });
        let mut bytes = command.into_bytes();
        bytes.push(b'\r');
        self.write_foreign_shell_bytes(&bytes);
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
        // Focus and dispatch are one gesture: after activating a different pane,
        // continue resolving this same click in that pane's camera/content.
        let mut focus_changed = false;
        if let Some(pane_id) = self.pane_at_screen(x, y) {
            // TF-01 deliberate exit: a canvas click is editor keyboard entry,
            // releasing any terminal/overlay key ownership before dispatch.
            self.set_application_focus(keyboard_focus::focus_after_canvas_click(pane_id));
            if pane_id != self.workspace().ui.layout.focused {
                self.swap_pane_focus(|layout| layout.focused = pane_id);
                self.log_review_event(format!("click-to-focus pane {}", pane_id.0));
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) focus-swapped to pane {}",
                    pane_id.0
                ));
                focus_changed = true;
            }
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
        if self.terminal_clipboard_menu_active()
            && !matches!(
                prepared_target.as_ref(),
                Some(
                    HitTarget::TerminalClipboardCopy
                        | HitTarget::TerminalClipboardPaste
                        | HitTarget::TerminalLinkCopy
                        | HitTarget::TerminalLinkOpen
                )
            )
        {
            self.dismiss_terminal_clipboard_menu();
            return true;
        }
        if let Some(target) = prepared_target {
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) prepared target {target:?}; prepare {}ms; dock {:?}",
                prepared_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return self.select_hit_target(&target) || focus_changed;
        }
        if let Some((world_point, SceneSurface::Schematic)) = world_point {
            // S3/UVT-004 plumbing: a focused schematic-pane click now resolves a
            // world point in the schematic camera AND hit-tests the symbol regions.
            // Firing selection off it is S5, so this resolves+traces only; the board
            // path below stays byte-identical.
            return self.resolve_schematic_primary_click((x, y), world_point) || focus_changed;
        }
        if let Some((world_point, SceneSurface::Board)) = world_point {
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
                return self.select_hit_target(&target) || focus_changed;
            }
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) world ({}, {}) no retained target; prepare {}ms; retained {}ms; dock {:?}",
                world_point.x,
                world_point.y,
                prepared_elapsed.as_millis(),
                retained_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return focus_changed;
        }
        self.trace_click(format!(
            "primary click ({x:.1}, {y:.1}) no prepared or viewport target; prepare {}ms; dock {:?}",
            prepared_elapsed.as_millis(),
            self.workspace().ui.active_dock_tab
        ));
        focus_changed
    }

    fn trace_click(&self, message: String) {
        if std::env::var_os("DATUM_TRACE_CLICKS").is_some() {
            eprintln!("[datum-click] {message}");
        }
    }

    fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        let started = std::time::Instant::now();
        let handled = self.select_hit_target_inner(target);
        // T0-C02 deliberate entry (spec §5): clicking the terminal SCREEN cell
        // rectangle — or a session action that expects terminal typing next —
        // hands key ownership to the terminal. Programmatic dock opens never do.
        let next_focus =
            keyboard_focus::focus_after_hit_target(self.application_focus(), handled, target);
        if next_focus != self.application_focus() {
            self.set_application_focus(next_focus);
        }
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
                    self.session.workspace_mut().ui.hovered_object = None;
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
                    self.session.workspace_mut().ui.hovered_object =
                        target.clone().map(|object_id| HoverTarget {
                            object_id,
                            surface: PaneContent::Board,
                        });
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
            HitTarget::TerminalTab => {
                // Owner decision 2026-08-14 (bead
                // dat-pan-trace-terminal-pollution-0j0): tab-click is
                // deliberate terminal entry, so the click stays handled even
                // when the dock is already showing the terminal —
                // `select_hit_target` then arms focus via
                // `hit_target_is_terminal_entry`.
                self.set_active_dock(DockTab::Terminal);
                true
            }
            HitTarget::TerminalSessionTab(session_id) => self.activate_terminal_session(session_id),
            HitTarget::TerminalSessionClose(session_id) => self.close_terminal_session(session_id),
            HitTarget::TerminalSessionNew => self.spawn_terminal_session_tab(),
            target @ (HitTarget::TerminalSessionTerminateActive
            | HitTarget::TerminalSessionForceKillActive
            | HitTarget::TerminalSessionRetryTermination
            | HitTarget::TerminalShutdownCancel) => self.handle_terminal_lifecycle_target(target),
            HitTarget::TerminalScreen => self.click_terminal_screen(),
            HitTarget::TerminalClipboardCopy => {
                self.dismiss_terminal_clipboard_menu();
                self.copy_terminal_scrollback();
                true
            }
            HitTarget::TerminalClipboardPaste => {
                self.dismiss_terminal_clipboard_menu();
                self.paste_terminal_input();
                true
            }
            HitTarget::TerminalLinkCopy => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.copy_terminal_link_target(&target);
                }
                true
            }
            HitTarget::TerminalLinkOpen => {
                let target = self.terminal_clipboard_link_target();
                self.dismiss_terminal_clipboard_menu();
                if let Some(target) = target {
                    self.arm_terminal_link_target(target);
                }
                true
            }
            HitTarget::TerminalLinkConfirmOpen => self.confirm_terminal_link_open(),
            HitTarget::TerminalLinkCancel => {
                self.cancel_terminal_link_confirmation();
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
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_review_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                self.write_foreign_shell_bytes(&bytes);
                self.log_review_event(format!("ran production output command {}", handoff.command));
                true
            }
            HitTarget::ProductionTerminalCommand(handoff) => {
                self.set_active_dock(DockTab::Terminal);
                let command = prepare_terminal_command_execution(
                    self.terminal_sessions.active(),
                    "production_terminal_command",
                    handoff,
                )
                .unwrap_or_else(|err| {
                    self.log_review_event(format!("terminal handoff prepare failed: {err}"));
                    handoff.command.clone()
                });
                let mut bytes = command.into_bytes();
                bytes.push(b'\r');
                self.write_foreign_shell_bytes(&bytes);
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
        // TF-01: the marking menu is a transient Overlay key owner; dismissing
        // it restores keyboard ownership to the editor.
        let pane = self.workspace().ui.layout.focused;
        self.set_application_focus(ApplicationFocus::Editor(pane));
        self.invalidate_frame();
        true
    }

    fn toggle_menu(&mut self, menu: &str) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        ui.terminal_clipboard_menu = None;
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
            self.log_review_event(format!("menu item {menu_name}/{label} unavailable"));
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
        self.log_review_event(format!("{menu_name}/{label} disabled: {reason}"));
        self.invalidate_frame();
        true
    }

    fn trace_timing(&self, message: String) {
        if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
            eprintln!("[datum-timing] {message}");
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
    } else if !(-157.5..157.5).contains(&angle) {
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
