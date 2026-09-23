//! Fixed-capacity CPU upload plans charged before allocation and through use.
use super::budget::{Budget, Permit, staging_process};
use std::sync::Arc;

pub(crate) struct StagingVec<T> {
    // Release elements and storage before their admission permits.
    values: Vec<T>,
    _permits: [Permit; 2],
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
        let permits = [host.reserve(bytes)?, staging_process().reserve(bytes)?];
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

    pub fn push(&mut self, value: T) {
        assert!(
            self.values.len() < self.values.capacity(),
            "upload plan exceeds admitted capacity"
        );
        self.values.push(value);
    }
}

impl<T> std::ops::Deref for StagingVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
