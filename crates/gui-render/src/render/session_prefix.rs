//! Image validity follows immutable preparation and successful completion only.
use super::frame_revision::Target;
use crate::gpu_surface::CompositionIdentity;
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
    composition: CompositionIdentity,
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
    encoded: Option<CompositionIdentity>,
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
        // A direct full render has no matching session receipt. It cannot
        // authorize reuse of the previous successfully presented A/C pair.
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

    pub(crate) fn damage(&self, composition: CompositionIdentity) -> Option<Pixels> {
        let valid = self.valid?;
        (Some(valid.key) == self.request && valid.composition == composition).then_some(())?;
        if valid.revision == self.revision {
            Some(Pixels::default())
        } else {
            valid.support?.union(self.support?)?.disjoint()
        }
    }
    pub(crate) fn encoded(
        &mut self,
        composition: CompositionIdentity,
        reused: bool,
        copy_bytes: u64,
    ) {
        // Request/support/revision are already owned by this immutable attempt.
        // Store only the new image identity until the linear receipt takes it.
        self.encoded = self.request.map(|_| composition);
        self.reused = reused;
        self.copy_bytes = copy_bytes;
    }
    pub(super) fn take_encoded(&mut self) -> Option<Completed> {
        let key = self.request.take();
        let composition = self.encoded.take();
        Some(Completed {
            key: key?,
            composition: composition?,
            support: self.support,
            revision: self.revision,
        })
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

// Include the linear receipt's key/completion fields, transient tile plan and
// image-owner handle containers. Global allocation/queue-observer machinery has
// its existing independent accounting; it is not hidden in a validity list.
const IMAGE_METADATA_BYTES: usize = std::mem::size_of::<Prefix>()
    + std::mem::size_of::<Option<Completed>>()
    + std::mem::size_of::<Option<Key>>()
    + std::mem::size_of::<crate::renderer_state::damage::regional::Plan>()
    + crate::gpu_surface::IMAGE_HANDLE_METADATA_BYTES;
const _: () = assert!(IMAGE_METADATA_BYTES <= 4096);

#[cfg(test)]
mod metadata_tests {
    #[test]
    fn regional_image_metadata_includes_receipt_plan_and_handles() {
        const { assert!(super::IMAGE_METADATA_BYTES <= 4096) };
        eprintln!(
            "regional image metadata: {} bytes",
            super::IMAGE_METADATA_BYTES
        );
    }
}
