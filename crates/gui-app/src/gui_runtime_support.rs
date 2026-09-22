#[path = "native_frame_probe.rs"]
pub(crate) mod native_frame_probe;
#[path = "native_queue_owner.rs"]
mod native_queue_owner;
#[path = "native_recovery.rs"]
pub(crate) mod native_recovery;
#[path = "native_surface_transaction.rs"]
pub(crate) mod native_surface_transaction;
#[path = "phase_probe.rs"]
pub(crate) mod phase_probe;
#[path = "presented_hit_regions.rs"]
pub(crate) mod presented_hit_regions;

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[cfg(feature = "visual")]
use anyhow::Result;
use winit::event::WindowEvent;

pub(crate) fn install_gui_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        append_gui_diagnostic_line(format!("panic: {panic_info}"));
        eprintln!("datum-gui panic: {panic_info}");
    }));
}

pub(crate) fn gui_diagnostic_log_path() -> PathBuf {
    std::env::var_os("DATUM_GUI_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("datum-gui-last.log"))
}

pub(crate) fn reset_gui_diagnostic_log() {
    let path = gui_diagnostic_log_path();
    let _ = fs::write(
        path,
        format!(
            "datum-gui diagnostic log pid={} start={:?}\n",
            std::process::id(),
            std::time::SystemTime::now()
        ),
    );
}

pub(crate) fn append_gui_diagnostic_line(message: impl AsRef<str>) {
    let path = gui_diagnostic_log_path();
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(
        file,
        "[{:?}] {}",
        std::time::SystemTime::now(),
        message.as_ref()
    );
}

/// Construct diagnostics only when enabled; callers must keep message-only work
/// inside the closure so disabled tracing adds no formatting or string allocation.
pub(crate) fn append_gui_verbose_diagnostic_line<M: AsRef<str>>(message: impl FnOnce() -> M) {
    if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_some() {
        append_gui_diagnostic_line(message());
    }
}

pub(crate) fn window_event_diagnostic_label(event: &WindowEvent) -> Option<String> {
    match event {
        WindowEvent::CloseRequested => Some("close requested".to_string()),
        WindowEvent::Destroyed => Some("destroyed".to_string()),
        WindowEvent::Resized(size) => Some(format!("resized {}x{}", size.width, size.height)),
        WindowEvent::Focused(focused) => Some(format!("focused {focused}")),
        WindowEvent::Occluded(occluded) => Some(format!("occluded {occluded}")),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(format!("scale factor {scale_factor}"))
        }
        WindowEvent::RedrawRequested => Some("redraw requested".to_string()),
        WindowEvent::MouseInput { state, button, .. } => {
            Some(format!("mouse input {button:?} {state:?}"))
        }
        WindowEvent::MouseWheel { delta, phase, .. } => {
            Some(format!("mouse wheel delta={delta:?} phase={phase:?}"))
        }
        WindowEvent::CursorMoved { position, .. } => {
            Some(format!("cursor moved {:.2},{:.2}", position.x, position.y))
        }
        WindowEvent::KeyboardInput {
            event,
            is_synthetic,
            ..
        } => Some(format!(
            "keyboard physical={:?} logical={:?} state={:?} repeat={} text={:?} synthetic={is_synthetic}",
            event.physical_key, event.logical_key, event.state, event.repeat, event.text,
        )),
        WindowEvent::Ime(ime) => Some(format!("ime {ime:?}")),
        WindowEvent::ModifiersChanged(modifiers) => {
            Some(format!("modifiers changed {:?}", modifiers.state()))
        }
        _ => None,
    }
}

pub(crate) fn trace_startup_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-startup] {message}");
    }
}

pub(crate) fn log_surface_identity(window: &winit::window::Window, adapter: &wgpu::Adapter) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let backend = match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Wayland(_)) => "wayland",
        Ok(RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) => "x11",
        Ok(_) => "other",
        Err(_) => "unavailable",
    };
    let info = adapter.get_info();
    append_gui_diagnostic_line(format!(
        "surface identity window={:?} window_backend={backend} gpu_backend={:?} adapter={:?} device_type={:?} driver={:?} driver_info={:?}",
        window.id(),
        info.backend,
        info.name,
        info.device_type,
        info.driver,
        info.driver_info
    ));
}

pub(crate) fn select_msaa_samples(adapter: &wgpu::Adapter, format: wgpu::TextureFormat) -> u32 {
    let format_features = adapter.get_texture_format_features(format);
    let supported = format_features.flags.supported_sample_counts();
    [8, 4, 1]
        .into_iter()
        .find(|sample_count| supported.contains(sample_count))
        .unwrap_or(1)
}

#[cfg(feature = "visual")]
pub(crate) fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

#[cfg(feature = "visual")]
pub(crate) fn convert_texture_pixels_to_rgba(
    pixels: &mut [u8],
    format: wgpu::TextureFormat,
) -> Result<()> {
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

/// Explicit measurement override using backends already built into wgpu.
/// Ordinary startup retains the existing adapter-selection policy.
pub(crate) fn diagnostic_instance() -> wgpu::Instance {
    let backends = match std::env::var("DATUM_GPU_DIAGNOSTIC_BACKEND").as_deref() {
        Ok("gl") => wgpu::Backends::GL,
        Ok("vulkan") => wgpu::Backends::VULKAN,
        Err(std::env::VarError::NotPresent) => return wgpu::Instance::default(),
        other => panic!("invalid DATUM_GPU_DIAGNOSTIC_BACKEND: {other:?}; expected gl or vulkan"),
    };
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    })
}

/// Common native-host configuration. Owned windows can prefer the main format;
/// otherwise retain the existing sRGB/FIFO selection and capability fallbacks.
pub(crate) fn surface_configuration(
    caps: &wgpu::SurfaceCapabilities,
    size: winit::dpi::PhysicalSize<u32>,
    preferred: Option<wgpu::TextureFormat>,
) -> wgpu::SurfaceConfiguration {
    let format = preferred
        .filter(|format| caps.formats.contains(format))
        .or_else(|| {
            caps.formats
                .iter()
                .copied()
                .find(wgpu::TextureFormat::is_srgb)
        })
        .unwrap_or(caps.formats[0]);
    let present_mode = caps
        .present_modes
        .iter()
        .copied()
        .find(|mode| *mode == wgpu::PresentMode::Fifo)
        .unwrap_or(caps.present_modes[0]);
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: diagnostic_frame_latency(),
    }
}

/// Bounded swapchain-depth experiment; absent flag preserves the product default.
fn diagnostic_frame_latency() -> u32 {
    match std::env::var("DATUM_GPU_DIAGNOSTIC_LATENCY").as_deref() {
        Err(std::env::VarError::NotPresent) => 2,
        Ok(value @ ("1" | "2")) => {
            append_gui_diagnostic_line(format!("diagnostic maximum frame latency hint={value}"));
            value.parse().expect("validated latency")
        }
        other => panic!("invalid DATUM_GPU_DIAGNOSTIC_LATENCY: {other:?}; expected 1 or 2"),
    }
}
