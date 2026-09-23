//! Construction leases bridge text allocation and retained-cache publication.
use crate::text_buffer_cache::budget::{Construction, Owner};

pub(crate) struct Admission<'a> {
    pub owner: Option<&'a Owner>,
    pub host: &'a std::sync::Arc<crate::text_gpu::budget::Budget>,
}

pub(super) struct ProducedLine {
    pub layout: glyphon::LayoutLine,
    pub construction: Option<Construction>,
}

pub(super) struct Storage<T> {
    // Free allocation before its reservation; publication explicitly transfers it.
    values: Vec<T>,
    lease: Option<Construction>,
}
impl<T> Default for Storage<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            lease: None,
        }
    }
}
impl<T> Storage<T> {
    pub fn with_capacity(capacity: usize, owner: Option<&Owner>) -> anyhow::Result<Self> {
        let bytes = crate::text_gpu::staging_vec::StagingVec::<T>::capacity_bytes(capacity)?;
        let lease = owner.map(|o| o.reserve(bytes as usize)).transpose()?;
        let mut values = Vec::new();
        values.try_reserve_exact(capacity)?;
        anyhow::ensure!(
            crate::cpu_alloc::heap::capacity_bytes::<T>(values.capacity()) == bytes as usize,
            "text allocation differs from admitted capacity"
        );
        Ok(Self { values, lease })
    }
    pub fn push(&mut self, value: T, owner: Option<&Owner>) -> anyhow::Result<()> {
        if self.values.len() == self.values.capacity() {
            let capacity = self
                .values
                .len()
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("text allocation count overflow"))?;
            let grown = capacity.max(self.values.capacity().saturating_mul(2));
            let mut replacement = Self::with_capacity(grown, owner).or_else(|error| {
                if grown == capacity {
                    Err(error)
                } else {
                    Self::with_capacity(capacity, owner)
                }
            })?;
            replacement.values.append(&mut self.values);
            *self = replacement;
        }
        self.values.push(value);
        Ok(())
    }
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }
    pub fn clear(&mut self) {
        self.values.clear();
    }
    pub fn published(&mut self) {
        self.lease = None;
    }
    pub fn into_parts(self) -> (Vec<T>, Option<Construction>) {
        (self.values, self.lease)
    }
}
impl<T> std::ops::Deref for Storage<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T> std::ops::DerefMut for Storage<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}
