//! One event-loop device replacement attempt; logical application state survives.
use super::*;
use anyhow::Context as _;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(super) struct DeviceHealth(Arc<AtomicU8>);
impl DeviceHealth {
    pub(super) fn observe(
        device: &wgpu::Device,
        wake: winit::event_loop::EventLoopProxy<()>,
    ) -> Self {
        let health = Self(Arc::new(AtomicU8::new(0)));
        let lost = health.clone();
        let lost_wake = wake.clone();
        device.set_device_lost_callback(move |_, _| {
            if lost.signal(1) {
                let _ = lost_wake.send_event(());
            }
        });
        let errors = health.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            let code = match error {
                wgpu::Error::OutOfMemory { .. } => 2,
                wgpu::Error::Validation { .. } => 3,
                wgpu::Error::Internal { .. } => 4,
            };
            if errors.signal(code) {
                let _ = wake.send_event(());
            }
        }));
        health
    }
    pub(super) fn failed(&self) -> bool {
        self.0.load(Ordering::Acquire) != 0
    }
    pub(super) fn allocation_failed(&self) {
        self.signal(2);
    }
    fn mark_failed(&self) {
        self.signal(4);
    }
    fn signal(&self, code: u8) -> bool {
        self.0
            .compare_exchange(0, code, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}
struct EventWake(winit::event_loop::EventLoopProxy<()>);
impl Wake for EventWake {
    fn wake(self: Arc<Self>) {
        let _ = self.0.send_event(());
    }
}
struct Pending {
    future: Pin<Box<dyn Future<Output = Result<native_gpu::Bundle>>>>,
    active: Duration,
    since: Option<Instant>,
}
pub(super) struct DeviceRecovery {
    pending: Option<Pending>,
    attempted: bool,
    reported: bool,
    inject: bool,
    verify_state: bool,
}
impl Default for DeviceRecovery {
    fn default() -> Self {
        let inject = match std::env::var("DATUM_DIAGNOSTIC_DEVICE_LOSS").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => false,
            Ok("once") => true,
            other => panic!("invalid DATUM_DIAGNOSTIC_DEVICE_LOSS: {other:?}"),
        };
        Self {
            pending: None,
            attempted: false,
            reported: false,
            inject,
            verify_state: inject,
        }
    }
}

impl DeviceRecovery {
    pub(super) fn pending(&self) -> bool {
        self.pending.is_some()
    }
}
impl Runtime {
    fn replace_native_gpu(
        &mut self,
        bundle: native_gpu::Bundle,
        wake: winit::event_loop::EventLoopProxy<()>,
    ) -> Result<()> {
        let (instance, surface, adapter, device, queue) = bundle;
        let health = DeviceHealth::observe(&device, wake);
        let config = gui_runtime_support::surface_configuration(
            &surface.get_capabilities(&adapter),
            self.window.inner_size(),
            None,
        );
        let mut renderer = Renderer::new(
            &device,
            &queue,
            config.format,
            select_msaa_samples(&adapter, config.format),
        );
        let measurements =
            native_gpu_measurements::Host::new(&mut renderer, &device, &queue, self.window, None)?;
        anyhow::ensure!(
            !health.failed(),
            "replacement device failed during renderer initialization"
        );
        self.instance = instance;
        self.surface = surface;
        self.adapter = adapter;
        self.device = device;
        self.queue = queue;
        self.config = config;
        self.renderer = renderer;
        self.measurements = measurements;
        self.device_health = health;
        self.surface_transaction = SurfaceTransaction::new(self.window);
        self.invalidate_scene();
        Ok(())
    }
}
impl App {
    pub(super) fn native_device_available(&self) -> bool {
        self.runtime
            .as_ref()
            .is_some_and(|runtime| !runtime.device_health.failed())
    }

    pub(super) fn retry_failed_device(&mut self) -> bool {
        if self
            .runtime
            .as_ref()
            .is_none_or(|runtime| !runtime.device_health.failed())
        {
            return false;
        }
        if self.device_recovery.pending.is_none() {
            self.device_recovery.attempted = false;
            self.device_recovery.reported = false;
        }
        true
    }

    pub(super) fn service_device_recovery(&mut self) -> Option<Instant> {
        let runtime = self.runtime.as_ref()?;
        if !self.frames.contains_host(runtime.window.id()) {
            self.device_recovery.pending = None;
            return None;
        }
        if self.device_recovery.inject && runtime.surface_transaction.has_presented() {
            self.device_recovery.inject = false;
            append_gui_diagnostic_line("native device loss injected");
            runtime.device.destroy();
            // Exercise the registered backend callback, rather than setting the
            // application's failure flag on the diagnostic's behalf.
            let _ = runtime.device.poll(wgpu::PollType::Poll);
        }
        if !runtime.device_health.failed() {
            if runtime.surface_transaction.has_presented()
                || [
                    &self.global_preferences_surface,
                    &self.project_preferences_surface,
                    &self.new_project_surface,
                ]
                .into_iter()
                .flatten()
                .any(|surface| surface.surface_transaction.has_presented())
            {
                self.device_recovery.attempted = false;
            }
            return None;
        }
        self.frames.fail_device();
        if !self.frames.has_drawable_host() {
            if let Some(pending) = &mut self.device_recovery.pending
                && let Some(since) = pending.since.take()
            {
                pending.active += since.elapsed();
            }
            return None;
        }
        if !self.device_recovery.attempted {
            let main_window = runtime.window;
            let fault_code = runtime.device_health.0.load(Ordering::Acquire);
            for window in [
                Some(main_window.id()),
                self.global_preferences_window
                    .as_ref()
                    .map(|window| window.id()),
                self.project_preferences_window
                    .as_ref()
                    .map(|window| window.id()),
                self.new_project_window.as_ref().map(|window| window.id()),
            ]
            .into_iter()
            .flatten()
            {
                self.cancel_native_host_gestures(window);
            }
            self.device_recovery.attempted = true;
            self.device_recovery.pending = Some(Pending {
                future: Box::pin(native_gpu::create(main_window)),
                active: Duration::ZERO,
                since: Some(Instant::now()),
            });
            append_gui_diagnostic_line(format!(
                "native device replacement begin fault_code={fault_code}"
            ));
        }
        let mut pending = self.device_recovery.pending.take()?;
        if let Some(since) = pending.since.replace(Instant::now()) {
            pending.active += since.elapsed();
        }
        if pending.active >= Duration::from_secs(2) {
            self.report_device_recovery_failure(
                "device replacement exceeded two drawable-active seconds",
            );
            return None;
        }
        let waker = Waker::from(Arc::new(EventWake(self.terminal_event_proxy.clone())));
        let result = pending
            .future
            .as_mut()
            .poll(&mut Context::from_waker(&waker));
        let expired = pending.active + pending.since.expect("drawable replacement clock").elapsed()
            >= Duration::from_secs(2);
        match result {
            Poll::Pending if !expired => {
                self.device_recovery.pending = Some(pending);
                Some(Instant::now() + Duration::from_millis(2))
            }
            Poll::Ready(Ok(bundle)) if !expired => {
                match self.install_replacement_device(bundle) {
                    Ok(()) => append_gui_diagnostic_line("native device replacement committed"),
                    Err(error) => self.report_device_recovery_failure(error),
                }
                None
            }
            Poll::Ready(Err(error)) => {
                self.report_device_recovery_failure(error);
                None
            }
            _ => {
                self.report_device_recovery_failure("device replacement exceeded two seconds");
                None
            }
        }
    }

    fn report_device_recovery_failure(&mut self, error: impl std::fmt::Display) {
        if let Some(runtime) = &self.runtime {
            runtime.device_health.mark_failed();
        }
        self.frames.fail_device();
        if !self.device_recovery.reported {
            self.device_recovery.reported = true;
            let message = format!(
                "native device replacement failed: {error}. State retained; press F5 to Retry or close the window."
            );
            append_gui_diagnostic_line(&message);
            eprintln!("datum-gui error: {message}");
        }
    }

    fn install_replacement_device(&mut self, bundle: native_gpu::Bundle) -> Result<()> {
        let runtime = self
            .runtime
            .as_mut()
            .context("runtime closed during device replacement")?;
        let preserved = self.device_recovery.verify_state.then(|| {
            (
                runtime.workspace().clone(),
                std::ptr::addr_of!(runtime.terminal_sessions),
                runtime.terminal_sessions.len(),
            )
        });
        runtime.replace_native_gpu(bundle, self.terminal_event_proxy.clone())?;
        // Stage all auxiliary GPU state before admitting any host again. Existing
        // windows, settings, scroll offsets and engine/terminal state stay owned.
        let global = self
            .global_preferences_surface
            .as_ref()
            .map(|surface| surface.replacement(runtime))
            .transpose()?;
        let project = self
            .project_preferences_surface
            .as_ref()
            .map(|surface| surface.replacement(runtime))
            .transpose()?;
        let new = self
            .new_project_surface
            .as_ref()
            .map(|surface| surface.replacement(runtime))
            .transpose()?;
        anyhow::ensure!(
            !runtime.device_health.failed(),
            "replacement device failed while rebuilding auxiliary renderers"
        );
        self.global_preferences_surface = global;
        self.project_preferences_surface = project;
        self.new_project_surface = new;
        if let Some((workspace, registry, sessions)) = preserved {
            anyhow::ensure!(
                runtime.workspace() == &workspace,
                "device replacement changed workspace state"
            );
            anyhow::ensure!(
                std::ptr::addr_of!(runtime.terminal_sessions) == registry
                    && runtime.terminal_sessions.len() == sessions,
                "device replacement changed terminal registry"
            );
            append_gui_diagnostic_line(
                "native device state preserved workspace=true terminal_registry=true",
            );
        }
        let epoch = runtime.measurements.epoch();
        self.frames.rebind_device(
            runtime.window.id(),
            epoch,
            runtime.surface_transaction.recovery.clone(),
        );
        for (surface, window) in [
            (
                &self.global_preferences_surface,
                &self.global_preferences_window,
            ),
            (
                &self.project_preferences_surface,
                &self.project_preferences_window,
            ),
            (&self.new_project_surface, &self.new_project_window),
        ] {
            if let (Some(surface), Some(window)) = (surface, window) {
                self.frames.rebind_device(
                    window.id(),
                    epoch,
                    surface.surface_transaction.recovery.clone(),
                );
            }
        }
        append_gui_diagnostic_line(format!("native device epoch replaced epoch={epoch}"));
        self.device_recovery.reported = false;
        self.request_redraw_if_needed();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retired_device_error_bursts_cannot_wake_or_fail_replacement_generation() {
        let old = DeviceHealth(Arc::new(AtomicU8::new(0)));
        let callback = old.clone();
        assert!(old.signal(1));
        for _ in 0..1000 {
            assert!(!callback.signal(3));
        }
        let replacement = DeviceHealth(Arc::new(AtomicU8::new(0)));
        drop(old);
        assert!(!callback.signal(2));
        assert!(!replacement.failed());
        assert!(replacement.signal(2));
        assert!(replacement.failed());
    }
}
