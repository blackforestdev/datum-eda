//! Bounded last-content ownership for immediate screen-space vertex uploads.
use super::vertex_allocation::VertexAllocation;

// One snapshot per semantic stream, not frame history. Larger streams still
// render normally but bypass CPU retention. Nine production slots bound total
// retained snapshots to 9 * 256 KiB per renderer.
const MAX_SNAPSHOT_BYTES: usize = 256 * 1024;

#[derive(Default)]
pub(crate) struct ScreenBuffer {
    snapshot: Box<[u8]>,
    allocation: VertexAllocation,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl ScreenBuffer {
    pub(crate) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.allocation.buffer()
    }

    /// Exact bytes are the key: equal size, allocator reuse, NaN payloads and
    /// signed zero cannot produce a false hit. Placement is already baked into
    /// these screen vertices; painter order stays in the caller's draw schedule.
    pub(crate) fn sync<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        vertices: &[T],
    ) -> usize {
        #[cfg(test)]
        {
            self.last_upload_bytes = 0;
        }
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.is_empty() {
            *self = Self::default();
            return 0;
        }
        if self.allocation.buffer().is_some() && self.snapshot.as_ref() == bytes {
            return 0;
        }
        let uploaded;
        if self.allocation.replace_if_needed(device, label, bytes) {
            uploaded = bytes.len();
        } else if let Some(buffer) = self.allocation.buffer() {
            uploaded = write_dirty_ranges(
                queue,
                buffer,
                &self.snapshot,
                bytes,
                std::mem::size_of::<T>(),
            );
        } else {
            unreachable!("missing buffers are replaced");
        }
        if bytes.len() <= MAX_SNAPSHOT_BYTES {
            if self.snapshot.len() == bytes.len() {
                self.snapshot.copy_from_slice(bytes);
            } else {
                self.snapshot = bytes.into();
            }
        } else {
            self.snapshot = Box::default();
        }
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        uploaded
    }
}

// A vertex is the update unit. Join adjacent changed vertices, preserving
// unchanged vertex gaps. This avoids one queue write per coordinate/color word
// during camera changes. Trim clean edge words within each resulting span;
// internal clean words remain coalesced, so this is not byte-minimal transfer.
// Production Vertex has a four-byte-aligned stride.
pub(crate) fn write_dirty_ranges(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    old: &[u8],
    new: &[u8],
    stride: usize,
) -> usize {
    assert!(stride > 0);
    assert_eq!(stride % wgpu::COPY_BUFFER_ALIGNMENT as usize, 0);
    let mut start = None;
    let mut uploaded = 0;
    for offset in (0..new.len()).step_by(stride) {
        let end = offset + stride;
        if old.get(offset..end) != Some(&new[offset..end]) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            uploaded += write_trimmed_span(queue, buffer, old, new, begin, offset);
        }
    }
    if let Some(begin) = start {
        uploaded += write_trimmed_span(queue, buffer, old, new, begin, new.len());
    }
    uploaded
}

// Preserve the existing number of queue writes. Word-by-word queue writes and
// thousands of individual buffer copies both regress moving-stream CPU cost.
fn write_trimmed_span(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    old: &[u8],
    new: &[u8],
    mut begin: usize,
    mut end: usize,
) -> usize {
    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    while begin < end && old.get(begin..begin + word) == Some(&new[begin..begin + word]) {
        begin += word;
    }
    while begin < end && old.get(end - word..end) == Some(&new[end - word..end]) {
        end -= word;
    }
    if begin < end {
        queue.write_buffer(buffer, begin as u64, &new[begin..end]);
    }
    end - begin
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    fn read(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: &wgpu::Buffer,
        len: u64,
    ) -> Vec<u8> {
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screen-upload-readback"),
            size: len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(source, 0, &target, 0, len);
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        target
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let bytes = target.slice(..).get_mapped_range().to_vec();
        target.unmap();
        bytes
    }

    #[test]
    #[ignore = "requires local GPU; run serially with visual feature"]
    fn screen_upload_reuses_exact_content_and_bounds_retention() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let mut owner = ScreenBuffer::default();
        let mut values = [1_u32, 2, 3, 4];
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 16);
        let first = owner.buffer().unwrap().clone();
        let relocated = values.to_vec();
        assert_ne!(relocated.as_ptr(), values.as_ptr());
        assert_eq!(owner.sync(&device, &queue, "proof", &relocated), 0);
        values[1] = 17; // same address AND length, different content
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 4);
        assert_eq!(owner.buffer(), Some(&first));
        assert_eq!(
            read(&device, &queue, &first, 16),
            bytemuck::cast_slice::<u32, u8>(&values)
        );
        values[0] = 18;
        values[3] = 19;
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 8);
        assert_eq!(
            read(&device, &queue, &first, 16),
            bytemuck::cast_slice::<u32, u8>(&values)
        );
        // Multi-word vertices: changing one byte transfers only its aligned
        // edge words. Internal gaps stay coalesced to bound queue-call overhead.
        let mut vertices = [[0_u32; 5]; 3];
        owner.sync(&device, &queue, "vertex-fields", &vertices);
        vertices[0][1] = 0x0100;
        vertices[0][4] = 0x0200;
        vertices[2][4] = 0x0300;
        assert_eq!(owner.sync(&device, &queue, "vertex-fields", &vertices), 20);
        assert_eq!(
            read(&device, &queue, owner.buffer().unwrap(), 60),
            bytemuck::cast_slice::<[u32; 5], u8>(&vertices)
        );
        assert_eq!(owner.sync(&device, &queue, "vertex-fields", &vertices), 0);
        // A retained allocation can grow its live prefix without reallocating.
        owner.sync(&device, &queue, "short-prefix", &vertices[..2]);
        vertices[0][0] = 7;
        assert_eq!(owner.sync(&device, &queue, "grow-prefix", &vertices), 24);
        assert_eq!(
            read(&device, &queue, owner.buffer().unwrap(), 60),
            bytemuck::cast_slice::<[u32; 5], u8>(&vertices)
        );
        let at_cap = vec![42_u32; MAX_SNAPSHOT_BYTES / 4];
        assert_eq!(
            owner.sync(&device, &queue, "proof", &at_cap),
            MAX_SNAPSHOT_BYTES
        );
        assert_eq!(owner.snapshot.len(), MAX_SNAPSHOT_BYTES);
        assert_eq!(owner.sync(&device, &queue, "proof", &at_cap), 0);
        let oversized = vec![43_u32; MAX_SNAPSHOT_BYTES / 4 + 1];
        for _ in 0..2 {
            assert_eq!(
                owner.sync(&device, &queue, "proof", &oversized),
                MAX_SNAPSHOT_BYTES + 4
            );
            assert!(
                owner.snapshot.is_empty(),
                "overlarge content bypasses retention"
            );
        }
        assert_eq!(
            read(
                &device,
                &queue,
                owner.buffer().unwrap(),
                oversized.len() as u64 * 4
            ),
            bytemuck::cast_slice::<u32, u8>(&oversized)
        );
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 16);
        assert_eq!(
            owner.buffer().unwrap().size(),
            16,
            "obsolete peak capacity released"
        );
        assert_ne!(owner.buffer(), Some(&first));
        // Clearing releases both owners; identical content must upload again.
        assert_eq!(owner.sync::<u32>(&device, &queue, "proof", &[]), 0);
        assert!(owner.buffer().is_none());
        assert!(owner.snapshot.is_empty());
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 16);
        owner = ScreenBuffer::default(); // renderer/device replacement
        assert_eq!(owner.sync(&device, &queue, "proof", &values), 16);
    }
}
