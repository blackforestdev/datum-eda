//! Shared surface acquisition/configuration using coordinator-owned recovery.
//! GPU completion is a resource-ownership signal, not compositor display proof.
use super::native_recovery::{AcquisitionFailure, RecoveryHandle, RetryReason};
use std::{cell::Cell, rc::Rc, time::Instant};

pub(crate) struct SurfaceTransaction {
    window: winit::window::WindowId,
    injected_fault: Option<InjectedFault>,
    queue_owner: super::native_queue_owner::QueueOwner,
    queue_host: u64,
    in_flight: u64,
    last_present_receipt: u64,
    last_submission: Option<wgpu::SubmissionIndex>,
    pub(crate) recovery: RecoveryHandle,
    drawable: bool,
    configured: Option<(u32, u32)>,
    configuration_generation: u64,
    texture_active: Rc<FrameCounts>,
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

#[derive(Default)]
struct FrameCounts {
    active: Cell<bool>,
    acquired: Cell<u64>,
    presented: Cell<u64>,
    frame_submissions: Cell<u64>,
    discarded: Cell<u64>,
    configure_attempts: Cell<u64>,
    acquire_attempts: Cell<u64>,
}
fn increment(count: &Cell<u64>) {
    count.set(
        count
            .get()
            .checked_add(1)
            .expect("native lifecycle count exhausted"),
    );
}

struct TextureLease {
    active: Rc<FrameCounts>,
    generation: u64,
    presented: bool,
    submission_receipt: Option<u64>,
}
impl TextureLease {
    fn acquire(active: &Rc<FrameCounts>, generation: u64) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !active.active.replace(true),
            "native surface already acquired"
        );
        increment(&active.acquired);
        Ok(Self {
            active: active.clone(),
            generation,
            presented: false,
            submission_receipt: None,
        })
    }
    fn submitted(&mut self, receipt: u64) {
        assert!(
            self.submission_receipt.is_none(),
            "native acquisition submitted twice"
        );
        self.submission_receipt = Some(receipt);
        increment(&self.active.frame_submissions);
    }

    fn belongs_to(&self, active: &Rc<FrameCounts>, generation: u64) -> bool {
        Rc::ptr_eq(&self.active, active) && self.generation == generation
    }
}
impl Drop for TextureLease {
    fn drop(&mut self) {
        self.active.active.set(false);
        increment(if self.presented {
            &self.active.presented
        } else {
            &self.active.discarded
        });
        debug_assert_eq!(
            self.active.acquired.get(),
            self.active.presented.get() + self.active.discarded.get()
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum InjectedFault {
    LostOnce,
    LostThenOutdated,
    OutdatedOnce,
    OtherOnce,
    Timeout,
    OutOfMemory,
}

impl SurfaceTransaction {
    pub(crate) fn new(
        window: &winit::window::Window,
        health: &crate::native_device_recovery::DeviceHealth,
    ) -> Self {
        let native_size = window.inner_size();
        // The recovered diagnostic implementation is now superseded by the
        // shared default policy; retain validation of the legacy switch.
        match std::env::var("DATUM_DIAGNOSTIC_RESIZE_TRANSACTION").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0" | "1") => {}
            other => panic!("invalid DATUM_DIAGNOSTIC_RESIZE_TRANSACTION: {other:?}"),
        }
        let queue_owner =
            super::native_queue_owner::QueueOwner::with_device_loss(health.device_loss_signal());
        let queue_host = queue_owner.register();
        let injected_fault = match std::env::var("DATUM_DIAGNOSTIC_SURFACE_FAULT").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("0") => None,
            Ok("other-once") => Some(InjectedFault::OtherOnce),
            Ok("lost-once") => Some(InjectedFault::LostOnce),
            Ok("lost-outdated") => Some(InjectedFault::LostThenOutdated),
            Ok("timeout") => Some(InjectedFault::Timeout),
            Ok("oom") => Some(InjectedFault::OutOfMemory),
            other => panic!("invalid DATUM_DIAGNOSTIC_SURFACE_FAULT: {other:?}"),
        };
        Self {
            window: window.id(),
            injected_fault,
            queue_owner,
            queue_host,
            in_flight: 0,
            last_present_receipt: 0,
            last_submission: None,
            recovery: RecoveryHandle::default(),
            drawable: native_size.width != 0 && native_size.height != 0,
            configured: None,
            configuration_generation: 0,
            texture_active: Rc::default(),
        }
    }

    pub(crate) fn has_presented(&self) -> bool {
        self.texture_active.presented.get() != 0
    }

    pub(crate) fn observe_attachment(
        &self,
        renderer: &datum_gui_render::Renderer,
        frame: &NativeSurfaceFrame,
    ) {
        let Some(attachment) = renderer.surface_attachment_snapshot() else {
            return;
        };
        assert!(
            frame
                .lease
                .belongs_to(&self.texture_active, self.configuration_generation)
        );
        self.queue_owner.observe_attachment(
            self.queue_host,
            attachment.owner,
            attachment.allocation,
            attachment.payload_bytes,
            frame.lease.submission_receipt.unwrap_or(0),
        );
        if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_none() {
            return;
        }
        self.queue_owner.trace_attachments();
        let (queue_epoch, _, _) = self.queue_owner.snapshot();
        super::append_gui_diagnostic_line(format!(
            "native surface attachment {}",
            serde_json::json!({
                "window": format!("{:?}", self.window), "host": self.queue_host, "queue_epoch": queue_epoch,
                "owner": attachment.owner, "allocation": attachment.allocation,
                "allocations_created": attachment.allocations_created, "extent": attachment.extent,
                "format": format!("{:?}", attachment.format), "samples": attachment.samples,
                "payload_bytes": attachment.payload_bytes, "state": "current_renderer_reference",
                "gpu_retirement_qualified": false
            })
        ));
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        // Native coordinator owns occlusion/suspend; zero extent is also safe
        // for direct diagnostic callers outside native dispatch.
        if width == 0 || height == 0 {
            self.set_drawable(false);
        }
    }

    pub(crate) fn share_queue_with(&mut self, other: &Self) {
        assert_eq!(
            self.texture_active.acquired.get(),
            0,
            "queue binding must precede acquisition"
        );
        assert_eq!(self.in_flight, 0, "queue binding must precede submission");
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
        increment(&self.texture_active.configure_attempts);
        surface.configure(device, config);
        drop(probe);
        super::append_gui_diagnostic_line("surface configure end");
        if health.failed() {
            self.trace_lifecycle("configure_failed");
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
        increment(&self.texture_active.acquire_attempts);
        let fault = self.injected_fault.filter(|fault| {
            !matches!(fault, InjectedFault::LostThenOutdated) || self.has_presented()
        });
        let result = if let Some(fault) = fault {
            super::append_gui_diagnostic_line(format!(
                "surface fault injected host={} kind={fault:?}",
                self.queue_host
            ));
            match fault {
                InjectedFault::LostOnce => {
                    self.injected_fault = None;
                    Err(wgpu::SurfaceError::Lost)
                }
                InjectedFault::LostThenOutdated => {
                    self.injected_fault = Some(InjectedFault::OutdatedOnce);
                    Err(wgpu::SurfaceError::Lost)
                }
                InjectedFault::OutdatedOnce => {
                    self.injected_fault = None;
                    Err(wgpu::SurfaceError::Outdated)
                }
                InjectedFault::OtherOnce => {
                    self.injected_fault = None;
                    Err(wgpu::SurfaceError::Other)
                }
                InjectedFault::Timeout => Err(wgpu::SurfaceError::Timeout),
                InjectedFault::OutOfMemory => Err(wgpu::SurfaceError::OutOfMemory),
            }
        } else {
            surface.get_current_texture()
        };
        drop(probe);
        match result {
            Ok(frame) => {
                let frame = NativeSurfaceFrame {
                    texture: Some(frame),
                    lease: TextureLease::acquire(
                        &self.texture_active,
                        self.configuration_generation,
                    )?,
                };
                if health.failed() {
                    drop(frame);
                    self.trace_lifecycle("acquired_after_failure");
                    Ok(None)
                } else {
                    Ok(Some(frame))
                }
            }
            Err(error) => {
                let action = self
                    .recovery
                    .borrow_mut()
                    .acquisition_failed(&error, Instant::now());
                match action {
                    AcquisitionFailure::Reconfigure => {
                        self.trace_lifecycle("acquire_lost_or_outdated");
                        self.configured = None;
                    }
                    AcquisitionFailure::Retry => {
                        self.trace_lifecycle("acquire_timeout");
                        super::append_gui_diagnostic_line("surface acquire timeout; frame skipped");
                    }
                    AcquisitionFailure::DeviceAllocation => {
                        self.trace_lifecycle("acquire_oom");
                        health.allocation_failed();
                    }
                    AcquisitionFailure::Fatal => {
                        self.trace_lifecycle("acquire_other_error");
                        anyhow::bail!("acquire native surface texture: {error}");
                    }
                }
                Ok(None)
            }
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
            !self.texture_active.active.get(),
            "cannot configure or acquire while a native surface texture is live"
        );
        if !self.drawable {
            self.trace_lifecycle("skip_undrawable");
            return Ok(false);
        }
        if !self.recovery.borrow_mut().ready(Instant::now()) {
            self.trace_lifecycle("skip_recovery");
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
                self.trace_lifecycle("wait_queue");
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

    /// Called synchronously at the renderer's queue.submit boundary, even when
    /// a later measurement or device-health check prevents presentation.
    pub(crate) fn submitted(
        &mut self,
        frame: &mut NativeSurfaceFrame,
        queue: &wgpu::Queue,
        submission: wgpu::SubmissionIndex,
    ) {
        assert!(
            frame
                .lease
                .belongs_to(&self.texture_active, self.configuration_generation)
        );
        assert!(frame.lease.submission_receipt.is_none());
        self.in_flight = self.queue_owner.submitted(queue);
        self.last_submission = Some(submission);
        frame.lease.submitted(self.in_flight);
        self.trace_lifecycle("submit");
    }

    /// The sole product notification/presentation boundary for acquired native
    /// frames. Success retires consumed damage only in the coordinator afterward.
    pub(crate) fn present(
        &mut self,
        mut frame: NativeSurfaceFrame,
        window: &winit::window::Window,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            window.id() == self.window,
            "native presentation window does not own this surface"
        );
        anyhow::ensure!(
            frame
                .lease
                .belongs_to(&self.texture_active, self.configuration_generation),
            "native frame belongs to another surface or configuration generation"
        );
        anyhow::ensure!(
            frame.lease.submission_receipt == Some(self.in_flight),
            "native frame has no matching submitted work"
        );
        window.pre_present_notify();
        frame
            .texture
            .take()
            .expect("unconsumed native frame")
            .present();
        frame.lease.presented = true;
        // Release acquisition before a subsequent configure can be admitted.
        drop(frame);
        self.last_present_receipt = self.in_flight;
        self.recovery.borrow_mut().success();
        self.trace_lifecycle("present");
        Ok(())
    }

    fn trace_lifecycle(&self, reason: &str) {
        if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_none() {
            return;
        }
        let (epoch, submitted, completed) = self.queue_owner.snapshot();
        let counts = &self.texture_active;
        super::append_gui_diagnostic_line(format!(
            "native_surface_lifecycle {}",
            serde_json::json!({
                "queue_epoch": epoch, "host": self.queue_host, "window": format!("{:?}", self.window), "reason": reason,
                "configuration_generation": self.configuration_generation, "configured_extent": self.configured,
                "configure_attempts": counts.configure_attempts.get(),
                "acquire_attempts": counts.acquire_attempts.get(),
                "acquired": counts.acquired.get(), "presented": counts.presented.get(),
                "discarded": counts.discarded.get(), "active": u8::from(counts.active.get()),
                "last_present_receipt": self.last_present_receipt,
                "last_frame_submission_receipt": self.in_flight, "wgpu_submission": format!("{:?}", self.last_submission),
                "frame_submissions": counts.frame_submissions.get(),
                "frame_submission_receipts_issued": submitted, "gpu_completed_through_receipt": completed
            })
        ));
    }
}

impl SurfaceTransaction {
    pub(super) fn queue_owner(&self) -> &super::native_queue_owner::QueueOwner {
        &self.queue_owner
    }
}

impl crate::App {
    pub(crate) fn sync_surface_drawability(&mut self) {
        if let (Some(runtime), Some(window)) = (&mut self.runtime, self.window.as_deref()) {
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
        for (window, transaction) in [
            (
                self.window.as_deref(),
                self.runtime.as_ref().map(|r| &r.surface_transaction),
            ),
            (
                self.global_preferences_window.as_deref(),
                self.global_preferences_surface
                    .as_ref()
                    .map(|s| &s.surface_transaction),
            ),
            (
                self.project_preferences_window.as_deref(),
                self.project_preferences_surface
                    .as_ref()
                    .map(|s| &s.surface_transaction),
            ),
            (
                self.new_project_window.as_deref(),
                self.new_project_surface
                    .as_ref()
                    .map(|s| &s.surface_transaction),
            ),
        ] {
            let (Some(window), Some(transaction)) = (window, transaction) else {
                continue;
            };
            let admitted = transaction
                .queue_owner
                .retry_ready(transaction.queue_host, transaction.in_flight);
            let (due, reported) = self.frames.service_recovery(window, now, admitted);
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
        if self.window.as_ref().is_some_and(|main| main.id() == window)
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
        self.trace_lifecycle("owner_drop");
        self.queue_owner.close_host(self.queue_host);
        self.queue_owner.trace_attachments();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquired_frames_reconcile_present_discard_and_live_ownership() {
        let counts = Rc::new(FrameCounts::default());
        increment(&counts.acquire_attempts);
        let mut shown = TextureLease::acquire(&counts, 1).unwrap();
        assert_eq!(counts.acquired.get(), 1);
        assert_eq!(counts.presented.get(), 0);
        shown.submitted(1);
        shown.presented = true;
        drop(shown);
        increment(&counts.acquire_attempts); // Backend failure produces no lease.
        increment(&counts.acquire_attempts);
        let abandoned = TextureLease::acquire(&counts, 1).unwrap();
        assert_eq!(
            counts.acquired.get(),
            counts.presented.get() + u64::from(counts.active.get())
        );
        drop(abandoned);
        assert_eq!(counts.acquire_attempts.get(), 3);
        assert_eq!(counts.acquired.get(), 2);
        assert_eq!(counts.presented.get(), 1);
        assert_eq!(counts.frame_submissions.get(), 1);
        assert_eq!(counts.discarded.get(), 1);
        assert!(!counts.active.get());
    }

    #[test]
    fn submitted_frame_discard_preserves_submission_without_claiming_presentation() {
        let counts = Rc::new(FrameCounts::default());
        let mut frame = TextureLease::acquire(&counts, 1).unwrap();
        frame.submitted(7);
        assert_eq!(frame.submission_receipt, Some(7));
        drop(frame); // A post-submit error skips presentation.
        assert_eq!(counts.frame_submissions.get(), 1);
        assert_eq!(counts.presented.get(), 0);
        assert_eq!(counts.discarded.get(), 1);
        assert!(!counts.active.get());
    }

    #[test]
    fn one_acquisition_lease_survives_rejected_overlap_and_releases_on_abort() {
        let active = Rc::new(FrameCounts::default());
        let first = TextureLease::acquire(&active, 1).unwrap();
        assert!(active.active.get());
        assert!(TextureLease::acquire(&active, 1).is_err());
        assert!(active.active.get());
        drop(first); // Preparation/error exit before presentation.
        assert!(!active.active.get());
        let next = TextureLease::acquire(&active, 2).unwrap();
        assert!(next.belongs_to(&active, 2));
        drop(next);
        assert!(!active.active.get());
    }

    #[test]
    fn foreign_and_retired_generation_frames_cannot_present_or_clear_new_ownership() {
        let old = Rc::new(FrameCounts::default());
        let replacement = Rc::new(FrameCounts::default());
        let first = TextureLease::acquire(&old, 7).unwrap();
        let next = TextureLease::acquire(&replacement, 7).unwrap();
        assert!(!first.belongs_to(&old, 8));
        assert!(!first.belongs_to(&replacement, 7));
        assert!(next.belongs_to(&replacement, 7));
        drop(first);
        assert!(replacement.active.get());
        drop(next);
        assert!(!replacement.active.get());
    }
}
