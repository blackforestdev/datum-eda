//! Opt-in resize transaction candidate shared by every native surface host.
//! GPU completion is a resource-ownership signal, not compositor display proof.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

pub(crate) struct SurfaceTransaction {
    enabled: bool,
    drawable: bool,
    configured: Option<(u32, u32)>,
    completed: Arc<AtomicBool>,
    retry_at: Option<Instant>,
    failures: u32,
    gpu_wait_attempts: u16,
}

impl SurfaceTransaction {
    pub(crate) fn new(
        config: &wgpu::SurfaceConfiguration,
        native_size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let enabled = match std::env::var("DATUM_DIAGNOSTIC_RESIZE_TRANSACTION").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => false,
            Ok("1") => true,
            other => panic!("invalid DATUM_DIAGNOSTIC_RESIZE_TRANSACTION: {other:?}"),
        };
        Self {
            enabled,
            drawable: native_size.width != 0 && native_size.height != 0,
            configured: (native_size.width != 0 && native_size.height != 0)
                .then_some((config.width, config.height)),
            completed: Arc::new(AtomicBool::new(true)),
            retry_at: None,
            failures: 0,
            gpu_wait_attempts: 0,
        }
    }

    pub(crate) fn configured_for(&self, width: u32, height: u32) -> bool {
        self.configured == Some((width, height))
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.drawable = width != 0 && height != 0;
        if !self.drawable {
            self.retry_at = None;
            self.failures = 0;
            self.gpu_wait_attempts = 0;
        }
    }

    pub(crate) fn configure_resize(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if !self.enabled {
            self.configure(surface, device, config);
        }
    }

    fn configure(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) {
        super::append_gui_diagnostic_line("surface configure begin");
        let probe = super::phase_probe::Probe::start("configure");
        surface.configure(device, config);
        drop(probe);
        super::append_gui_diagnostic_line("surface configure end");
        self.configured = Some((config.width, config.height));
    }

    pub(crate) fn begin_frame(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> anyhow::Result<bool> {
        if !self.enabled {
            return Ok(true);
        }
        if !self.drawable {
            return Ok(false);
        }
        self.check_retry_budget()?;
        if self.failures > 0 && self.retry_at.is_some_and(|due| due > Instant::now()) {
            return Ok(false);
        }
        if !self.completed.load(Ordering::Acquire) {
            device.poll(wgpu::PollType::Poll)?;
            if !self.completed.load(Ordering::Acquire) {
                self.note_gpu_wait()?;
                // Wake only while a requested frame waits for resource ownership.
                self.retry_at
                    .get_or_insert_with(|| Instant::now() + Duration::from_millis(2));
                return Ok(false);
            }
        }
        self.gpu_wait_attempts = 0;
        self.retry_at = None;
        if self.configured != Some((config.width, config.height)) {
            self.configure(surface, device, config);
        }
        Ok(true)
    }

    fn check_retry_budget(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.failures < 8,
            "surface acquisition failed eight attempts; stopping resize retry episode"
        );
        anyhow::ensure!(
            self.gpu_wait_attempts <= 1024,
            "GPU completion did not arrive in 1024 frame attempts; stopping resize retry episode"
        );
        Ok(())
    }

    fn note_gpu_wait(&mut self) -> anyhow::Result<()> {
        self.gpu_wait_attempts = self.gpu_wait_attempts.saturating_add(1);
        self.check_retry_budget()
    }

    pub(crate) fn presented(&mut self, queue: &wgpu::Queue) {
        if !self.enabled {
            return;
        }
        self.failures = 0;
        self.gpu_wait_attempts = 0;
        self.retry_at = None;
        self.completed.store(false, Ordering::Release);
        let completed = Arc::clone(&self.completed);
        queue.on_submitted_work_done(move || completed.store(true, Ordering::Release));
    }

    pub(crate) fn acquisition_failed(&mut self, reconfigure: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if reconfigure {
            self.configured = None;
        }
        self.failures = self.failures.saturating_add(1);
        let delay = (8_u64 << self.failures.saturating_sub(1).min(5)).min(250);
        self.retry_at = self
            .drawable
            .then(|| Instant::now() + Duration::from_millis(delay));
        true
    }

    pub(crate) fn poll_retry(&mut self, now: Instant) -> (bool, Option<Instant>) {
        match self.retry_at {
            Some(due) if due <= now => {
                self.retry_at = None;
                (true, None)
            }
            due => (false, due),
        }
    }
}

impl crate::App {
    pub(crate) fn service_surface_retries(&mut self) -> Option<Instant> {
        let now = Instant::now();
        let (redraw, mut next) = self
            .runtime
            .as_mut()
            .map(|runtime| runtime.surface_transaction.poll_retry(now))
            .unwrap_or((false, None));
        if redraw {
            self.request_main_redraw_if_needed();
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
                let (redraw, due) = surface.surface_transaction.poll_retry(now);
                if redraw {
                    self.frames.invalidate(window);
                }
                if let Some(due) = due {
                    next = Some(next.map_or(due, |old| old.min(due)));
                }
            }
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn transaction() -> SurfaceTransaction {
        SurfaceTransaction {
            enabled: true,
            drawable: true,
            configured: Some((1280, 800)),
            completed: Arc::new(AtomicBool::new(true)),
            retry_at: None,
            failures: 0,
            gpu_wait_attempts: 0,
        }
    }
    #[test]
    fn exhausted_retry_episodes_fail_without_clearing_pending_configuration() {
        let mut state = transaction();
        for _ in 0..7 {
            state.acquisition_failed(true);
            assert!(state.check_retry_budget().is_ok());
        }
        state.acquisition_failed(true);
        assert!(state.check_retry_budget().is_err());
        assert_eq!(state.configured, None);
        let mut gpu = transaction();
        for _ in 0..1024 {
            assert!(gpu.note_gpu_wait().is_ok());
        }
        assert!(gpu.note_gpu_wait().is_err());
        assert_eq!(gpu.configured, Some((1280, 800)));
    }

    #[test]
    fn failed_acquisition_has_one_retry_and_keeps_reconfiguration_required() {
        let mut state = transaction();
        assert!(state.acquisition_failed(true));
        let due = state.retry_at.unwrap();
        assert_eq!(state.configured, None);
        assert_eq!(
            state.poll_retry(due - Duration::from_nanos(1)),
            (false, Some(due))
        );
        assert_eq!(state.poll_retry(due), (true, None));
        assert_eq!(state.poll_retry(due), (false, None));
        assert_eq!(state.configured, None);
    }
    #[test]
    fn zero_extent_cancels_retry_until_native_size_returns() {
        let mut state = transaction();
        state.acquisition_failed(false);
        state.resize(0, 800);
        assert!(!state.drawable);
        assert_eq!(
            state.poll_retry(Instant::now() + Duration::from_secs(1)),
            (false, None)
        );
        state.resize(1280, 800);
        assert!(state.drawable);
        assert_eq!(state.configured, Some((1280, 800)));
    }
    #[test]
    fn host_retry_and_completion_ownership_are_independent() {
        let mut first = transaction();
        let mut second = transaction();
        first.completed.store(false, Ordering::Release);
        first.acquisition_failed(true);
        assert!(second.completed.load(Ordering::Acquire));
        assert_eq!(
            second.poll_retry(Instant::now() + Duration::from_secs(1)),
            (false, None)
        );
        assert_eq!(second.configured, Some((1280, 800)));
        drop(first);
        assert!(second.completed.load(Ordering::Acquire));
    }
}
