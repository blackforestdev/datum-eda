//! Shared surface acquisition/configuration using coordinator-owned recovery.
//! GPU completion is a resource-ownership signal, not compositor display proof.
use super::native_recovery::{RecoveryHandle, RetryReason};
use std::{cell::Cell, rc::Rc, time::Instant};

pub(crate) struct SurfaceTransaction {
    injected_fault: Option<InjectedFault>,
    queue_owner: super::native_queue_owner::QueueOwner,
    queue_host: u64,
    in_flight: u64,
    pub(crate) recovery: RecoveryHandle,
    drawable: bool,
    configured: Option<(u32, u32)>,
    configuration_generation: u64,
    texture_active: Rc<Cell<bool>>,
}

/// Presentation ownership is deliberately separate from GPU completion. Dropping
/// an unpresented frame releases acquisition; it does not retire application damage.
pub(crate) struct NativeSurfaceFrame {
    // Field order drops the backend texture before releasing the acquisition lease.
    texture: Option<wgpu::SurfaceTexture>,
    lease: TextureLease,
}

impl NativeSurfaceFrame {
    pub(crate) fn view(&self) -> wgpu::TextureView {
        self.texture
            .as_ref()
            .expect("unconsumed native frame")
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}

struct TextureLease {
    active: Rc<Cell<bool>>,
    generation: u64,
}
impl TextureLease {
    fn acquire(active: &Rc<Cell<bool>>, generation: u64) -> anyhow::Result<Self> {
        anyhow::ensure!(!active.replace(true), "native surface already acquired");
        Ok(Self {
            active: active.clone(),
            generation,
        })
    }
    fn belongs_to(&self, active: &Rc<Cell<bool>>, generation: u64) -> bool {
        Rc::ptr_eq(&self.active, active) && self.generation == generation
    }
}
impl Drop for TextureLease {
    fn drop(&mut self) {
        self.active.set(false);
    }
}

#[derive(Clone, Copy, Debug)]
enum InjectedFault {
    LostOnce,
    Timeout,
    OutOfMemory,
}

impl SurfaceTransaction {
    pub(crate) fn new(
        _config: &wgpu::SurfaceConfiguration,
        native_size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        // The recovered diagnostic implementation is now superseded by the
        // shared default policy; retain validation of the legacy switch.
        match std::env::var("DATUM_DIAGNOSTIC_RESIZE_TRANSACTION").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0" | "1") => {}
            other => panic!("invalid DATUM_DIAGNOSTIC_RESIZE_TRANSACTION: {other:?}"),
        }
        let queue_owner = super::native_queue_owner::QueueOwner::default();
        let queue_host = queue_owner.register();
        let injected_fault = match std::env::var("DATUM_DIAGNOSTIC_SURFACE_FAULT").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => None,
            Ok("lost-once") => Some(InjectedFault::LostOnce),
            Ok("timeout") => Some(InjectedFault::Timeout),
            Ok("oom") => Some(InjectedFault::OutOfMemory),
            other => panic!("invalid DATUM_DIAGNOSTIC_SURFACE_FAULT: {other:?}"),
        };
        Self {
            injected_fault,
            queue_owner,
            queue_host,
            in_flight: 0,
            recovery: RecoveryHandle::default(),
            drawable: native_size.width != 0 && native_size.height != 0,
            configured: None,
            configuration_generation: 0,
            texture_active: Rc::default(),
        }
    }

    pub(crate) fn has_presented(&self) -> bool {
        self.in_flight != 0
    }

    pub(crate) fn configured_for(&self, width: u32, height: u32) -> bool {
        self.configured == Some((width, height))
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        // Native coordinator owns occlusion/suspend; zero extent is also safe
        // for direct diagnostic callers outside native dispatch.
        if width == 0 || height == 0 {
            self.set_drawable(false);
        }
    }

    pub(crate) fn share_queue_with(&mut self, other: &Self) {
        self.queue_owner.cancel(self.queue_host);
        self.queue_owner = other.queue_owner.clone();
        self.queue_host = self.queue_owner.register();
    }

    pub(crate) fn set_drawable(&mut self, drawable: bool) {
        self.drawable = drawable;
        self.recovery
            .borrow_mut()
            .set_drawable(drawable, Instant::now());
        if !drawable || self.recovery.borrow().failed() {
            self.queue_owner.cancel(self.queue_host);
        }
    }

    fn configure(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        health: &crate::native_device_recovery::DeviceHealth,
    ) -> bool {
        super::append_gui_diagnostic_line("surface configure begin");
        let probe = super::phase_probe::Probe::start("configure");
        surface.configure(device, config);
        drop(probe);
        super::append_gui_diagnostic_line("surface configure end");
        if health.failed() {
            return false;
        }
        self.configuration_generation = self
            .configuration_generation
            .checked_add(1)
            .expect("surface configuration generation exhausted");
        self.configured = Some((config.width, config.height));
        self.queue_owner.configured(self.queue_host);
        true
    }

    /// Common acquisition and error classification for every native product host.
    /// Rendering adapters retain their damage when this returns no texture.
    pub(crate) fn acquire(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        health: &crate::native_device_recovery::DeviceHealth,
    ) -> anyhow::Result<Option<NativeSurfaceFrame>> {
        if !self.begin_frame(surface, device, config, health)?
            || health.failed()
            || !self.recovery.borrow_mut().ready(Instant::now())
        {
            return Ok(None);
        }
        let probe = super::phase_probe::Probe::start("acquire");
        // Explicit fault qualification uses the same production classification,
        // recovery owner and native dispatcher as a real backend error.
        let result = if let Some(fault) = self.injected_fault {
            super::append_gui_diagnostic_line(format!(
                "surface fault injected host={} kind={fault:?}",
                self.queue_host
            ));
            match fault {
                InjectedFault::LostOnce => {
                    self.injected_fault = None;
                    Err(wgpu::SurfaceError::Lost)
                }
                InjectedFault::Timeout => Err(wgpu::SurfaceError::Timeout),
                InjectedFault::OutOfMemory => Err(wgpu::SurfaceError::OutOfMemory),
            }
        } else {
            surface.get_current_texture()
        };
        drop(probe);
        match result {
            Ok(frame) if !health.failed() => Ok(Some(NativeSurfaceFrame {
                texture: Some(frame),
                lease: TextureLease::acquire(&self.texture_active, self.configuration_generation)?,
            })),
            Ok(_) => Ok(None),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.configured = None;
                self.recovery
                    .borrow_mut()
                    .defer(RetryReason::Acquisition, Instant::now());
                Ok(None)
            }
            Err(wgpu::SurfaceError::Timeout) => {
                self.recovery
                    .borrow_mut()
                    .defer(RetryReason::Acquisition, Instant::now());
                super::append_gui_diagnostic_line("surface acquire timeout; frame skipped");
                Ok(None)
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                health.allocation_failed();
                Ok(None)
            }
            Err(error) => anyhow::bail!("acquire native surface texture: {error}"),
        }
    }

    pub(crate) fn begin_frame(
        &mut self,
        surface: &wgpu::Surface<'_>,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        health: &crate::native_device_recovery::DeviceHealth,
    ) -> anyhow::Result<bool> {
        anyhow::ensure!(
            !self.texture_active.get(),
            "cannot configure or acquire while a native surface texture is live"
        );
        if !self.drawable {
            return Ok(false);
        }
        if !self.recovery.borrow_mut().ready(Instant::now()) {
            return Ok(false);
        }
        let now = Instant::now();
        device.poll(wgpu::PollType::Poll)?;
        if health.failed() {
            return Ok(false);
        }
        match self.queue_owner.admit(
            self.queue_host,
            self.configured != Some((config.width, config.height)),
            self.in_flight,
        ) {
            super::native_queue_owner::Admission::Wait => {
                self.recovery.borrow_mut().defer(RetryReason::Queue, now);
                return Ok(false);
            }
            super::native_queue_owner::Admission::Configure => {
                if !self.configure(surface, device, config, health) {
                    return Ok(false);
                }
                // The next FIFO ticket owns the queue as soon as ours finishes.
                // Do not submit this host's frame ahead of its configuration.
                if self
                    .queue_owner
                    .admit(self.queue_host, false, self.in_flight)
                    == super::native_queue_owner::Admission::Wait
                {
                    self.recovery.borrow_mut().defer(RetryReason::Queue, now);
                    return Ok(false);
                }
            }
            super::native_queue_owner::Admission::Frame => {}
        }
        Ok(true)
    }

    /// The sole product notification/presentation boundary for acquired native
    /// frames. Success retires consumed damage only in the coordinator afterward.
    pub(crate) fn present(
        &mut self,
        mut frame: NativeSurfaceFrame,
        window: &winit::window::Window,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            frame
                .lease
                .belongs_to(&self.texture_active, self.configuration_generation),
            "native frame belongs to another surface or configuration generation"
        );
        window.pre_present_notify();
        frame
            .texture
            .take()
            .expect("unconsumed native frame")
            .present();
        // Release acquisition before a subsequent configure can be admitted.
        drop(frame);
        self.in_flight = self.queue_owner.submitted(queue);
        self.recovery.borrow_mut().success();
        Ok(())
    }
}

impl crate::App {
    pub(crate) fn sync_surface_drawability(&mut self) {
        if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window) {
            runtime
                .surface_transaction
                .set_drawable(self.frames.is_drawable(window.id()));
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
                surface
                    .surface_transaction
                    .set_drawable(self.frames.is_drawable(window.id()));
            }
        }
    }

    pub(crate) fn service_surface_retries(&mut self) -> Option<Instant> {
        self.sync_surface_drawability();
        if self.device_recovery.pending() {
            return None;
        }
        let now = Instant::now();
        let mut next: Option<Instant> = None;
        let mut failed = Vec::new();
        for window in [
            self.window,
            self.global_preferences_window.as_deref(),
            self.project_preferences_window.as_deref(),
            self.new_project_window.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let (due, reported) = self.frames.service_recovery(window, now);
            if reported {
                failed.push(window.id());
            }
            if let Some(due) = due {
                next = Some(next.map_or(due, |old| old.min(due)));
            }
        }
        for window in failed {
            self.cancel_native_host_gestures(window);
        }
        next
    }

    pub(crate) fn cancel_native_host_gestures(&mut self, window: winit::window::WindowId) {
        self.frames.cancel_capture(window);
        if self.window.is_some_and(|main| main.id() == window)
            && let Some(runtime) = &mut self.runtime
        {
            runtime.pan_gesture.cancel();
            runtime.cancel_terminal_tab_drag();
            // The host coordinator now consumes the cancelled gesture release.
            runtime.terminal_tab_drag_release_suppressed = false;
            runtime.cancel_terminal_text_selection_drag();
            runtime.terminal_split_drag = None;
            runtime.divider_drag = None;
            runtime.dock_drag_active = false;
            runtime.terminal_mouse_button = None;
            self.apply_cursor_icon(winit::window::CursorIcon::Default);
        }
        for (surface, native) in [
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
            if native.as_ref().is_some_and(|native| native.id() == window)
                && let Some(surface) = surface
            {
                surface.cancel_capture();
            }
        }
    }
}

impl Drop for SurfaceTransaction {
    fn drop(&mut self) {
        self.queue_owner.cancel(self.queue_host);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_acquisition_lease_survives_rejected_overlap_and_releases_on_abort() {
        let active = Rc::new(Cell::new(false));
        let first = TextureLease::acquire(&active, 1).unwrap();
        assert!(active.get());
        assert!(TextureLease::acquire(&active, 1).is_err());
        assert!(active.get());
        drop(first); // Preparation/error exit before presentation.
        assert!(!active.get());
        let next = TextureLease::acquire(&active, 2).unwrap();
        assert!(next.belongs_to(&active, 2));
        drop(next);
        assert!(!active.get());
    }

    #[test]
    fn foreign_and_retired_generation_frames_cannot_present_or_clear_new_ownership() {
        let old = Rc::new(Cell::new(false));
        let replacement = Rc::new(Cell::new(false));
        let first = TextureLease::acquire(&old, 7).unwrap();
        let next = TextureLease::acquire(&replacement, 7).unwrap();
        assert!(!first.belongs_to(&old, 8));
        assert!(!first.belongs_to(&replacement, 7));
        assert!(next.belongs_to(&replacement, 7));
        drop(first);
        assert!(replacement.get());
        drop(next);
        assert!(!replacement.get());
    }
}
