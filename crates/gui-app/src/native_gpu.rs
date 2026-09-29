//! Shared native device creation and explicit measurement feature admission.
#[global_allocator]
static TEXT_ACCOUNTING_ALLOCATOR: datum_gui_render::cpu_alloc::Allocator =
    datum_gui_render::cpu_alloc::Allocator;

use super::*;

pub(super) type Bundle = (
    wgpu::Instance,
    wgpu::Surface<'static>,
    wgpu::Adapter,
    wgpu::Device,
    wgpu::Queue,
);

/// Borrowed GPU ownership shared by initial and staged auxiliary construction.
/// No application/model state or authority is available through this view.
#[derive(Clone, Copy)]
pub(super) struct DeviceView<'a> {
    pub instance: &'a wgpu::Instance,
    pub adapter: &'a wgpu::Adapter,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub health: &'a native_device_recovery::DeviceHealth,
    pub transaction: &'a SurfaceTransaction,
    pub epoch: u64,
}

impl Runtime {
    pub(super) fn native_device_view(&self) -> DeviceView<'_> {
        DeviceView {
            instance: &self.instance,
            adapter: &self.adapter,
            device: &self.device,
            queue: &self.queue,
            config: &self.config,
            health: &self.device_health,
            transaction: &self.surface_transaction,
            epoch: self.measurements.epoch(),
        }
    }
}

pub(super) async fn create(window: std::sync::Arc<Window>) -> Result<Bundle> {
    append_gui_diagnostic_line("wgpu instance create begin");
    let instance = gui_runtime_support::diagnostic_instance();
    let surface = instance
        .create_surface(window.clone())
        .context("create surface")?;
    append_gui_diagnostic_line("wgpu request adapter begin");
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .context("request adapter")?;
    gui_runtime_support::log_surface_identity(&window, &adapter);
    append_gui_diagnostic_line("wgpu request device begin");
    let adapter_format_features =
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES & adapter.features();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("datum-m7-spike-device"),
            required_features: adapter_format_features
                | super::native_gpu_measurements::required_features(&adapter)?,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        })
        .await
        .context("request device")?;
    append_gui_diagnostic_line("wgpu request device end");
    Ok((instance, surface, adapter, device, queue))
}

#[cfg(all(test, target_os = "linux"))]
mod regional_capability_tests {
    #[test]
    #[ignore = "r4 group4 native X11/Vulkan copy-target capability; single approved batch only"]
    #[allow(deprecated)] // Hidden capability fixture; production uses ActiveEventLoop.
    fn regional_reference_surface_supports_copy_destination() {
        use winit::platform::x11::EventLoopBuilderExtX11;
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
            .with_x11()
            .with_any_thread(true)
            .build()
            .unwrap();
        let window = std::sync::Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_visible(false)
                        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
                )
                .unwrap(),
        );
        let (_instance, surface, adapter, _device, _queue) =
            pollster::block_on(super::create(window.clone())).unwrap();
        let identity = adapter.get_info();
        assert!(identity.name.contains("P630"));
        assert_eq!(identity.backend, wgpu::Backend::Vulkan);
        let caps = surface.get_capabilities(&adapter);
        assert!(
            caps.usages.contains(wgpu::TextureUsages::COPY_DST),
            "reference native surface refuses COPY_DST"
        );
        let config =
            crate::gui_runtime_support::surface_configuration(&caps, window.inner_size(), None);
        assert!(
            config
                .usage
                .contains(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT)
        );
        assert!(
            adapter
                .get_texture_format_features(config.format)
                .flags
                .sample_count_supported(8)
        );
        eprintln!(
            "r4 native capability passed: {:?}, {:?}, {:?}",
            identity.name, config.format, config.usage
        );
    }
}
