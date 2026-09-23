//! Bounded scalar text measurements and per-thread retained ownership.
use super::TextFace;
use crate::WidthMeasurementCacheUsage;
use crate::cpu_alloc::heap::capacity_bytes;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

const MAX_MEASUREMENTS: usize = 256;
const MAX_MEASUREMENT_TEXT_BYTES: usize = 64 * 1024;
const MAX_OWNED_BYTES: usize = 128 * 1024;
type Entry = (String, u32, TextFace, MeasurementKind, f32);

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
static OWNERS: Mutex<Vec<WidthMeasurementCacheUsage>> = Mutex::new(Vec::new());

pub(crate) fn usage() -> Vec<WidthMeasurementCacheUsage> {
    OWNERS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MeasurementKind {
    Width,
    WrappedHeight(u32),
}

pub(super) struct MeasurementCache {
    entries: std::collections::VecDeque<Entry>,
    text_bytes: usize,
    text_heap_bytes: usize,
    owner_id: u64,
    #[cfg(test)]
    pub(super) misses: usize,
}

impl Default for MeasurementCache {
    fn default() -> Self {
        let cache = Self {
            entries: Default::default(),
            text_bytes: 0,
            text_heap_bytes: 0,
            owner_id: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            #[cfg(test)]
            misses: 0,
        };
        OWNERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(WidthMeasurementCacheUsage {
                owner_id: cache.owner_id,
                thread: std::thread::current().id(),
                entries: 0,
                key_bytes: 0,
                retained_bytes: cache.retained_bytes(),
            });
        cache
    }
}

impl Drop for MeasurementCache {
    fn drop(&mut self) {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        owners.retain(|owner| owner.owner_id != self.owner_id);
        // Accounting must not retain storage for an obsolete thread peak.
        if owners.capacity() > owners.len().saturating_mul(4) {
            *owners = std::mem::take(&mut *owners).into_boxed_slice().into_vec();
        }
    }
}

impl MeasurementCache {
    fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + capacity_bytes::<Entry>(self.entries.capacity())
            + self.text_heap_bytes
    }

    fn publish_usage(&self) {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        let owner = owners
            .iter_mut()
            .find(|owner| owner.owner_id == self.owner_id)
            .expect("measurement owner registered until drop");
        owner.entries = self.entries.len();
        owner.key_bytes = self.text_bytes;
        owner.retained_bytes = self.retained_bytes();
    }

    pub(super) fn measure(
        &mut self,
        text: &str,
        size: f32,
        face: TextFace,
        miss: impl FnOnce() -> f32,
    ) -> f32 {
        self.measure_kind(text, size, face, MeasurementKind::Width, miss)
    }

    pub(super) fn measure_kind(
        &mut self,
        text: &str,
        size: f32,
        face: TextFace,
        kind: MeasurementKind,
        miss: impl FnOnce() -> f32,
    ) -> f32 {
        if let Some(index) = self
            .entries
            .iter()
            .position(|(label, bits, font, cached_kind, _)| {
                *bits == size.to_bits()
                    && *font == face
                    && *cached_kind == kind
                    && label.as_str() == text
            })
        {
            let entry = self.entries.remove(index).expect("matched entry exists");
            let width = entry.4;
            self.entries.push_front(entry);
            return width;
        }
        #[cfg(test)]
        {
            self.misses += 1;
        }
        let width = miss();
        // Arbitrary user text must not turn scalar measurement reuse into an
        // unbounded string history. Oversized strings are measured but not held.
        if text.len() <= MAX_MEASUREMENT_TEXT_BYTES {
            if self.entries.len() < MAX_MEASUREMENTS
                && self.entries.len() == self.entries.capacity()
                && self.entries.try_reserve(1).is_err()
            {
                return width;
            }
            while self.entries.len() >= MAX_MEASUREMENTS
                || self.text_bytes + text.len() > MAX_MEASUREMENT_TEXT_BYTES
                || self.retained_bytes() + capacity_bytes::<u8>(text.len()) > MAX_OWNED_BYTES
            {
                let old = self.entries.pop_back().expect("bounded cache has entries");
                self.text_bytes -= old.0.len();
                self.text_heap_bytes -= capacity_bytes::<u8>(old.0.capacity());
            }
            // Retention is optional after measurement. Avoid an infallible
            // boxed-string conversion/shrink and account actual key capacity.
            let mut key = String::new();
            if key.try_reserve_exact(text.len()).is_ok() {
                key.push_str(text);
                let key_bytes = capacity_bytes::<u8>(key.capacity());
                if self.retained_bytes().saturating_add(key_bytes) <= MAX_OWNED_BYTES {
                    self.text_bytes += key.len();
                    self.text_heap_bytes += key_bytes;
                    self.entries
                        .push_front((key, size.to_bits(), face, kind, width));
                }
            }
            // Eviction or deque growth may have changed ownership even if the
            // optional key allocation failed. Always publish that final state.
            self.publish_usage();
        }
        width
    }
}

#[cfg(test)]
mod tests {
    use super::super::measure_uncached;
    use super::*;
    #[test]
    fn exact_measurements_reuse_only_identical_font_size_and_text() {
        let mut cache = MeasurementCache::default();
        for (text, size, face) in [
            ("Project Preferences", 13.0, TextFace::Ui),
            ("Project Preferences", 14.0, TextFace::Ui),
            ("Project Preferences", 13.0, TextFace::UiStrong),
            ("Project Preferences", 13.0, TextFace::UiMedium),
            ("Project Preferences", 13.0, TextFace::Terminal),
            ("Global Preferences", 13.0, TextFace::Ui),
            ("Units · µm", 13.0, TextFace::Mono),
        ] {
            let actual = measure_uncached(text, size, face);
            let first = cache.measure(text, size, face, || actual);
            assert_eq!(first.to_bits(), actual.to_bits());
            assert_eq!(
                cache
                    .measure(text, size, face, || panic!("cache hit reshaped text"))
                    .to_bits(),
                actual.to_bits()
            );
        }
        assert_eq!(cache.entries.len(), 7);
    }

    #[test]
    fn measurement_cache_bounds_changing_labels_and_keeps_recent_hits() {
        let mut cache = MeasurementCache::default();
        for n in 0..1000 {
            let kind = if n % 2 == 0 {
                MeasurementKind::WrappedHeight(100.0_f32.to_bits())
            } else {
                MeasurementKind::Width
            };
            cache.measure_kind(&format!("label-{n}"), 13.0, TextFace::Ui, kind, || n as f32);
            assert!(cache.entries.len() <= MAX_MEASUREMENTS);
            assert!(cache.text_bytes <= MAX_MEASUREMENT_TEXT_BYTES);
            assert!(cache.retained_bytes() <= MAX_OWNED_BYTES);
        }
        assert_eq!(
            cache.measure("label-999", 13.0, TextFace::Ui, || panic!(
                "recent label evicted"
            )),
            999.0
        );
        let large = "x".repeat(MAX_MEASUREMENT_TEXT_BYTES + 1);
        cache.measure(&large, 13.0, TextFace::Ui, || 1.0);
        assert!(!cache.entries.iter().any(|entry| entry.0.as_str() == large));
        for n in 0..20 {
            cache.measure(
                &format!("{n}{}", "x".repeat(8000)),
                13.0,
                TextFace::Ui,
                || 1.0,
            );
            assert!(cache.text_bytes <= MAX_MEASUREMENT_TEXT_BYTES);
            assert!(cache.retained_bytes() <= MAX_OWNED_BYTES);
        }
    }
    #[test]
    fn thread_local_owners_report_exact_retention_and_retire_at_thread_exit() {
        use std::sync::{Arc, Barrier, mpsc};
        let barrier = Arc::new(Barrier::new(3));
        let (send, receive) = mpsc::channel();
        let mut threads = Vec::new();
        for text in ["owned width alpha", "owned width beta"] {
            let barrier = barrier.clone();
            let send = send.clone();
            threads.push(std::thread::spawn(move || {
                super::super::measured_text_run_width_px(text, 13.0, TextFace::Ui);
                let thread = std::thread::current().id();
                send.send((thread, text.len())).unwrap();
                barrier.wait();
            }));
        }
        let identities = [receive.recv().unwrap(), receive.recv().unwrap()];
        let report = crate::Renderer::width_measurement_cache_usage();
        barrier.wait();
        for thread in threads {
            thread.join().unwrap();
        }

        let mut ids = Vec::new();
        for (thread, key_bytes) in identities {
            let owners: Vec<_> = report.iter().filter(|r| r.thread == thread).collect();
            assert_eq!(owners.len(), 1);
            let owner = owners[0];
            assert_eq!(owner.entries, 1);
            assert_eq!(owner.key_bytes, key_bytes);
            assert!(owner.retained_bytes >= key_bytes + std::mem::size_of::<Entry>());
            assert!(owner.retained_bytes <= MAX_OWNED_BYTES);
            ids.push(owner.owner_id);
        }
        assert_ne!(ids[0], ids[1]);
        assert!(
            crate::Renderer::width_measurement_cache_usage()
                .iter()
                .all(|owner| !ids.contains(&owner.owner_id))
        );
    }

    #[test]
    fn ownership_report_matches_eviction_and_oversize_bypass() {
        let mut cache = MeasurementCache::default();
        for n in 0..1000 {
            cache.measure(
                &format!("{n}{}", "x".repeat(8000)),
                13.0,
                TextFace::Ui,
                || 7.0,
            );
        }
        let report = || {
            usage()
                .into_iter()
                .find(|r| r.owner_id == cache.owner_id)
                .unwrap()
        };
        let before = report();
        assert_eq!(before.entries, cache.entries.len());
        assert_eq!(
            before.key_bytes,
            cache.entries.iter().map(|e| e.0.len()).sum::<usize>()
        );
        assert_eq!(before.retained_bytes, cache.retained_bytes());
        let id = cache.owner_id;
        cache.measure(
            &"x".repeat(MAX_MEASUREMENT_TEXT_BYTES + 1),
            13.0,
            TextFace::Ui,
            || 9.0,
        );
        let after = usage().into_iter().find(|r| r.owner_id == id).unwrap();
        assert_eq!(after.retained_bytes, before.retained_bytes);
        assert_eq!(after.key_bytes, before.key_bytes);
        drop(cache);
        assert!(usage().iter().all(|r| r.owner_id != id));
    }
}

#[cfg(test)]
#[path = "measurement_heap_tests.rs"]
mod heap_tests;
