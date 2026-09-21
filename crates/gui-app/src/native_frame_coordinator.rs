//! Application-owned native redraw tokens and damage generations.
//!
//! Native events still update input/model state before requesting a frame. A
//! frame retires only its captured damage; host identity prevents late completion
//! from changing a closed or replacement host. Each host owns its recovery clock;
//! queue configuration admission is shared separately by the native device.
use crate::gui_runtime_support::native_recovery::RecoveryHandle;
use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};
use winit::window::{Window, WindowId};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FrameReceipt {
    window: WindowId,
    host_generation: u64,
    device_generation: u64,
    damage: u64,
    attempt: u64,
}

// Counts belong to a host/device generation. Suppression (hidden, recovering,
// or clean) takes precedence over token coalescing. Native tokens may originate
// in the compositor, so received is not expected to equal requested.
#[derive(Debug, Default, serde::Serialize)]
struct RedrawCounts {
    requested: u64,
    coalesced: u64,
    suppressed: u64,
    received: u64,
    duplicate_tokens: u64,
    presentations: u64,
}

#[derive(Debug)]
struct Host {
    counts: RedrawCounts,
    recovery: RecoveryHandle,
    primary_down: bool,
    suppress_release: bool,
    extent: PhysicalSize<u32>,
    occluded: bool,
    minimized: bool,
    restore_pending: bool,
    generation: u64,
    device_generation: u64,
    damage: u64,
    presented: u64,
    pending: bool,
    ready: bool,
    attempt: u64,
    rendering: Option<u64>,
}

#[derive(Default)]
pub(super) struct NativeFrameCoordinator {
    generation: u64,
    hosts: HashMap<WindowId, Host>,
    suspended: bool,
    wake_deadline: Option<Instant>,
    order: VecDeque<WindowId>,
}

impl NativeFrameCoordinator {
    /// Producers contribute eligible work; no producer can postpone another.
    pub(super) fn wake_at(&mut self, deadline: Option<Instant>) {
        if let Some(deadline) = deadline {
            self.wake_deadline = Some(self.wake_deadline.map_or(deadline, |old| old.min(deadline)));
        }
    }

    /// Consume this round's deadlines once at the native wait boundary. The
    /// next round must re-register eligible work, so canceled timers cannot spin.
    pub(super) fn take_control_flow(&mut self) -> winit::event_loop::ControlFlow {
        self.wake_deadline.take().map_or(
            winit::event_loop::ControlFlow::Wait,
            winit::event_loop::ControlFlow::WaitUntil,
        )
    }

    pub(super) fn register(
        &mut self,
        window: WindowId,
        device_generation: u64,
        extent: PhysicalSize<u32>,
        recovery: RecoveryHandle,
    ) {
        recovery.borrow_mut().set_drawable(
            !self.suspended && extent.width != 0 && extent.height != 0,
            Instant::now(),
        );
        self.generation = self
            .generation
            .checked_add(1)
            .expect("host generation exhausted");
        if !self.hosts.contains_key(&window) {
            self.order.push_back(window);
        }
        if let Some(previous) = self.hosts.remove(&window) {
            previous.trace(window, "replaced");
        }
        self.hosts.insert(
            window,
            Host {
                counts: RedrawCounts::default(),
                recovery,
                primary_down: false,
                suppress_release: false,
                extent,
                occluded: false,
                minimized: false,
                restore_pending: false,
                generation: self.generation,
                device_generation,
                damage: 1,
                presented: 0,
                pending: false,
                ready: false,
                attempt: 0,
                rendering: None,
            },
        );
    }

    fn drawable(host: &Host, suspended: bool) -> bool {
        !suspended
            && !host.occluded
            && !host.minimized
            && host.extent.width != 0
            && host.extent.height != 0
    }

    /// Observe native visibility without applying, dropping or coalescing input.
    /// Actual content handlers still reconcile size, DPI and authoritative state.
    pub(super) fn window_event(&mut self, window: WindowId, event: &WindowEvent) {
        if matches!(event, WindowEvent::Destroyed) {
            self.close(window);
            return;
        }
        let Some(host) = self.hosts.get_mut(&window) else {
            return;
        };
        let changed = match event {
            WindowEvent::Resized(size) if *size != host.extent => {
                host.extent = *size;
                true
            }
            WindowEvent::Occluded(occluded) if *occluded != host.occluded => {
                host.occluded = *occluded;
                true
            }
            _ => false,
        };
        if !changed {
            return;
        }
        Self::visibility_changed(host, self.suspended);
    }

    pub(super) fn observe_minimized(&mut self, window: WindowId, minimized: bool) {
        let Some(host) = self.hosts.get_mut(&window) else {
            return;
        };
        if host.minimized == minimized {
            return;
        }
        host.minimized = minimized;
        Self::visibility_changed(host, self.suspended);
        crate::append_gui_verbose_diagnostic_line(|| {
            format!("native host {window:?} minimized {minimized}")
        });
    }

    fn visibility_changed(host: &mut Host, suspended: bool) {
        host.damage = host
            .damage
            .checked_add(1)
            .expect("damage generation exhausted");
        host.restore_pending = Self::drawable(host, suspended);
        host.recovery
            .borrow_mut()
            .set_drawable(host.restore_pending, Instant::now());
        if !host.restore_pending {
            host.pending = false;
            host.ready = false;
        }
    }

    pub(super) fn set_suspended(&mut self, suspended: bool) {
        if self.suspended == suspended {
            return;
        }
        self.suspended = suspended;
        for host in self.hosts.values_mut() {
            host.damage = host
                .damage
                .checked_add(1)
                .expect("damage generation exhausted");
            host.restore_pending = Self::drawable(host, suspended);
            host.recovery
                .borrow_mut()
                .set_drawable(host.restore_pending, Instant::now());
            if suspended {
                host.pending = false;
                host.ready = false;
            }
        }
    }

    /// Flush only visibility/extent restoration. Failed acquisitions are still
    /// owned by recovery, so this cannot create an immediate retry loop.
    pub(super) fn request_restored(&mut self, window: &Window) {
        if self.take_restore_request(window.id()) {
            window.request_redraw();
        }
    }

    fn take_restore_request(&mut self, window: WindowId) -> bool {
        let Some(host) = self.hosts.get_mut(&window) else {
            return false;
        };
        if !host.restore_pending || !Self::drawable(host, self.suspended) {
            return false;
        }
        host.restore_pending = false;
        Self::request(host, self.suspended)
    }

    /// Cancel the gesture without turning its trailing release into a click.
    pub(super) fn cancel_capture(&mut self, window: WindowId) {
        if let Some(host) = self.hosts.get_mut(&window) {
            host.suppress_release |= host.primary_down;
        }
    }

    /// Native primary-button tracking is shared by all adapters. A new press
    /// starts a new gesture even if a cancelled release was never delivered.
    pub(super) fn consume_cancelled_release(
        &mut self,
        window: WindowId,
        event: &WindowEvent,
    ) -> bool {
        let Some(host) = self.hosts.get_mut(&window) else {
            return false;
        };
        if let WindowEvent::MouseInput {
            state,
            button: MouseButton::Left,
            ..
        } = event
        {
            match state {
                ElementState::Pressed => {
                    host.primary_down = true;
                    host.suppress_release = false;
                }
                ElementState::Released => {
                    host.primary_down = false;
                    return std::mem::take(&mut host.suppress_release);
                }
            }
        }
        false
    }

    pub(super) fn is_drawable(&self, window: WindowId) -> bool {
        self.hosts
            .get(&window)
            .is_some_and(|host| Self::drawable(host, self.suspended))
    }

    pub(super) fn has_drawable_host(&self) -> bool {
        self.hosts
            .values()
            .any(|host| Self::drawable(host, self.suspended))
    }

    pub(super) fn contains_host(&self, window: WindowId) -> bool {
        self.hosts.contains_key(&window)
    }

    pub(super) fn service_recovery(
        &mut self,
        window: &Window,
        now: Instant,
        admitted: bool,
    ) -> (Option<Instant>, bool) {
        let Some(host) = self.hosts.get_mut(&window.id()) else {
            return (None, false);
        };
        let (ready, due) = host.recovery.borrow_mut().poll_admitted(now, admitted);
        let failed = host.recovery.borrow_mut().take_failure();
        if failed {
            host.pending = false;
            host.ready = false;
            host.restore_pending = false;
            let message = format!(
                "Rendering paused for native host {:?}: automatic recovery stopped. Press F5 to Retry, or close this window. Application state is retained.",
                window.id()
            );
            crate::append_gui_diagnostic_line(&message);
            eprintln!("datum-gui error: {message}");
        }
        if ready && Self::request(host, self.suspended) {
            window.request_redraw();
        }
        (due, failed)
    }

    /// Stop only this host on a non-device render error. Input state and other
    /// hosts survive; the ordinary recovery reporter exposes Retry/Close.
    pub(super) fn render_failed(&mut self, window: WindowId, error: &impl std::fmt::Display) {
        if let Some(host) = self.hosts.get_mut(&window) {
            host.recovery.borrow_mut().fail();
            host.pending = false;
            host.ready = false;
            host.restore_pending = false;
            crate::append_gui_diagnostic_line(format!(
                "native host {window:?} render error: {error}"
            ));
        }
    }

    pub(super) fn fail_device(&mut self) {
        for host in self.hosts.values_mut() {
            host.recovery.borrow_mut().fail();
            host.pending = false;
            host.ready = false;
            host.rendering = None;
        }
    }

    pub(super) fn rebind_device(&mut self, window: WindowId, epoch: u64, recovery: RecoveryHandle) {
        let Some(previous) = self.hosts.get(&window).map(|host| {
            (
                host.extent,
                host.occluded,
                host.primary_down,
                host.suppress_release,
                host.minimized,
            )
        }) else {
            return;
        };
        self.register(window, epoch, previous.0, recovery);
        let host = self.hosts.get_mut(&window).expect("rebound native host");
        host.occluded = previous.1;
        host.primary_down = previous.2;
        host.suppress_release = previous.3;
        host.minimized = previous.4;
        host.recovery
            .borrow_mut()
            .set_drawable(Self::drawable(host, self.suspended), Instant::now());
    }

    pub(super) fn manual_retry(&mut self, window: WindowId) -> bool {
        self.hosts
            .get_mut(&window)
            .is_some_and(|host| host.recovery.borrow_mut().manual_retry())
    }

    pub(super) fn rendering_failed(&self, window: WindowId) -> bool {
        self.hosts
            .get(&window)
            .is_some_and(|host| host.recovery.borrow().failed())
    }

    pub(super) fn close(&mut self, window: WindowId) {
        if let Some(host) = self.hosts.remove(&window) {
            host.trace(window, "closed");
        }
        self.order.retain(|id| *id != window);
    }

    /// The sole product request_redraw boundary. This does not apply or coalesce
    /// input; callers have already committed their complete view-state change.
    pub(super) fn invalidate(&mut self, window: &Window) {
        if self.invalidate_id(window.id()) {
            window.request_redraw();
        }
    }

    fn invalidate_id(&mut self, window: WindowId) -> bool {
        let Some(host) = self.hosts.get_mut(&window) else {
            return false;
        };
        host.damage = host
            .damage
            .checked_add(1)
            .expect("damage generation exhausted");
        Self::request(host, self.suspended)
    }

    fn request(host: &mut Host, suspended: bool) -> bool {
        if !Self::drawable(host, suspended)
            || !host.recovery.borrow_mut().ready(Instant::now())
            || host.damage == host.presented
        {
            host.counts.suppressed += 1;
            return false;
        }
        if host.pending || host.rendering.is_some() {
            host.counts.coalesced += 1;
            return false;
        }
        host.counts.requested += 1;
        host.pending = true;
        host.restore_pending = false;
        true
    }

    /// Accept only native redraw tokens. Input-only event-loop wakeups cannot
    /// manufacture ready frames; duplicate native tokens coalesce until dispatch.
    pub(super) fn redraw_received(&mut self, window: WindowId) {
        if let Some(host) = self.hosts.get_mut(&window) {
            host.counts.received += 1;
            host.counts.duplicate_tokens += u64::from(host.ready);
            host.ready = true;
            host.pending = true;
        }
    }

    /// Snapshot one bounded round after native event delivery. Rotate the first
    /// host, not just the requests, so repeated compositor ordering cannot give
    /// the same busy host first service in every round.
    pub(super) fn ready_round(&mut self, now: Instant) -> Vec<WindowId> {
        let mut ready = Vec::new();
        let mut first_served = None;
        for (index, id) in self.order.iter().enumerate() {
            let host = self.hosts.get_mut(id).expect("registered dispatch host");
            if !std::mem::take(&mut host.ready) {
                continue;
            }
            if Self::drawable(host, self.suspended) && host.recovery.borrow_mut().ready(now) {
                // Keep the delivered token reserved until this host begins.
                first_served.get_or_insert(index);
                ready.push(*id);
            } else {
                host.pending = false;
            }
        }
        if let Some(index) = first_served {
            // Idle/hidden/recovering registrations do not consume a turn.
            self.order.rotate_left(index + 1);
        }
        ready
    }

    /// A compositor may request repaint without a preceding application request.
    pub(super) fn begin_frame(&mut self, window: WindowId) -> Option<FrameReceipt> {
        let host = self.hosts.get_mut(&window)?;
        if !Self::drawable(host, self.suspended)
            || !host.recovery.borrow_mut().ready(Instant::now())
        {
            host.pending = false;
            host.ready = false;
            return None;
        }
        if host.rendering.is_some() {
            return None;
        }
        host.pending = false;
        host.restore_pending = false;
        if host.damage == host.presented {
            host.damage = host
                .damage
                .checked_add(1)
                .expect("damage generation exhausted");
        }
        host.attempt = host
            .attempt
            .checked_add(1)
            .expect("frame attempt exhausted");
        host.rendering = Some(host.attempt);
        Some(FrameReceipt {
            window,
            host_generation: host.generation,
            device_generation: host.device_generation,
            damage: host.damage,
            attempt: host.attempt,
        })
    }

    pub(super) fn frame_finished(
        &mut self,
        window: &Window,
        receipt: FrameReceipt,
        presented: bool,
    ) {
        if window.id() == receipt.window && self.finish(receipt, presented) {
            window.request_redraw();
        }
    }

    fn finish(&mut self, receipt: FrameReceipt, presented: bool) -> bool {
        let Some(host) = self.hosts.get_mut(&receipt.window) else {
            return false;
        };
        if host.generation != receipt.host_generation
            || host.device_generation != receipt.device_generation
            || host.rendering != Some(receipt.attempt)
        {
            return false;
        }
        host.rendering = None;
        if !presented {
            // Recovery owns the retry deadline. Retain damage without turning
            // an acquire failure into an immediate native redraw spin.
            host.trace(receipt.window, "not_presented");
            return false;
        }
        host.presented = receipt.damage;
        host.counts.presentations += 1;
        let requested = Self::request(host, self.suspended);
        host.trace(receipt.window, "presented");
        requested
    }
}

impl Host {
    fn trace(&self, window: WindowId, reason: &str) {
        crate::gui_runtime_support::append_gui_verbose_diagnostic_line(|| {
            format!(
                "native redraw ledger {}",
                serde_json::json!({
                    "window": format!("{window:?}"), "reason": reason,
                    "host_generation": self.generation, "device_generation": self.device_generation,
                    "damage": self.damage, "presented_damage": self.presented,
                    "attempts": self.attempt, "pending": self.pending,
                    "ready": self.ready, "rendering": self.rendering,
                    "counts": self.counts,
                })
            )
        });
    }
}

#[cfg(test)]
#[path = "native_frame_dispatch_tests.rs"]
mod dispatch_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn primary(state: ElementState) -> WindowEvent {
        WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state,
            button: MouseButton::Left,
        }
    }

    #[test]
    fn cancelled_capture_cannot_activate_a_control_after_dpi_or_device_replacement() {
        let mut frames = NativeFrameCoordinator::default();
        for id in 1..=4 {
            let window = WindowId::from(id);
            frames.register(window, 1, PhysicalSize::new(1280, 800), Default::default());
            assert!(!frames.consume_cancelled_release(window, &primary(ElementState::Pressed)));
        }
        // Only the affected native host loses its current coordinate gesture.
        frames.cancel_capture(WindowId::from(2));
        assert!(
            !frames.consume_cancelled_release(WindowId::from(1), &primary(ElementState::Released))
        );
        for id in 2..=4 {
            let window = WindowId::from(id);
            frames.cancel_capture(window);
            // Cancellation survives GPU replacement, which preserves native input identity.
            frames.rebind_device(window, 2, Default::default());
            assert!(frames.consume_cancelled_release(window, &primary(ElementState::Released)));
            assert!(!frames.consume_cancelled_release(window, &primary(ElementState::Released)));
            assert!(!frames.consume_cancelled_release(window, &primary(ElementState::Pressed)));
            assert!(!frames.consume_cancelled_release(window, &primary(ElementState::Released)));
        }
    }

    #[test]
    fn missing_cancelled_release_does_not_poison_new_gesture_or_reopened_host() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        frames.consume_cancelled_release(id, &primary(ElementState::Pressed));
        frames.cancel_capture(id);
        frames.consume_cancelled_release(id, &primary(ElementState::Pressed));
        assert!(!frames.consume_cancelled_release(id, &primary(ElementState::Released)));
        frames.consume_cancelled_release(id, &primary(ElementState::Pressed));
        frames.cancel_capture(id);
        frames.close(id);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        assert!(!frames.consume_cancelled_release(id, &primary(ElementState::Released)));
    }

    #[test]
    fn device_rebind_preserves_visibility_and_rejects_old_or_closed_receipts() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(7);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        let old = frames.begin_frame(id).unwrap();
        frames.window_event(id, &WindowEvent::Resized(PhysicalSize::new(1440, 900)));
        frames.window_event(id, &WindowEvent::Occluded(true));
        frames.fail_device();
        frames.rebind_device(id, 2, Default::default());
        assert!(!frames.finish(old, true));
        assert_eq!(frames.hosts[&id].extent, PhysicalSize::new(1440, 900));
        assert!(frames.begin_frame(id).is_none());
        frames.window_event(id, &WindowEvent::Occluded(false));
        let current = frames.begin_frame(id).unwrap();
        assert_eq!(current.device_generation, 2);
        assert!(!frames.finish(current, true));
        frames.close(id);
        frames.rebind_device(id, 3, Default::default());
        assert!(!frames.contains_host(id));
    }

    #[test]
    fn zero_extent_retains_damage_and_restores_one_current_frame() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        assert!(frames.invalidate_id(id));
        frames.window_event(id, &WindowEvent::Resized(PhysicalSize::new(0, 800)));
        assert!(!frames.hosts[&id].pending);
        for _ in 0..20 {
            assert!(!frames.invalidate_id(id));
            assert!(frames.begin_frame(id).is_none());
            assert!(!frames.take_restore_request(id));
        }
        let latest = frames.hosts[&id].damage;
        frames.window_event(id, &WindowEvent::Resized(PhysicalSize::new(1440, 900)));
        assert!(frames.take_restore_request(id));
        assert!(!frames.take_restore_request(id));
        let receipt = frames.begin_frame(id).unwrap();
        assert!(receipt.damage > latest);
        assert!(!frames.finish(receipt, true));
        assert_eq!(frames.hosts[&id].presented, frames.hosts[&id].damage);
    }

    #[test]
    fn occlusion_and_application_suspend_require_both_to_clear() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        frames.window_event(id, &WindowEvent::Occluded(true));
        frames.set_suspended(true);
        assert!(!frames.invalidate_id(id));
        frames.window_event(id, &WindowEvent::Occluded(false));
        assert!(!frames.take_restore_request(id));
        assert!(frames.begin_frame(id).is_none());
        frames.set_suspended(false);
        assert!(frames.take_restore_request(id));
        let receipt = frames.begin_frame(id).unwrap();
        assert!(!frames.finish(receipt, false));
        // Restoration was consumed by the actual attempt, not a retry timer.
        for _ in 0..20 {
            assert!(!frames.take_restore_request(id));
        }
    }

    #[test]
    fn initially_zero_and_closed_hosts_cannot_be_revived_by_old_events() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(0, 0), Default::default());
        assert!(!frames.invalidate_id(id));
        assert!(frames.begin_frame(id).is_none());
        frames.window_event(id, &WindowEvent::Resized(PhysicalSize::new(1, 1)));
        assert!(frames.take_restore_request(id));
        frames.window_event(id, &WindowEvent::Destroyed);
        frames.window_event(id, &WindowEvent::Occluded(false));
        frames.window_event(id, &WindowEvent::Resized(PhysicalSize::new(1280, 800)));
        frames.set_suspended(true);
        frames.set_suspended(false);
        assert!(!frames.take_restore_request(id));
        assert!(frames.begin_frame(id).is_none());
    }

    #[test]
    fn render_error_pauses_only_its_host_and_manual_retry_keeps_damage() {
        let mut frames = NativeFrameCoordinator::default();
        let failed = WindowId::from(1);
        let peer = WindowId::from(2);
        for id in [failed, peer] {
            frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        }
        let attempt = frames.begin_frame(failed).unwrap();
        frames.render_failed(failed, &"backend acquisition error");
        assert!(!frames.finish(attempt, false));
        assert!(frames.rendering_failed(failed));
        assert!(frames.begin_frame(failed).is_none());
        assert!(frames.begin_frame(peer).is_some());
        assert!(frames.manual_retry(failed));
        let retry = frames.begin_frame(failed).unwrap();
        assert_eq!(retry.damage, attempt.damage);
        assert!(!frames.finish(retry, true));
        assert_eq!(frames.hosts[&failed].presented, attempt.damage);
        frames.close(failed);
        frames.render_failed(failed, &"late error");
        assert!(!frames.contains_host(failed));
    }

    #[test]
    fn native_hosts_coalesce_requests_without_losing_latest_damage() {
        let mut frames = NativeFrameCoordinator::default();
        for id in 1..=4 {
            let window = WindowId::from(id);
            frames.register(window, 1, PhysicalSize::new(1280, 800), Default::default());
            assert!(frames.invalidate_id(window));
            for _ in 0..100 {
                assert!(!frames.invalidate_id(window));
            }
            frames.redraw_received(window);
            frames.redraw_received(window);
            let _ = frames.ready_round(Instant::now());
            let first = frames.begin_frame(window).unwrap();
            assert!(!frames.invalidate_id(window));
            assert!(frames.finish(first, true));
            assert!(!frames.invalidate_id(window));
            frames.redraw_received(window);
            let _ = frames.ready_round(Instant::now());
            let last = frames.begin_frame(window).unwrap();
            assert!(!frames.finish(last, true));
            let counts = &frames.hosts[&window].counts;
            assert_eq!(counts.requested, 2);
            assert_eq!(counts.coalesced, 102);
            assert_eq!(counts.suppressed, 1);
            assert_eq!(counts.received, 3);
            assert_eq!(counts.duplicate_tokens, 1);
            assert_eq!(counts.presentations, 2);
            assert_eq!(
                frames.hosts[&window].damage,
                frames.hosts[&window].presented
            );
        }
    }

    #[test]
    fn local_damage_does_not_request_other_hosts() {
        let mut frames = NativeFrameCoordinator::default();
        for id in 1..=4 {
            frames.register(
                WindowId::from(id),
                1,
                PhysicalSize::new(1280, 800),
                Default::default(),
            );
        }
        assert!(frames.invalidate_id(WindowId::from(1)));
        for id in 2..=4 {
            assert!(!frames.hosts[&WindowId::from(id)].pending);
        }
    }

    #[test]
    fn failed_acquisition_retains_damage_without_busy_retry() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        assert!(frames.invalidate_id(id));
        let receipt = frames.begin_frame(id).unwrap();
        assert!(!frames.finish(receipt, false));
        assert_ne!(frames.hosts[&id].damage, frames.hosts[&id].presented);
        assert!(!frames.hosts[&id].pending);
        assert!(frames.invalidate_id(id));
        let retry = frames.begin_frame(id).unwrap();
        assert!(!frames.finish(retry, true));
        assert_eq!(frames.hosts[&id].damage, frames.hosts[&id].presented);
    }

    #[test]
    fn closed_replaced_and_duplicate_completions_cannot_clear_new_damage() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        let old = frames.begin_frame(id).unwrap();
        frames.close(id);
        assert!(!frames.finish(old, true));
        assert!(!frames.invalidate_id(id));
        frames.register(id, 2, PhysicalSize::new(1280, 800), Default::default());
        let new = frames.begin_frame(id).unwrap();
        assert!(!frames.finish(old, true));
        assert_eq!(frames.hosts[&id].counts.presentations, 0);
        assert_eq!(frames.hosts[&id].counts.requested, 0);
        assert_eq!(frames.hosts[&id].rendering, Some(new.attempt));
        assert!(!frames.finish(new, true));
        assert!(frames.invalidate_id(id));
        assert!(!frames.finish(new, true));
        assert!(frames.hosts[&id].pending);
        assert_eq!(frames.hosts[&id].counts.presentations, 1);
        assert_eq!(frames.hosts[&id].counts.requested, 1);
    }

    #[test]
    fn late_failed_attempt_cannot_complete_retry_of_identical_damage() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1, PhysicalSize::new(1280, 800), Default::default());
        let failed = frames.begin_frame(id).unwrap();
        assert!(!frames.finish(failed, false));
        let retry = frames.begin_frame(id).unwrap();
        assert_eq!(retry.damage, failed.damage);
        assert_ne!(retry.attempt, failed.attempt);
        assert!(!frames.finish(failed, true));
        assert_eq!(frames.hosts[&id].rendering, Some(retry.attempt));
        assert_eq!(frames.hosts[&id].presented, 0);
        assert!(!frames.finish(retry, true));
        assert_eq!(frames.hosts[&id].presented, retry.damage);
    }
}

#[cfg(test)]
#[path = "native_frame_deadline_tests.rs"]
mod deadline_tests;
