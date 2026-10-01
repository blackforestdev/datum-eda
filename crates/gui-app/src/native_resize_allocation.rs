//! Disabled-by-default current-host resize allocation diagnostic.
//! Logical sizes remain authoritative; only presentation storage is rounded.
use std::time::{Duration, Instant};

pub(super) struct ResizeAllocation {
    enabled: bool,
    last_extent: [u32; 2],
    quiet: Option<Duration>,
    deadline: Option<Instant>,
}

impl ResizeAllocation {
    pub(super) fn new(window: &winit::window::Window) -> Self {
        let enabled = match std::env::var("DATUM_DIAGNOSTIC_RESIZE_ALLOCATION").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("exact") => false,
            Ok("quantized") => true,
            other => panic!("invalid DATUM_DIAGNOSTIC_RESIZE_ALLOCATION: {other:?}"),
        };
        if enabled {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            assert!(matches!(
                window.window_handle().unwrap().as_raw(),
                RawWindowHandle::Wayland(_)
            ));
            assert!(
                window.scale_factor().fract() != 0.0,
                "diagnostic requires fractional-scale viewporter"
            );
        }
        let size = window.inner_size();
        Self {
            enabled,
            last_extent: [size.width, size.height],
            quiet: None,
            deadline: None,
        }
    }

    pub(super) fn initialize_output(&mut self, window: &winit::window::Window) {
        if !self.enabled || self.quiet.is_some() {
            return;
        }
        // A Wayland surface receives an output association after mapping its
        // first buffer. Stay exact until that actual output is known.
        let Some(monitor) = window.current_monitor() else {
            return;
        };
        let Some(refresh) = monitor.refresh_rate_millihertz() else {
            return;
        };
        assert!(refresh != 0);
        self.quiet = Some(Duration::from_nanos(3_000_000_000_000 / u64::from(refresh)));
        super::super::append_gui_diagnostic_line(format!(
            "DIAGNOSTIC resize allocation quantum=32 refresh_millihertz={refresh} quiet_display_intervals=3 scale={} monitor={:?}",
            window.scale_factor(),
            monitor.name()
        ));
    }

    pub(super) fn resized(&mut self, extent: [u32; 2], now: Instant) {
        if extent.contains(&0) {
            self.cancel();
        } else if extent != self.last_extent
            && self.enabled
            && let Some(quiet) = self.quiet
        {
            self.deadline = Some(now + quiet);
        }
        self.last_extent = extent;
    }

    pub(super) fn extent(&self, logical: [u32; 2]) -> [u32; 2] {
        if self.deadline.is_some() {
            logical.map(|v| {
                v.div_ceil(32)
                    .checked_mul(32)
                    .expect("resize extent overflow")
            })
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

impl super::SurfaceTransaction {
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
            runtime
                .surface_transaction
                .resize_allocation
                .initialize_output(&runtime.window);
            if runtime
                .surface_transaction
                .resize_allocation
                .settle(Instant::now())
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
    fn storage_rounding_never_changes_input_extent_and_settles_once() {
        let now = Instant::now();
        let mut state = ResizeAllocation {
            enabled: true,
            last_extent: [1344, 840],
            quiet: Some(Duration::from_millis(50)),
            deadline: None,
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
}
