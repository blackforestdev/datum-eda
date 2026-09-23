//! Explicit mapped staging for texture copies in the caller's frame submission.
use super::budget::Budget;
use super::lifetime::{Kind, Owner, Tracked};
use std::sync::Arc;

pub(super) struct TextureUpload<'a> {
    pub texture: &'a wgpu::Texture,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub stride: u32,
    pub pixels: &'a [u8],
}

pub(crate) struct Batch {
    // Drop encoded resource references before releasing their accounting.
    command: Option<wgpu::CommandBuffer>,
    buffer: Tracked<wgpu::Buffer>,
}

impl Batch {
    pub fn command(&mut self) -> wgpu::CommandBuffer {
        self.command.take().expect("upload command submitted once")
    }

    pub fn hold(self, queue: &wgpu::Queue) {
        super::hold_until_done(queue, vec![self.buffer.submission_ref()]);
    }
}

pub(super) fn textures(
    device: &wgpu::Device,
    owner: &Owner,
    generation: u64,
    host_budget: &Arc<Budget>,
    uploads: &[TextureUpload<'_>],
) -> anyhow::Result<Option<Batch>> {
    if uploads.is_empty() {
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
    // Actual padded API capacity, not just source payload, is charged before allocation.
    let host = host_budget.reserve(capacity)?;
    let process = super::budget::staging_process().reserve(capacity)?;
    let gpu = super::budget::gpu_process().reserve(capacity)?;
    let buffer = owner.track_with_permits(
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-texture-upload-staging"),
            size: capacity,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        }),
        capacity,
        generation,
        Kind::Staging,
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
    Ok(Some(Batch {
        command: Some(encoder.finish()),
        buffer,
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
#[path = "upload_tests.rs"]
mod tests;
