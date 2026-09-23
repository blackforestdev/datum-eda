//! Terminal texture identity, pending upload and submission ownership.
use super::{PreparedTerminalGraphic, TerminalGraphicTextureKey};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};
#[path = "terminal_rgba_upload.rs"]
mod upload;

pub(super) struct CachedTerminalGraphicTexture {
    pub(super) key: TerminalGraphicTextureKey,
    pub(super) bind_group: wgpu::BindGroup,
    // Bindings drop first; the record survives through the final GPU handle.
    texture: Tracked<wgpu::Texture>,
    // Preserve pixel allocation identity while its address is in the cache key.
    source: datum_terminal_core::RenderGraphic,
    pub(super) pending: bool,
}

impl CachedTerminalGraphicTexture {
    pub(super) fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        graphic: &PreparedTerminalGraphic,
        key: TerminalGraphicTextureKey,
    ) -> anyhow::Result<CachedTerminalGraphicTexture> {
        let bytes = u64::from(key.width)
            .checked_mul(u64::from(key.height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| anyhow::anyhow!("terminal texture capacity overflow"))?;
        anyhow::ensure!(
            bytes == std::mem::size_of_val(graphic.graphic.placement().pixels()) as u64,
            "terminal texture extent does not match RGBA payload"
        );
        let terminal_permit = crate::text_gpu::budget::terminal_process().reserve(bytes)?;
        let permit = crate::text_gpu::budget::gpu_process().reserve(bytes)?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("datum-terminal-graphic-texture"),
            size: wgpu::Extent3d {
                width: key.width,
                height: key.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("datum-terminal-graphic-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-terminal-graphic-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Ok(CachedTerminalGraphicTexture {
            key,
            bind_group,
            texture: Owner::new().track_with_permits(
                texture,
                bytes,
                1,
                Kind::TerminalTexture,
                vec![terminal_permit, permit],
            ),
            source: graphic.graphic.clone(),
            pending: true,
        })
    }

    pub(super) fn submission_ref(&self) -> SubmissionRef {
        self.texture.submission_ref()
    }

    pub(super) fn flush_upload(&mut self, queue: &wgpu::Queue) {
        if !self.pending {
            return;
        }
        write_rgba(
            queue,
            &self.texture,
            self.source.placement().pixels(),
            self.key.width,
            self.key.height,
        );
        self.pending = false;
    }
}

fn write_rgba(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    pixels: &[datum_terminal_core::Rgba8],
    width: u32,
    height: u32,
) {
    upload::chunks(pixels, width, height, |x, y, w, h, bytes| {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    });
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires local GPU; borrowed terminal upload row boundaries"]
    fn multi_chunk_odd_width_upload_matches_decoded_pixels() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let (width, height) = (257_u32, 513_u32);
        let pixels: Vec<_> = (0..width * height)
            .map(|i| datum_terminal_core::Rgba8 {
                red: i as u8,
                green: (i / 7) as u8,
                blue: (i / 257) as u8,
                alpha: 255 - i as u8,
            })
            .collect();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terminal-chunk-proof"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        write_rgba(&queue, &texture, &pixels, width, height);
        let stride = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terminal-chunk-readback"),
            size: u64::from(stride) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let data = readback.slice(..).get_mapped_range();
        let expected: Vec<_> = pixels
            .iter()
            .flat_map(|p| [p.red, p.green, p.blue, p.alpha])
            .collect();
        for y in 0..height as usize {
            assert_eq!(
                &data[y * stride as usize..y * stride as usize + width as usize * 4],
                &expected[y * width as usize * 4..(y + 1) * width as usize * 4]
            );
        }
    }
}
