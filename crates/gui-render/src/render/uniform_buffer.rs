//! Fixed-size uniform ownership with exact aligned changed-range uploads.
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};
use wgpu::util::DeviceExt;

pub(crate) struct UniformBuffer<T> {
    buffer: Tracked<wgpu::Buffer>,
    value: Option<T>,
    pending: Option<T>,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub(crate) fn new(device: &wgpu::Device, label: &str, value: T) -> anyhow::Result<Self> {
        let permit = reserve::<T>()?;
        Ok(Self {
            buffer: tracked(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytemuck::bytes_of(&value),
                    usage: uniform_usage(),
                }),
                permit,
            ),
            value: Some(value),
            pending: None,
            #[cfg(test)]
            last_upload_bytes: std::mem::size_of::<T>(),
        })
    }

    pub(crate) fn empty(device: &wgpu::Device, label: &str) -> anyhow::Result<Self> {
        let permit = reserve::<T>()?;
        Ok(Self {
            buffer: tracked(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: std::mem::size_of::<T>() as u64,
                    usage: uniform_usage(),
                    mapped_at_creation: false,
                }),
                permit,
            ),
            value: None,
            pending: None,
            #[cfg(test)]
            last_upload_bytes: 0,
        })
    }

    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn submission_ref(&self) -> SubmissionRef {
        self.buffer.submission_ref()
    }

    pub(crate) fn cancel_uploads(&mut self) {
        self.pending = None;
    }

    pub(crate) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        if let Some(value) = self.pending.take() {
            write_changed_ranges(
                self.value.as_ref().map(bytemuck::bytes_of),
                bytemuck::bytes_of(&value),
                |offset, data| queue.write_buffer(&self.buffer, offset as u64, data),
            );
            self.value = Some(value);
        }
    }

    /// Planned bytes; queue writes occur only in flush_uploads before submission.
    pub(crate) fn sync(&mut self, _queue: &wgpu::Queue, value: T) -> usize {
        let uploaded = write_changed_ranges(
            self.value.as_ref().map(bytemuck::bytes_of),
            bytemuck::bytes_of(&value),
            |_, _| {},
        );
        self.pending = (uploaded != 0).then_some(value);
        #[cfg(test)]
        {
            self.last_upload_bytes = uploaded;
        }
        uploaded
    }
}

fn uniform_usage() -> wgpu::BufferUsages {
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    if cfg!(test) {
        usage | wgpu::BufferUsages::COPY_SRC
    } else {
        usage
    }
}

fn reserve<T>() -> anyhow::Result<crate::text_gpu::budget::Permit> {
    crate::text_gpu::budget::gpu_process()
        .reserve((std::mem::size_of::<T>() as u64).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT))
}

fn tracked(buffer: wgpu::Buffer, permit: crate::text_gpu::budget::Permit) -> Tracked<wgpu::Buffer> {
    let bytes = buffer.size();
    Owner::new().track_with_permits(buffer, bytes, 1, Kind::Uniform, vec![permit])
}

/// Keep bindings and their allocation together; drop binding references first.
pub(crate) struct UniformBinding<T> {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) buffer: UniformBuffer<T>,
}
impl<T: bytemuck::Pod> UniformBinding<T> {
    pub(crate) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        label: &str,
        value: Option<T>,
    ) -> anyhow::Result<Self> {
        let buffer = match value {
            Some(value) => UniformBuffer::new(device, label, value)?,
            None => UniformBuffer::empty(device, label)?,
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.buffer().as_entire_binding(),
            }],
        });
        Ok(Self { bind_group, buffer })
    }
}

impl crate::Renderer {
    pub(crate) fn cancel_uniform_uploads(&mut self) {
        self.uniform_buffer.cancel_uploads();
        for binding in &mut self.surface_scene_uniforms {
            binding.buffer.cancel_uploads();
        }
    }
    pub(crate) fn flush_uniform_uploads(&mut self, queue: &wgpu::Queue) {
        self.uniform_buffer.flush_uploads(queue);
        for binding in &mut self.surface_scene_uniforms {
            binding.buffer.flush_uploads(queue);
        }
    }
    pub(crate) fn uniform_submission_refs(&self) -> impl Iterator<Item = SubmissionRef> + '_ {
        [
            self.uniform_buffer.submission_ref(),
            self.scene_bind_group.buffer.submission_ref(),
            self.schematic_scene_bind_group.buffer.submission_ref(),
        ]
        .into_iter()
        .chain(
            self.surface_scene_uniforms
                .iter()
                .map(|binding| binding.buffer.submission_ref()),
        )
    }
}

// Uniforms are small fixed records (16-byte screen and 64-byte camera), unlike
// large vertex streams. Coalesce adjacent dirty words without transferring
// internal clean gaps; queue offsets and sizes obey COPY_BUFFER_ALIGNMENT.
fn write_changed_ranges(
    old: Option<&[u8]>,
    new: &[u8],
    mut write: impl FnMut(usize, &[u8]),
) -> usize {
    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    assert_eq!(new.len() % word, 0);
    let Some(old) = old else {
        write(0, new);
        return new.len();
    };
    assert_eq!(old.len(), new.len());
    if old == new {
        return 0;
    }
    let mut uploaded = 0;
    let mut start = None;
    for offset in (0..new.len()).step_by(word) {
        if old[offset..offset + word] != new[offset..offset + word] {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            write(begin, &new[begin..offset]);
            uploaded += offset - begin;
        }
    }
    if let Some(begin) = start {
        write(begin, &new[begin..]);
        uploaded += new.len() - begin;
    }
    uploaded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "visual")]
    #[test]
    #[ignore = "requires local GPU; deferred uniform and lifetime readback"]
    fn cancelled_uniform_updates_leave_submitted_bytes_and_retirement_intact() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let read = |buffer: &wgpu::Buffer| {
            let target = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("uniform-readback"),
                size: 16,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_buffer_to_buffer(buffer, 0, &target, 0, 16);
            queue.submit([encoder.finish()]);
            let (tx, rx) = std::sync::mpsc::channel();
            target
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            rx.recv().unwrap().unwrap();
            let data = target.slice(..).get_mapped_range().to_vec();
            target.unmap();
            data
        };
        let mut owner = UniformBuffer::new(&device, "uniform", [1_u32; 4]).unwrap();
        let id = owner.buffer.id();
        assert_eq!(owner.sync(&queue, [2_u32; 4]), 16);
        assert_eq!(
            read(owner.buffer()),
            bytemuck::cast_slice::<u32, u8>(&[1; 4])
        );
        owner.cancel_uploads();
        assert_eq!(owner.sync(&queue, [1_u32; 4]), 0);
        owner.flush_uploads(&queue);
        assert_eq!(
            read(owner.buffer()),
            bytemuck::cast_slice::<u32, u8>(&[1; 4])
        );
        assert_eq!(owner.sync(&queue, [1, 2, 1, 3]), 8);
        owner.flush_uploads(&queue);
        assert_eq!(
            read(owner.buffer()),
            bytemuck::cast_slice::<u32, u8>(&[1, 2, 1, 3])
        );
        assert_eq!(owner.sync(&queue, [1, 2, 1, 3]), 0);
        let held = owner.submission_ref();
        drop(owner);
        assert!(
            crate::Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == id && r.kind == Kind::Uniform && r.bytes == 16 && r.retiring)
        );
        drop(held);
        assert!(
            !crate::Renderer::gpu_process_allocations()
                .iter()
                .any(|r| r.id == id)
        );
    }

    #[test]
    fn uniform_ranges_transfer_only_dirty_aligned_words() {
        for mask in 0_u32..256 {
            let old = [0_u8; 32];
            let mut new = old;
            for index in 0..8 {
                if mask & (1 << index) != 0 {
                    new[4 * index + index % 4] = 1;
                }
            }
            let mut result = old;
            let mut writes = 0;
            let bytes = write_changed_ranges(Some(&old), &new, |offset, data| {
                assert_eq!(offset % 4, 0);
                assert_eq!(data.len() % 4, 0);
                for word in data.chunks_exact(4) {
                    assert_ne!(word, [0; 4]);
                }
                result[offset..offset + data.len()].copy_from_slice(data);
                writes += 1;
            });
            assert_eq!(result, new);
            assert_eq!(bytes, mask.count_ones() as usize * 4);
            assert_eq!(writes, (mask & !(mask << 1)).count_ones());
        }
        let mut writes = 0;
        assert_eq!(
            write_changed_ranges(None, &[0; 64], |offset, data| {
                assert_eq!(offset, 0);
                assert_eq!(data, [0; 64]);
                writes += 1;
            }),
            64
        );
        assert_eq!(writes, 1, "new storage must be initialized in full");
    }
}
