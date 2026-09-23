//! Process ownership and admission for owned text-cache capacities.
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

const PROCESS_LIMIT: usize = 32 * 1024 * 1024;
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
static OWNERS: Mutex<Registry> = Mutex::new(Registry(Vec::new()));

struct Registry(Vec<TextCacheOwnerUsage>);
impl Registry {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Mutex<Self>>()
            + crate::cpu_alloc::heap::capacity_bytes::<TextCacheOwnerUsage>(self.0.capacity())
    }
    fn insert(&mut self, id: u64, usage: TextCacheOwnerUsage) {
        if let Some(owner) = self.0.iter_mut().find(|owner| owner.owner_id == id) {
            *owner = usage;
        } else {
            self.0.push(usage);
        }
    }
    fn remove(&mut self, id: u64) {
        self.0.retain(|owner| owner.owner_id != id);
        if self.0.capacity() > self.0.len().saturating_mul(4) {
            self.0 = std::mem::take(&mut self.0).into_boxed_slice().into_vec();
        }
    }
}

/// Public retained capacities plus measured Arc/Datum headers; private scratch is separate.
#[derive(Clone, Copy, Debug)]
pub struct TextCacheOwnerUsage {
    pub owner_id: u64,
    pub bytes: usize,
    /// This owner is preparing a frame, including a warm frame with unchanged storage.
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
            .remove(self.0);
    }
}

fn allowance(owners: &Registry, id: u64, limit: usize) -> usize {
    limit.saturating_sub(owners.bytes()).saturating_sub(
        owners
            .0
            .iter()
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

pub(crate) struct Admission {
    pub required_bytes: usize,
    pub retained_bytes: usize,
}

/// Current-frame layouts must be admitted before glyph preparation. Eviction
/// runs under the same process lock as post-frame retention.
pub(crate) fn admit(id: u64, trim: impl FnOnce(usize) -> Admission) -> anyhow::Result<()> {
    let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    let available = allowance(&owners, id, PROCESS_LIMIT).min(8 * 1024 * 1024);
    let Admission {
        required_bytes,
        retained_bytes,
    } = trim(available);
    owners.insert(
        id,
        TextCacheOwnerUsage {
            owner_id: id,
            bytes: retained_bytes,
            preparing: required_bytes <= available,
            retention_overflow: required_bytes > available,
        },
    );
    anyhow::ensure!(
        required_bytes <= available,
        "required text layout exceeds CPU cache admission (requested {required_bytes} bytes; available {available})"
    );
    Ok(())
}

pub(crate) fn preparing(id: u64) {
    set_preparing(id, true);
}

pub(crate) fn submitted(id: u64) {
    set_preparing(id, false);
}

fn set_preparing(id: u64, preparing: bool) {
    if let Some(owner) = OWNERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .0
        .iter_mut()
        .find(|owner| owner.owner_id == id)
    {
        owner.preparing = preparing;
    }
}

impl crate::Renderer {
    /// Enumerate CPU text capacities and accounted headers for every live renderer cache.
    /// Preparing owners may exceed retention caps; this is not scratch accounting.
    /// Add text_cache_registry_bytes once for total process ownership.
    pub fn text_cache_process_usage() -> Vec<TextCacheOwnerUsage> {
        OWNERS.lock().unwrap_or_else(|e| e.into_inner()).0.clone()
    }

    /// Shared owner registry, including spare capacity and Datum allocation headers.
    /// Charged once to the process allowance, not to each renderer's local cap.
    pub fn text_cache_registry_bytes() -> usize {
        OWNERS.lock().unwrap_or_else(|e| e.into_inner()).bytes()
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
        let mut owners = Registry(Vec::new());
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
        let limit = 32 + owners.bytes();
        assert_eq!(allowance(&owners, 5, limit), 0);
        owners.remove(1);
        assert_eq!(allowance(&owners, 5, limit), 8);
        assert_eq!(allowance(&owners, 2, limit), 8);
    }
    #[test]
    fn registry_reports_actual_heap_through_growth_update_and_close() {
        let scope = crate::cpu_alloc::Scope::new("text-owner-registry");
        let mut registry = Registry(Vec::new());
        let assert_heap = |registry: &Registry| {
            let usage = scope.usage();
            assert_eq!(
                registry.bytes() - std::mem::size_of::<Mutex<Registry>>(),
                (usage.payload_bytes + usage.tracking_bytes) as usize
            );
        };
        for id in 1..=64 {
            scope.with(|| {
                registry.insert(
                    id,
                    TextCacheOwnerUsage {
                        owner_id: id,
                        bytes: 8,
                        preparing: true,
                        retention_overflow: false,
                    },
                )
            });
            assert_heap(&registry);
        }
        let before = registry.bytes();
        scope.with(|| {
            registry.insert(
                64,
                TextCacheOwnerUsage {
                    owner_id: 64,
                    bytes: 16,
                    preparing: false,
                    retention_overflow: false,
                },
            )
        });
        assert_eq!(registry.bytes(), before, "warm updates do not allocate");
        assert_eq!(allowance(&registry, 64, before + 63 * 8 + 16), 16);
        assert_eq!(allowance(&registry, 64, before + 63 * 8 + 15), 15);
        for id in 1..=64 {
            scope.with(|| registry.remove(id));
            assert_heap(&registry);
        }
        assert_eq!(registry.0.capacity(), 0);
        assert_eq!(scope.usage().allocations, 0);
    }
}
