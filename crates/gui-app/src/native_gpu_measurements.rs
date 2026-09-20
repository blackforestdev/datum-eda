//! Opt-in measurement lifecycle for the four existing production surface hosts.
//! Does not schedule redraws, change presentation, or claim display latency.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

fn enabled() -> Result<bool> {
    match std::env::var("DATUM_GPU_MEASUREMENTS").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0") => Ok(false),
        Ok("1") => Ok(true),
        other => anyhow::bail!("invalid DATUM_GPU_MEASUREMENTS: {other:?}"),
    }
}

pub(super) fn required_features(adapter: &wgpu::Adapter) -> Result<wgpu::Features> {
    if !enabled()? {
        return Ok(wgpu::Features::empty());
    }
    anyhow::ensure!(
        adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY),
        "GPU timestamp measurement unavailable on adapter"
    );
    Ok(wgpu::Features::TIMESTAMP_QUERY)
}

pub(super) struct Host {
    epoch: u64,
    enabled: bool,
    drawable: bool,
    occluded: bool,
    suspended: bool,
}

impl Host {
    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }

    pub(super) fn new(
        renderer: &mut Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &Window,
        shared_epoch: Option<u64>,
    ) -> Result<Self> {
        static ID: AtomicU64 = AtomicU64::new(1);
        let enabled = enabled()?;
        let id = ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| anyhow::anyhow!("native measurement identity exhausted"))?;
        let epoch = shared_epoch.unwrap_or(id);
        let host = Self {
            epoch,
            enabled,
            drawable: window.inner_size().width > 0 && window.inner_size().height > 0,
            occluded: false,
            suspended: false,
        };
        if enabled {
            renderer.enable_gpu_measurements(device, queue, id, epoch, Box::new(|cancelled| {
                let record = serde_json::json!({"host":cancelled.host,"device_epoch":cancelled.device_epoch,"frame":cancelled.frame,"submission":cancelled.submission,"status":"incomplete","reason":"host_or_device_closed_before_collection"});
                // Always leave a stderr receipt even if the diagnostic file is unavailable.
                eprintln!("gpu_measurement_incomplete {record}");
                let result = std::fs::OpenOptions::new().append(true).open(gui_runtime_support::gui_diagnostic_log_path())
                    .and_then(|mut file| writeln!(file, "gpu_measurement_incomplete {record}"));
                if let Err(error) = result { eprintln!("gpu_measurement_log_failed {error}"); }
            }))?;
            host.sync(renderer);
            append_gui_diagnostic_line(format!(
                "gpu_measurement_host host={id} epoch={epoch} window={:?} timestamp_period_ns={}",
                window.id(),
                queue.get_timestamp_period()
            ));
        }
        Ok(host)
    }

    fn sync(&self, renderer: &mut Renderer) {
        if self.enabled {
            renderer.set_gpu_measurement_visibility(self.drawable, self.occluded, self.suspended);
        }
    }

    fn event(&mut self, renderer: &mut Renderer, event: &WindowEvent) {
        if !self.enabled {
            return;
        }
        match event {
            WindowEvent::Resized(size) => self.drawable = size.width != 0 && size.height != 0,
            WindowEvent::Occluded(occluded) => self.occluded = *occluded,
            _ => return,
        }
        self.sync(renderer);
    }

    fn poll(&self, renderer: &mut Renderer, device: &wgpu::Device) -> Result<Option<Instant>> {
        if !self.enabled {
            return Ok(None);
        }
        for sample in renderer.poll_gpu_measurements(device)? {
            // Existing native diagnostic output includes process and adapter identity.
            // A write error fails the trial instead of silently losing a sample.
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(gui_runtime_support::gui_diagnostic_log_path())?;
            writeln!(
                file,
                "gpu_measurement {}",
                serde_json::json!({
                    "host": sample.host, "device_epoch": sample.device_epoch,
                    "frame": sample.frame, "submission": sample.submission,
                    "timestamp_period_ns": sample.period_ns, "raw_ticks": sample.raw_ticks,
                    "passes_ns": sample.passes_ns, "own_pass_sum_ns": sample.own_pass_sum_ns,
                    "frame_span_ns": sample.frame_span_ns,
                })
            )?;
        }
        Ok(renderer.gpu_measurement_poll_deadline())
    }
}

impl App {
    pub(super) fn measurement_window_event(&mut self, id: WindowId, event: &WindowEvent) {
        if let Some(runtime) = self.runtime.as_mut() {
            if runtime.window.id() == id {
                runtime.measurements.event(&mut runtime.renderer, event);
            }
        }
        for (surface, window) in [
            (
                &mut self.global_preferences_surface,
                &self.global_preferences_window,
            ),
            (
                &mut self.project_preferences_surface,
                &self.project_preferences_window,
            ),
            (&mut self.new_project_surface, &self.new_project_window),
        ] {
            if let (Some(surface), Some(window)) = (surface, window) {
                if window.id() == id {
                    surface.measurements.event(&mut surface.renderer, event);
                }
            }
        }
    }

    pub(super) fn measurement_suspend(&mut self, suspended: bool) {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.measurements.suspended = suspended;
            runtime.measurements.sync(&mut runtime.renderer);
        }
        for surface in [
            &mut self.global_preferences_surface,
            &mut self.project_preferences_surface,
            &mut self.new_project_surface,
        ]
        .into_iter()
        .flatten()
        {
            surface.measurements.suspended = suspended;
            surface.measurements.sync(&mut surface.renderer);
        }
    }

    pub(super) fn service_gpu_measurements(&mut self) -> Result<Option<Instant>> {
        let Some(runtime) = self.runtime.as_mut() else {
            return Ok(None);
        };
        let mut next = runtime
            .measurements
            .poll(&mut runtime.renderer, &runtime.device)?;
        for surface in [
            &mut self.global_preferences_surface,
            &mut self.project_preferences_surface,
            &mut self.new_project_surface,
        ]
        .into_iter()
        .flatten()
        {
            if let Some(due) = surface
                .measurements
                .poll(&mut surface.renderer, &runtime.device)?
            {
                next = Some(next.map_or(due, |old| old.min(due)));
            }
        }
        Ok(next)
    }
}
