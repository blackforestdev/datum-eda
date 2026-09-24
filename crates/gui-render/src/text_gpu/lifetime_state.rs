//! Bounded allocation lifecycle accounting; no resource/event history is retained.
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RetirementReason {
    OwnerDropped = 1,
    Replaced,
    Cleared,
    Uncached,
    Submitted,
}
impl RetirementReason {
    fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::OwnerDropped),
            2 => Some(Self::Replaced),
            3 => Some(Self::Cleared),
            4 => Some(Self::Uncached),
            5 => Some(Self::Submitted),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReleasedAllocations {
    pub allocations: u64,
    pub capacity_bytes: u64,
}

pub(super) struct Metadata {
    pub consumers: AtomicU16,
    pub retiring_consumers: AtomicU16,
    prepared_consumers: [AtomicU64; 14],
    submitted_consumers: [AtomicU64; 14],
    pub payload: AtomicU64,
    pub released: AtomicBool,
    reason: AtomicU8,
    pub prepared: AtomicU64,
    pub submitted: AtomicU64,
}
impl Metadata {
    pub fn new(payload: u64) -> Self {
        Self {
            retiring_consumers: AtomicU16::new(0),
            consumers: AtomicU16::new(super::super::allocation_host::consumers().bits()),
            prepared_consumers: std::array::from_fn(|_| AtomicU64::new(0)),
            submitted_consumers: std::array::from_fn(|_| AtomicU64::new(0)),
            payload: AtomicU64::new(payload),
            released: AtomicBool::new(false),
            reason: AtomicU8::new(0),
            prepared: AtomicU64::new(0),
            submitted: AtomicU64::new(0),
        }
    }
    pub fn retire(&self, reason: RetirementReason) -> bool {
        self.reason
            .compare_exchange(0, reason as u8, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub fn reason(&self) -> Option<RetirementReason> {
        RetirementReason::from_code(self.reason.load(Ordering::Acquire))
    }
}

#[derive(Clone, Copy)]
pub(super) enum ReferenceKind {
    Prepared,
    Submission,
}
impl Metadata {
    pub fn counter(&self, kind: ReferenceKind) -> &AtomicU64 {
        match kind {
            ReferenceKind::Prepared => &self.prepared,
            ReferenceKind::Submission => &self.submitted,
        }
    }
}

pub(super) fn released_snapshot(
    totals: [ReleasedAllocations; 5],
) -> [(RetirementReason, ReleasedAllocations); 5] {
    [
        RetirementReason::OwnerDropped,
        RetirementReason::Replaced,
        RetirementReason::Cleared,
        RetirementReason::Uncached,
        RetirementReason::Submitted,
    ]
    .map(|reason| (reason, totals[reason as usize - 1]))
}

impl Metadata {
    fn incidence(&self, kind: ReferenceKind) -> &[AtomicU64; 14] {
        match kind {
            ReferenceKind::Prepared => &self.prepared_consumers,
            ReferenceKind::Submission => &self.submitted_consumers,
        }
    }
    pub fn retain_consumers(
        &self,
        kind: ReferenceKind,
        consumers: crate::resource_consumers::Consumers,
    ) {
        for consumer in consumers.iter() {
            self.incidence(kind)[consumer as usize].fetch_add(1, Ordering::AcqRel);
        }
    }
    pub fn release_consumers(
        &self,
        kind: ReferenceKind,
        consumers: crate::resource_consumers::Consumers,
    ) {
        for consumer in consumers.iter() {
            self.incidence(kind)[consumer as usize].fetch_sub(1, Ordering::AcqRel);
        }
    }
    pub fn referenced_consumers(
        &self,
        kind: ReferenceKind,
    ) -> crate::resource_consumers::Consumers {
        let mut result = crate::resource_consumers::Consumers::default();
        for consumer in crate::resource_consumers::Consumer::ALL {
            if self.incidence(kind)[consumer as usize].load(Ordering::Acquire) > 0 {
                result.insert(consumer);
            }
        }
        result
    }
}
