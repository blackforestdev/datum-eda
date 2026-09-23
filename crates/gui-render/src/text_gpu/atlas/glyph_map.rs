//! Datum-owned glyph lookup with explicit, pre-admitted bucket storage.
use super::*;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

#[derive(Clone, Copy)]
struct Entry {
    hash: u64,
    key: CacheKey,
    value: Option<GlyphLocation>,
}

#[derive(Default)]
pub(super) struct GlyphMap {
    slots: StagingVec<Option<Entry>>,
    len: usize,
    hash: RandomState,
}

impl GlyphMap {
    pub fn allocated_bytes(&self) -> u64 {
        self.slots.allocated_bytes()
    }

    pub fn get(&self, key: &CacheKey) -> Option<&Option<GlyphLocation>> {
        if self.slots.is_empty() {
            return None;
        }
        let hash = self.hash.hash_one(key);
        let index = Self::slot(&self.slots, key, hash);
        self.slots[index].as_ref().map(|entry| &entry.value)
    }

    /// Keep at least half the buckets empty. Old/new tables remain charged
    /// together during growth; cached hashes avoid hashing every key again.
    pub fn reserve_one(
        &mut self,
        host: &std::sync::Arc<super::super::budget::Budget>,
    ) -> anyhow::Result<()> {
        if self.len < self.slots.len() / 2 {
            return Ok(());
        }
        let capacity = self
            .slots
            .len()
            .checked_mul(2)
            .ok_or_else(|| anyhow::anyhow!("glyph lookup capacity overflow"))?
            .max(64);
        let mut replacement = StagingVec::new(capacity, host)?;
        replacement.extend(std::iter::repeat_n(None, capacity));
        for entry in self.slots.iter().flatten() {
            let index = Self::slot(&replacement, &entry.key, entry.hash);
            replacement[index] = Some(*entry);
        }
        self.slots = replacement;
        Ok(())
    }

    pub fn insert(&mut self, key: CacheKey, value: Option<GlyphLocation>) {
        assert!(
            !self.slots.is_empty(),
            "reserve glyph lookup before insertion"
        );
        let hash = self.hash.hash_one(key);
        let index = Self::slot(&self.slots, &key, hash);
        if self.slots[index].is_none() {
            assert!(
                self.len < self.slots.len() / 2,
                "glyph lookup exceeds admitted load"
            );
            self.len += 1;
        }
        self.slots[index] = Some(Entry { hash, key, value });
    }

    pub fn clear(&mut self) {
        self.slots.fill(None);
        self.len = 0;
    }

    fn slot(slots: &[Option<Entry>], key: &CacheKey, hash: u64) -> usize {
        let mask = slots.len() - 1;
        let mut index = hash as usize & mask;
        while let Some(entry) = &slots[index] {
            if entry.hash == hash && entry.key == *key {
                break;
            }
            index = (index + 1) & mask;
        }
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_lookup_growth_refusal_and_repack_preserve_exact_keys_and_capacity() {
        let fonts = crate::load_datum_fonts();
        let font = fonts.db().faces().next().unwrap().id;
        let key = |id| {
            CacheKey::new(
                font,
                id,
                18.0,
                (0.0, 0.0),
                glyphon::cosmic_text::fontdb::Weight::NORMAL,
                glyphon::cosmic_text::CacheKeyFlags::empty(),
            )
            .0
        };
        let small = StagingVec::<Option<Entry>>::capacity_bytes(64).unwrap();
        let large = StagingVec::<Option<Entry>>::capacity_bytes(128).unwrap();
        let host = super::super::super::budget::Budget::new(small + large);
        // Process admission is shared infrastructure, not bucket storage. Create
        // its singleton outside this owner's scope even when this test runs alone.
        let process = super::super::super::budget::staging_process();
        let process_baseline = process.used();
        let scope = crate::cpu_alloc::Scope::new("glyph-lookup-buckets");
        let mut map = GlyphMap::default();
        for id in 0..32 {
            scope.with(|| map.reserve_one(&host)).unwrap();
            map.insert(key(id), None);
        }
        let blocker = host.reserve(1).unwrap();
        assert!(map.reserve_one(&host).is_err());
        assert_eq!(map.allocated_bytes(), small);
        for id in 0..32 {
            assert!(matches!(map.get(&key(id)), Some(None)));
        }
        assert!(map.get(&key(32)).is_none());
        drop(blocker);
        scope.with(|| map.reserve_one(&host)).unwrap();
        for id in 32..64 {
            map.insert(
                key(id),
                Some(GlyphLocation {
                    page: 1,
                    origin: [id as u32, 9],
                    size: [2, 3],
                    bearing: [0, 0],
                    color: false,
                }),
            );
        }
        assert_eq!(map.allocated_bytes(), large);
        assert_eq!(host.used(), large);
        assert_eq!(process.used(), process_baseline + large);
        let usage = scope.usage();
        assert_eq!(usage.payload_bytes + usage.tracking_bytes, large);
        for id in 0..64 {
            assert_eq!(
                map.get(&key(id)).map(|v| v.map(|x| x.origin)),
                Some((id >= 32).then_some([id as u32, 9]))
            );
        }
        map.insert(
            key(7),
            Some(GlyphLocation {
                page: 0,
                origin: [7, 11],
                size: [1, 1],
                bearing: [0, 0],
                color: true,
            }),
        );
        assert_eq!(map.len, 64);
        assert_eq!(map.get(&key(7)).unwrap().unwrap().origin, [7, 11]);
        map.clear();
        assert!(map.get(&key(7)).is_none());
        assert_eq!(
            host.used(),
            large,
            "repack retains and charges reusable buckets"
        );
        map.reserve_one(&host).unwrap();
        map.insert(key(7), None);
        map.clear();
        // 129 distinct keys in 128 initial buckets guarantee a collision.
        let mut buckets = [None; 128];
        let mut collision = None;
        for id in 0..=128 {
            let candidate = key(id);
            let bucket = map.hash.hash_one(candidate) as usize & 127;
            if let Some(previous) = buckets[bucket] {
                collision = Some((previous, candidate));
                break;
            }
            buckets[bucket] = Some(candidate);
        }
        let (first, second) = collision.unwrap();
        map.insert(first, None);
        map.insert(second, None);
        assert!(matches!(map.get(&first), Some(None)));
        assert!(matches!(map.get(&second), Some(None)));
        assert_eq!(map.len, 2);
        drop(map);
        assert_eq!(host.used(), 0);
        assert_eq!(process.used(), process_baseline);
        assert_eq!(
            scope.usage().payload_bytes + scope.usage().tracking_bytes,
            0
        );
    }
}
