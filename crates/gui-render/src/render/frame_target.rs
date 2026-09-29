//! Explicit full-texture presentation capability; arbitrary views never grant copy access.
#[derive(Clone)]
pub struct FrameTarget {
    view: wgpu::TextureView,
    copy_destination: Option<wgpu::Texture>,
}

impl FrameTarget {
    /// The caller supplies the actual acquired/capture texture, not a view from
    /// which the renderer guesses a subresource. Unsupported copy usage retains
    /// ordinary render-attachment fallback.
    pub fn full_texture(texture: &wgpu::Texture) -> anyhow::Result<Self> {
        anyhow::ensure!(
            texture.dimension() == wgpu::TextureDimension::D2
                && texture.depth_or_array_layers() == 1
                && texture.sample_count() == 1
                && texture
                    .usage()
                    .contains(wgpu::TextureUsages::RENDER_ATTACHMENT),
            "frame target must be a single-layer, single-sample 2D render attachment"
        );
        Ok(Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                ..Default::default()
            }),
            copy_destination: texture
                .usage()
                .contains(wgpu::TextureUsages::COPY_DST)
                .then(|| texture.clone()),
        })
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Copies must match the prepared target exactly; never scale, reinterpret
    /// its color format, or write outside a caller-selected view.
    pub fn copy_destination(
        &self,
        extent: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> Option<&wgpu::Texture> {
        self.copy_destination.as_ref().filter(|texture| {
            [texture.width(), texture.height()] == extent && texture.format() == format
        })
    }
}

impl From<wgpu::TextureView> for FrameTarget {
    fn from(view: wgpu::TextureView) -> Self {
        Self {
            view,
            copy_destination: None,
        }
    }
}

impl From<&wgpu::TextureView> for FrameTarget {
    fn from(view: &wgpu::TextureView) -> Self {
        Self::from(view.clone())
    }
}

impl From<&FrameTarget> for FrameTarget {
    fn from(target: &FrameTarget) -> Self {
        target.clone()
    }
}
