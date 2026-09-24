//! Process ownership and admission for owned text-cache capacities.
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

pub(super) const LOCAL_LIMIT: usize = 8 * 1024 * 1024;
const PROCESS_LIMIT: usize = 32 * 1024 * 1024;
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
static OWNERS: Mutex<Registry> = Mutex::new(Registry(Vec::new()));

struct Registry(Vec<TextCacheOwnerUsage>);
impl Registry {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Mutex<Self>>()
            + crate::cpu_alloc::heap::capacity_bytes::<TextCacheOwnerUsage>(self.0.capacity())
    }
    fn insert(&mut self, id: u64, mut usage: TextCacheOwnerUsage) {
        if let Some(owner) = self.0.iter_mut().find(|owner| owner.owner_id == id) {
            usage.constructing_bytes = owner.constructing_bytes;
            usage.epoch = owner.epoch;
            *owner = usage;
        } else {
            self.0.push(usage);
        }
    }
    fn register(&mut self, bytes: usize) -> anyhow::Result<Owner> {
        anyhow::ensure!(bytes <= LOCAL_LIMIT, "text owner exceeds local capacity");
        let live = self
            .0
            .iter()
            .try_fold(bytes, |total, owner| {
                total
                    .checked_add(owner.bytes)?
                    .checked_add(owner.constructing_bytes)
            })
            .ok_or_else(|| anyhow::anyhow!("text owner capacity overflow"))?;
        let existing = self.bytes();
        anyhow::ensure!(
            live.saturating_add(existing) <= PROCESS_LIMIT,
            "text owner exceeds process capacity"
        );
        if self.0.len() == self.0.capacity() {
            let needed = self
                .0
                .len()
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("text owner count overflow"))?;
            let preferred = self.0.capacity().saturating_mul(2).max(4).max(needed);
            let mut capacity = preferred;
            let replacement_bytes = loop {
                let amount =
                    crate::text_gpu::staging_vec::StagingVec::<TextCacheOwnerUsage>::capacity_bytes(
                        capacity,
                    )? as usize;
                if live.saturating_add(existing).saturating_add(amount) <= PROCESS_LIMIT {
                    break amount;
                }
                anyhow::ensure!(
                    capacity != needed,
                    "text registry replacement exceeds process capacity"
                );
                capacity = needed;
            };
            // The registry lock serializes this reservation with every other
            // owner admission. Charge both buffers until the old one is dropped.
            let mut replacement = Vec::new();
            replacement.try_reserve_exact(capacity)?;
            anyhow::ensure!(
                crate::cpu_alloc::heap::capacity_bytes::<TextCacheOwnerUsage>(
                    replacement.capacity()
                ) == replacement_bytes,
                "text registry capacity differs from admission"
            );
            replacement.extend_from_slice(&self.0);
            self.0 = replacement;
        }
        let owner = Owner(NEXT_OWNER.fetch_add(1, Ordering::Relaxed));
        self.0.push(TextCacheOwnerUsage {
            owner_id: owner.0,
            bytes,
            constructing_bytes: 0,
            epoch: 0,
            preparing: true,
            retention_overflow: false,
        });
        Ok(owner)
    }

    fn remove(&mut self, id: u64) {
        self.0.retain(|owner| owner.owner_id != id);
        // Removing an owner must not allocate outside admission. Preserve the
        // charged slots for another host; release storage after the final owner.
        if self.0.is_empty() {
            self.0 = Vec::new();
        }
    }
}

/// Public retained capacities plus measured Arc/Datum headers; private scratch is separate.
#[derive(Clone, Copy, Debug)]
pub struct TextCacheOwnerUsage {
    pub owner_id: u64,
    pub bytes: usize,
    /// Admitted allocations under construction, not yet included in retained bytes.
    pub constructing_bytes: usize,
    epoch: u64,
    /// This owner is preparing a frame, including a warm frame with unchanged storage.
    pub preparing: bool,
    /// Required owner storage could not fit even after cache retirement.
    pub retention_overflow: bool,
}

pub(crate) struct Owner(u64);
impl Owner {
    pub fn try_new(bytes: usize) -> anyhow::Result<Self> {
        OWNERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .register(bytes)
    }
    pub fn reserve(&self, bytes: usize) -> anyhow::Result<Construction> {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        let available = allowance(&owners, self.0, PROCESS_LIMIT).min(LOCAL_LIMIT);
        let owner = owners
            .0
            .iter_mut()
            .find(|o| o.owner_id == self.0)
            .expect("live text owner must be registered");
        let next = owner
            .constructing_bytes
            .checked_add(bytes)
            .ok_or_else(|| anyhow::anyhow!("text construction reservation overflow"))?;
        if owner.bytes.checked_add(next).is_none_or(|n| n > available) {
            return Err(ConstructionRefusal { bytes }.into());
        }
        owner.constructing_bytes = next;
        Ok(Construction {
            owner: self.0,
            epoch: owner.epoch,
            bytes,
        })
    }

    pub fn id(&self) -> u64 {
        self.0
    }
    /// Atomically transfer all this preparation's surviving allocations into
    /// retained usage. Old leases become inert; observers never count both.
    pub fn publish(&self, bytes: usize) {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(owner) = owners.0.iter_mut().find(|o| o.owner_id == self.0) {
            owner.bytes = bytes;
            owner.constructing_bytes = 0;
            owner.epoch = owner
                .epoch
                .checked_add(1)
                .expect("text publication epoch exhausted");
            owner.preparing = true;
            owner.retention_overflow = false;
        } else {
            owners.insert(
                self.0,
                TextCacheOwnerUsage {
                    owner_id: self.0,
                    bytes,
                    constructing_bytes: 0,
                    epoch: 0,
                    preparing: true,
                    retention_overflow: false,
                },
            );
        }
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

#[derive(Debug)]
pub(crate) struct ConstructionRefusal {
    pub bytes: usize,
}
impl std::fmt::Display for ConstructionRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "text construction exceeds CPU cache admission (requested {} bytes)",
            self.bytes
        )
    }
}
impl std::error::Error for ConstructionRefusal {}

/// Lives through allocation and publication; replacement reserves old/new overlap.
/// Drop after publishing retained capacity, or after releasing refused storage.
pub(crate) struct Construction {
    owner: u64,
    epoch: u64,
    bytes: usize,
}
impl Construction {
    /// Release conservative construction headroom after measuring actual capacity.
    /// Publication still transfers the remaining lease atomically to retained usage.
    pub(crate) fn shrink_to(&mut self, bytes: usize) {
        assert!(bytes <= self.bytes, "construction exceeded admitted bound");
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(owner) = owners.0.iter_mut().find(|o| o.owner_id == self.owner)
            && owner.epoch == self.epoch
        {
            owner.constructing_bytes -= self.bytes - bytes;
        }
        self.bytes = bytes;
    }
}

impl Drop for Construction {
    fn drop(&mut self) {
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(owner) = owners.0.iter_mut().find(|o| o.owner_id == self.owner)
            && owner.epoch == self.epoch
        {
            owner.constructing_bytes = owner
                .constructing_bytes
                .checked_sub(self.bytes)
                .expect("text construction reservation released twice");
        }
    }
}

fn allowance(owners: &Registry, id: u64, limit: usize) -> usize {
    limit.saturating_sub(owners.bytes()).saturating_sub(
        owners
            .0
            .iter()
            .filter(|o| o.owner_id != id)
            .map(|o| o.bytes.saturating_add(o.constructing_bytes))
            .sum(),
    )
}

/// Serialize admission across render threads. The callback only trims its own
/// cache and must not publish recursively. Active other owners remain charged.
pub(crate) fn settle(id: u64, trim: impl FnOnce(usize) -> usize) {
    let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    let available = allowance(&owners, id, PROCESS_LIMIT).min(LOCAL_LIMIT);
    let bytes = trim(available);
    owners.insert(
        id,
        TextCacheOwnerUsage {
            owner_id: id,
            bytes,
            constructing_bytes: 0,
            epoch: 0,
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
    let available = allowance(&owners, id, PROCESS_LIMIT).min(LOCAL_LIMIT);
    let Admission {
        required_bytes,
        retained_bytes,
    } = trim(available);
    owners.insert(
        id,
        TextCacheOwnerUsage {
            owner_id: id,
            bytes: retained_bytes,
            constructing_bytes: 0,
            epoch: 0,
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
    /// Includes in-progress reservations; this is not scratch accounting.
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
    impl Owner {
        /// Test fixtures can synthesize full/overfull owners without allocating payload.
        pub fn new(bytes: usize) -> Self {
            let owner = Self(NEXT_OWNER.fetch_add(1, Ordering::Relaxed));
            owner.publish(bytes);
            owner
        }
    }

    #[test]
    #[ignore = "requires serial process-wide text admission"]
    fn construction_is_admitted_across_owners_and_transfers_once_on_publication() {
        let owner = Owner::new(128);
        let lease = owner.reserve(LOCAL_LIMIT - 128).unwrap();
        assert!(owner.reserve(1).is_err());
        let inspect = || {
            crate::Renderer::text_cache_process_usage()
                .into_iter()
                .find(|entry| entry.owner_id == owner.id())
                .unwrap()
        };
        assert_eq!(inspect().bytes, 128);
        assert_eq!(inspect().constructing_bytes, LOCAL_LIMIT - 128);
        owner.publish(LOCAL_LIMIT);
        assert_eq!(inspect().constructing_bytes, 0);
        drop(lease);
        assert_eq!(inspect().bytes, LOCAL_LIMIT);
        assert_eq!(inspect().constructing_bytes, 0);
        owner.publish(0);
        let other = Owner::new(0);
        let others: usize = crate::Renderer::text_cache_process_usage()
            .iter()
            .filter(|entry| entry.owner_id != owner.id() && entry.owner_id != other.id())
            .map(|entry| entry.bytes + entry.constructing_bytes)
            .sum();
        let filler = Owner::new(0);
        let registry = crate::Renderer::text_cache_registry_bytes();
        filler.publish(PROCESS_LIMIT - registry - others - 1000);
        let first = owner.reserve(600).unwrap();
        assert!(other.reserve(401).is_err());
        let second = other.reserve(400).unwrap();
        drop(first);
        let replacement = owner.reserve(600).unwrap();
        drop((second, replacement));
        assert_eq!(inspect().constructing_bytes, 0);
    }

    #[test]
    #[ignore = "requires serial process-wide text admission"]
    fn owner_creation_admits_registry_overlap_and_refuses_without_mutation() {
        let filler = Owner::new(0);
        let mut slots = Vec::new();
        loop {
            let full = {
                let owners = OWNERS.lock().unwrap();
                owners.0.len() == owners.0.capacity()
            };
            if full {
                break;
            }
            slots.push(Owner::new(0));
        }
        let (len, capacity, registry, others) = {
            let owners = OWNERS.lock().unwrap();
            (
                owners.0.len(),
                owners.0.capacity(),
                owners.bytes(),
                owners
                    .0
                    .iter()
                    .filter(|o| o.owner_id != filler.id())
                    .map(|o| o.bytes + o.constructing_bytes)
                    .sum::<usize>(),
            )
        };
        let incoming = 128;
        filler.publish(PROCESS_LIMIT - registry - others - incoming);
        assert!(Owner::try_new(LOCAL_LIMIT + 1).is_err());
        assert!(Owner::try_new(incoming).is_err());
        {
            let owners = OWNERS.lock().unwrap();
            assert_eq!(
                (owners.0.len(), owners.0.capacity(), owners.bytes()),
                (len, capacity, registry)
            );
        }
        let replacement = crate::cpu_alloc::heap::capacity_bytes::<TextCacheOwnerUsage>(len + 1);
        filler.publish(PROCESS_LIMIT - registry - others - incoming - replacement);
        let admitted = Owner::try_new(incoming).unwrap();
        {
            let owners = OWNERS.lock().unwrap();
            assert_eq!(
                owners.0.capacity(),
                len + 1,
                "tight headroom uses exact growth"
            );
            assert!(
                owners.bytes()
                    + owners
                        .0
                        .iter()
                        .map(|o| o.bytes + o.constructing_bytes)
                        .sum::<usize>()
                    <= PROCESS_LIMIT
            );
        }
        drop((admitted, slots, filler));
    }

    #[test]
    fn admission_counts_other_owners_including_active_frames() {
        let mut owners = Registry(Vec::new());
        for id in 1..=5 {
            owners.insert(
                id,
                TextCacheOwnerUsage {
                    owner_id: id,
                    bytes: 8,
                    constructing_bytes: 0,
                    epoch: 0,
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
                        constructing_bytes: 0,
                        epoch: 0,
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
                    constructing_bytes: 0,
                    epoch: 0,
                    preparing: false,
                    retention_overflow: false,
                },
            )
        });
        assert_eq!(registry.bytes(), before, "warm updates do not allocate");
        assert_eq!(allowance(&registry, 64, before + 63 * 8 + 16), 16);
        assert_eq!(allowance(&registry, 64, before + 63 * 8 + 15), 15);
        let pointer = registry.0.as_ptr();
        for id in 1..=64 {
            scope.with(|| registry.remove(id));
            assert_heap(&registry);
            if id < 64 {
                assert_eq!(registry.bytes(), before, "spare slots stay charged");
                assert_eq!(
                    registry.0.as_ptr(),
                    pointer,
                    "removal does not replace storage"
                );
            }
        }
        assert_eq!(registry.0.capacity(), 0);
        assert_eq!(scope.usage().allocations, 0);
    }
}
