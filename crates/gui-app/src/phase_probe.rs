//! Opt-in diagnostic CPU time windows, not production acceptance measurements.
use std::sync::OnceLock;
use std::time::Instant;

pub(crate) struct Probe {
    name: &'static str,
    wall: Instant,
    process: u64,
    thread: u64,
}

fn cpu_ns(clock: libc::clockid_t) -> Option<u64> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: value is writable and clock is a libc CPU-clock constant.
    if unsafe { libc::clock_gettime(clock, &mut value) } != 0 {
        return None;
    }
    Some(value.tv_sec as u64 * 1_000_000_000 + value.tv_nsec as u64)
}

impl Probe {
    pub(crate) fn start(name: &'static str) -> Option<Self> {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        if !*ENABLED.get_or_init(|| std::env::var_os("DATUM_RESIZE_CPU_PROBE").is_some()) {
            return None;
        }
        Some(Self {
            name,
            wall: Instant::now(),
            process: cpu_ns(libc::CLOCK_PROCESS_CPUTIME_ID)?,
            thread: cpu_ns(libc::CLOCK_THREAD_CPUTIME_ID)?,
        })
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let wall = self.wall.elapsed().as_nanos();
        if let (Some(process), Some(thread)) = (
            cpu_ns(libc::CLOCK_PROCESS_CPUTIME_ID),
            cpu_ns(libc::CLOCK_THREAD_CPUTIME_ID),
        ) {
            super::append_gui_diagnostic_line(format!(
                "cpu_probe phase={} wall_ns={} process_ns={} thread_ns={}",
                self.name,
                wall,
                process.saturating_sub(self.process),
                thread.saturating_sub(self.thread)
            ));
        }
    }
}
