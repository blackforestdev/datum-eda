use anyhow::{Context, Result};
use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use clap::Parser;
use datum_gui_protocol::{
    DockTab, LiveDesignSession, LiveReviewRequest, PointNm, SceneBounds,
    SessionCommand, SessionEvent, WorkspaceTool, ensure_known_good_demo_request,
    load_live_workspace_state,
};
use datum_gui_render::{
    CameraState, HitTarget, PreparedScene, Renderer, RetainedScene, ShellLayout,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

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
}

struct App {
    args: GuiArgs,
    window: Option<&'static Window>,
    runtime: Option<Runtime>,
}

struct TerminalSession {
    rx: Receiver<String>,
}

struct AssistantSession {
    _child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    rx: Receiver<AssistantBridgeOutput>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AssistantBridgeConfig {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct AssistantBridgeInput {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<AssistantContext>,
}

#[derive(Debug, Deserialize)]
struct AssistantBridgeOutput {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    actions: Vec<AssistantBridgeAction>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AssistantBridgeAction {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    tab: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    action_id: Option<String>,
    #[serde(default)]
    dx_nm: Option<i64>,
    #[serde(default)]
    dy_nm: Option<i64>,
    #[serde(default)]
    command: Option<String>,
}

#[derive(Debug, Serialize)]
struct AssistantContext {
    board_name: String,
    project_name: String,
    tool: &'static str,
    selection: String,
    active_review_target_id: String,
    project_root: Option<String>,
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
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Datum M7 Spike")
                    .with_inner_size(LogicalSize::new(1280.0, 800.0)),
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
                        runtime.invalidate_scene();
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
                        state: ElementState::Released,
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
                        state: ElementState::Released,
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
                    let input_was_empty = runtime
                        .current_dock_input()
                        .map_or(true, |s| s.is_empty());
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
                    runtime.fit_camera();
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
            } if text.eq_ignore_ascii_case("t") => {
                if let Some(runtime) = &mut self.runtime
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
                    if let Err(err) = runtime.render() {
                        fatal_gui_error(event_loop, "render failed", err);
                    }
                }
            }
            _ => {}
        }
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
    prepared_scene: Option<PreparedScene>,
    scene_dirty: bool,
    terminal: TerminalSession,
    assistant: AssistantSession,
    assistant_config_path: PathBuf,
    assistant_config: AssistantBridgeConfig,
    clipboard: Option<Clipboard>,
}

impl Runtime {
    fn workspace(&self) -> &datum_gui_protocol::ReviewWorkspaceState {
        self.session.workspace()
    }

    async fn new(window: &'static Window, args: &GuiArgs) -> Result<Self> {
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
        let msaa_samples = if want_msaa8.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
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
        let renderer = Renderer::new(&device, &queue, config.format, msaa_samples);
        let request = args
            .resolve_request()
            .context("resolve GUI launch review context")?;
        let assistant_config_path = assistant_config_path(&request);
        let assistant_config =
            load_assistant_config(&assistant_config_path).with_context(|| {
                format!("load assistant config {}", assistant_config_path.display())
            })?;
        let state =
            load_live_workspace_state(&request).context("load live M7 review workspace state")?;
        let camera = CameraState::fit_to_bounds(&state.scene.bounds);
        let terminal = spawn_terminal_session().context("spawn integrated terminal lane")?;
        let assistant = spawn_assistant_session(&assistant_config)
            .context("spawn integrated assistant lane")?;
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
            prepared_scene: None,
            scene_dirty: true,
            terminal,
            assistant,
            assistant_config_path,
            assistant_config,
            clipboard: Clipboard::new().ok(),
        };
        runtime.sync_assistant_context();
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
        self.invalidate_scene();
    }

    fn render(&mut self) -> Result<()> {
        let frame = self
            .surface
            .get_current_texture()
            .context("acquire next surface texture")?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
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
            .context("retained scene should exist before render")?;
        let prepared = self
            .prepared_scene
            .as_ref()
            .context("prepared scene should exist before render")?;
        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            &prepared,
            retained,
            self.config.width,
            self.config.height,
        )?;
        frame.present();
        Ok(())
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

    fn invalidate_scene(&mut self) {
        self.retained_scene = None;
        self.prepared_scene = None;
        self.scene_dirty = true;
    }

    fn invalidate_frame(&mut self) {
        self.prepared_scene = None;
        self.scene_dirty = true;
    }

    fn poll_terminal_output(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.terminal.rx.try_recv() {
                Ok(line) => {
                    self.push_terminal_line(line);
                    changed = true;
                }
                Err(TryRecvError::Empty) => return changed,
                Err(TryRecvError::Disconnected) => {
                    self.push_terminal_line("terminal session ended".to_string());
                    return true;
                }
            }
        }
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
            let overflow = ui.terminal.lines.len() - 240;
            ui.terminal.lines.drain(0..overflow);
        }
        ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    fn push_assistant_message(&mut self, role: &str, content: String) {
        let ui = &mut self.session.workspace_mut().ui;
        ui.assistant.transcript.push(datum_gui_protocol::AssistantMessage {
            role: role.to_string(),
            content,
        });
        if ui.assistant.transcript.len() > 80 {
            let overflow = ui.assistant.transcript.len() - 80;
            ui.assistant.transcript.drain(0..overflow);
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

    fn is_paste_shortcut(&self, event: &KeyEvent) -> bool {
        if event.state != ElementState::Released {
            return false;
        }
        if self.modifiers.control_key() {
            if let PhysicalKey::Code(KeyCode::KeyV) = event.physical_key {
                return true;
            }
        }
        self.modifiers.shift_key() && matches!(event.logical_key, Key::Named(NamedKey::Insert))
    }

    fn is_copy_shortcut(&self, event: &KeyEvent) -> bool {
        event.state == ElementState::Released
            && self.modifiers.control_key()
            && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyC))
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
        let active = self.workspace().ui.active_dock_tab?;
        match active {
            DockTab::Terminal => None,
            DockTab::Assistant => Some(&self.workspace().ui.assistant.input),
        }
    }

    fn current_dock_input_mut(&mut self) -> Option<&mut String> {
        let active = self.workspace().ui.active_dock_tab?;
        let ui = &mut self.session.workspace_mut().ui;
        match active {
            DockTab::Terminal => None,
            DockTab::Assistant => Some(&mut ui.assistant.input),
        }
    }

    fn copy_dock_input(&mut self) -> bool {
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
            "assistant config: api-key={api_key}, model={model}. Use /config api-key <key> and /config model <model>."
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
            selection: match &workspace.selection {
                datum_gui_protocol::SelectionTarget::None => "none".to_string(),
                datum_gui_protocol::SelectionTarget::ReviewAction(id) => format!("review:{id}"),
                datum_gui_protocol::SelectionTarget::AuthoredObject(id) => format!("object:{id}"),
            },
            active_review_target_id: workspace.active_review_target_id.clone(),
            project_root,
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
                "move_selected_by" | "begin_route_selected_proposal" | "apply_selected_route"
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

    fn apply_session_result(&mut self, result: datum_gui_protocol::SessionCommandResult) -> bool {
        if !result.handled {
            return false;
        }
        for event in result.events {
            match event {
                SessionEvent::SceneChanged => self.invalidate_scene(),
                SessionEvent::SelectionChanged(_) => self.invalidate_scene(),
                SessionEvent::FrameChanged => self.invalidate_frame(),
            }
        }
        self.sync_assistant_context();
        true
    }

    fn dispatch_session_command(&mut self, command: SessionCommand) -> bool {
        let result = self.session.apply(command);
        self.apply_session_result(result)
    }

    fn needs_redraw(&self) -> bool {
        self.scene_dirty
    }

    fn handle_primary_click(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        let prepared = self.prepared_scene();
        if let Some(target) = prepared.hit_test(x, y).cloned() {
            return self.select_hit_target(&target);
        }
        if let Some(world_point) = prepared.world_point_at_screen(x, y) {
            let retained = self.retained_scene.get_or_insert_with(|| {
                RetainedScene::from_workspace(
                    self.session.workspace(),
                    self.config.width,
                    self.config.height,
                )
            });
            if let Some(target) = retained.hit_test_authored_world(world_point).cloned() {
                return self.select_hit_target(&target);
            }
        }
        false
    }

    fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::ReviewAction(action_id) => {
                let handled =
                    self.dispatch_session_command(SessionCommand::SelectReviewAction(action_id.clone()));
                if handled {
                    self.log_review_event(format!("selected review action {action_id}"));
                }
                handled
            }
            HitTarget::AuthoredObject(object_id) => {
                let handled = self
                    .dispatch_session_command(SessionCommand::SelectAuthoredObject(object_id.clone()));
                if handled {
                    self.session.workspace_mut().ui.hovered_object_id = None;
                    self.log_review_event(format!("selected authored object {object_id}"));
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
                let handled = self
                    .dispatch_session_command(SessionCommand::ToggleLayerVisibility(layer_id.clone()));
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
            HitTarget::TerminalTab => self.set_active_dock(DockTab::Terminal),
            HitTarget::AssistantTab => self.set_active_dock(DockTab::Assistant),
            HitTarget::DockResizeHandle => false, // handled in mouse press
        }
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
        self.invalidate_scene();
        true
    }

    fn handle_dock_scroll(&mut self, scroll_lines: f32) -> bool {
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        let delta = if scroll_lines > 0.0 { 1_usize } else { 0_usize };
        let ui = &mut self.session.workspace_mut().ui;
        match active {
            DockTab::Terminal => {
                if scroll_lines > 0.0 {
                    // scroll up (more history)
                    let max = ui.terminal.lines.len();
                    ui.terminal.scroll_offset = (ui.terminal.scroll_offset + delta).min(max);
                } else {
                    // scroll down (toward latest)
                    ui.terminal.scroll_offset = ui.terminal.scroll_offset.saturating_sub(1);
                }
            }
            DockTab::Assistant => {
                if scroll_lines > 0.0 {
                    let max = ui.assistant.transcript.len();
                    ui.assistant.scroll_offset = (ui.assistant.scroll_offset + delta).min(max);
                } else {
                    ui.assistant.scroll_offset = ui.assistant.scroll_offset.saturating_sub(1);
                }
            }
        }
        self.invalidate_frame();
        true
    }
}

fn spawn_terminal_session() -> Result<TerminalSession> {
    let (_tx, rx) = mpsc::channel();
    Ok(TerminalSession { rx })
}

fn spawn_assistant_session(config: &AssistantBridgeConfig) -> Result<AssistantSession> {
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/datum_assistant_bridge.py");
    let mut command = Command::new("/usr/bin/python3");
    command
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(api_key) = &config.api_key {
        command.env("OPENAI_API_KEY", api_key);
    }
    if let Some(model) = &config.model {
        command.env("DATUM_ASSISTANT_MODEL", model);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("spawn assistant bridge {}", script.display()))?;
    let stdin = child.stdin.take().context("take assistant bridge stdin")?;
    let stdout = child
        .stdout
        .take()
        .context("take assistant bridge stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("take assistant bridge stderr")?;
    let stdin = Arc::new(Mutex::new(stdin));
    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                match serde_json::from_str::<AssistantBridgeOutput>(&line) {
                    Ok(message) => {
                        let _ = tx.send(message);
                    }
                    Err(err) => {
                        let _ = tx.send(AssistantBridgeOutput {
                            kind: "error".to_string(),
                            message: format!("assistant bridge emitted invalid JSON: {err}"),
                            actions: Vec::new(),
                        });
                    }
                }
            }
        });
    }
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx.send(AssistantBridgeOutput {
                kind: "error".to_string(),
                message: line,
                actions: Vec::new(),
            });
        }
    });
    Ok(AssistantSession {
        _child: child,
        stdin,
        rx,
    })
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
