//! Terminal texture identity, pending upload and submission ownership.
use super::{PreparedTerminalGraphic, TerminalGraphicTextureKey};
use crate::text_gpu::lifetime::{Kind, Owner, SubmissionRef, Tracked};

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
        let bytes = u64::from(key.width) * u64::from(key.height) * 4;
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
                u64::from(key.width) * u64::from(key.height) * 4,
                1,
                Kind::TerminalTexture,
                vec![permit],
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
        let bytes = self
            .source
            .placement()
            .pixels()
            .iter()
            .flat_map(|pixel| [pixel.red, pixel.green, pixel.blue, pixel.alpha])
            .collect::<Vec<_>>();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.key.width * 4),
                rows_per_image: Some(self.key.height),
            },
            wgpu::Extent3d {
                width: self.key.width,
                height: self.key.height,
                depth_or_array_layers: 1,
            },
        );
        self.pending = false;
    }
}
