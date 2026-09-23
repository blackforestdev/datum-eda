//! Fixed-size uniform ownership with exact aligned changed-range uploads.
use crate::text_gpu::budget::{Budget, GpuReservation};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub(crate) struct UniformBuffer<T> {
    buffer: Tracked<wgpu::Buffer>,
    pub(crate) generation_budget: Arc<Budget>,
    value: Option<T>,
    pending: Option<T>,
    #[cfg(test)]
    pub(crate) last_upload_bytes: usize,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn new(
        device: &wgpu::Device,
        label: &str,
        value: T,
        screen_budget: &Arc<Budget>,
    ) -> anyhow::Result<Self> {
        Self::new_in_generation(device, label, value, screen_budget, Budget::new(2))
    }

    pub(crate) fn new_in_generation(
        device: &wgpu::Device,
        label: &str,
        value: T,
        screen_budget: &Arc<Budget>,
        generation_budget: Arc<Budget>,
    ) -> anyhow::Result<Self> {
        let permits = reserve::<T>(screen_budget, &generation_budget)?;
        Ok(Self {
            generation_budget,
            buffer: tracked(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytemuck::bytes_of(&value),
                    usage: uniform_usage(),
                }),
                permits,
            ),
            value: Some(value),
            pending: None,
            #[cfg(test)]
            last_upload_bytes: std::mem::size_of::<T>(),
        })
    }

    pub(crate) fn empty_in_generation(
        device: &wgpu::Device,
        label: &str,
        screen_budget: &Arc<Budget>,
        generation_budget: Arc<Budget>,
    ) -> anyhow::Result<Self> {
        let permits = reserve::<T>(screen_budget, &generation_budget)?;
        Ok(Self {
            generation_budget,
            buffer: tracked(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: std::mem::size_of::<T>() as u64,
                    usage: uniform_usage(),
                    mapped_at_creation: false,
                }),
                permits,
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

    pub(crate) fn append_uploads<'a>(
        &'a self,
        out: &mut impl Extend<crate::text_gpu::upload::BufferUpload<'a>>,
    ) {
        if let Some(value) = &self.pending {
            write_changed_ranges(
                self.value.as_ref().map(bytemuck::bytes_of),
                bytemuck::bytes_of(value),
                |offset, bytes| {
                    out.extend(std::iter::once(crate::text_gpu::upload::BufferUpload {
                        buffer: &self.buffer,
                        offset: offset as u64,
                        bytes,
                    }))
                },
            );
        }
    }

    pub(crate) fn finish_uploads(&mut self) {
        if let Some(value) = self.pending.take() {
            self.value = Some(value);
        }
    }

    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn flush_uploads(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut uploads = Vec::new();
        self.append_uploads(&mut uploads);
        crate::text_gpu::upload::submit_buffers_for_test(device, queue, &uploads);
        self.finish_uploads();
    }

    /// Planned bytes; explicit staging copies occur only at successful frame submission.
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

fn reserve<T>(
    screen_budget: &Arc<Budget>,
    generations: &Arc<Budget>,
) -> anyhow::Result<GpuReservation> {
    let generation = generations.reserve(1).map_err(|_| {
        anyhow::anyhow!("uniform has two live GPU allocations; wait for retirement before recovery")
    })?;
    let bytes = (std::mem::size_of::<T>() as u64).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);
    let host = screen_budget.reserve(bytes)?;
    GpuReservation::new(bytes, vec![generation, host])
}

fn tracked(buffer: wgpu::Buffer, permits: GpuReservation) -> Tracked<wgpu::Buffer> {
    Owner::new().track_reserved(buffer, 1, Kind::Uniform, permits)
}

/// Keep bindings and their allocation together; drop binding references first.
pub(crate) struct UniformBinding<T> {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) buffer: UniformBuffer<T>,
}
impl<T: bytemuck::Pod> UniformBinding<T> {
    pub(crate) fn from_buffer(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        label: &str,
        buffer: UniformBuffer<T>,
    ) -> anyhow::Result<Self> {
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
fn write_changed_ranges<'a>(
    old: Option<&[u8]>,
    new: &'a [u8],
    mut write: impl FnMut(usize, &'a [u8]),
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
#[path = "uniform_buffer_tests.rs"]
mod tests;
