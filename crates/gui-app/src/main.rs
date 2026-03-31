use anyhow::{Context, Result};
use clap::Parser;
use datum_gui_protocol::{
    LiveReviewRequest, ensure_known_good_demo_request, load_live_workspace_state,
};
use datum_gui_render::{HitTarget, PreparedScene, Renderer};
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

fn main() -> Result<()> {
    let args = GuiArgs::parse();
    let event_loop = EventLoop::new().context("failed to create event loop")?;
    let mut app = App::new(args);
    event_loop.run_app(&mut app).context("failed to run app")
}

#[derive(Debug, Clone, Parser)]
#[command(name = "datum-gui", about = "Datum M7 route-proposal review spike")]
struct GuiArgs {
    #[arg(long = "demo-known-good", default_value_t = false)]
    demo_known_good: bool,
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

impl App {
    fn new(args: GuiArgs) -> Self {
        Self {
            args,
            window: None,
            runtime: None,
        }
    }
}

impl GuiArgs {
    fn resolve_request(&self) -> Result<LiveReviewRequest> {
        if self.demo_known_good {
            return ensure_known_good_demo_request();
        }
        Ok(LiveReviewRequest {
            project_root: self.project_root.clone().ok_or_else(|| {
                anyhow::anyhow!("--project-root is required unless --demo-known-good is used")
            })?,
            net_uuid: self.net_uuid.clone().ok_or_else(|| {
                anyhow::anyhow!("--net is required unless --demo-known-good is used")
            })?,
            from_anchor_pad_uuid: self.from_anchor_pad_uuid.clone().ok_or_else(|| {
                anyhow::anyhow!("--from-anchor is required unless --demo-known-good is used")
            })?,
            to_anchor_pad_uuid: self.to_anchor_pad_uuid.clone().ok_or_else(|| {
                anyhow::anyhow!("--to-anchor is required unless --demo-known-good is used")
            })?,
            profile: self.profile.clone(),
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Datum M7 Spike")
                    .with_inner_size(LogicalSize::new(1280.0, 800.0)),
            )
            .expect("window creation should succeed");
        let window_ref: &'static Window = Box::leak(Box::new(window));
        let runtime = pollster::block_on(Runtime::new(window_ref, &self.args))
            .expect("runtime creation should succeed");
        window_ref.request_redraw();
        self.runtime = Some(runtime);
        self.window = Some(window_ref);
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
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window) {
                    runtime.resize(size.width, size.height);
                    if runtime.needs_redraw() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.last_cursor_pos = Some((position.x as f32, position.y as f32));
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window) {
                    if runtime.handle_primary_click() {
                        runtime.invalidate_scene();
                        window.request_redraw();
                    }
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
                if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window) {
                    if !matches!(
                        runtime.state.selection,
                        datum_gui_protocol::SelectionTarget::None
                    ) {
                        runtime.state.clear_selection();
                        runtime.invalidate_scene();
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(runtime) = &mut self.runtime {
                    runtime.render().expect("render should succeed");
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
    state: datum_gui_protocol::ReviewWorkspaceState,
    last_cursor_pos: Option<(f32, f32)>,
    prepared_scene: Option<PreparedScene>,
    scene_dirty: bool,
}

impl Runtime {
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
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("datum-m7-spike-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .context("request device")?;
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
        let renderer = Renderer::new(&device, &queue, config.format);
        let request = args
            .resolve_request()
            .context("resolve GUI launch review context")?;
        let state =
            load_live_workspace_state(&request).context("load live M7 review workspace state")?;
        Ok(Self {
            surface,
            device,
            queue,
            config,
            renderer,
            state,
            last_cursor_pos: None,
            prepared_scene: None,
            scene_dirty: true,
        })
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
        let prepared = self.prepared_scene().clone();
        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            &prepared,
            self.config.width,
            self.config.height,
        )?;
        frame.present();
        Ok(())
    }

    fn prepared_scene(&mut self) -> &PreparedScene {
        self.prepared_scene.get_or_insert_with(|| {
            self.scene_dirty = false;
            PreparedScene::from_workspace(&self.state, self.config.width, self.config.height)
        })
    }

    fn invalidate_scene(&mut self) {
        self.prepared_scene = None;
        self.scene_dirty = true;
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
        false
    }

    fn select_hit_target(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::ReviewAction(action_id) => self.state.select_review_action(action_id),
            HitTarget::AuthoredObject(object_id) => self.state.select_authored_object(object_id),
            HitTarget::TerminalTab | HitTarget::AssistantTab => false,
        }
    }
}
