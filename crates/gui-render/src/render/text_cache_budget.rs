//! Process ownership and admission for owned text-cache capacities.
use std::collections::BTreeMap;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

const PROCESS_LIMIT: usize = 32 * 1024 * 1024;
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
static OWNERS: Mutex<BTreeMap<u64, TextCacheOwnerUsage>> = Mutex::new(BTreeMap::new());

/// Public retained capacities plus measured Arc/Datum headers; private scratch is separate.
#[derive(Clone, Copy, Debug)]
pub struct TextCacheOwnerUsage {
    pub owner_id: u64,
    pub bytes: usize,
    /// Changed CPU layout exists before successful submission/retention admission.
    pub preparing: bool,
    /// Required owner storage could not fit even after cache retirement.
    pub retention_overflow: bool,
}

pub(crate) struct Owner(u64);
impl Owner {
    pub fn new(bytes: usize) -> Self {
        let owner = Self(NEXT_OWNER.fetch_add(1, Ordering::Relaxed));
        owner.publish(bytes);
        owner
    }
    pub fn id(&self) -> u64 {
        self.0
    }
    pub fn publish(&self, bytes: usize) {
        OWNERS.lock().unwrap_or_else(|e| e.into_inner()).insert(
            self.0,
            TextCacheOwnerUsage {
                owner_id: self.0,
                bytes,
                preparing: true,
                retention_overflow: false,
            },
        );
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        OWNERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.0);
    }
}

fn allowance(owners: &BTreeMap<u64, TextCacheOwnerUsage>, id: u64, limit: usize) -> usize {
    limit.saturating_sub(
        owners
            .values()
            .filter(|o| o.owner_id != id)
            .map(|o| o.bytes)
            .sum(),
    )
}

/// Serialize admission across render threads. The callback only trims its own
/// cache and must not publish recursively. Active other owners remain charged.
pub(crate) fn settle(id: u64, trim: impl FnOnce(usize) -> usize) {
    let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    let available = allowance(&owners, id, PROCESS_LIMIT).min(8 * 1024 * 1024);
    let bytes = trim(available);
    owners.insert(
        id,
        TextCacheOwnerUsage {
            owner_id: id,
            bytes,
            preparing: false,
            retention_overflow: bytes > available,
        },
    );
}

/// Current-frame layouts must be admitted before glyph preparation. Eviction
/// runs under the same process lock as post-frame retention.
pub(crate) fn admit(id: u64, trim: impl FnOnce(usize) -> usize) -> anyhow::Result<()> {
    let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    let available = allowance(&owners, id, PROCESS_LIMIT).min(8 * 1024 * 1024);
    let bytes = trim(available);
    owners.insert(
        id,
        TextCacheOwnerUsage {
            owner_id: id,
            bytes,
            preparing: true,
            retention_overflow: bytes > available,
        },
    );
    anyhow::ensure!(
        bytes <= available,
        "required text layout exceeds CPU cache admission (requested {bytes} bytes; available {available})"
    );
    Ok(())
}

pub(crate) fn submitted(id: u64) {
    if let Some(owner) = OWNERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(&id)
    {
        owner.preparing = false;
    }
}

impl crate::Renderer {
    /// Enumerate CPU text capacities and accounted headers for every live renderer cache.
    /// Preparing owners may exceed retention caps; this is not scratch accounting.
    pub fn text_cache_process_usage() -> Vec<TextCacheOwnerUsage> {
        OWNERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .copied()
            .collect()
    }

    pub fn text_cache_key_usage(&self) -> crate::TextCacheKeyUsage {
        self.text_buffers.key_usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_counts_other_owners_including_active_frames() {
        let mut owners = BTreeMap::new();
        for id in 1..=5 {
            owners.insert(
                id,
                TextCacheOwnerUsage {
                    owner_id: id,
                    bytes: 8,
                    preparing: id == 2,
                    retention_overflow: false,
                },
            );
        }
        assert_eq!(allowance(&owners, 5, 32), 0);
        owners.remove(&1);
        assert_eq!(allowance(&owners, 5, 32), 8);
        assert_eq!(allowance(&owners, 2, 32), 8);
    }
}
