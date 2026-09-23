//! GPU allocation lifetime shared by retained-world and screen vertex streams.
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};

#[derive(Default)]
pub(crate) struct VertexAllocation {
    buffer: Option<Tracked<wgpu::Buffer>>,
    owner: Option<Owner>,
    generation: u64,
    generation_budget: Option<std::sync::Arc<crate::text_gpu::budget::Budget>>,
    retention_budget: Option<std::sync::Arc<crate::text_gpu::budget::Budget>>,
    uncached: bool,
    budgets: Vec<std::sync::Arc<crate::text_gpu::budget::Budget>>,
}

impl VertexAllocation {
    pub(crate) fn with_budget(budget: std::sync::Arc<crate::text_gpu::budget::Budget>) -> Self {
        Self::with_budgets(vec![budget])
    }

    pub(crate) fn with_budgets(
        budgets: Vec<std::sync::Arc<crate::text_gpu::budget::Budget>>,
    ) -> Self {
        Self {
            budgets,
            ..Self::default()
        }
    }

    pub(crate) fn with_generation_limit(
        mut self,
        budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> Self {
        self.generation_budget = Some(budget);
        self
    }

    pub(crate) fn with_retention_budget(
        mut self,
        budget: std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> Self {
        self.retention_budget = Some(budget);
        self
    }

    /// Call only after establishing the frame submission references.
    pub(crate) fn retire_uncached(&mut self) {
        if self.uncached {
            self.clear();
        }
    }

    pub(crate) fn clear(&mut self) {
        self.uncached = false;
        self.buffer = None;
    }

    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_deref()
    }

    pub(crate) fn submission_ref(&self) -> Option<SubmissionRef> {
        self.buffer.as_ref().map(Tracked::submission_ref)
    }

    /// A replacement requires a complete upload at the submission boundary. Keep nearby capacity to avoid churn,
    /// but release historical peaks above four times the latest live payload.
    /// Device/renderer teardown drops this owner and its handle.
    pub(crate) fn replace_if_needed(
        &mut self,
        device: &wgpu::Device,
        label: &str,
        bytes: &[u8],
    ) -> anyhow::Result<bool> {
        debug_assert!(
            !bytes.is_empty(),
            "empty streams drop their allocation owner"
        );
        let live = bytes.len() as u64;
        if self
            .buffer
            .as_ref()
            .is_some_and(|buffer| buffer.size() >= live && buffer.size() <= live.saturating_mul(4))
        {
            return Ok(false);
        }
        let mut usage =
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE;
        if cfg!(test) {
            usage |= wgpu::BufferUsages::COPY_SRC;
        }
        let capacity = live.next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);
        let mut permits = Vec::with_capacity(self.budgets.len() + 3);
        if let Some(budget) = &self.generation_budget {
            permits.push(budget.reserve(1).map_err(|_| {
                anyhow::anyhow!("retained world stream already has two live GPU allocations")
            })?);
        }
        for budget in &self.budgets {
            permits.push(budget.reserve(capacity)?);
        }
        permits.push(crate::text_gpu::budget::gpu_process().reserve(capacity)?);
        // Retention refusal bypasses the cache, never the hard frame budgets.
        let mut uncached = false;
        if let Some(budget) = &self.retention_budget {
            match budget.reserve(capacity) {
                Ok(permit) => permits.push(permit),
                Err(_) => uncached = true,
            }
        }
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: capacity,
            usage,
            mapped_at_creation: false,
        });
        self.uncached = uncached;
        self.generation += 1;
        let bytes = buffer.size();
        self.buffer = Some(
            self.owner
                .get_or_insert_with(Owner::new)
                .track_with_permits(buffer, bytes, self.generation, Kind::Vertex, permits),
        );
        Ok(true)
    }
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires local GPU and serial execution: exercises process admission"]
    fn refusal_preserves_allocation_and_retirement_keeps_reservation() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let budget = crate::text_gpu::budget::gpu_process();
        let baseline = budget.used();
        let mut allocation = VertexAllocation::default();
        allocation
            .replace_if_needed(&device, "admission", &[0; 16])
            .unwrap();
        let held = allocation.submission_ref().unwrap();
        let original = allocation.owner.as_ref().unwrap().records();
        let filler = budget.reserve(512 * 1024 * 1024 - budget.used()).unwrap();
        assert!(
            allocation
                .replace_if_needed(&device, "refused", &[0; 64])
                .is_err()
        );
        assert_eq!(allocation.owner.as_ref().unwrap().records(), original);
        assert_eq!(allocation.buffer().unwrap().size(), 16);
        assert_eq!(budget.used(), 512 * 1024 * 1024);
        drop(filler);
        allocation
            .replace_if_needed(&device, "retry", &[0; 64])
            .unwrap();
        assert_eq!(budget.used(), baseline + 80);
        let records = allocation.owner.as_ref().unwrap().records();
        assert_eq!(records.len(), 2);
        assert!(records.iter().any(|r| r.id == original[0].id && r.retiring));
        drop(held);
        assert_eq!(budget.used(), baseline + 64);
        drop(allocation);
        assert_eq!(budget.used(), baseline);
    }
}
