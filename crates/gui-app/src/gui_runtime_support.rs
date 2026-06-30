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

pub(crate) fn append_gui_verbose_diagnostic_line(message: impl AsRef<str>) {
    if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_some() {
        append_gui_diagnostic_line(message);
    }
}

pub(crate) fn window_event_diagnostic_label(event: &WindowEvent) -> Option<String> {
    match event {
        WindowEvent::CloseRequested => Some("close requested".to_string()),
        WindowEvent::Destroyed => Some("destroyed".to_string()),
        WindowEvent::Resized(size) => Some(format!("resized {}x{}", size.width, size.height)),
        WindowEvent::Focused(focused) => Some(format!("focused {focused}")),
        WindowEvent::RedrawRequested => Some("redraw requested".to_string()),
        WindowEvent::MouseInput { state, button, .. } => {
            Some(format!("mouse input {button:?} {state:?}"))
        }
        WindowEvent::MouseWheel { .. } => Some("mouse wheel".to_string()),
        WindowEvent::KeyboardInput { event, .. } => Some(format!(
            "keyboard {:?} {:?}",
            event.logical_key, event.state
        )),
        WindowEvent::ModifiersChanged(_) => Some("modifiers changed".to_string()),
        _ => None,
    }
}

pub(crate) fn trace_startup_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-startup] {message}");
    }
}

pub(crate) fn terminal_raw_input_should_handle(
    terminal_accepts_raw_input: bool,
    paste_shortcut: bool,
    copy_shortcut: bool,
) -> bool {
    terminal_accepts_raw_input && !paste_shortcut && !copy_shortcut
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
pub(crate) fn convert_texture_pixels_to_rgba(pixels: &mut [u8], format: wgpu::TextureFormat) -> Result<()> {
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

