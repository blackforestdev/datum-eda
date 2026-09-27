//! Cache identity is metadata, not part of PreparedScene's semantic equality.
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone)]
pub(crate) struct CompositionRevision(u64);

impl Default for CompositionRevision {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(
            NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("prepared composition identity exhausted"),
        )
    }
}

impl PartialEq for CompositionRevision {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl CompositionRevision {
    pub fn identity(&self) -> u64 {
        self.0
    }
    pub fn invalidate(&mut self) {
        *self = Self::default();
    }
}
