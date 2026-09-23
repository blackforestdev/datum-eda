//! Explicit mapped staging for texture and buffer copies in the caller's frame submission.
use super::budget::Budget;
use super::lifetime::{Kind, Owner, Tracked};
use super::staging_vec::StagingVec;
use std::sync::Arc;

pub(crate) struct TextureUpload<'a> {
    pub target: Option<super::lifetime::UploadTarget<'a>>,
    pub texture: &'a wgpu::Texture,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub stride: u32,
    pub pixels: &'a [u8],
}

pub(crate) struct BufferUpload<'a> {
    pub target: Option<super::lifetime::UploadTarget<'a>>,
    pub buffer: &'a wgpu::Buffer,
    pub offset: u64,
    pub bytes: &'a [u8],
}

/// Allocation-free first pass over the same producers that fill the upload plan.
#[derive(Default)]
pub(crate) struct UploadCount {
    pub entries: usize,
    pub bytes: u64,
}
impl<'a> Extend<BufferUpload<'a>> for UploadCount {
    fn extend<I: IntoIterator<Item = BufferUpload<'a>>>(&mut self, uploads: I) {
        for upload in uploads {
            self.entries += 1;
            self.bytes += upload.bytes.len() as u64;
        }
    }
}

impl<'a> Extend<TextureUpload<'a>> for UploadCount {
    fn extend<I: IntoIterator<Item = TextureUpload<'a>>>(&mut self, uploads: I) {
        for upload in uploads {
            self.entries += 1;
            self.bytes += upload.pixels.len() as u64;
        }
    }
}

pub(crate) struct Batch {
    // Drop encoded resource references before releasing their accounting.
    command: Option<wgpu::CommandBuffer>,
    owner: Owner,
    totals: super::upload_totals::UploadTotals,
    buffers: StagingVec<Tracked<wgpu::Buffer>>,
    destinations: StagingVec<super::lifetime::SubmittedUpload>,
}

impl Batch {
    pub fn command(&mut self) -> wgpu::CommandBuffer {
        self.command.take().expect("upload command submitted once")
    }

    /// Called immediately after queue.submit at every production upload site.
    pub fn hold(mut self, queue: &wgpu::Queue) {
        assert!(
            self.command.is_none(),
            "upload must be submitted before its hold"
        );
        self.owner.record_upload(self.totals);
        for destination in self.destinations.drain_all() {
            destination.commit();
        }
        // Move the already admitted owners into completion; no second reference
        // vector is allocated after submission.
        self.buffers.iter().for_each(Tracked::mark_retiring);
        queue.on_submitted_work_done(move || {
            drop(self.buffers);
            drop(self.destinations);
        });
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

/// CPU owners retained by a nonempty batch, separately from GPU copy capacity.
pub(crate) fn retention_metadata_bytes(
    buffers: &[BufferUpload<'_>],
    scatter: bool,
) -> anyhow::Result<u64> {
    Ok(
        StagingVec::<Tracked<wgpu::Buffer>>::capacity_bytes(allocation_count(buffers, scatter))?
            + destination_metadata_bytes(super::sparse_upload::groups(buffers).count())?,
    )
}

pub(crate) fn destination_metadata_bytes(count: usize) -> anyhow::Result<u64> {
    StagingVec::<super::lifetime::SubmittedUpload>::capacity_bytes(count)
}

fn allocation_count(buffers: &[BufferUpload<'_>], scatter: bool) -> usize {
    1 + super::sparse_upload::groups(buffers)
        .filter(|g| scatter && g.sparse)
        .count()
}

pub(crate) fn required_bytes(uploads: &[TextureUpload<'_>], buffers: &[BufferUpload<'_>]) -> u64 {
    uploads.iter().map(padded_bytes).sum::<u64>()
        + super::sparse_upload::groups(buffers)
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
        .clone()
        .filter(|g| g.sparse && scatter.is_some())
        .map(|g| g.packet_bytes())
        .sum();
    // Sparse packets carry (destination word, value), and both their mapped
    // upload buffer and storage buffer remain charged until completion.
    capacity += sparse_bytes * 3 / 2;
    for group in groups.clone().filter(|g| g.sparse && scatter.is_some()) {
        anyhow::ensure!(
            group.packet_bytes() <= u64::from(device.limits().max_storage_buffer_binding_size)
                && group.uploads[0].buffer.size()
                    <= u64::from(device.limits().max_storage_buffer_binding_size),
            "fragmented upload exceeds storage binding limit"
        );
    }
    let host = host_budget.reserve(capacity)?;
    let process = super::budget::staging_process().reserve(capacity)?;
    let mut reservation = super::budget::GpuReservation::new(capacity, vec![host, process])?;
    let direct_bytes = capacity - sparse_bytes * 2;
    let mut destinations = StagingVec::new(
        uploads.len() + super::sparse_upload::groups(buffers).count(),
        host_budget,
    )?;
    for upload in uploads {
        if let Some(target) = upload.target {
            destinations.push(target.receipt(upload.pixels.len() as u64, padded_bytes(upload)));
        }
    }
    for group in super::sparse_upload::groups(buffers) {
        if let Some(target) = group.uploads[0].target {
            let source = group
                .uploads
                .iter()
                .map(|upload| upload.bytes.len() as u64)
                .sum();
            let transfer = if group.sparse && scatter.is_some() {
                group.packet_bytes()
            } else {
                source
            };
            destinations.push(target.receipt(source, transfer));
        }
    }
    let mut allocations =
        StagingVec::new(allocation_count(buffers, scatter.is_some()), host_budget)?;
    let mapped_reservation = reservation.split(direct_bytes + sparse_bytes)?;
    let buffer = owner.track_reserved(
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-texture-upload-staging"),
            size: direct_bytes + sparse_bytes,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        }),
        generation,
        Kind::Staging,
        // Every allocation shares the admitted batch lifetime, independently
        // of the order in which mapped and scatter owners are retired.
        mapped_reservation,
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
        for group in groups.clone() {
            if group.sparse && scatter.is_some() {
                continue;
            }
            for upload in group.uploads {
                mapped[offset..offset + upload.bytes.len()].copy_from_slice(upload.bytes);
                offset += upload.bytes.len();
            }
        }
        for group in groups.clone().filter(|g| g.sparse && scatter.is_some()) {
            let len = group.packet_bytes() as usize;
            group.fill(&mut mapped[offset..offset + len]);
            offset += len;
        }
    }
    buffer.unmap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("datum-texture-upload-copies"),
    });
    let mut totals = super::upload_totals::UploadTotals {
        batches: 1,
        staging_capacity_bytes: capacity,
        ..Default::default()
    };
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
        totals.texture_source_bytes += upload.pixels.len() as u64;
        totals.texture_padding_bytes += padded_bytes(upload) - upload.pixels.len() as u64;
        totals.texture_copies += 1;
        offset += padded_bytes(upload);
    }
    for group in groups.clone() {
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
            totals.buffer_payload_bytes += upload.bytes.len() as u64;
            totals.buffer_copy_bytes += upload.bytes.len() as u64;
            totals.buffer_copies += 1;
            offset += upload.bytes.len() as u64;
        }
    }
    if let Some(scatter) = scatter {
        for group in groups.clone().filter(|g| g.sparse) {
            let bytes = group.packet_bytes();
            let packet_reservation = reservation.split(bytes)?;
            let packet = owner.track_reserved(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-scatter-packet"),
                    size: bytes,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }),
                generation,
                Kind::Staging,
                packet_reservation,
            );
            encoder.copy_buffer_to_buffer(&buffer, offset, &packet, 0, bytes);
            scatter.encode(device, &mut encoder, &packet, group.uploads[0].buffer);
            totals.buffer_payload_bytes += bytes / 2;
            totals.scatter_index_bytes += bytes / 2;
            totals.buffer_copy_bytes += bytes;
            totals.buffer_copies += 1;
            totals.scatter_dispatches += 1;
            offset += bytes;
            allocations.push(packet);
        }
    }
    allocations.push(buffer);
    Ok(Some(Batch {
        command: Some(encoder.finish()),
        owner: owner.clone(),
        totals,
        buffers: allocations,
        destinations,
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
