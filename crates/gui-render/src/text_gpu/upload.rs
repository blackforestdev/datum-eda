//! Explicit mapped staging for texture and buffer copies in the caller's frame submission.
use super::budget::Budget;
use super::lifetime::{Kind, Owner, Tracked};
use std::sync::Arc;

pub(crate) struct TextureUpload<'a> {
    pub texture: &'a wgpu::Texture,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub stride: u32,
    pub pixels: &'a [u8],
}

pub(crate) struct BufferUpload<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub offset: u64,
    pub bytes: &'a [u8],
}

pub(crate) struct Batch {
    // Drop encoded resource references before releasing their accounting.
    command: Option<wgpu::CommandBuffer>,
    buffers: Vec<Tracked<wgpu::Buffer>>,
}

impl Batch {
    pub fn command(&mut self) -> wgpu::CommandBuffer {
        self.command.take().expect("upload command submitted once")
    }

    pub fn hold(self, queue: &wgpu::Queue) {
        super::hold_until_done(
            queue,
            self.buffers.iter().map(Tracked::submission_ref).collect(),
        );
    }
}

pub(crate) fn batch(
    device: &wgpu::Device,
    owner: &Owner,
    generation: u64,
    host_budget: &Arc<Budget>,
    uploads: &[TextureUpload<'_>],
    buffers: &[BufferUpload<'_>],
) -> anyhow::Result<Option<Batch>> {
    batch_with_scatter(
        device,
        owner,
        generation,
        host_budget,
        uploads,
        buffers,
        None,
    )
}

pub(crate) fn required_bytes(uploads: &[TextureUpload<'_>], buffers: &[BufferUpload<'_>]) -> u64 {
    uploads.iter().map(padded_bytes).sum::<u64>()
        + super::sparse_upload::groups(buffers)
            .iter()
            .map(|group| {
                if group.sparse {
                    group.packet_bytes() * 2
                } else {
                    group.uploads.iter().map(|u| u.bytes.len() as u64).sum()
                }
            })
            .sum::<u64>()
}

pub(crate) fn batch_with_scatter(
    device: &wgpu::Device,
    owner: &Owner,
    generation: u64,
    host_budget: &Arc<Budget>,
    uploads: &[TextureUpload<'_>],
    buffers: &[BufferUpload<'_>],
    scatter: Option<&super::sparse_upload::Scatter>,
) -> anyhow::Result<Option<Batch>> {
    if uploads.is_empty() && buffers.is_empty() {
        return Ok(None);
    }
    let mut capacity = 0_u64;
    for upload in uploads {
        anyhow::ensure!(
            upload.size[1] > 0 && upload.stride > 0,
            "empty texture upload"
        );
        anyhow::ensure!(
            upload.stride <= u32::MAX - (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1),
            "texture upload row pitch overflow"
        );
        anyhow::ensure!(
            upload.pixels.len() as u64 == u64::from(upload.stride) * u64::from(upload.size[1]),
            "texture upload payload mismatch"
        );
        capacity = capacity
            .checked_add(padded_bytes(upload))
            .ok_or_else(|| anyhow::anyhow!("texture staging size overflow"))?;
    }
    for upload in buffers {
        let bytes = upload.bytes.len() as u64;
        anyhow::ensure!(
            bytes > 0
                && bytes.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
                && upload.offset.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT)
                && upload
                    .offset
                    .checked_add(bytes)
                    .is_some_and(|end| end <= upload.buffer.size()),
            "invalid buffer upload range"
        );
        capacity = capacity
            .checked_add(bytes)
            .ok_or_else(|| anyhow::anyhow!("buffer staging size overflow"))?;
    }
    let groups = super::sparse_upload::groups(buffers);
    let sparse_bytes: u64 = groups
        .iter()
        .filter(|g| g.sparse && scatter.is_some())
        .map(|g| g.packet_bytes())
        .sum();
    // Sparse packets carry (destination word, value), and both their mapped
    // upload buffer and storage buffer remain charged until completion.
    capacity += sparse_bytes * 3 / 2;
    for group in groups.iter().filter(|g| g.sparse && scatter.is_some()) {
        anyhow::ensure!(
            group.packet_bytes() <= u64::from(device.limits().max_storage_buffer_binding_size)
                && group.uploads[0].buffer.size()
                    <= u64::from(device.limits().max_storage_buffer_binding_size),
            "fragmented upload exceeds storage binding limit"
        );
    }
    let host = host_budget.reserve(capacity)?;
    let process = super::budget::staging_process().reserve(capacity)?;
    let gpu = super::budget::gpu_process().reserve(capacity)?;
    let direct_bytes = capacity - sparse_bytes * 2;
    let mut allocations = Vec::new();
    let buffer = owner.track_with_permits(
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-texture-upload-staging"),
            size: direct_bytes + sparse_bytes,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        }),
        direct_bytes + sparse_bytes,
        generation,
        Kind::Staging,
        // Aggregate reservations are held on the mapped owner. Batch keeps all
        // storage owners alive until the same completion boundary.
        vec![host, process, gpu],
    );
    {
        let mut mapped = buffer.slice(..).get_mapped_range_mut();
        let mut offset = 0;
        for upload in uploads {
            let pitch = padded_stride(upload) as usize;
            for row in upload.pixels.chunks_exact(upload.stride as usize) {
                mapped[offset..offset + row.len()].copy_from_slice(row);
                offset += pitch;
            }
        }
        for group in &groups {
            if group.sparse && scatter.is_some() {
                continue;
            }
            for upload in group.uploads {
                mapped[offset..offset + upload.bytes.len()].copy_from_slice(upload.bytes);
                offset += upload.bytes.len();
            }
        }
        for group in groups.iter().filter(|g| g.sparse && scatter.is_some()) {
            let len = group.packet_bytes() as usize;
            group.fill(&mut mapped[offset..offset + len]);
            offset += len;
        }
    }
    buffer.unmap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("datum-texture-upload-copies"),
    });
    let mut offset = 0;
    for upload in uploads {
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset,
                    bytes_per_row: Some(padded_stride(upload)),
                    rows_per_image: Some(upload.size[1]),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: upload.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: upload.origin[0],
                    y: upload.origin[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: upload.size[0],
                height: upload.size[1],
                depth_or_array_layers: 1,
            },
        );
        offset += padded_bytes(upload);
    }
    for group in &groups {
        if group.sparse && scatter.is_some() {
            continue;
        }
        for upload in group.uploads {
            encoder.copy_buffer_to_buffer(
                &buffer,
                offset,
                upload.buffer,
                upload.offset,
                upload.bytes.len() as u64,
            );
            offset += upload.bytes.len() as u64;
        }
    }
    if let Some(scatter) = scatter {
        for group in groups.iter().filter(|g| g.sparse) {
            let bytes = group.packet_bytes();
            let packet = owner.track_with_permits(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-scatter-packet"),
                    size: bytes,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }),
                bytes,
                generation,
                Kind::Staging,
                Vec::new(),
            );
            encoder.copy_buffer_to_buffer(&buffer, offset, &packet, 0, bytes);
            scatter.encode(device, &mut encoder, &packet, group.uploads[0].buffer);
            offset += bytes;
            allocations.push(packet);
        }
    }
    allocations.push(buffer);
    Ok(Some(Batch {
        command: Some(encoder.finish()),
        buffers: allocations,
    }))
}

fn padded_stride(upload: &TextureUpload<'_>) -> u32 {
    upload
        .stride
        .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
}

fn padded_bytes(upload: &TextureUpload<'_>) -> u64 {
    u64::from(padded_stride(upload)) * u64::from(upload.size[1])
}

#[cfg(all(test, feature = "visual"))]
pub(crate) fn submit_buffers_for_test(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffers: &[BufferUpload<'_>],
) {
    if let Some(mut batch) = batch_with_scatter(
        device,
        &Owner::new(),
        1,
        &Budget::new(16 * 1024 * 1024),
        &[],
        buffers,
        Some(&super::sparse_upload::Scatter::default()),
    )
    .unwrap()
    {
        queue.submit([batch.command()]);
        batch.hold(queue);
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "upload_tests.rs"]
mod tests;
