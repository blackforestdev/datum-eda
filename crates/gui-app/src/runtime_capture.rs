//! Native capture target and readback ownership.
use super::*;

impl Runtime {
    #[cfg(feature = "visual")]
    pub(super) fn write_visual_screenshot(&mut self, path: &Path) -> Result<()> {
        let image = self.capture_visual_screenshot()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create screenshot directory {}", parent.display()))?;
        }
        image
            .save(path)
            .with_context(|| format!("write visual shell screenshot {}", path.display()))
    }

    #[cfg(not(feature = "visual"))]
    pub(super) fn write_visual_screenshot(&mut self, _path: &Path) -> Result<()> {
        anyhow::bail!("datum-gui visual screenshots require the datum-gui-app visual feature")
    }

    #[cfg(feature = "visual")]
    fn capture_visual_screenshot(&mut self) -> Result<image::RgbaImage> {
        let target = datum_gui_render::capture_resource::CaptureTarget::new(
            &self.device,
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            self.config.format,
        )?;
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        self.renderer.render_session_mut().retry_content();
        if self.renderer.render_session().prepared().is_none() {
            self.scene_dirty = false;
            self.ensure_retained_scene();
            self.renderer.render_session_mut().check_content_budget()?;
            let prepared = self.build_terminal_prepared_scene()?;
            self.renderer
                .render_session_mut()
                .install_prepared(prepared);
        }
        self.renderer.render_session_mut().check_content_budget()?;
        self.ensure_schematic_retained_scene();
        self.renderer.render_session_mut().check_content_budget()?;
        let retained = self
            .renderer
            .render_session()
            .board()
            .cloned()
            .context("retained scene should exist before visual screenshot")?;
        let schematic_retained = self.renderer.render_session().schematic().cloned();
        let owner = self.renderer.resource_owner_id();
        let receipt = self
            .renderer
            .render_session_mut()
            .begin_frame(owner, owner, 0, false);
        let rendered = self.renderer.with_prepared_scene(|renderer, prepared| {
            renderer.render(
                &self.device,
                &self.queue,
                &target_view,
                prepared,
                &retained,
                schematic_retained.as_ref(),
                self.config.width,
                self.config.height,
            )
        });
        target.hold_submission(&self.queue);
        rendered?;
        let result = self.read_visual_texture(&target);
        self.renderer
            .render_session_mut()
            .complete_frame(receipt, owner, owner, 0, result.is_ok());
        result
    }

    #[cfg(feature = "visual")]
    fn read_visual_texture(
        &self,
        texture: &datum_gui_render::capture_resource::CaptureTarget,
    ) -> Result<image::RgbaImage> {
        let width = self.config.width;
        let height = self.config.height;
        let unpadded_bytes_per_row = width * COPY_BYTES_PER_PIXEL;
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, WGPU_COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_bytes_per_row as u64 * height as u64;
        let output_buffer =
            datum_gui_render::capture_resource::CaptureReadback::new(&self.device, buffer_size)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("datum-gui-layer-b-visual-readback-encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        texture.hold_submission(&self.queue);
        output_buffer.hold_submission(&self.queue);

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("poll device for visual shell readback")?;
        receiver
            .recv()
            .context("wait for visual shell readback mapping")?
            .context("map visual shell readback buffer")?;

        let mapped = buffer_slice.get_mapped_range();
        let mut pixels = vec![0_u8; (width * height * COPY_BYTES_PER_PIXEL) as usize];
        for row in 0..height as usize {
            let source_start = row * padded_bytes_per_row as usize;
            let source_end = source_start + unpadded_bytes_per_row as usize;
            let dest_start = row * unpadded_bytes_per_row as usize;
            let dest_end = dest_start + unpadded_bytes_per_row as usize;
            pixels[dest_start..dest_end].copy_from_slice(&mapped[source_start..source_end]);
        }
        drop(mapped);
        output_buffer.unmap();

        convert_texture_pixels_to_rgba(&mut pixels, self.config.format)?;
        image::RgbaImage::from_raw(width, height, pixels)
            .context("construct visual shell image from readback pixels")
    }
}
