//! Image validity follows immutable preparation and successful completion only.
use super::frame_revision::Target;
use crate::gpu_surface::PairIdentity;
#[path = "session_damage.rs"]
pub(crate) mod damage;
use damage::Pixels;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Key {
    pub preparation: u64,
    pub strong_revision: u64,
    pub target: Target,
}

#[derive(Clone, Copy)]
pub(super) struct Completed {
    key: Key,
    pair: PairIdentity,
    support: Option<Pixels>,
    revision: u64,
}

/// Fixed inline metadata; no per-frame lists, hashes, or unaccounted image history.
#[derive(Default)]
pub(crate) struct Prefix {
    valid: Option<Completed>,
    request: Option<Key>,
    support: Option<Pixels>,
    revision: u64,
    encoded: Option<Completed>,
    reused: bool,
    copy_bytes: u64,
    world_bundles: std::cell::Cell<usize>,
}
impl Prefix {
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn fragmented_control(&self) -> Option<Pixels> {
        Pixels::fragmented_control()
    }

    pub(super) fn begin(&mut self, request: Option<Key>, support: Option<Pixels>, revision: u64) {
        self.support = support;
        self.revision = revision;
        self.request = request;
        self.encoded = None;
        self.reused = false;
        self.copy_bytes = 0;
        self.world_bundles.set(0);
    }
    pub(crate) fn reset_work(&mut self) {
        // A direct full render may overwrite/discard working B without a session
        // receipt. Its old presented composition can no longer authorize reuse.
        if self.request.is_none() {
            self.valid = None;
        }
        self.encoded = None;
        self.reused = false;
        self.copy_bytes = 0;
        self.world_bundles.set(0);
    }
    pub(crate) fn world_bundle_executed(&self) {
        self.world_bundles.set(
            self.world_bundles
                .get()
                .checked_add(1)
                .expect("world execution count exhausted"),
        );
    }
    pub(crate) fn world_executions(&self) -> usize {
        self.world_bundles.get()
    }
    pub(crate) fn requested(&self) -> bool {
        self.request.is_some()
    }
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn desired_support(&self) -> Option<Pixels> {
        self.support
    }

    pub(crate) fn damage(&self, pair: PairIdentity) -> Option<Pixels> {
        let valid = self.valid?;
        (Some(valid.key) == self.request && valid.pair == pair).then_some(())?;
        if valid.revision == self.revision {
            Some(Pixels::default())
        } else {
            valid.support?.union(self.support?)?.disjoint()
        }
    }
    pub(crate) fn encoded(&mut self, pair: PairIdentity, reused: bool, copy_bytes: u64) {
        self.encoded = self.request.map(|key| Completed {
            key,
            pair,
            support: self.support,
            revision: self.revision,
        });
        self.reused = reused;
        self.copy_bytes = copy_bytes;
    }
    pub(super) fn take_encoded(&mut self) -> Option<Completed> {
        self.request = None;
        self.encoded.take()
    }
    pub(super) fn complete(&mut self, candidate: Option<Completed>, key: Key, success: bool) {
        self.valid = candidate.filter(|c| success && c.key == key);
    }
    pub(crate) fn reused(&self) -> bool {
        self.reused
    }
    pub(crate) fn copy_bytes(&self) -> u64 {
        self.copy_bytes
    }
}

// Image validity metadata is fixed inline storage, with no owned lists.
const _: () =
    assert!(std::mem::size_of::<Prefix>() + std::mem::size_of::<Option<Completed>>() <= 4096);
