//! Fixed-capacity CPU upload plans charged before allocation and through use.
use super::budget::{Budget, Permit, staging_process};
use std::sync::Arc;

pub(crate) struct StagingVec<T> {
    // Release elements and storage before their admission permits.
    values: Vec<T>,
    _permits: Option<[Permit; 2]>,
}

impl<T> Default for StagingVec<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            _permits: None,
        }
    }
}

impl<T> StagingVec<T> {
    pub fn capacity_bytes(capacity: usize) -> anyhow::Result<u64> {
        let layout = std::alloc::Layout::array::<T>(capacity)?;
        // Leave room for Datum's allocator prefix and alignment padding.
        anyhow::ensure!(
            layout.size() <= isize::MAX as usize - layout.align() - std::mem::size_of::<usize>(),
            "upload metadata allocation layout overflow"
        );
        Ok(crate::cpu_alloc::heap::capacity_bytes::<T>(capacity) as u64)
    }

    pub fn new(capacity: usize, host: &Arc<Budget>) -> anyhow::Result<Self> {
        let bytes = Self::capacity_bytes(capacity)?;
        let permits = if bytes == 0 {
            None
        } else {
            Some([host.reserve(bytes)?, staging_process().reserve(bytes)?])
        };
        let mut values = Vec::new();
        values.try_reserve_exact(capacity)?;
        anyhow::ensure!(
            crate::cpu_alloc::heap::capacity_bytes::<T>(values.capacity()) as u64 == bytes,
            "upload metadata capacity differs from admitted layout"
        );
        Ok(Self {
            values,
            _permits: permits,
        })
    }

    pub fn allocated_bytes(&self) -> u64 {
        crate::cpu_alloc::heap::capacity_bytes::<T>(self.values.capacity()) as u64
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Preserve admitted storage on reuse; reserve old/new overlap before growth.
    pub fn ensure_capacity(&mut self, capacity: usize, host: &Arc<Budget>) -> anyhow::Result<()> {
        if capacity > self.values.capacity() {
            let grown = capacity.max(self.values.capacity().saturating_mul(2));
            let mut replacement = Self::new(grown, host).or_else(|error| {
                if grown == capacity {
                    Err(error)
                } else {
                    Self::new(capacity, host)
                }
            })?;
            replacement.values.append(&mut self.values);
            *self = replacement;
        }
        Ok(())
    }

    pub fn try_push(&mut self, value: T, host: &Arc<Budget>) -> anyhow::Result<()> {
        self.ensure_capacity(
            self.len()
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("staging element count overflow"))?,
            host,
        )?;
        self.push(value);
        Ok(())
    }

    pub fn push(&mut self, value: T) {
        assert!(
            self.values.len() < self.values.capacity(),
            "upload plan exceeds admitted capacity"
        );
        self.values.push(value);
    }
}

impl<T: Copy> StagingVec<T> {
    pub fn from_slice(values: &[T], host: &Arc<Budget>) -> anyhow::Result<Self> {
        let mut result = Self::new(values.len(), host)?;
        result.values.extend_from_slice(values);
        Ok(result)
    }

    pub fn copy_from_slice(&mut self, values: &[T]) {
        self.values.copy_from_slice(values);
    }
}

impl<T> AsRef<[T]> for StagingVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.values
    }
}

impl<T> Extend<T> for StagingVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        for value in values {
            self.push(value);
        }
    }
}

impl<T> std::ops::Deref for StagingVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}

impl<T> std::ops::DerefMut for StagingVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_admits_old_new_overlap_and_reuse_preserves_storage() {
        let old = StagingVec::<u64>::capacity_bytes(4).unwrap();
        let new = StagingVec::<u64>::capacity_bytes(8).unwrap();
        let host = Budget::new(old + new);
        let mut plan = StagingVec::new(4, &host).unwrap();
        plan.extend(0_u64..4);
        let blocker = host.reserve(1).unwrap();
        assert!(plan.ensure_capacity(8, &host).is_err());
        assert_eq!(&*plan, &[0, 1, 2, 3]);
        assert_eq!(plan.allocated_bytes(), old);
        drop(blocker);
        plan.ensure_capacity(8, &host).unwrap();
        assert_eq!(host.used(), new);
        assert_eq!(&*plan, &[0, 1, 2, 3]);
        let pointer = plan.as_ptr();
        plan.clear();
        plan.ensure_capacity(8, &host).unwrap();
        plan.extend(4_u64..12);
        assert_eq!(plan.as_ptr(), pointer);
        drop(plan);
        assert_eq!(host.used(), 0);
    }

    #[test]
    fn staging_plan_charges_actual_heap_and_refuses_before_growth() {
        let process = staging_process();
        let baseline = process.used();
        let bytes = StagingVec::<u64>::capacity_bytes(17).unwrap();
        let host = Budget::new(bytes);
        let scope = crate::cpu_alloc::Scope::new("upload-plan-capacity");
        let mut plan = scope.with(|| StagingVec::<u64>::new(17, &host)).unwrap();
        for value in 0..17 {
            plan.push(value);
        }
        let usage = scope.usage();
        assert_eq!(usage.payload_bytes + usage.tracking_bytes, bytes);
        assert_eq!(host.used(), bytes);
        assert_eq!(process.used(), baseline + bytes);
        assert!(StagingVec::<u64>::new(1, &host).is_err());
        assert_eq!(&*plan, &(0..17).collect::<Vec<_>>());
        drop(plan);
        assert_eq!(
            scope.usage().payload_bytes + scope.usage().tracking_bytes,
            0
        );
        assert_eq!(host.used(), 0);
        assert_eq!(process.used(), baseline);
        assert!(StagingVec::<u64>::new(usize::MAX, &host).is_err());
    }
}
