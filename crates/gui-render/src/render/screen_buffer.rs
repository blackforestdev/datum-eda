//! Bounded last-content ownership for immediate screen-space vertex uploads.
use super::vertex_allocation::VertexAllocation;

// Submitted and prepared snapshots are separate so cancellation preserves the
// comparison baseline. Each is bounded by this stream's admitted GPU capacity;
// all screen streams therefore retain at most twice their shared GPU allowance
// in snapshot payload, plus allocation headers. Shrink/uncached retirement trims
// obsolete storage instead of retaining each stream's historical peak. Their
// actual capacities and headers also share host/process staging admission.

pub(crate) struct ScreenBuffer {
    snapshot: crate::text_gpu::staging_vec::StagingVec<u8>,
    allocation: VertexAllocation,
    pending: crate::text_gpu::staging_vec::StagingVec<std::ops::Range<usize>>,
    staging_budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    prepared: crate::text_gpu::staging_vec::StagingVec<u8>,
    has_prepared: bool,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl Default for ScreenBuffer {
    fn default() -> Self {
        Self {
            snapshot: Default::default(),
            allocation: VertexAllocation::default()
                .with_generation_limit(crate::text_gpu::budget::Budget::new(2)),
            pending: Default::default(),
            staging_budget: crate::text_gpu::budget::Budget::new(16 * 1024 * 1024),
            prepared: Default::default(),
            has_prepared: false,
            #[cfg(test)]
            last_upload_bytes: 0,
        }
    }
}

impl ScreenBuffer {
    pub(crate) fn with_budgets(
        budgets: Vec<std::sync::Arc<crate::text_gpu::budget::Budget>>,
    ) -> Self {
        Self {
            allocation: VertexAllocation::with_budgets(budgets)
                .with_generation_limit(crate::text_gpu::budget::Budget::new(2)),
            ..Self::default()
        }
    }

    pub(crate) fn with_budget(budget: std::sync::Arc<crate::text_gpu::budget::Budget>) -> Self {
        Self::with_budgets(vec![budget])
    }

    pub(crate) fn with_staging_budget(
        mut self,
        budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> Self {
        assert_eq!(
            self.pending.allocated_bytes(),
            0,
            "assign staging owner before use"
        );
        self.staging_budget = budget;
        self
    }

    pub(crate) fn replacement(&self) -> Self {
        Self {
            allocation: self.allocation.replacement(),
            staging_budget: self.staging_budget.clone(),
            ..Self::default()
        }
    }

    pub(crate) fn with_generation_limit(
        mut self,
        budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> Self {
        self.allocation = self.allocation.with_generation_limit(budget);
        self
    }

    pub(crate) fn with_retention_budget(
        mut self,
        budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> Self {
        self.allocation = self.allocation.with_retention_budget(budget);
        self
    }

    pub(crate) fn retire_uncached_gpu(&mut self) {
        self.allocation.retire_uncached();
        if self.allocation.buffer().is_none() {
            self.pending = Default::default();
            self.snapshot = Default::default();
            self.prepared = Default::default();
        }
    }

    pub(crate) fn snapshot_bytes(&self) -> u64 {
        self.snapshot.allocated_bytes() + self.prepared.allocated_bytes()
    }

    pub(crate) fn pending_metadata_bytes(&self) -> u64 {
        self.pending.allocated_bytes()
    }

    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.allocation.buffer()
    }

    pub(crate) fn submission_ref(&self) -> Option<crate::text_gpu::lifetime::SubmissionRef> {
        self.allocation.submission_ref()
    }

    pub(crate) fn cancel_uploads(&mut self) {
        self.pending.clear();
        self.has_prepared = false;
        if self.prepared.len() as u64 > self.allocation.buffer().map_or(0, wgpu::Buffer::size) {
            self.prepared = Default::default();
        }
    }

    pub(crate) fn append_uploads<'a>(
        &'a self,
        out: &mut impl Extend<crate::text_gpu::upload::BufferUpload<'a>>,
    ) {
        let bytes = &self.prepared;
        for range in self.pending.iter() {
            let end = range.end.min(bytes.len());
            if range.start < end {
                out.extend(std::iter::once(crate::text_gpu::upload::BufferUpload {
                    buffer: self.allocation.buffer().unwrap(),
                    offset: range.start as u64,
                    bytes: &bytes[range.start..end],
                }));
            }
        }
    }

    pub(crate) fn finish_uploads(&mut self) {
        if self.has_prepared {
            std::mem::swap(&mut self.snapshot, &mut self.prepared);
        }
        self.cancel_uploads();
    }

    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn flush_uploads(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut uploads = Vec::new();
        self.append_uploads(&mut uploads);
        crate::text_gpu::upload::submit_buffers_for_test(device, queue, &uploads);
        self.finish_uploads();
    }

    /// Exact bytes are the key: equal size, allocator reuse, NaN payloads and
    /// signed zero cannot produce a false hit. Placement is already baked into
    /// these screen vertices; painter order stays in the caller's draw schedule.
    pub(crate) fn sync<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        label: &str,
        vertices: &[T],
    ) -> anyhow::Result<usize> {
        #[cfg(test)]
        {
            self.last_upload_bytes = 0;
        }
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.is_empty() {
            self.allocation.clear();
            self.snapshot = Default::default();
            self.pending = Default::default();
            self.prepared = Default::default();
            self.has_prepared = false;
            return Ok(0);
        }
        if self.allocation.buffer().is_some() && self.snapshot.as_ref() == bytes {
            self.cancel_uploads();
            return Ok(0);
        }
        if self.has_prepared && self.prepared.as_ref() == bytes {
            return Ok(0);
        }
        if self.allocation.replace_if_needed(device, label, bytes)? {
            // A fresh allocation has no submitted content, even if the prior
            // allocation's snapshot happens to match a later preparation.
            self.snapshot = Default::default();
            self.cancel_uploads();
        }
        let mut ranges = 0;
        dirty_ranges(&self.snapshot, bytes, std::mem::size_of::<T>(), |_, _| {
            ranges += 1
        });
        self.pending.ensure_capacity(ranges, &self.staging_budget)?;
        // Keep the previous plan and its bytes intact until replacement storage
        // is admitted. Both old/new capacities count during allocation.
        let prepared = if self.prepared.len() != bytes.len() {
            Some(crate::text_gpu::staging_vec::StagingVec::from_slice(
                bytes,
                &self.staging_budget,
            )?)
        } else {
            None
        };
        self.pending.clear();
        let uploaded = dirty_ranges(
            &self.snapshot,
            bytes,
            std::mem::size_of::<T>(),
            |offset, bytes| {
                self.pending
                    .push(offset as usize..offset as usize + bytes.len())
            },
        );
        if let Some(prepared) = prepared {
            self.prepared = prepared;
        } else {
            self.prepared.copy_from_slice(bytes);
        }
        self.has_prepared = true;
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        Ok(uploaded)
    }
}

// Transfer exactly the changed COPY_BUFFER_ALIGNMENT words. Adjacent dirty
// words share a copy; clean words split it even inside a vertex or instance.
// The caller batches these ranges into one explicit staging allocation.
pub(crate) fn dirty_ranges<'a>(
    old: &[u8],
    new: &'a [u8],
    stride: usize,
    mut write: impl FnMut(u64, &'a [u8]),
) -> usize {
    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    assert!(stride > 0);
    assert_eq!(stride % word, 0);
    assert_eq!(new.len() % stride, 0);
    let mut start = None;
    let mut uploaded = 0;
    for offset in (0..new.len()).step_by(word) {
        let end = offset + word;
        if old.get(offset..end) != Some(&new[offset..end]) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            write(begin as u64, &new[begin..offset]);
            uploaded += offset - begin;
        }
    }
    if let Some(begin) = start {
        write(begin as u64, &new[begin..]);
        uploaded += new.len() - begin;
    }
    uploaded
}

#[cfg(all(test, feature = "visual"))]
#[path = "screen_buffer/tests.rs"]
mod tests;
