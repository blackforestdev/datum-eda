//! Declared monotonic workload phases for causal frame/submission attribution.
//! Loaded once; no log-arrival classification, input scheduling or per-event I/O.
use anyhow::{Context, Result, ensure};
#[cfg(test)]
use winit::event::WindowEvent;

#[derive(Clone, Copy, serde::Deserialize)]
pub(super) struct Schedule {
    pub epoch: u64,
    pub warmup_ns: u64,
    pub active_ns: u64,
    pub still_ns: u64,
    pub drain_ns: u64,
    #[serde(skip)]
    closed: bool,
}
impl Schedule {
    fn validate(self) -> Result<Self> {
        ensure!(
            self.epoch != 0 && self.warmup_ns != 0,
            "workload epoch/start missing"
        );
        ensure!(
            self.warmup_ns.checked_add(5_000_000_000) == Some(self.active_ns)
                && self.active_ns.checked_add(30_000_000_000) == Some(self.still_ns)
                && self.still_ns.checked_add(5_000_000_000) == Some(self.drain_ns),
            "workload must declare exact 5s/30s/5s boundaries"
        );
        Ok(self)
    }
    pub(super) fn from_environment() -> Result<Option<Self>> {
        std::env::var_os("DATUM_WORKLOAD_MANIFEST")
            .map(|path| {
                let bytes = std::fs::read(path).context("read declared workload manifest")?;
                serde_json::from_slice::<Self>(&bytes)
                    .context("decode workload manifest")?
                    .validate()
            })
            .transpose()
    }
    #[cfg(test)]
    pub(super) fn tag(&mut self, event: &WindowEvent, now: u64, sequence: u64) -> [u64; 3] {
        self.demand(matches!(event, WindowEvent::CloseRequested), now, sequence)
    }
    pub(super) fn demand(&mut self, close: bool, now: u64, sequence: u64) -> [u64; 3] {
        self.closed |= close;
        let phase = if self.closed {
            6
        } else if now < self.warmup_ns {
            1
        } else if now < self.active_ns {
            2
        } else if now < self.still_ns {
            3
        } else if now < self.drain_ns {
            4
        } else {
            5
        };
        [self.epoch, phase, sequence]
    }
    pub(super) fn value(&self) -> serde_json::Value {
        serde_json::json!({"epoch":self.epoch,"warmup_ns":self.warmup_ns,"active_ns":self.active_ns,"still_ns":self.still_ns,"drain_ns":self.drain_ns,"phases":["startup","warmup","active","still","drain","close"],"clock":"CLOCK_MONOTONIC"})
    }
}
pub(super) fn monotonic_ns() -> Result<u64> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: CLOCK_MONOTONIC is valid and the initialized record is writable.
    ensure!(
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } == 0,
        "workload monotonic clock unavailable"
    );
    u64::try_from(time.tv_sec)?
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(time.tv_nsec as u64))
        .context("workload monotonic clock overflow")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declared_phases_and_explicit_close_are_not_log_arrival_times() {
        let mut s = Schedule {
            epoch: 1,
            warmup_ns: 10,
            active_ns: 5_000_000_010,
            still_ns: 35_000_000_010,
            drain_ns: 40_000_000_010,
            closed: false,
        }
        .validate()
        .unwrap();
        for (now, phase) in [
            (0, 1),
            (10, 2),
            (s.active_ns, 3),
            (s.still_ns, 4),
            (s.drain_ns, 5),
        ] {
            assert_eq!(s.tag(&WindowEvent::RedrawRequested, now, 2), [1, phase, 2]);
        }
        assert_eq!(
            s.tag(&WindowEvent::CloseRequested, s.drain_ns + 1, 3),
            [1, 6, 3]
        );
        assert_eq!(
            s.tag(&WindowEvent::RedrawRequested, s.drain_ns + 2, 4),
            [1, 6, 4]
        );
        s.active_ns += 1;
        assert!(s.validate().is_err());
    }
}
