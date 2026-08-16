#[cfg(test)]
use super::limits::{HUP_GRACE_MS, KILL_VERIFY_MS, TERM_GRACE_MS};
#[cfg(test)]
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShutdownPhase {
    Running,
    Hup,
    Term,
    Kill,
    Closed,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShutdownRequest {
    Graceful,
    Force,
}

#[cfg(test)]
fn phase_deadline(phase: ShutdownPhase, now: Instant) -> Instant {
    let milliseconds = match phase {
        ShutdownPhase::Hup => HUP_GRACE_MS,
        ShutdownPhase::Term => TERM_GRACE_MS,
        ShutdownPhase::Kill => KILL_VERIFY_MS,
        _ => 0,
    };
    now + Duration::from_millis(milliseconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_ratified_phase_deadlines_total_six_seconds() {
        let start = Instant::now();
        let hup = phase_deadline(ShutdownPhase::Hup, start);
        let term = phase_deadline(ShutdownPhase::Term, hup);
        let kill = phase_deadline(ShutdownPhase::Kill, term);
        assert_eq!(kill.duration_since(start), Duration::from_secs(6));
    }
}
