use super::Renderer;

impl Renderer {
    pub(crate) fn ensure_msaa(&mut self, device: &wgpu::Device, width: u32, height: u32) -> &wgpu::TextureView {
        if self.msaa_size != (width, height) || self.msaa_view.is_none() {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("datum-gui-render-msaa"),
                size: wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 },
                mip_level_count: 1, sample_count: self.msaa_samples,
                dimension: wgpu::TextureDimension::D2, format: self.msaa_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
            });
            self.msaa_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            self.msaa_size = (width, height);
        }
        self.msaa_view.as_ref().expect("MSAA view initialized")
    }
}
