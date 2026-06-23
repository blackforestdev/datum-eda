use anyhow::{Context, Result};
use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use clap::Parser;
use datum_gui_protocol::{
    BoardTextAlignmentField, BoardTextBooleanField, BoardTextCycleField, BoardTextHeightStep,
    BoardTextLineSpacingStep, BoardTextRotationStep, DockTab, LiveDesignSession, LiveReviewRequest,
    PointNm, RectNm, SceneBounds, SessionCommand, SessionEvent, WorkspaceTool,
    ensure_known_good_demo_request, load_board_editor_workspace_state, load_live_workspace_state,
};
use datum_gui_render::{
    CameraState, HitTarget, PreparedScene, Renderer, RetainedScene, ShellLayout,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::TryRecvError;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

mod artifact_preview_controls;
mod assistant_bridge;
mod board_text_terminal_commands;
mod production_status_refresh;
mod runtime_terminal_context;
mod terminal_activity_snapshot;
mod terminal_agent_launcher;
mod terminal_context;
mod terminal_input;
mod terminal_screen;
mod terminal_session;
mod terminal_session_events;
use assistant_bridge::{
    AssistantBridgeAction, AssistantBridgeConfig, AssistantBridgeInput, AssistantSession,
    spawn_assistant_session,
};
use board_text_terminal_commands::{
    BoardTextEditTerminalField, BoardTextQuickEditTerminalAction, board_text_edit_terminal_command,
    board_text_quick_edit_terminal_command,
};
use terminal_activity_snapshot::load_terminal_activity_summary_lines;
use terminal_input::{TerminalKeyAction, terminal_key_action};
use terminal_screen::{TerminalScreen, terminal_scrollback_copy_text};
use terminal_session::{
    TerminalLaunchContext, TerminalSession, refresh_terminal_session_context_from_state,
    restart_terminal_session as restart_pty_terminal_session, spawn_terminal_session,
    terminal_launch_context_from_state,
    terminate_terminal_session as terminate_pty_terminal_session,
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
    dock_height_px: u32,
    show_authored: bool,
    show_proposed: bool,
    show_unrouted: bool,
    dim_unrelated: bool,
    layer_visibility: BTreeMap<String, bool>,
    selection: String,
}

fn main() -> Result<()> {
    let args = GuiArgs::parse();
    let event_loop = EventLoop::new().context("failed to create event loop")?;
    let mut app = App::new(args);
    event_loop.run_app(&mut app).context("failed to run app")
}

fn fatal_gui_error(event_loop: &ActiveEventLoop, context: &str, err: impl std::fmt::Display) -> ! {
    eprintln!("datum-gui error: {context}: {err}");
    event_loop.exit();
    std::process::exit(1);
}

fn trace_startup_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-startup] {message}");
    }
}

fn parse_window_size(value: &str) -> Result<(u32, u32)> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| anyhow::anyhow!("window size must use <width>x<height>"))?;
    let width = width
        .parse::<u32>()
        .with_context(|| format!("parse window width from {value:?}"))?;
    let height = height
        .parse::<u32>()
        .with_context(|| format!("parse window height from {value:?}"))?;
    if width == 0 || height == 0 {
        anyhow::bail!("window size dimensions must be non-zero");
    }
    Ok((width, height))
}

fn terminal_raw_input_should_handle(
    terminal_accepts_raw_input: bool,
    paste_shortcut: bool,
    copy_shortcut: bool,
) -> bool {
    terminal_accepts_raw_input && !paste_shortcut && !copy_shortcut
}

fn selection_cache_key(workspace: &datum_gui_protocol::ReviewWorkspaceState) -> String {
    match &workspace.selection {
        datum_gui_protocol::SelectionTarget::None => "none".to_string(),
        datum_gui_protocol::SelectionTarget::ReviewAction(id) => format!("review:{id}"),
        datum_gui_protocol::SelectionTarget::AuthoredObject(id) => format!("object:{id}"),
        datum_gui_protocol::SelectionTarget::CheckFinding(id) => format!("finding:{id}"),
        datum_gui_protocol::SelectionTarget::Finding(id) => format!("finding:{id}"),
        datum_gui_protocol::SelectionTarget::JournalEntry(id) => format!("journal:{id}"),
        datum_gui_protocol::SelectionTarget::Relationship(id) => format!("relationship:{id}"),
    }
}

fn retained_selection_cache_key(
    workspace: &datum_gui_protocol::ReviewWorkspaceState,
    selection: &datum_gui_protocol::SelectionTarget,
) -> String {
    match selection {
        datum_gui_protocol::SelectionTarget::None => "none".to_string(),
        datum_gui_protocol::SelectionTarget::ReviewAction(id) => format!("review:{id}"),
        datum_gui_protocol::SelectionTarget::CheckFinding(id) => format!("finding:{id}"),
        datum_gui_protocol::SelectionTarget::AuthoredObject(id) => {
            let lightweight = workspace
                .scene
                .board_texts
                .iter()
                .any(|text| &text.object_id == id)
                || workspace
                    .scene
                    .outline
                    .iter()
                    .any(|outline| &outline.object_id == id)
                || workspace
                    .scene
                    .board_graphics
                    .iter()
                    .any(|graphic| &graphic.object_id == id);
            if lightweight && !workspace.ui.filters.dim_unrelated {
                "none".to_string()
            } else if lightweight {
                "lightweight-authored".to_string()
            } else {
                format!("object:{id}")
            }
        }
        datum_gui_protocol::SelectionTarget::Finding(id) => format!("finding:{id}"),
        datum_gui_protocol::SelectionTarget::JournalEntry(id) => format!("journal:{id}"),
        datum_gui_protocol::SelectionTarget::Relationship(id) => format!("relationship:{id}"),
    }
}

#[cfg(feature = "visual")]
fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

#[cfg(feature = "visual")]
fn convert_texture_pixels_to_rgba(pixels: &mut [u8], format: wgpu::TextureFormat) -> Result<()> {
    match format {
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => Ok(()),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
            Ok(())
        }
        other => anyhow::bail!("unsupported visual screenshot surface format: {other:?}"),
    }
}

#[derive(Debug, Clone, Parser)]
#[command(name = "datum-gui", about = "Datum M7 route-proposal review spike")]
struct GuiArgs {
    #[arg(long = "demo-known-good", default_value_t = false)]
    demo_known_good: bool,
    #[arg(long = "board", help = "Open a KiCad .kicad_pcb board file directly")]
    board_file: Option<PathBuf>,
    #[arg(long = "artifact")]
    artifact_path: Option<PathBuf>,
    #[arg(long = "project-root")]
    project_root: Option<PathBuf>,
    #[arg(long = "net")]
    net_uuid: Option<String>,
    #[arg(long = "from-anchor")]
    from_anchor_pad_uuid: Option<String>,
    #[arg(long = "to-anchor")]
    to_anchor_pad_uuid: Option<String>,
    #[arg(long = "profile")]
    profile: Option<String>,
    #[arg(long = "visual-test", default_value_t = false)]
    visual_test: bool,
    #[arg(long = "window-size", default_value = "1280x768")]
    window_size: String,
    #[arg(long = "screenshot-out")]
    screenshot_out: Option<PathBuf>,
    #[arg(long = "exit-after-screenshot", default_value_t = false)]
    exit_after_screenshot: bool,
}

struct App {
    args: GuiArgs,
    window: Option<&'static Window>,
    runtime: Option<Runtime>,
}

#[derive(Debug, Serialize)]
struct AssistantContext {
    board_name: String,
    project_name: String,
    tool: &'static str,
    selection: String,
    active_review_target_id: String,
    project_root: Option<String>,
    terminal_activity_summary: Vec<String>,
    selected_component: Option<AssistantSelectedComponent>,
    selected_review_action: Option<AssistantSelectedReviewAction>,
    components: Vec<AssistantComponentSummary>,
    review_actions: Vec<AssistantReviewActionSummary>,
}

#[derive(Debug, Serialize)]
struct AssistantSelectedComponent {
    reference: String,
    value: Option<String>,
    object_id: String,
    component_uuid: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Serialize)]
struct AssistantSelectedReviewAction {
    action_id: String,
    net_name: String,
    layer: i32,
    width_nm: i64,
}

#[derive(Debug, Serialize)]
struct AssistantComponentSummary {
    reference: String,
    value: Option<String>,
    object_id: String,
    component_uuid: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Serialize)]
struct AssistantReviewActionSummary {
    action_id: String,
    net_name: String,
    layer: i32,
    width_nm: i64,
}

impl App {
    fn new(args: GuiArgs) -> Self {
        Self {
            args,
            window: None,
            runtime: None,
        }
    }

    fn request_redraw_if_needed(&mut self) {
        if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window)
            && !runtime.redraw_pending
        {
            runtime.redraw_pending = true;
            window.request_redraw();
        }
    }
}

impl GuiArgs {
    fn visual_window_size(&self) -> Result<(u32, u32)> {
        parse_window_size(&self.window_size)
    }

    fn validate_visual_args(&self) -> Result<()> {
        if !self.visual_test {
            return Ok(());
        }
        if self.screenshot_out.is_none() {
            anyhow::bail!("--visual-test requires --screenshot-out");
        }
        self.visual_window_size()?;
        Ok(())
    }

    fn wants_plain_project_board_view(&self) -> bool {
        self.project_root.is_some()
            && self.board_file.is_none()
            && self.artifact_path.is_none()
            && self.net_uuid.is_none()
            && self.from_anchor_pad_uuid.is_none()
            && self.to_anchor_pad_uuid.is_none()
    }

    fn resolve_request(&self) -> Result<LiveReviewRequest> {
        if self.demo_known_good {
            return ensure_known_good_demo_request();
        }
        if let Some(board_file) = &self.board_file {
            let project_root = self
                .project_root
                .clone()
                .or_else(|| board_file.parent().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from("."));
            return Ok(LiveReviewRequest {
                project_root,
                board_file: Some(board_file.clone()),
                artifact_path: None,
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: None,
            });
        }
        if let Some(artifact_path) = &self.artifact_path {
            let project_root = self
                .project_root
                .clone()
                .or_else(|| artifact_path.parent().map(PathBuf::from))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "--project-root is required when --artifact has no parent directory"
                    )
                })?;
            return Ok(LiveReviewRequest {
                project_root,
                board_file: None,
                artifact_path: Some(artifact_path.clone()),
                net_uuid: None,
                from_anchor_pad_uuid: None,
                to_anchor_pad_uuid: None,
                profile: self.profile.clone(),
            });
        }
        Ok(LiveReviewRequest {
            project_root: self.project_root.clone().ok_or_else(|| {
                anyhow::anyhow!("--project-root, --board, or --demo-known-good is required")
            })?,
            board_file: None,
            artifact_path: None,
            net_uuid: self.net_uuid.clone(),
            from_anchor_pad_uuid: self.from_anchor_pad_uuid.clone(),
            to_anchor_pad_uuid: self.to_anchor_pad_uuid.clone(),
            profile: self.profile.clone(),
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
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
        let (window_width, window_height) = self
            .args
            .visual_window_size()
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "window size invalid", err));
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Datum M7 Spike")
                    .with_inner_size(LogicalSize::new(window_width as f64, window_height as f64)),
            )
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "window creation failed", err));
        window.set_ime_allowed(true);
        let window_ref: &'static Window = Box::leak(Box::new(window));
        let runtime = pollster::block_on(Runtime::new(window_ref, &self.args))
            .unwrap_or_else(|err| fatal_gui_error(event_loop, "runtime creation failed", err));
        self.runtime = Some(runtime);
        self.window = Some(window_ref);
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
        if let Some(runtime) = &mut self.runtime
            && runtime.poll_terminal_output()
        {
            self.request_redraw_if_needed();
        }
        if let Some(runtime) = &mut self.runtime
            && runtime.poll_assistant_output()
        {
            self.request_redraw_if_needed();
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
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
                if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window) {
                    runtime.resize(size.width, size.height);
                    if runtime.needs_redraw() {
                        let _ = window;
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    let next_pos = (position.x as f32, position.y as f32);
                    let mut changed = false;
                    if runtime.dock_drag_active {
                        changed = runtime.handle_dock_resize_drag(next_pos);
                    } else if runtime.middle_drag_active || runtime.right_drag_active {
                        changed = runtime.handle_pan_drag(next_pos);
                    }
                    runtime.last_cursor_pos = Some(next_pos);
                    // Update hover state
                    if !runtime.dock_drag_active
                        && !runtime.middle_drag_active
                        && !runtime.right_drag_active
                    {
                        changed = runtime.update_hover(next_pos) || changed;
                    }
                    runtime.refresh_terminal_context_snapshot();
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
                    runtime.middle_drag_active = state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.right_drag_active = state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(runtime) = &mut self.runtime {
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
                .is_some_and(|runtime| runtime.workspace().ui.active_dock_tab.is_some()) =>
            {
                if let Some(runtime) = &mut self.runtime {
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
                            Some(DockTab::Assistant) => {
                                ui.assistant.input.clear();
                                ui.assistant.cursor = 0;
                            }
                            Some(DockTab::Outputs) => {}
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
                    if !matches!(
                        runtime.workspace().selection,
                        datum_gui_protocol::SelectionTarget::None
                    ) {
                        let _ = runtime.dispatch_session_command(SessionCommand::ClearSelection);
                        runtime.invalidate_scene();
                        self.request_redraw_if_needed();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.redraw_pending = false;
                    let render_started = std::time::Instant::now();
                    if let Err(err) = runtime.render() {
                        fatal_gui_error(event_loop, "render failed", err);
                    }
                    runtime.trace_timing(format!(
                        "redraw render {}ms",
                        render_started.elapsed().as_millis()
                    ));
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
    renderer: Renderer,
    session: LiveDesignSession,
    camera: CameraState,
    last_cursor_pos: Option<(f32, f32)>,
    middle_drag_active: bool,
    right_drag_active: bool,
    dock_drag_active: bool,
    modifiers: ModifiersState,
    redraw_pending: bool,
    retained_scene: Option<RetainedScene>,
    retained_scene_cache: Vec<(RetainedSceneCacheKey, RetainedScene)>,
    prepared_scene: Option<PreparedScene>,
    scene_dirty: bool,
    terminal: TerminalSession,
    terminal_screen: TerminalScreen,
    assistant: AssistantSession,
    terminal_disconnected_reported: bool,
    assistant_disconnected_reported: bool,
    assistant_config_path: PathBuf,
    assistant_config: AssistantBridgeConfig,
    terminal_launch_context: TerminalLaunchContext,
    terminal_production_refresh_pending: bool,
    terminal_production_refresh_due: Option<std::time::Instant>,
    terminal_production_refresh_attempts: u8,
    clipboard: Option<Clipboard>,
}

impl Runtime {
    fn workspace(&self) -> &datum_gui_protocol::ReviewWorkspaceState {
        self.session.workspace()
    }

    async fn new(window: &'static Window, args: &GuiArgs) -> Result<Self> {
        let runtime_started = std::time::Instant::now();
        let wgpu_started = std::time::Instant::now();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).context("create surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("request adapter")?;
        let want_msaa8 =
            wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES & adapter.features();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("datum-m7-spike-device"),
                required_features: want_msaa8,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .context("request device")?;
        trace_startup_timing(format!(
            "wgpu init {}ms",
            wgpu_started.elapsed().as_millis()
        ));
        let msaa_samples =
            if want_msaa8.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
                8
            } else {
                4
            };
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let renderer_started = std::time::Instant::now();
        let renderer = Renderer::new(&device, &queue, config.format, msaa_samples);
        trace_startup_timing(format!(
            "renderer init {}ms",
            renderer_started.elapsed().as_millis()
        ));
        let request_started = std::time::Instant::now();
        let request = args
            .resolve_request()
            .context("resolve GUI launch review context")?;
        trace_startup_timing(format!(
            "request resolve {}ms",
            request_started.elapsed().as_millis()
        ));
        let config_started = std::time::Instant::now();
        let assistant_config_path = assistant_config_path(&request);
        let assistant_config =
            load_assistant_config(&assistant_config_path).with_context(|| {
                format!("load assistant config {}", assistant_config_path.display())
            })?;
        trace_startup_timing(format!(
            "assistant config load {}ms",
            config_started.elapsed().as_millis()
        ));
        let workspace_started = std::time::Instant::now();
        let state = if args.wants_plain_project_board_view() {
            load_board_editor_workspace_state(&request)
                .context("load board editor workspace state")?
        } else {
            load_live_workspace_state(&request).context("load live M7 review workspace state")?
        };
        trace_startup_timing(format!(
            "workspace load {}ms",
            workspace_started.elapsed().as_millis()
        ));
        let camera_started = std::time::Instant::now();
        let camera = CameraState::fit_to_bounds(&state.scene.bounds);
        trace_startup_timing(format!(
            "camera fit {}ms",
            camera_started.elapsed().as_millis()
        ));
        let terminal_launch_context =
            terminal_launch_context_from_state(&request.project_root, &state);
        let terminal_started = std::time::Instant::now();
        let terminal = spawn_terminal_session(&terminal_launch_context)
            .context("spawn integrated terminal lane")?;
        trace_startup_timing(format!(
            "terminal spawn {}ms",
            terminal_started.elapsed().as_millis()
        ));
        let assistant_started = std::time::Instant::now();
        let assistant = spawn_assistant_session(&assistant_config)
            .context("spawn integrated assistant lane")?;
        trace_startup_timing(format!(
            "assistant spawn {}ms",
            assistant_started.elapsed().as_millis()
        ));
        let mut runtime = Self {
            surface,
            device,
            queue,
            config,
            renderer,
            session: LiveDesignSession::new(state),
            camera,
            last_cursor_pos: None,
            middle_drag_active: false,
            right_drag_active: false,
            dock_drag_active: false,
            modifiers: ModifiersState::empty(),
            redraw_pending: false,
            retained_scene: None,
            retained_scene_cache: Vec::new(),
            prepared_scene: None,
            scene_dirty: true,
            terminal,
            terminal_screen: TerminalScreen::default(),
            assistant,
            terminal_disconnected_reported: false,
            assistant_disconnected_reported: false,
            assistant_config_path,
            assistant_config,
            terminal_launch_context,
            terminal_production_refresh_pending: false,
            terminal_production_refresh_due: None,
            terminal_production_refresh_attempts: 0,
            clipboard: Clipboard::new().ok(),
        };
        let assistant_context_started = std::time::Instant::now();
        runtime.sync_assistant_context();
        trace_startup_timing(format!(
            "assistant context sync {}ms",
            assistant_context_started.elapsed().as_millis()
        ));
        trace_startup_timing(format!(
            "runtime total {}ms",
            runtime_started.elapsed().as_millis()
        ));
        Ok(runtime)
    }

    fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.resize_terminal_to_dock();
        self.invalidate_scene();
    }

    fn render(&mut self) -> Result<()> {
        let render_started = std::time::Instant::now();
        let acquire_started = std::time::Instant::now();
        let frame = self
            .surface
            .get_current_texture()
            .context("acquire next surface texture")?;
        let acquire_elapsed = acquire_started.elapsed();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let scene_started = std::time::Instant::now();
        let retained_was_cached = self.retained_scene.is_some();
        let prepared_was_cached = self.prepared_scene.is_some();
        let mut retained_build_ms = 0;
        let mut prepared_build_ms = 0;
        if self.prepared_scene.is_none() {
            self.scene_dirty = false;
            if self.retained_scene.is_none() {
                let retained_started = std::time::Instant::now();
                self.retained_scene = Some(RetainedScene::from_workspace(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                ));
                retained_build_ms = retained_started.elapsed().as_millis();
            }
            let retained = self
                .retained_scene
                .as_ref()
                .context("retained scene should exist before prepared scene rebuild")?;
            let prepared_started = std::time::Instant::now();
            self.prepared_scene = Some(PreparedScene::from_workspace(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.camera,
                retained,
            ));
            prepared_build_ms = prepared_started.elapsed().as_millis();
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
        let present_started = std::time::Instant::now();
        frame.present();
        let present_elapsed = present_started.elapsed();
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
                RetainedScene::from_workspace(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                )
            });
            self.prepared_scene = Some(PreparedScene::from_workspace(
                self.session.workspace(),
                self.config.width,
                self.config.height,
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
            RetainedScene::from_workspace(
                self.session.workspace(),
                self.config.width,
                self.config.height,
            )
        });
        self.prepared_scene.get_or_insert_with(|| {
            self.scene_dirty = false;
            PreparedScene::from_workspace(
                self.session.workspace(),
                self.config.width,
                self.config.height,
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

    fn poll_assistant_output(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.assistant.rx.try_recv() {
                Ok(output) => {
                    changed = true;
                    match output.kind.as_str() {
                        "ready" | "error" => {
                            self.push_assistant_message("assistant", output.message);
                        }
                        "response" => {
                            if !output.message.is_empty() {
                                self.push_assistant_message("assistant", output.message);
                            }
                            let summary = self.apply_assistant_actions(&output.actions);
                            if !summary.is_empty() {
                                self.push_assistant_message("system", summary);
                            }
                        }
                        other => {
                            self.push_assistant_message(
                                "system",
                                format!("assistant bridge sent unknown event `{other}`"),
                            );
                        }
                    }
                }
                Err(TryRecvError::Empty) => return changed,
                Err(TryRecvError::Disconnected) => {
                    if self.assistant_disconnected_reported {
                        return false;
                    }
                    self.assistant_disconnected_reported = true;
                    self.push_assistant_message("system", "assistant session ended".to_string());
                    return true;
                }
            }
        }
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

    fn refresh_terminal_activity_summary(&mut self) -> bool {
        let next = match load_terminal_activity_summary_lines(&self.terminal.event_log_path(), 4) {
            Ok(lines) => lines,
            Err(err) => vec![format!("activity summary unavailable: {err}")],
        };
        let ui = &mut self.session.workspace_mut().ui;
        if ui.terminal.activity_summary == next {
            return false;
        }
        ui.terminal.activity_summary = next;
        self.invalidate_frame();
        true
    }

    fn push_assistant_message(&mut self, role: &str, content: String) {
        let ui = &mut self.session.workspace_mut().ui;
        ui.assistant
            .transcript
            .push(datum_gui_protocol::AssistantMessage {
                role: role.to_string(),
                content,
            });
        if ui.assistant.transcript.len() > 80 {
            ui.assistant
                .transcript
                .drain(0..ui.assistant.transcript.len() - 80);
        }
        ui.assistant.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn set_active_dock(&mut self, tab: DockTab) -> bool {
        let ui = &mut self.session.workspace_mut().ui;
        if ui.active_dock_tab == Some(tab) {
            return false;
        }
        ui.active_dock_tab = Some(tab);
        self.invalidate_scene();
        self.refresh_terminal_context_snapshot();
        if matches!(tab, DockTab::Terminal) {
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
        self.refresh_terminal_context_snapshot();
        true
    }

    fn dock_accepts_text_input(&self) -> bool {
        self.workspace().ui.active_dock_tab.is_some()
    }

    fn terminal_accepts_raw_input(&self) -> bool {
        matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal))
    }

    fn handle_terminal_key_input(&mut self, event: &KeyEvent) -> bool {
        match terminal_key_action(event, self.modifiers) {
            TerminalKeyAction::Write(bytes) => self.write_terminal_bytes(&bytes),
            TerminalKeyAction::Interrupt => {
                if let Err(err) = self.terminal.interrupt() {
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
            TerminalKeyAction::ConsumeRelease => true,
            TerminalKeyAction::LetPasteShortcutHandle
            | TerminalKeyAction::LetCopyShortcutHandle
            | TerminalKeyAction::Ignore => false,
        }
    }

    fn write_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        if bytes.iter().any(|byte| matches!(byte, b'\n' | b'\r')) {
            self.mark_terminal_production_refresh_pending();
        }
        if let Err(err) = self.terminal.write_bytes(bytes) {
            self.push_terminal_line(format!("terminal write failed: {err}"));
        }
        true
    }

    fn terminate_terminal_session(&mut self) {
        match terminate_pty_terminal_session(
            &self.terminal,
            &mut self.session.workspace_mut().ui.terminal,
        ) {
            Ok(()) => {}
            Err(err) => self.push_terminal_line(format!("terminal terminate failed: {err}")),
        }
        self.invalidate_frame();
    }

    fn restart_terminal_session(&mut self) {
        match restart_pty_terminal_session(
            &mut self.terminal,
            &mut self.terminal_screen,
            &mut self.session.workspace_mut().ui.terminal,
            &self.terminal_launch_context,
        ) {
            Ok(()) => self.resize_terminal_to_dock(),
            Err(err) => self.push_terminal_line(format!("terminal restart failed: {err}")),
        }
        self.terminal_disconnected_reported = false;
        self.terminal_production_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.invalidate_frame();
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
            Some(DockTab::Assistant) => !self.modifiers.shift_key(),
            Some(DockTab::Outputs) | None => false,
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
        if !matches!(active, DockTab::Assistant) {
            return false;
        }
        if text.chars().all(|ch| !ch.is_control()) {
            let ui = &mut self.session.workspace_mut().ui;
            let (input, cursor) = (&mut ui.assistant.input, &mut ui.assistant.cursor);
            let byte_pos = char_to_byte_pos(input, *cursor);
            input.insert_str(byte_pos, text);
            *cursor += text.chars().count();
            self.invalidate_frame();
            return true;
        }
        false
    }

    fn current_dock_input(&self) -> Option<&str> {
        matches!(
            self.workspace().ui.active_dock_tab,
            Some(DockTab::Assistant)
        )
        .then_some(self.workspace().ui.assistant.input.as_str())
    }

    fn current_dock_input_mut(&mut self) -> Option<&mut String> {
        matches!(
            self.workspace().ui.active_dock_tab,
            Some(DockTab::Assistant)
        )
        .then_some(&mut self.session.workspace_mut().ui.assistant.input)
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
            self.push_assistant_message("system", "clipboard copy failed".to_string());
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
            self.push_assistant_message("system", "clipboard cut failed".to_string());
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
            self.push_assistant_message("system", "clipboard paste failed".to_string());
            return false;
        };
        if text.is_empty() {
            return false;
        }
        if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            return self.write_terminal_bytes(text.as_bytes());
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
        if !matches!(active, DockTab::Assistant) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = (&mut ui.assistant.input, &mut ui.assistant.cursor);
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
        if !matches!(active, DockTab::Assistant) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = (&ui.assistant.input, &mut ui.assistant.cursor);
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
        if !matches!(active, DockTab::Assistant) {
            return false;
        }
        let ui = &mut self.session.workspace_mut().ui;
        let (input, cursor) = (&ui.assistant.input, &mut ui.assistant.cursor);
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
            Some(DockTab::Assistant) => self.complete_assistant_input(),
            Some(DockTab::Terminal) => false,
            Some(DockTab::Outputs) => false,
            None => false,
        }
    }

    fn complete_assistant_input(&mut self) -> bool {
        let current = self.session.workspace().ui.assistant.input.clone();
        let completed = self.complete_assistant_text(&current);
        if completed == current {
            return false;
        }
        let cursor = completed.chars().count();
        let ui = &mut self.session.workspace_mut().ui;
        ui.assistant.input = completed;
        ui.assistant.cursor = cursor;
        self.invalidate_frame();
        true
    }

    fn complete_assistant_text(&self, input: &str) -> String {
        const CONFIG_COMMANDS: &[&str] = &[
            "/activity",
            "/config status",
            "/config api-key ",
            "/config api-key",
            "/config clear-api-key",
            "/config model ",
            "/config clear-model",
            "/config cancel",
        ];
        let trimmed_start = input.trim_start();
        if !trimmed_start.starts_with('/') {
            return input.to_string();
        }
        if "/activity".starts_with(trimmed_start) {
            return replace_tail(input, trimmed_start, "/activity");
        }
        if !trimmed_start.starts_with("/config") {
            if "/config".starts_with(trimmed_start) {
                return replace_tail(input, trimmed_start, "/config ");
            }
            return input.to_string();
        }
        if let Some(completed) = best_completion(trimmed_start, CONFIG_COMMANDS) {
            let replaced = replace_tail(input, trimmed_start, &completed);
            if replaced != input {
                return replaced;
            }
        }

        if let Some(rest) = trimmed_start.strip_prefix("/config model ") {
            let models = ["gpt-5.4-mini", "gpt-5.4", "gpt-5.3-codex-spark"];
            if let Some(model) = best_completion(rest, &models) {
                return replace_tail(input, trimmed_start, &format!("/config model {model}"));
            }
        }

        if let Some(rest) = trimmed_start.strip_prefix("/config select ") {
            let mut references: Vec<&str> = self
                .workspace()
                .scene
                .components
                .iter()
                .map(|component| component.reference.as_str())
                .collect();
            references.sort_unstable();
            references.dedup();
            if let Some(reference) = best_completion(rest, &references) {
                return replace_tail(input, trimmed_start, &format!("/config select {reference}"));
            }
        }

        input.to_string()
    }

    fn submit_dock_input(&mut self) -> bool {
        match self.workspace().ui.active_dock_tab {
            Some(DockTab::Terminal) => false,
            Some(DockTab::Assistant) => self.submit_assistant_input(),
            Some(DockTab::Outputs) => false,
            None => false,
        }
    }

    fn submit_assistant_input(&mut self) -> bool {
        let input = {
            let ui = &mut self.session.workspace_mut().ui;
            let input = ui.assistant.input.trim().to_string();
            ui.assistant.input.clear();
            ui.assistant.cursor = 0;
            input
        };
        if self.workspace().ui.assistant.awaiting_api_key {
            return self.submit_assistant_api_key(input);
        }
        if input.is_empty() {
            self.invalidate_frame();
            return true;
        }
        if self.handle_assistant_meta_command(&input) {
            return true;
        }
        {
            self.push_assistant_message("user", input.clone());
        }
        if let Err(err) = self.send_assistant_message(AssistantBridgeInput {
            kind: "user_message",
            text: Some(input),
            context: Some(self.assistant_context()),
        }) {
            self.push_assistant_message("assistant", format!("assistant bridge failed: {err}"));
        }
        true
    }

    fn submit_assistant_api_key(&mut self, input: String) -> bool {
        self.session.workspace_mut().ui.assistant.awaiting_api_key = false;
        if input.is_empty() {
            self.push_assistant_message(
                "assistant",
                "assistant api-key entry canceled".to_string(),
            );
            return true;
        }
        self.assistant_config.api_key = Some(input);
        match self.persist_and_restart_assistant() {
            Ok(()) => self.push_assistant_message(
                "assistant",
                "assistant API key saved locally; bridge restarted".to_string(),
            ),
            Err(err) => self.push_assistant_message(
                "assistant",
                format!("failed to save assistant API key: {err}"),
            ),
        }
        true
    }

    fn handle_assistant_meta_command(&mut self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed == "/activity" {
            self.push_assistant_message("user", trimmed.to_string());
            self.set_active_dock(DockTab::Terminal);
            if let Err(err) = record_manual_terminal_command_handoff(
                &self.terminal,
                "assistant_activity",
                "datum.gui.session_activity",
                "execute",
                ASSISTANT_ACTIVITY_COMMAND,
            ) {
                self.push_terminal_line(format!("terminal handoff event write failed: {err}"));
            }
            self.write_terminal_bytes(assistant_activity_command_bytes().as_bytes());
            self.log_review_event("ran assistant activity command".to_string());
            return true;
        }
        if !trimmed.starts_with("/config") {
            return false;
        }
        let redacted = redact_assistant_config_command(trimmed);
        self.push_assistant_message("user", redacted);
        let mut parts = trimmed.split_whitespace();
        let _ = parts.next();
        match parts.next() {
            Some("status") | None => {
                self.push_assistant_message("assistant", self.assistant_status_summary());
            }
            Some("api-key") => {
                let key = if let Some(key) = parts.next() {
                    key.to_string()
                } else {
                    self.session.workspace_mut().ui.assistant.awaiting_api_key = true;
                    self.push_assistant_message(
                        "assistant",
                        "enter assistant API key and press Enter; input will be hidden".to_string(),
                    );
                    return true;
                };
                self.assistant_config.api_key = Some(key);
                match self.persist_and_restart_assistant() {
                    Ok(()) => self.push_assistant_message(
                        "assistant",
                        "assistant API key saved locally; bridge restarted".to_string(),
                    ),
                    Err(err) => self.push_assistant_message(
                        "assistant",
                        format!("failed to save assistant API key: {err}"),
                    ),
                }
            }
            Some("clear-api-key") => {
                self.assistant_config.api_key = None;
                match self.persist_and_restart_assistant() {
                    Ok(()) => self.push_assistant_message(
                        "assistant",
                        "assistant API key cleared; bridge restarted".to_string(),
                    ),
                    Err(err) => self.push_assistant_message(
                        "assistant",
                        format!("failed to clear assistant API key: {err}"),
                    ),
                }
            }
            Some("model") => {
                let Some(model) = parts.next() else {
                    self.push_assistant_message(
                        "assistant",
                        "usage: /config model <model>".to_string(),
                    );
                    return true;
                };
                self.assistant_config.model = Some(model.to_string());
                match self.persist_and_restart_assistant() {
                    Ok(()) => self.push_assistant_message(
                        "assistant",
                        format!("assistant model set to {model}; bridge restarted"),
                    ),
                    Err(err) => self.push_assistant_message(
                        "assistant",
                        format!("failed to save assistant model: {err}"),
                    ),
                }
            }
            Some("clear-model") => {
                self.assistant_config.model = None;
                match self.persist_and_restart_assistant() {
                    Ok(()) => self.push_assistant_message(
                        "assistant",
                        "assistant model cleared; bridge restarted".to_string(),
                    ),
                    Err(err) => self.push_assistant_message(
                        "assistant",
                        format!("failed to clear assistant model: {err}"),
                    ),
                }
            }
            Some("cancel") => {
                self.session.workspace_mut().ui.assistant.awaiting_api_key = false;
                self.push_assistant_message(
                    "assistant",
                    "assistant config entry canceled".to_string(),
                );
            }
            Some(other) => {
                self.push_assistant_message(
                    "assistant",
                    format!(
                        "unknown config command `{other}`. Use /config status | /config api-key | /config api-key <key> | /config clear-api-key | /config model <model> | /config clear-model | /config cancel"
                    ),
                );
            }
        }
        true
    }

    fn assistant_status_summary(&self) -> String {
        let api_key = if self.assistant_config.api_key.is_some() {
            "configured"
        } else {
            "missing"
        };
        let model = self
            .assistant_config
            .model
            .as_deref()
            .unwrap_or("gpt-5.4-mini");
        format!(
            "assistant config: api-key={api_key}, model={model}. Use /activity for recent terminal session activity, /config api-key <key>, and /config model <model>."
        )
    }

    fn persist_and_restart_assistant(&mut self) -> Result<()> {
        save_assistant_config(&self.assistant_config_path, &self.assistant_config)?;
        self.assistant =
            spawn_assistant_session(&self.assistant_config).context("restart assistant bridge")?;
        self.sync_assistant_context();
        Ok(())
    }

    fn send_assistant_message(&mut self, message: AssistantBridgeInput) -> Result<()> {
        let payload =
            serde_json::to_string(&message).context("serialize assistant bridge input")?;
        let mut stdin = self
            .assistant
            .stdin
            .lock()
            .map_err(|_| anyhow::anyhow!("lock assistant stdin"))?;
        stdin
            .write_all(payload.as_bytes())
            .context("write assistant bridge input")?;
        stdin
            .write_all(b"\n")
            .context("terminate assistant bridge input")?;
        stdin.flush().context("flush assistant bridge input")?;
        Ok(())
    }

    fn sync_assistant_context(&mut self) {
        let _ = self.send_assistant_message(AssistantBridgeInput {
            kind: "context",
            text: None,
            context: Some(self.assistant_context()),
        });
    }

    fn assistant_context(&self) -> AssistantContext {
        let workspace = self.workspace();
        let project_root = workspace
            .backing
            .as_ref()
            .map(|backing| backing.request.project_root.to_string_lossy().into_owned());
        let selected_component =
            workspace
                .selected_component()
                .map(|component| AssistantSelectedComponent {
                    reference: component.reference.clone(),
                    value: component.value.clone(),
                    object_id: component.object_id.clone(),
                    component_uuid: component.component_uuid.clone(),
                    x_nm: component.position.x,
                    y_nm: component.position.y,
                });
        let selected_review_action =
            workspace
                .selected_review_action()
                .map(|action| AssistantSelectedReviewAction {
                    action_id: action.action_id.clone(),
                    net_name: action.net_name.clone(),
                    layer: action.layer,
                    width_nm: action.width_nm,
                });
        AssistantContext {
            board_name: workspace.scene.board_name.clone(),
            project_name: workspace.scene.project_name.clone(),
            tool: match workspace.tool {
                WorkspaceTool::Select => "select",
            },
            selection: selection_cache_key(workspace),
            active_review_target_id: workspace.active_review_target_id.clone(),
            project_root,
            terminal_activity_summary: workspace.ui.terminal.activity_summary.clone(),
            selected_component,
            selected_review_action,
            components: workspace
                .scene
                .components
                .iter()
                .map(|component| AssistantComponentSummary {
                    reference: component.reference.clone(),
                    value: component.value.clone(),
                    object_id: component.object_id.clone(),
                    component_uuid: component.component_uuid.clone(),
                    x_nm: component.position.x,
                    y_nm: component.position.y,
                })
                .collect(),
            review_actions: workspace
                .review
                .proposal_actions
                .iter()
                .map(|action| AssistantReviewActionSummary {
                    action_id: action.action_id.clone(),
                    net_name: action.net_name.clone(),
                    layer: action.layer,
                    width_nm: action.width_nm,
                })
                .collect(),
        }
    }

    fn apply_assistant_actions(&mut self, actions: &[AssistantBridgeAction]) -> String {
        if actions.is_empty() {
            return String::new();
        }
        let mut outcomes = Vec::new();
        for action in actions {
            let outcome = match action.kind.as_str() {
                "open_dock" => match action.tab.as_deref() {
                    Some("terminal") => {
                        self.set_active_dock(DockTab::Terminal);
                        "opened terminal".to_string()
                    }
                    Some("assistant") => {
                        self.set_active_dock(DockTab::Assistant);
                        "focused assistant".to_string()
                    }
                    _ => "assistant requested unknown dock".to_string(),
                },
                "set_tool" => "tool changes are disabled in read-only M7 review".to_string(),
                "select_component_reference" => {
                    if let Some(reference) = action.reference.as_deref() {
                        let object_id = self
                            .workspace()
                            .scene
                            .components
                            .iter()
                            .find(|component| component.reference.eq_ignore_ascii_case(reference))
                            .map(|component| component.object_id.clone());
                        match object_id {
                            Some(object_id) => {
                                if self.dispatch_session_command(
                                    SessionCommand::SelectAuthoredObject(object_id),
                                ) {
                                    format!("selected {reference}")
                                } else {
                                    format!("could not select {reference}")
                                }
                            }
                            None => format!("component not found: {reference}"),
                        }
                    } else {
                        "assistant omitted component reference".to_string()
                    }
                }
                "select_review_action" => {
                    if let Some(action_id) = action.action_id.as_deref() {
                        if self.dispatch_session_command(SessionCommand::SelectReviewAction(
                            action_id.to_string(),
                        )) {
                            format!("selected review action {action_id}")
                        } else {
                            format!("review action not found: {action_id}")
                        }
                    } else {
                        "assistant omitted review action id".to_string()
                    }
                }
                "move_selected_by"
                | "begin_route_selected_proposal"
                | "apply_selected_route"
                | "cancel_active_edit" => {
                    "mutating assistant actions are disabled in read-only M7 review".to_string()
                }
                "run_terminal_command" => {
                    if let Some(command) = action.command.as_deref() {
                        format!("blocked terminal command `{command}` in read-only mode")
                    } else {
                        "assistant omitted terminal command".to_string()
                    }
                }
                other => format!("assistant requested unsupported action `{other}`"),
            };
            outcomes.push(outcome);
        }
        self.sync_assistant_context();
        outcomes.join("; ")
    }

    fn log_review_event(&mut self, message: impl Into<String>) {
        self.push_terminal_line(message.into());
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
            }
        }
        self.sync_assistant_context();
        self.refresh_terminal_context_snapshot();
        true
    }

    fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
        let previous_retained_key = self.retained_scene_cache_key();
        let result = self.session.apply(command);
        self.apply_session_result(result, Some(previous_retained_key))
    }

    fn needs_redraw(&self) -> bool {
        self.scene_dirty
    }

    fn handle_primary_click(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            self.trace_click("primary click ignored: no cursor position".to_string());
            return false;
        };
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
                    RetainedScene::from_workspace(
                        self.session.workspace(),
                        self.config.width,
                        self.config.height,
                    )
                });
                retained
                    .hit_test_authored_world(world_point, self.session.workspace())
                    .cloned()
            };
            let retained_elapsed = retained_started.elapsed();
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
            HitTarget::AssistantTab => self.open_terminal_agent_launcher(),
            HitTarget::OutputsTab => self.set_active_dock(DockTab::Outputs),
            HitTarget::TerminalActivitySummary(summary) => {
                self.set_active_dock(DockTab::Assistant);
                self.push_assistant_message(
                    "system",
                    format!("selected terminal activity span: {summary}"),
                );
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
                    &self.terminal,
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
                    &self.terminal,
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
            HitTarget::DockResizeHandle => false, // handled in mouse press
        }
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
            &self.terminal,
            "board_text_terminal_command",
            "datum.gui.board_text.edit_prefill",
            "prefill",
            &command,
        ) {
            self.push_terminal_line(format!("terminal handoff event write failed: {err}"));
        }
        self.write_terminal_bytes(command.as_bytes());
        self.invalidate_scene();
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

    fn current_layout(&self) -> ShellLayout {
        ShellLayout::for_window(
            self.config.width,
            self.config.height,
            if self.workspace().ui.active_dock_tab.is_some() {
                Some(self.workspace().ui.dock_height_px)
            } else {
                None
            },
        )
    }

    fn scene_viewport(&self) -> datum_gui_render::RectPx {
        self.current_layout().scene_viewport()
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
        let new_height = (window_height - next_cursor_pos.1).clamp(44.0, window_height * 0.6);
        self.session.workspace_mut().ui.dock_height_px = new_height as u32;
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }

    fn resize_terminal_to_dock(&mut self) {
        let ui = &self.session.workspace().ui;
        let cols = ((self.config.width as f32 - 24.0) / 7.5).floor().max(20.0) as u16;
        let rows = ((ui.dock_height_px as f32 - 76.0) / 16.0).floor().max(4.0) as u16;
        if let Err(err) = self.terminal.resize(cols, rows) {
            self.push_terminal_line(format!("terminal resize failed: {err}"));
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn assistant_activity_command_is_session_scoped() {
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("context session-activity"));
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("$DATUM_SESSION_ID"));
        assert!(ASSISTANT_ACTIVITY_COMMAND.contains("--limit 20"));
        assert_eq!(
            assistant_activity_command_bytes(),
            "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20\r"
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

fn assistant_config_path(request: &LiveReviewRequest) -> PathBuf {
    request.project_root.join(".datum").join("assistant.json")
}

fn load_assistant_config(path: &Path) -> Result<AssistantBridgeConfig> {
    if !path.exists() {
        return Ok(AssistantBridgeConfig::default());
    }
    let text = fs::read_to_string(path)
        .with_context(|| format!("read assistant config {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse assistant config {}", path.display()))
}

fn save_assistant_config(path: &Path, config: &AssistantBridgeConfig) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("assistant config path missing parent"))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create assistant config dir {}", parent.display()))?;
    let text = serde_json::to_string_pretty(config).context("serialize assistant config")?;
    fs::write(path, format!("{text}\n"))
        .with_context(|| format!("write assistant config {}", path.display()))?;
    Ok(())
}

fn redact_assistant_config_command(command: &str) -> String {
    if let Some(rest) = command.strip_prefix("/config api-key ") {
        return format!("/config api-key {}", redact_secret(rest.trim()));
    }
    command.to_string()
}

fn assistant_activity_command_bytes() -> String {
    format!("{ASSISTANT_ACTIVITY_COMMAND}\r")
}

fn redact_secret(secret: &str) -> String {
    if secret.is_empty() {
        return "<empty>".to_string();
    }
    let tail_len = secret.len().min(4);
    let tail = &secret[secret.len() - tail_len..];
    format!("***{tail}")
}

fn best_completion(prefix: &str, candidates: &[&str]) -> Option<String> {
    let trimmed = prefix.trim();
    let mut matches = candidates
        .iter()
        .filter(|candidate| candidate.starts_with(trimmed))
        .copied()
        .collect::<Vec<_>>();
    matches.sort_unstable();
    matches.dedup();
    matches.first().map(|value| (*value).to_string())
}

fn replace_tail(original: &str, tail: &str, replacement: &str) -> String {
    if let Some(prefix) = original.strip_suffix(tail) {
        return format!("{prefix}{replacement}");
    }
    replacement.to_string()
}

fn char_to_byte_pos(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
