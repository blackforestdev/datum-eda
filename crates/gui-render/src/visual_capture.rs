use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, anyhow};
use datum_gui_protocol::ReviewWorkspaceState;
use image::RgbaImage;

use crate::{CameraState, PreparedScene, Renderer, RetainedScene};

const DEFAULT_MSAA_SAMPLES: u32 = 4;
const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const COPY_BYTES_PER_PIXEL: u32 = 4;
const WGPU_COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;
const READBACK_TIMEOUT: Duration = Duration::from_secs(30);

/// Layer-A visual capture path for deterministic renderer goldens.
///
/// This owns the GPU setup and readback only. Scene construction and drawing
/// still go through the production renderer so visual tests exercise the same
/// code that interactive Datum uses.
pub struct OffscreenRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    width: u32,
    height: u32,
}

impl OffscreenRenderer {
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        if width == 0 || height == 0 {
            return Err(anyhow!("offscreen capture dimensions must be non-zero"));
        }

        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))
        .or_else(|_| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
        })
        .context("request offscreen wgpu adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("datum-gui-visual-capture-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .context("request offscreen wgpu device")?;
        let renderer = Renderer::new(&device, &queue, OUTPUT_FORMAT, DEFAULT_MSAA_SAMPLES);

        Ok(Self {
            device,
            queue,
            renderer,
            width,
            height,
        })
    }

    pub fn render_workspace(
        &mut self,
        state: &ReviewWorkspaceState,
        camera: Option<CameraState>,
    ) -> anyhow::Result<RgbaImage> {
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("datum-gui-visual-capture-target"),
            size: self.extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OUTPUT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let retained = RetainedScene::from_workspace(state, self.width, self.height);
        let camera = camera.unwrap_or_else(|| CameraState::fit_to_bounds(&state.scene.bounds));
        let prepared =
            PreparedScene::from_workspace(state, self.width, self.height, camera, &retained);

        self.renderer.render(
            &self.device,
            &self.queue,
            &target_view,
            &prepared,
            &retained,
            self.width,
            self.height,
        )?;

        self.read_texture(&target)
    }

    fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    fn read_texture(&self, texture: &wgpu::Texture) -> anyhow::Result<RgbaImage> {
        let unpadded_bytes_per_row = self.width * COPY_BYTES_PER_PIXEL;
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, WGPU_COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = padded_bytes_per_row as u64 * self.height as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("datum-gui-visual-capture-readback-buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("datum-gui-visual-capture-readback-encoder"),
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
                    rows_per_image: Some(self.height),
                },
            },
            self.extent(),
        );
        self.queue.submit([encoder.finish()]);

        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(READBACK_TIMEOUT),
            })
            .context("poll device for visual readback")?;
        receiver
            .recv_timeout(READBACK_TIMEOUT)
            .context("wait for visual readback mapping")?
            .context("map visual readback buffer")?;

        let mapped = buffer_slice.get_mapped_range();
        let mut pixels = vec![0_u8; (self.width * self.height * COPY_BYTES_PER_PIXEL) as usize];
        for row in 0..self.height as usize {
            let source_start = row * padded_bytes_per_row as usize;
            let source_end = source_start + unpadded_bytes_per_row as usize;
            let dest_start = row * unpadded_bytes_per_row as usize;
            let dest_end = dest_start + unpadded_bytes_per_row as usize;
            pixels[dest_start..dest_end].copy_from_slice(&mapped[source_start..source_end]);
        }
        drop(mapped);
        output_buffer.unmap();

        RgbaImage::from_raw(self.width, self.height, pixels)
            .context("construct visual capture image from readback pixels")
    }
}

fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_to_preserves_aligned_rows() {
        assert_eq!(align_to(1024, 256), 1024);
        assert_eq!(align_to(1025, 256), 1280);
    }

    #[test]
    #[ignore = "requires a working local wgpu adapter; intended for explicit visual harness smoke runs"]
    fn offscreen_capture_renders_fixture_workspace() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let mut renderer = OffscreenRenderer::new(640, 360).expect("create offscreen renderer");
        let image = renderer
            .render_workspace(&state, None)
            .expect("render fixture workspace");

        assert_eq!(image.width(), 640);
        assert_eq!(image.height(), 360);
        assert!(image.pixels().any(|pixel| pixel.0 != [0, 0, 0, 0]));
    }
}
