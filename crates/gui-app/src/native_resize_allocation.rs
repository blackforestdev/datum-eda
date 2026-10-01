//! Shared opt-in resize storage reuse (PM052) and preserved diagnostics.
//! Logical sizes remain authoritative; only presentation storage is rounded.
use std::time::{Duration, Instant};

pub(super) struct ResizeAllocation {
    enabled: bool,
    retain_peak: bool,
    peak_extent: Option<[u32; 2]>,
    last_extent: [u32; 2],
    quiet: Option<Duration>,
    deadline: Option<Instant>,
    output: Option<(winit::monitor::MonitorHandle, u64, Option<u32>)>,
}

impl ResizeAllocation {
    pub(super) fn new(window: &winit::window::Window) -> Self {
        let (enabled, retain_peak) =
            match std::env::var("DATUM_DIAGNOSTIC_RESIZE_ALLOCATION").as_deref() {
                Err(std::env::VarError::NotPresent) | Ok("exact") => (false, false),
                Ok("quantized") => (true, false),
                Ok("quantized-retained") => (true, true),
                other => panic!("invalid DATUM_DIAGNOSTIC_RESIZE_ALLOCATION: {other:?}"),
            };
        let size = window.inner_size();
        Self {
            enabled,
            retain_peak,
            peak_extent: None,
            last_extent: [size.width, size.height],
            quiet: None,
            deadline: None,
            output: None,
        }
    }

    pub(super) fn initialize_output(&mut self, window: &winit::window::Window) -> bool {
        if !self.enabled {
            return false;
        }
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let wayland = window
            .window_handle()
            .is_ok_and(|handle| matches!(handle.as_raw(), RawWindowHandle::Wayland(_)));
        let scale = window.scale_factor();
        let output = window.current_monitor().map(|monitor| {
            let refresh = monitor.refresh_rate_millihertz();
            (monitor, scale.to_bits(), refresh)
        });
        let quiet = quiet_duration(wayland, scale, output.as_ref().and_then(|o| o.2));
        let changed = self.output != output;
        let invalidated = self.refresh_context(changed, quiet);
        self.output = output;
        if changed && let Some(quiet) = quiet {
            super::super::append_gui_diagnostic_line(format!(
                "DIAGNOSTIC resize allocation quantum=32 refresh_millihertz={} quiet_display_intervals=3 scale={scale} monitor={:?} retain_peak={} quiet_ns={}",
                self.output.as_ref().unwrap().2.unwrap(),
                window.current_monitor().and_then(|m| m.name()),
                self.retain_peak,
                quiet.as_nanos()
            ));
        }
        invalidated
    }

    fn refresh_context(&mut self, changed: bool, quiet: Option<Duration>) -> bool {
        let invalidated = (changed || quiet.is_none()) && self.deadline.is_some();
        if changed || quiet.is_none() {
            self.cancel();
        }
        self.quiet = quiet;
        invalidated
    }

    fn reset_output(&mut self) {
        self.cancel();
        self.quiet = None;
        self.output = None;
    }

    pub(super) fn resized(&mut self, extent: [u32; 2], now: Instant) {
        if extent.contains(&0) {
            self.cancel();
        } else if extent != self.last_extent
            && self.enabled
            && let Some(quiet) = self.quiet
        {
            if self.retain_peak {
                let mut peak = self.peak_extent.unwrap_or(self.last_extent);
                for axis in 0..2 {
                    if extent[axis] > peak[axis] {
                        peak[axis] = round_storage(extent[axis]);
                    }
                }
                self.peak_extent = Some(peak);
            }
            self.deadline = Some(now + quiet);
        }
        self.last_extent = extent;
    }

    pub(super) fn extent(&self, logical: [u32; 2]) -> [u32; 2] {
        if self.deadline.is_some() {
            if let Some(peak) = self.peak_extent {
                assert!(peak[0] >= logical[0] && peak[1] >= logical[1]);
                peak
            } else {
                logical.map(round_storage)
            }
        } else {
            logical
        }
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(super) fn settle(&mut self, now: Instant) -> bool {
        if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.cancel();
            true
        } else {
            false
        }
    }

    pub(super) fn cancel(&mut self) {
        self.deadline = None;
        self.peak_extent = None;
    }

    pub(super) fn configured(&self, logical: [u32; 2], physical: [u32; 2]) {
        if self.enabled {
            super::super::append_gui_diagnostic_line(format!(
                "diagnostic raster configure logical={}x{} buffer={}x{}",
                logical[0], logical[1], physical[0], physical[1]
            ));
        }
    }
}

fn round_storage(value: u32) -> u32 {
    value
        .div_ceil(32)
        .checked_mul(32)
        .expect("resize extent overflow")
}

fn quiet_duration(wayland: bool, scale: f64, refresh: Option<u32>) -> Option<Duration> {
    if !wayland || !scale.is_finite() || scale <= 0.0 || scale.fract() == 0.0 {
        return None;
    }
    refresh
        .filter(|&value| value != 0)
        .map(|value| Duration::from_nanos(3_000_000_000_000 / u64::from(value)))
}

impl super::SurfaceTransaction {
    pub(crate) fn enable_resize_buffer_reuse(&mut self) {
        self.resize_allocation.enabled = true;
        self.resize_allocation.retain_peak = true;
        self.resize_allocation.reset_output();
    }
    pub(crate) fn inherit_resize_allocation_policy(&mut self, other: &Self) {
        self.resize_allocation.enabled = other.resize_allocation.enabled;
        self.resize_allocation.retain_peak = other.resize_allocation.retain_peak;
        self.resize_allocation.reset_output();
    }
    pub(crate) fn reset_resize_output(&mut self) {
        self.resize_allocation.reset_output();
    }
    pub(crate) fn refresh_resize_output(&mut self, window: &winit::window::Window) -> bool {
        self.resize_allocation.initialize_output(window)
    }

    pub(crate) fn render_extent(&self, config: &wgpu::SurfaceConfiguration) -> [u32; 2] {
        self.resize_allocation.extent([config.width, config.height])
    }
}

impl crate::App {
    pub(crate) fn service_resize_allocation(&mut self) {
        if !self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.surface_transaction.resize_allocation.enabled)
        {
            return;
        }
        self.sync_surface_drawability();
        if let Some(runtime) = self.runtime.as_mut() {
            let output_changed = runtime
                .surface_transaction
                .refresh_resize_output(&runtime.window);
            if runtime
                .surface_transaction
                .resize_allocation
                .settle(Instant::now())
                || output_changed
            {
                runtime.invalidate_surface_size();
                self.frames.invalidate(&runtime.window);
            }
            self.frames
                .wake_at(runtime.surface_transaction.resize_allocation.deadline());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_output_is_exact_and_context_change_releases_old_peak() {
        assert!(quiet_duration(false, 1.05, Some(60_000)).is_none());
        assert!(quiet_duration(true, 1.0, Some(60_000)).is_none());
        assert!(quiet_duration(true, 1.05, None).is_none());
        assert!(quiet_duration(true, 1.05, Some(0)).is_none());
        let quiet = quiet_duration(true, 1.05, Some(60_000));
        let now = Instant::now();
        let mut state = ResizeAllocation {
            enabled: true,
            retain_peak: true,
            peak_extent: None,
            last_extent: [1344, 840],
            quiet,
            deadline: None,
            output: None,
        };
        state.resized([1400, 840], now);
        assert_eq!(state.extent(state.last_extent), [1408, 840]);
        assert!(!state.refresh_context(false, quiet));
        assert!(state.refresh_context(true, quiet_duration(true, 1.25, Some(120_000))));
        assert_eq!(state.extent(state.last_extent), [1400, 840]);
        state.resized([1420, 840], now);
        assert!(state.refresh_context(false, None));
        assert_eq!(state.extent(state.last_extent), [1420, 840]);
        state.resized([1500, 840], now);
        assert!(state.deadline.is_none());
        state.reset_output();
        assert!(state.quiet.is_none() && state.peak_extent.is_none());
    }

    #[test]
    fn storage_rounding_never_changes_input_extent_and_settles_once() {
        let now = Instant::now();
        let mut state = ResizeAllocation {
            enabled: true,
            retain_peak: false,
            peak_extent: None,
            last_extent: [1344, 840],
            quiet: Some(Duration::from_millis(50)),
            deadline: None,
            output: None,
        };
        assert_eq!(state.extent([1344, 840]), [1344, 840]);
        state.resized([1347, 840], now);
        assert_eq!(state.last_extent, [1347, 840]);
        assert_eq!(state.extent(state.last_extent), [1376, 864]);
        state.resized([1357, 840], now + Duration::from_millis(40));
        assert!(!state.settle(now + Duration::from_millis(50)));
        assert!(state.settle(now + Duration::from_millis(90)));
        assert_eq!(state.extent(state.last_extent), [1357, 840]);
        assert!(!state.settle(now + Duration::from_secs(1)));
        assert!(state.deadline().is_none());
        state.resized([1358, 840], now);
        state.resized([0, 0], now);
        assert!(state.deadline().is_none());
    }
    #[test]
    fn peak_storage_reuses_shrink_and_preserves_static_axis_then_releases() {
        let now = Instant::now();
        let mut state = ResizeAllocation {
            enabled: true,
            retain_peak: true,
            peak_extent: None,
            last_extent: [1344, 840],
            quiet: Some(Duration::from_millis(50)),
            deadline: None,
            output: None,
        };
        state.resized([1339, 840], now);
        assert_eq!(state.extent(state.last_extent), [1344, 840]);
        state.resized([1400, 840], now + Duration::from_millis(1));
        assert_eq!(state.extent(state.last_extent), [1408, 840]);
        state.resized([1347, 840], now + Duration::from_millis(2));
        assert_eq!(state.extent(state.last_extent), [1408, 840]);
        state.resized([1347, 900], now + Duration::from_millis(3));
        assert_eq!(state.last_extent, [1347, 900]);
        assert_eq!(state.extent(state.last_extent), [1408, 928]);
        assert!(state.settle(now + Duration::from_millis(53)));
        assert_eq!(state.extent(state.last_extent), [1347, 900]);
        assert!(state.peak_extent.is_none());
        state.resized([1348, 900], now + Duration::from_millis(54));
        assert_eq!(state.extent(state.last_extent), [1376, 900]);
        state.resized([0, 0], now + Duration::from_millis(55));
        assert!(state.peak_extent.is_none() && state.deadline().is_none());
    }
}
