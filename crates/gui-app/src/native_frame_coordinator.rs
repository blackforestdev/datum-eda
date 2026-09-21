//! Application-owned native redraw tokens and damage generations.
//!
//! Native events still update input/model state before requesting a frame. A
//! frame retires only its captured damage; host identity prevents late completion
//! from changing a closed or replacement host. Queue/recovery policy is separate.
use std::collections::HashMap;
use winit::window::{Window, WindowId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FrameReceipt {
    window: WindowId,
    host_generation: u64,
    device_generation: u64,
    damage: u64,
    attempt: u64,
}

#[derive(Debug)]
struct Host {
    generation: u64,
    device_generation: u64,
    damage: u64,
    presented: u64,
    pending: bool,
    attempt: u64,
    rendering: Option<u64>,
}

#[derive(Default)]
pub(super) struct NativeFrameCoordinator {
    generation: u64,
    hosts: HashMap<WindowId, Host>,
}

impl NativeFrameCoordinator {
    pub(super) fn register(&mut self, window: WindowId, device_generation: u64) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("host generation exhausted");
        self.hosts.insert(
            window,
            Host {
                generation: self.generation,
                device_generation,
                damage: 1,
                presented: 0,
                pending: false,
                attempt: 0,
                rendering: None,
            },
        );
    }

    pub(super) fn close(&mut self, window: WindowId) {
        self.hosts.remove(&window);
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
        Self::request(host)
    }

    fn request(host: &mut Host) -> bool {
        if host.pending || host.rendering.is_some() || host.damage == host.presented {
            return false;
        }
        host.pending = true;
        true
    }

    /// A compositor may request repaint without a preceding application request.
    pub(super) fn redraw_received(&mut self, window: WindowId) -> Option<FrameReceipt> {
        let host = self.hosts.get_mut(&window)?;
        if host.rendering.is_some() {
            return None;
        }
        host.pending = false;
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
            return false;
        }
        host.presented = receipt.damage;
        Self::request(host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_hosts_coalesce_requests_without_losing_latest_damage() {
        let mut frames = NativeFrameCoordinator::default();
        for id in 1..=4 {
            let window = WindowId::from(id);
            frames.register(window, 1);
            assert!(frames.invalidate_id(window));
            for _ in 0..100 {
                assert!(!frames.invalidate_id(window));
            }
            let first = frames.redraw_received(window).unwrap();
            assert!(!frames.invalidate_id(window));
            assert!(frames.finish(first, true));
            assert!(!frames.invalidate_id(window));
            let last = frames.redraw_received(window).unwrap();
            assert!(!frames.finish(last, true));
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
            frames.register(WindowId::from(id), 1);
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
        frames.register(id, 1);
        assert!(frames.invalidate_id(id));
        let receipt = frames.redraw_received(id).unwrap();
        assert!(!frames.finish(receipt, false));
        assert_ne!(frames.hosts[&id].damage, frames.hosts[&id].presented);
        assert!(!frames.hosts[&id].pending);
        assert!(frames.invalidate_id(id));
        let retry = frames.redraw_received(id).unwrap();
        assert!(!frames.finish(retry, true));
        assert_eq!(frames.hosts[&id].damage, frames.hosts[&id].presented);
    }

    #[test]
    fn closed_replaced_and_duplicate_completions_cannot_clear_new_damage() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1);
        let old = frames.redraw_received(id).unwrap();
        frames.close(id);
        assert!(!frames.finish(old, true));
        assert!(!frames.invalidate_id(id));
        frames.register(id, 2);
        let new = frames.redraw_received(id).unwrap();
        assert!(!frames.finish(old, true));
        assert_eq!(frames.hosts[&id].rendering, Some(new.attempt));
        assert!(!frames.finish(new, true));
        assert!(frames.invalidate_id(id));
        assert!(!frames.finish(new, true));
        assert!(frames.hosts[&id].pending);
    }

    #[test]
    fn late_failed_attempt_cannot_complete_retry_of_identical_damage() {
        let mut frames = NativeFrameCoordinator::default();
        let id = WindowId::from(1);
        frames.register(id, 1);
        let failed = frames.redraw_received(id).unwrap();
        assert!(!frames.finish(failed, false));
        let retry = frames.redraw_received(id).unwrap();
        assert_eq!(retry.damage, failed.damage);
        assert_ne!(retry.attempt, failed.attempt);
        assert!(!frames.finish(failed, true));
        assert_eq!(frames.hosts[&id].rendering, Some(retry.attempt));
        assert_eq!(frames.hosts[&id].presented, 0);
        assert!(!frames.finish(retry, true));
        assert_eq!(frames.hosts[&id].presented, retry.damage);
    }
}
