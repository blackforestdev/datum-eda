//! Diagnostic-only independent native loop: no Datum scene, layout, or renderer.
//! A screen-space grid makes movement visible; presentation calls are not display proof.
use crate::gui_runtime_support as support;
use anyhow::{Context, Result};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub(super) fn run_if_requested() -> Option<Result<()>> {
    match std::env::var("DATUM_DIAGNOSTIC_MINIMAL_WINDOW").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0") => None,
        Ok("1") => Some(run()),
        other => Some(Err(anyhow::anyhow!(
            "invalid minimal-window flag: {other:?}"
        ))),
    }
}

fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = MinimalApp::default();
    event_loop.run_app(&mut app)?;
    app.error.map_or(Ok(()), Err)
}

#[derive(Default)]
struct MinimalApp {
    state: Option<State>,
    error: Option<anyhow::Error>,
}

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    pending: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("DATUM MINIMAL RESIZE REPRO - FIXED PIXEL GRID")
                    .with_inner_size(winit::dpi::LogicalSize::new(1280, 800)),
            )?,
        );
        let instance = support::diagnostic_instance();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        support::log_surface_identity(&window, &adapter);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("minimal-resize-repro"),
                required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    & adapter.features(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let pending = window.inner_size();
        let config =
            support::surface_configuration(&surface.get_capabilities(&adapter), pending, None);
        surface.configure(&device, &config);
        support::append_gui_diagnostic_line(format!(
            "initial surface configure begin {}x{}",
            config.width, config.height
        ));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fixed-pixel-grid"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
@vertex fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    let p = array<vec2f, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
    return vec4f(p[index], 0., 1.);
}
@fragment fn fs(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let cell = vec2u(p.xy) / 32u;
    let bright = ((cell.x + cell.y) % 2u) == 0u;
    return select(vec4f(0.06, 0.09, 0.12, 1.), vec4f(0.35, 0.65, 0.75, 1.), bright);
}
"#
                .into(),
            ),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fixed-pixel-grid"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(config.format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        window.request_redraw();
        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            pending,
        })
    }

    fn render(&mut self) -> Result<()> {
        if self.pending.width == 0 || self.pending.height == 0 {
            return Ok(());
        }
        if (self.config.width, self.config.height) != (self.pending.width, self.pending.height) {
            self.config.width = self.pending.width;
            self.config.height = self.pending.height;
            support::append_gui_diagnostic_line("surface configure begin");
            let probe = support::phase_probe::Probe::start("minimal_configure");
            self.surface.configure(&self.device, &self.config);
            drop(probe);
            support::append_gui_diagnostic_line("surface configure end");
        }
        // Stop on acquisition error: this diagnostic must not hide recovery churn.
        let probe = support::phase_probe::Probe::start("minimal_acquire");
        let frame = self
            .surface
            .get_current_texture()
            .context("minimal surface acquisition")?;
        drop(probe);
        let probe = support::phase_probe::Probe::start("minimal_encode");
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fixed-pixel-grid"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }
        let commands = encoder.finish();
        drop(probe);
        let probe = support::phase_probe::Probe::start("minimal_submit");
        self.queue.submit([commands]);
        drop(probe);
        let probe = support::phase_probe::Probe::start("minimal_present");
        self.window.pre_present_notify();
        frame.present();
        drop(probe);
        eprintln!("[datum-timing] runtime render diagnostic_minimal=true");
        Ok(())
    }
}

impl ApplicationHandler for MinimalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        match pollster::block_on(State::new(event_loop)) {
            Ok(state) => self.state = Some(state),
            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut().filter(|s| s.window.id() == id) else {
            return;
        };
        if let Some(label) = support::window_event_diagnostic_label(&event) {
            support::append_gui_verbose_diagnostic_line(|| format!("window event {label}"));
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                support::append_gui_diagnostic_line(format!(
                    "resize apply {}x{} -> {}x{}",
                    state.pending.width, state.pending.height, size.width, size.height
                ));
                state.pending = size;
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = state.render() {
                    self.error = Some(error);
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
