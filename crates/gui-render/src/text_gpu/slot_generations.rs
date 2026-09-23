//! Shared slot identity for resources that may retire after their consumer closes.
use super::budget::Budget;
use std::sync::{Arc, Mutex, Weak};

/// Slot identity survives consumer close and renderer recovery. Weak entries keep
/// neither allocations nor allowances alive after their final submission retires.
#[derive(Clone, Default)]
pub(crate) struct SlotGenerations(Arc<Mutex<Vec<Weak<Budget>>>>);

impl SlotGenerations {
    pub(crate) fn for_slot(&self, slot: usize) -> Arc<Budget> {
        let mut slots = self.0.lock().unwrap_or_else(|e| e.into_inner());
        while slots.last().is_some_and(|owner| owner.strong_count() == 0) {
            slots.pop();
        }
        if slots.capacity() > slots.len().saturating_mul(4) {
            slots.shrink_to_fit();
        }
        if slots.len() <= slot {
            slots.resize_with(slot + 1, Weak::new);
        }
        if let Some(owner) = slots[slot].upgrade() {
            return owner;
        }
        let owner = Budget::new(2);
        slots[slot] = Arc::downgrade(&owner);
        owner
    }
}
