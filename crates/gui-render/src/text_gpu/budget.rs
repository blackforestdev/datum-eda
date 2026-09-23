//! Reserve texture capacity before API allocation; release after the last hold.
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicU64, Ordering},
};

pub(crate) struct Budget {
    limit: u64,
    used: AtomicU64,
}
impl Budget {
    pub fn new(limit: u64) -> Arc<Self> {
        Arc::new(Self {
            limit,
            used: AtomicU64::new(0),
        })
    }
    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }
    pub fn reserve(self: &Arc<Self>, bytes: u64) -> anyhow::Result<Permit> {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .map_err(|_| {
                anyhow::anyhow!(
                    "GPU resource budget exhausted (requested {bytes} bytes; limit {})",
                    self.limit
                )
            })?;
        Ok(Permit {
            budget: self.clone(),
            bytes,
        })
    }
}

pub(crate) struct Permit {
    budget: Arc<Budget>,
    bytes: u64,
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

pub(super) fn process() -> Arc<Budget> {
    static BUDGET: OnceLock<Arc<Budget>> = OnceLock::new();
    BUDGET
        .get_or_init(|| Budget::new(128 * 1024 * 1024))
        .clone()
}

/// Shared admission for migrated application GPU allocations, not driver residency.
pub(crate) fn gpu_process() -> Arc<Budget> {
    static BUDGET: OnceLock<Arc<Budget>> = OnceLock::new();
    BUDGET
        .get_or_init(|| Budget::new(512 * 1024 * 1024))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_second_reservation_rolls_back_first() {
        let atlas = Budget::new(32);
        let gpu = Budget::new(8);
        let attempt = || -> anyhow::Result<Vec<Permit>> {
            let first = atlas.reserve(16)?;
            let second = gpu.reserve(16)?;
            Ok(vec![first, second])
        };
        assert!(attempt().is_err());
        assert_eq!(atlas.used(), 0);
        assert_eq!(gpu.used(), 0);
    }

    #[test]
    fn concurrent_admission_cannot_overbook_or_release_early() {
        let budget = Budget::new(64);
        let barrier = Arc::new(std::sync::Barrier::new(9));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let budget = budget.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let permit = budget.reserve(16).ok();
                    barrier.wait();
                    barrier.wait();
                    permit
                })
            })
            .collect();
        barrier.wait();
        assert_eq!(budget.used(), 64);
        assert!(budget.reserve(1).is_err());
        barrier.wait();
        let permits: Vec<_> = workers
            .into_iter()
            .filter_map(|w| w.join().unwrap())
            .collect();
        assert_eq!(permits.len(), 4);
        assert!(budget.reserve(u64::MAX).is_err());
        drop(permits);
        assert_eq!(budget.used(), 0);
        assert!(budget.reserve(64).is_ok());
    }
}
