//! One immutable CPU source and its reusable GPU allocation per renderer/device.
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub(crate) struct RetainedBuffer<T> {
    source: Option<Arc<[T]>>,
    buffer: Option<wgpu::Buffer>,
    capacity_bytes: usize,
}

impl<T> Default for RetainedBuffer<T> {
    fn default() -> Self {
        Self {
            source: None,
            buffer: None,
            capacity_bytes: 0,
        }
    }
}

impl<T: bytemuck::Pod> RetainedBuffer<T> {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    /// Return actual source bytes submitted for upload, zero for a warm source.
    /// Retaining the Arc makes allocator-address reuse impossible until the old
    /// identity is replaced. Buffer capacity is never used as content identity.
    pub(crate) fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        source: &Arc<[T]>,
    ) -> usize {
        if source.is_empty() {
            self.clear();
            return 0;
        }
        if self.buffer.is_some()
            && self
                .source
                .as_ref()
                .is_some_and(|old| Arc::ptr_eq(old, source))
        {
            return 0;
        }
        let bytes = bytemuck::cast_slice(source.as_ref());
        if self.buffer.is_none() || self.capacity_bytes < bytes.len() {
            let mut usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
            if cfg!(test) {
                usage |= wgpu::BufferUsages::COPY_SRC;
            }
            self.buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytes,
                    usage,
                }),
            );
            self.capacity_bytes = self.buffer.as_ref().unwrap().size() as usize;
        } else if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytes);
        }
        self.source = Some(source.clone());
        bytes.len()
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
    #[ignore = "requires local GPU; run serially with visual feature"]
    fn retained_gpu_buffer_reuses_uploads_but_replaces_equal_sized_content() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut retained = RetainedBuffer::default();
        let source: Arc<[u32]> = Arc::from([1, 2, 3, 4]);
        let weak = Arc::downgrade(&source);
        assert_eq!(retained.sync(&device, &queue, "proof", &source), 16);
        let buffer = retained.buffer().unwrap().clone();
        assert_eq!(retained.sync(&device, &queue, "proof", &source.clone()), 0);
        drop(source);
        assert!(
            weak.upgrade().is_some(),
            "upload owner prevents address reuse"
        );
        let replacement: Arc<[u32]> = Arc::from([5, 6, 7, 8]);
        assert_eq!(retained.sync(&device, &queue, "proof", &replacement), 16);
        assert!(
            weak.upgrade().is_none(),
            "old CPU source released on replacement"
        );
        assert_eq!(
            retained.buffer(),
            Some(&buffer),
            "same-size GPU capacity reused"
        );
        assert_eq!(
            read(&device, &queue, &buffer, 16),
            bytemuck::cast_slice::<u32, u8>(replacement.as_ref())
        );
        assert_eq!(retained.sync(&device, &queue, "proof", &replacement), 0);
        let weak = Arc::downgrade(&replacement);
        retained.clear();
        assert!(retained.buffer().is_none());
        assert_eq!(retained.capacity_bytes, 0);
        drop(replacement);
        assert!(weak.upgrade().is_none());
        let empty: Arc<[u32]> = Arc::from([]);
        assert_eq!(retained.sync(&device, &queue, "proof", &empty), 0);
        assert!(retained.buffer().is_none());
        let replacement: Arc<[u32]> = Arc::from([9, 10, 11, 12]);
        assert_eq!(retained.sync(&device, &queue, "proof", &replacement), 16);
        assert_ne!(
            retained.buffer(),
            Some(&buffer),
            "reset cannot reuse stale GPU identity"
        );
    }
}
