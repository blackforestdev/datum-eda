//! Coalesced rendering damage and linear completion receipts.
//! Platform delivery damage remains owned by the native coordinator.
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Interaction,
    Composition,
    Content,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub host: u64,
    pub device: u64,
    pub configuration: u64,
}

#[derive(Debug)]
pub struct Receipt {
    owner: u64,
    attempt: u64,
    revision: u64,
    target: Target,
    native: bool,
    mailbox: Arc<AtomicU64>,
    completed: bool,
}

impl Receipt {
    pub(super) fn target(&self) -> Target {
        self.target
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }
}

impl Drop for Receipt {
    fn drop(&mut self) {
        if !self.completed {
            let _ =
                self.mailbox
                    .compare_exchange(self.attempt, 0, Ordering::AcqRel, Ordering::Acquire);
        }
    }
}

struct Active {
    attempt: u64,
    target: Target,
}

pub struct Revisions {
    owner: u64,
    revision: u64,
    acknowledged: u64,
    strong_revision: u64,
    attempt: u64,
    active: Option<Active>,
    mailbox: Arc<AtomicU64>,
}

impl Default for Revisions {
    fn default() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let owner = NEXT
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .expect("render session identities exhausted");
        // One fixed mailbox per session. The allocator owner follows its live
        // allocation, including a receipt outliving the session; no per-frame
        // allocation or guessed Arc header size enters the warm path.
        let storage = crate::cpu_alloc::Scope::new("render-session-receipt");
        let mailbox = storage.with(|| Arc::new(AtomicU64::new(0)));
        Self {
            owner,
            revision: 1,
            acknowledged: 0,
            strong_revision: 1,
            attempt: 0,
            active: None,
            mailbox,
        }
    }
}

impl Revisions {
    pub fn update(&mut self, change: Change) -> u64 {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("render revisions exhausted");
        if change != Change::Interaction {
            self.strong_revision = self.revision;
        }
        self.revision
    }

    pub(super) fn matches(&self, receipt: &Receipt) -> bool {
        receipt.owner == self.owner
            && self.active.as_ref().is_some_and(|active| {
                active.attempt == receipt.attempt
                    && active.target == receipt.target
                    && self.mailbox.load(Ordering::Acquire) == active.attempt
            })
    }

    pub fn current(&self) -> u64 {
        self.revision
    }
    pub fn pending(&self) -> bool {
        self.revision != self.acknowledged || self.abandoned()
    }
    pub fn interaction_only(&self) -> bool {
        self.pending() && !self.abandoned() && self.strong_revision <= self.acknowledged
    }

    fn abandoned(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| self.mailbox.load(Ordering::Acquire) != active.attempt)
    }

    pub fn begin(&mut self, target: Target, native: bool) -> Receipt {
        // Superseding an unfinished attempt cannot carry image validity forward.
        if self.active.take().is_some() {
            self.update(Change::Target);
        }
        self.attempt = self
            .attempt
            .checked_add(1)
            .expect("render attempts exhausted");
        self.mailbox.store(self.attempt, Ordering::Release);
        self.active = Some(Active {
            attempt: self.attempt,
            target,
        });
        Receipt {
            owner: self.owner,
            attempt: self.attempt,
            revision: self.revision,
            target,
            native,
            mailbox: self.mailbox.clone(),
            completed: false,
        }
    }

    /// A stale/foreign/capture receipt cannot retire native damage. New changes
    /// remain pending even when an older submitted revision presents successfully.
    pub fn complete(&mut self, mut receipt: Receipt, actual: Target, presented: bool) -> bool {
        if receipt.owner != self.owner
            || !self.active.as_ref().is_some_and(|active| {
                active.attempt == receipt.attempt && active.target == receipt.target
            })
        {
            return false;
        }
        self.active = None;
        self.mailbox.store(0, Ordering::Release);
        receipt.completed = true;
        if !presented || !receipt.native || actual != receipt.target {
            self.update(Change::Target);
            return false;
        }
        self.acknowledged = receipt.revision;
        true
    }

    pub fn retire_target(&mut self) {
        self.active = None;
        self.mailbox.store(0, Ordering::Release);
        self.update(Change::Target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TARGET: Target = Target {
        host: 1,
        device: 1,
        configuration: 1,
    };

    #[test]
    fn mixed_damage_cannot_be_downgraded_by_pointer_order() {
        for changes in [
            [Change::Composition, Change::Interaction],
            [Change::Interaction, Change::Composition],
        ] {
            let mut state = Revisions::default();
            let receipt = state.begin(TARGET, true);
            assert!(state.complete(receipt, TARGET, true));
            state.update(changes[0]);
            state.update(changes[1]);
            assert!(!state.interaction_only());
            let receipt = state.begin(TARGET, true);
            state.update(Change::Interaction);
            assert!(state.complete(receipt, TARGET, true));
            assert!(state.pending());
            assert!(state.interaction_only());
        }
    }

    #[test]
    fn failed_capture_foreign_and_retired_receipts_preserve_damage() {
        let mut state = Revisions::default();
        let receipt = state.begin(TARGET, true);
        assert!(!state.complete(receipt, TARGET, false));
        let capture = state.begin(TARGET, false);
        assert!(!state.complete(capture, TARGET, true));
        assert!(state.pending());
        let mut foreign = Revisions::default();
        let receipt = foreign.begin(TARGET, true);
        assert!(!state.complete(receipt, TARGET, true));
        let stale = state.begin(TARGET, true);
        state.retire_target();
        let current = state.begin(
            Target {
                device: 2,
                ..TARGET
            },
            true,
        );
        assert!(!state.complete(stale, TARGET, true));
        assert!(state.pending());
        assert!(state.complete(
            current,
            Target {
                device: 2,
                ..TARGET
            },
            true
        ));
        assert!(!state.pending());
    }

    #[test]
    fn dropped_and_superseded_attempts_never_acknowledge() {
        let mut state = Revisions::default();
        {
            let _abandoned = state.begin(TARGET, true);
        }
        assert!(state.pending());
        let old = state.begin(TARGET, true);
        let latest = state.begin(TARGET, true);
        assert!(!state.complete(old, TARGET, true));
        assert!(state.complete(latest, TARGET, true));
    }
    #[test]
    fn warm_failure_and_abandoned_exposure_require_full_recovery() {
        let mut state = Revisions::default();
        let receipt = state.begin(TARGET, true);
        assert!(state.complete(receipt, TARGET, true));
        state.update(Change::Interaction);
        assert!(state.interaction_only());
        let receipt = state.begin(TARGET, true);
        assert!(!state.complete(receipt, TARGET, false));
        assert!(state.pending());
        assert!(!state.interaction_only());
        let receipt = state.begin(TARGET, true);
        assert!(state.complete(receipt, TARGET, true));
        assert!(!state.pending());
        {
            let _exposure = state.begin(TARGET, true);
        }
        assert!(state.pending());
        assert!(!state.interaction_only());
        let receipt = state.begin(TARGET, true);
        assert!(state.complete(receipt, TARGET, true));
        assert!(!state.pending());
    }

    #[test]
    fn actual_present_target_must_match_every_identity_component() {
        for actual in [
            Target { host: 2, ..TARGET },
            Target {
                device: 2,
                ..TARGET
            },
            Target {
                configuration: 2,
                ..TARGET
            },
        ] {
            let mut state = Revisions::default();
            let receipt = state.begin(TARGET, true);
            assert!(state.complete(receipt, TARGET, true));
            state.update(Change::Interaction);
            let receipt = state.begin(TARGET, true);
            assert!(!state.complete(receipt, actual, true));
            assert!(state.pending());
            assert!(!state.interaction_only());
        }
    }
    #[test]
    fn receipt_mailbox_has_no_per_attempt_allocation_and_stale_drop_is_isolated() {
        let mut state = Revisions::default();
        let scope = crate::cpu_alloc::Scope::new("receipt-warm-proof");
        let (old, current) = scope.with(|| (state.begin(TARGET, true), state.begin(TARGET, true)));
        assert_eq!(scope.usage().allocations, 0);
        assert_eq!(Arc::as_ptr(&old.mailbox), Arc::as_ptr(&current.mailbox));
        drop(old);
        assert!(!state.abandoned());
        assert!(state.complete(current, TARGET, true));
        assert!(!state.pending());
        let receipt = state.begin(TARGET, true);
        let mailbox = receipt.mailbox.clone();
        drop(state);
        assert_ne!(mailbox.load(Ordering::Acquire), 0);
        drop(receipt);
        assert_eq!(mailbox.load(Ordering::Acquire), 0);
    }
}
