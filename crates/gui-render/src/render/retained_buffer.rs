//! One immutable CPU source and its reusable GPU allocation per renderer/device.
use super::shared_geometry::SharedGeometry;
use super::vertex_allocation::VertexAllocation;

pub(crate) struct RetainedBuffer<T> {
    source: Option<SharedGeometry<T>>,
    allocation: VertexAllocation,
    pending: bool,
    document_budget: Option<std::sync::Arc<crate::text_gpu::budget::Budget>>,
}

impl<T> Default for RetainedBuffer<T> {
    fn default() -> Self {
        Self {
            source: None,
            allocation: VertexAllocation::default(),
            pending: false,
            document_budget: None,
        }
    }
}

impl<T: bytemuck::Pod> RetainedBuffer<T> {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.allocation.buffer()
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn submission_ref(&self) -> Option<crate::text_gpu::lifetime::SubmissionRef> {
        self.allocation.submission_ref()
    }

    pub(crate) fn cancel_uploads(&mut self) {
        if self.pending {
            self.source = None;
            self.pending = false;
        }
    }

    pub(crate) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        if self.pending {
            queue.write_buffer(
                self.buffer().unwrap(),
                0,
                bytemuck::cast_slice(self.source.as_ref().unwrap().as_ref()),
            );
            self.pending = false;
        }
    }

    /// Return source bytes planned for upload, zero for a warm source.
    /// Retaining the shared owner makes allocator-address reuse impossible until the old
    /// identity is replaced. Buffer capacity is never used as content identity.
    pub(crate) fn sync(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        label: &str,
        source: &SharedGeometry<T>,
    ) -> anyhow::Result<usize> {
        if source.is_empty() {
            self.clear();
            return Ok(0);
        }
        if self.allocation.buffer().is_some()
            && self.source.as_ref().is_some_and(|old| old.ptr_eq(source))
        {
            return Ok(0);
        }
        let bytes = bytemuck::cast_slice(source.as_ref());
        let budget = source.document_budget();
        let same_document = match (&self.document_budget, budget) {
            (Some(current), Some(next)) => std::sync::Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        };
        if same_document {
            self.allocation.replace_if_needed(device, label, bytes)?;
        } else {
            // A reused API buffer cannot stay charged to the previous document.
            // Admit the replacement before retiring the old working allocation.
            let mut replacement = budget.map_or_else(VertexAllocation::default, |budget| {
                VertexAllocation::with_budget(budget.clone())
            });
            replacement.replace_if_needed(device, label, bytes)?;
            self.allocation = replacement;
            self.document_budget = budget.cloned();
        }
        self.pending = true;
        self.source = Some(source.clone());
        Ok(bytes.len())
    }
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    fn read(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: &wgpu::Buffer,
        size: u64,
    ) -> Vec<u8> {
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("retained-buffer-proof"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(source, 0, &target, 0, size);
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        target
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let bytes = target.slice(..).get_mapped_range().to_vec();
        target.unmap();
        bytes
    }

    #[test]
    #[ignore = "requires local GPU; document admission across streams and retirement"]
    fn document_buffers_share_admission_and_switch_owners_atomically() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let a = SharedGeometry::for_document(vec![1_u32; 4], "document-admission-a");
        let revision = SharedGeometry::for_document(vec![2_u32; 4], "document-admission-a");
        let other = SharedGeometry::for_document(vec![3_u32; 4], "document-admission-b");
        let budget = a.document_budget().unwrap().clone();
        let other_budget = other.document_budget().unwrap().clone();
        assert!(std::sync::Arc::ptr_eq(
            &budget,
            revision.document_budget().unwrap()
        ));
        assert!(!std::sync::Arc::ptr_eq(&budget, &other_budget));
        let mut first = RetainedBuffer::default();
        let mut second = RetainedBuffer::default();
        first.sync(&device, &queue, "first", &a).unwrap();
        first.flush_uploads(&queue);
        second.sync(&device, &queue, "second", &revision).unwrap();
        assert_eq!(budget.used(), 32);
        let held = first.submission_ref().unwrap();
        let original = first.buffer().unwrap().clone();
        let filler = budget.reserve(64 * 1024 * 1024 - budget.used()).unwrap();
        let bigger = SharedGeometry::for_document(vec![4_u32; 8], "document-admission-a");
        assert!(first.sync(&device, &queue, "refused", &bigger).is_err());
        assert_eq!(first.buffer(), Some(&original));
        assert_eq!(
            read(&device, &queue, first.buffer().unwrap(), 16),
            bytemuck::cast_slice::<u32, u8>(&a)
        );
        // Equal-sized geometry belonging to another document needs a new charged owner.
        first.sync(&device, &queue, "other", &other).unwrap();
        assert_ne!(first.buffer(), Some(&original));
        assert_eq!(other_budget.used(), 16);
        assert_eq!(budget.used(), 64 * 1024 * 1024);
        drop(original);
        drop(held);
        assert_eq!(budget.used(), 64 * 1024 * 1024 - 16);
        drop(filler);
        first.sync(&device, &queue, "retry", &bigger).unwrap();
        assert_eq!(budget.used(), 48);
        assert_eq!(other_budget.used(), 0);
        drop(first);
        drop(second);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    #[ignore = "requires local GPU; run serially with visual feature"]
    fn retained_gpu_buffer_reuses_uploads_but_replaces_equal_sized_content() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut retained = RetainedBuffer::default();
        let source: SharedGeometry<u32> = vec![1, 2, 3, 4].into();
        let weak = source.downgrade();
        assert_eq!(
            retained.sync(&device, &queue, "proof", &source).unwrap(),
            16
        );
        let buffer = retained.buffer().unwrap().clone();
        assert_eq!(read(&device, &queue, &buffer, 16), vec![0; 16]);
        retained.cancel_uploads();
        assert_eq!(
            retained.sync(&device, &queue, "retry", &source).unwrap(),
            16
        );
        assert_eq!(
            retained
                .sync(&device, &queue, "proof", &source.clone())
                .unwrap(),
            0
        );
        drop(source);
        assert!(
            weak.upgrade().is_some(),
            "upload owner prevents address reuse"
        );
        let replacement: SharedGeometry<u32> = vec![5, 6, 7, 8].into();
        assert_eq!(
            retained
                .sync(&device, &queue, "proof", &replacement)
                .unwrap(),
            16
        );
        assert!(
            weak.upgrade().is_none(),
            "old CPU source released on replacement"
        );
        assert_eq!(
            retained.buffer(),
            Some(&buffer),
            "same-size GPU capacity reused"
        );
        retained.flush_uploads(&queue);
        assert_eq!(
            read(&device, &queue, &buffer, 16),
            bytemuck::cast_slice::<u32, u8>(replacement.as_ref())
        );
        assert_eq!(
            retained
                .sync(&device, &queue, "proof", &replacement)
                .unwrap(),
            0
        );
        let large: SharedGeometry<u32> = vec![42; 64].into();
        assert_eq!(
            retained.sync(&device, &queue, "proof", &large).unwrap(),
            256
        );
        let peak = retained.buffer().unwrap().clone();
        let quarter: SharedGeometry<u32> = vec![43; 16].into();
        assert_eq!(
            retained.sync(&device, &queue, "proof", &quarter).unwrap(),
            64
        );
        assert_eq!(
            retained.buffer(),
            Some(&peak),
            "reuse at four-times boundary"
        );
        let small: SharedGeometry<u32> = vec![44; 15].into();
        assert_eq!(retained.sync(&device, &queue, "proof", &small).unwrap(), 60);
        assert_eq!(
            retained.buffer().unwrap().size(),
            60,
            "release historical peak below quarter occupancy"
        );
        assert_ne!(retained.buffer(), Some(&peak));
        assert_eq!(retained.sync(&device, &queue, "proof", &small).unwrap(), 0);
        retained.flush_uploads(&queue);
        assert_eq!(
            read(&device, &queue, retained.buffer().unwrap(), 60),
            bytemuck::cast_slice::<u32, u8>(&small)
        );
        let weak = small.downgrade();
        retained.clear();
        drop(small);
        assert!(retained.buffer().is_none());
        assert!(retained.buffer().is_none());
        drop(replacement);
        assert!(weak.upgrade().is_none());
        let empty: SharedGeometry<u32> = Vec::new().into();
        assert_eq!(retained.sync(&device, &queue, "proof", &empty).unwrap(), 0);
        assert!(retained.buffer().is_none());
        let replacement: SharedGeometry<u32> = vec![9, 10, 11, 12].into();
        assert_eq!(
            retained
                .sync(&device, &queue, "proof", &replacement)
                .unwrap(),
            16
        );
        assert_ne!(
            retained.buffer(),
            Some(&buffer),
            "reset cannot reuse stale GPU identity"
        );
    }
}
