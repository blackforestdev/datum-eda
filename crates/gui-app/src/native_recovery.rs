//! Drawable-active recovery clock shared by coordinator and backend transitions.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
pub(crate) type RecoveryHandle = Rc<RefCell<Recovery>>;
const LIMIT: Duration = Duration::from_secs(2);
#[derive(Clone, Copy, Debug)]
pub(crate) enum RetryReason {
    Acquisition,
    Queue,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AcquisitionFailure {
    Reconfigure,
    Retry,
    DeviceAllocation,
    Fatal,
}

#[derive(Debug)]
pub(crate) struct Recovery {
    drawable: bool,
    episode: bool,
    active: Duration,
    since: Option<Instant>,
    due: Option<Instant>,
    attempts: u32,
    failed: bool,
    reported: bool,
}
impl Default for Recovery {
    fn default() -> Self {
        Self {
            drawable: true,
            episode: false,
            active: Duration::ZERO,
            since: None,
            due: None,
            attempts: 0,
            failed: false,
            reported: false,
        }
    }
}
impl Recovery {
    /// Classify the backend result and advance the same monotonic episode.
    /// Reconfiguration never grants a fresh failure budget.
    pub(crate) fn acquisition_failed(
        &mut self,
        error: &wgpu::SurfaceError,
        now: Instant,
    ) -> AcquisitionFailure {
        match error {
            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                self.defer(RetryReason::Acquisition, now);
                AcquisitionFailure::Reconfigure
            }
            wgpu::SurfaceError::Timeout => {
                self.defer(RetryReason::Acquisition, now);
                AcquisitionFailure::Retry
            }
            wgpu::SurfaceError::OutOfMemory => AcquisitionFailure::DeviceAllocation,
            wgpu::SurfaceError::Other => AcquisitionFailure::Fatal,
        }
    }

    fn advance(&mut self, now: Instant) {
        if let Some(since) = self.since {
            self.active += now.saturating_duration_since(since);
            self.since = Some(now);
        }
        if self.active >= LIMIT {
            self.failed = true;
            self.due = None;
            self.since = None;
        }
    }
    pub(crate) fn set_drawable(&mut self, drawable: bool, now: Instant) {
        self.advance(now);
        if self.drawable == drawable {
            return;
        }
        self.drawable = drawable;
        if !drawable {
            self.since = None;
            self.due = None;
        } else if self.episode && !self.failed {
            self.since = Some(now);
            self.attempts = 0;
            self.due = Some(now + Duration::from_millis(16).min(LIMIT - self.active));
        }
    }
    pub(crate) fn defer(&mut self, reason: RetryReason, now: Instant) {
        self.advance(now);
        if self.failed {
            return;
        }
        self.episode = true;
        if !self.drawable {
            return;
        }
        self.since.get_or_insert(now);
        let delay = match reason {
            RetryReason::Queue => Duration::from_millis(2),
            RetryReason::Acquisition => {
                let millis = (16_u64 << self.attempts.min(4)).min(250);
                self.attempts = self.attempts.saturating_add(1);
                Duration::from_millis(millis)
            }
        };
        self.due = Some(now + delay.min(LIMIT - self.active));
    }
    pub(crate) fn ready(&mut self, now: Instant) -> bool {
        self.advance(now);
        self.drawable && !self.failed && self.due.is_none_or(|due| due <= now)
    }
    pub(crate) fn poll(&mut self, now: Instant) -> (bool, Option<Instant>) {
        self.advance(now);
        if !self.drawable || self.failed {
            return (false, None);
        }
        match self.due {
            Some(due) if due <= now => {
                self.due = None;
                (true, None)
            }
            due => (false, due),
        }
    }
    /// A queue wait needs callback progress, not another native frame attempt.
    /// Preserve the episode and short wake while admission remains unavailable.
    pub(crate) fn poll_admitted(
        &mut self,
        now: Instant,
        admitted: bool,
    ) -> (bool, Option<Instant>) {
        let result = self.poll(now);
        if result.0 && !admitted {
            self.defer(RetryReason::Queue, now);
            (false, self.poll(now).1)
        } else {
            result
        }
    }

    pub(crate) fn fail(&mut self) {
        self.failed = true;
        self.due = None;
        self.since = None;
    }

    pub(crate) fn failed(&self) -> bool {
        self.failed
    }
    pub(crate) fn take_failure(&mut self) -> bool {
        if self.failed && !self.reported {
            self.reported = true;
            true
        } else {
            false
        }
    }
    pub(crate) fn success(&mut self) {
        let drawable = self.drawable;
        *self = Self::default();
        self.drawable = drawable;
    }
    pub(crate) fn manual_retry(&mut self) -> bool {
        if !self.failed {
            return false;
        }
        self.success();
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acquisition_backoff_stops_at_two_seconds_and_needs_manual_retry() {
        let start = Instant::now();
        let mut now = start;
        let mut r = Recovery::default();
        for delay in [16, 32, 64, 128, 250, 250, 250, 250, 250, 250, 250, 250] {
            r.defer(RetryReason::Acquisition, now);
            let due = r.due.unwrap();
            assert_eq!(
                due - now,
                Duration::from_millis(delay).min(LIMIT - (now - start))
            );
            assert!(!r.ready(due - Duration::from_nanos(1)));
            now = due;
            let _ = r.poll(now);
            if now - start == LIMIT {
                break;
            }
        }
        assert_eq!(now - start, LIMIT);
        assert!(r.failed());
        assert!(!r.ready(now));
        assert!(r.take_failure());
        assert!(!r.take_failure());
        assert!(r.manual_retry());
        assert!(r.ready(now));
    }
    #[test]
    fn visibility_cycles_preserve_accumulated_active_failure_time() {
        let start = Instant::now();
        let mut r = Recovery::default();
        r.defer(RetryReason::Acquisition, start);
        r.set_drawable(false, start + Duration::from_millis(900));
        r.set_drawable(true, start + Duration::from_secs(10));
        assert_eq!(r.due, Some(start + Duration::from_millis(10016)));
        r.set_drawable(true, start + Duration::from_millis(10500)); // size change cannot reset
        r.set_drawable(false, start + Duration::from_millis(10900));
        r.set_drawable(true, start + Duration::from_secs(20));
        assert!(r.ready(start + Duration::from_millis(20199)));
        assert!(!r.ready(start + Duration::from_millis(20200)));
        assert!(r.failed());
    }
    #[test]
    fn gpu_wait_is_bounded_by_same_episode_and_success_resets_it() {
        let now = Instant::now();
        let mut r = Recovery::default();
        r.defer(RetryReason::Queue, now);
        assert_eq!(r.poll(now), (false, Some(now + Duration::from_millis(2))));
        r.success();
        assert!(r.ready(now + Duration::from_secs(10)));
        r.defer(RetryReason::Queue, now + Duration::from_secs(10));
        assert_eq!(r.poll(now + Duration::from_secs(12)), (false, None));
        assert!(r.failed());
    }
}

#[cfg(test)]
mod acquisition_tests {
    use super::*;

    #[test]
    fn lost_outdated_and_timeout_share_one_episode_until_success() {
        let start = Instant::now();
        let mut recovery = Recovery::default();
        assert_eq!(
            recovery.acquisition_failed(&wgpu::SurfaceError::Lost, start),
            AcquisitionFailure::Reconfigure
        );
        let second = start + Duration::from_millis(16);
        assert_eq!(recovery.poll(second), (true, None));
        assert_eq!(
            recovery.acquisition_failed(&wgpu::SurfaceError::Outdated, second),
            AcquisitionFailure::Reconfigure
        );
        let third = second + Duration::from_millis(32);
        assert_eq!(recovery.poll(third), (true, None));
        assert_eq!(
            recovery.acquisition_failed(&wgpu::SurfaceError::Timeout, third),
            AcquisitionFailure::Retry
        );
        assert_eq!(
            recovery.poll(third),
            (false, Some(third + Duration::from_millis(64)))
        );
        // Later backend failures/reconfiguration did not restart the two-second clock.
        assert!(!recovery.ready(start + LIMIT));
        assert!(recovery.failed());
        assert!(recovery.manual_retry());
        assert!(recovery.ready(start + LIMIT));
        recovery.acquisition_failed(&wgpu::SurfaceError::Lost, start + LIMIT);
        recovery.success();
        assert!(recovery.ready(start + LIMIT * 2));
        let next = start + LIMIT * 2;
        recovery.acquisition_failed(&wgpu::SurfaceError::Outdated, next);
        assert_eq!(
            recovery.poll(next),
            (false, Some(next + Duration::from_millis(16)))
        );
    }

    #[test]
    fn allocation_and_fatal_errors_do_not_start_automatic_acquisition_retry() {
        let now = Instant::now();
        for (error, action) in [
            (
                wgpu::SurfaceError::OutOfMemory,
                AcquisitionFailure::DeviceAllocation,
            ),
            (wgpu::SurfaceError::Other, AcquisitionFailure::Fatal),
        ] {
            let mut recovery = Recovery::default();
            assert_eq!(recovery.acquisition_failed(&error, now), action);
            assert_eq!(recovery.poll(now), (false, None));
            assert!(!recovery.episode);
        }
    }
}
