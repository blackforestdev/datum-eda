//! Test-only independent full painter samples. Scratch images are not resource evidence.
use super::*;
use crate::renderer_state::damage::{clip::Clip, regional::Plan};

impl Renderer {
    pub(crate) fn regional_full_samples(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Vec<u32> {
        let alias = self.msaa_format.remove_srgb_suffix();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("r4-full-painter-sample-oracle"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 8,
            dimension: wgpu::TextureDimension::D2,
            format: self.msaa_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[alias],
        });
        let view = texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("r4-full-painter-every-sample"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: APP_BG[0] as f64,
                            g: APP_BG[1] as f64,
                            b: APP_BG[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            let prepared = self.render_session.prepared().unwrap();
            self.draw_frame_prefix(&mut pass, prepared, width, height, &mut None)
                .unwrap();
            self.draw_frame_suffix(
                &mut pass,
                prepared,
                width,
                height,
                &self.damage_masks.unrestricted.group,
                Clip::full(width, height),
            )
            .unwrap();
        }
        queue.submit([encoder.finish()]);
        let unorm = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(alias),
            ..Default::default()
        });
        crate::renderer_state::damage::restore::sample_tests::read_samples(
            device, queue, &unorm, width, height,
        )
    }

    pub(crate) fn regional_repaint_for_proof(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        extent: [u32; 2],
        plan: &Plan,
    ) {
        let images = self.surface_attachments.prefix_images(device).unwrap();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("r4-explicit-plan-output"),
            size: wgpu::Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.msaa_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let target = crate::render_input::FrameTarget::full_texture(&target).unwrap();
        let working = self
            .ensure_msaa(device, extent[0], extent[1])
            .unwrap()
            .clone();
        let mut encoder = device.create_command_encoder(&Default::default());
        self.encode_composition(
            device,
            &mut encoder,
            &target,
            &working,
            self.render_session.prepared().unwrap(),
            extent[0],
            extent[1],
            Some(&images),
            Some(plan),
            &mut None,
            crate::gpu_surface::prefix_negative_control::Fault::None,
        )
        .unwrap();
        queue.submit([encoder.finish()]);
        self.hold_frame_submission(queue, Some(&images));
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }

    pub(crate) fn assert_regional_samples(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        extent: [u32; 2],
        plan: &Plan,
        label: &str,
    ) {
        assert!(
            !plan.is_empty(),
            "sample comparison must cover actual tiles"
        );
        let images = self.surface_attachments.prefix_images(device).unwrap();
        let (_, atlas) = images.restoration_views().unwrap();
        let actual = crate::renderer_state::damage::restore::sample_tests::read_samples(
            device, queue, atlas, 512, 512,
        );
        let reference = self.regional_full_samples(device, queue, extent[0], extent[1]);
        let mut checked = 0;
        let mut covered = vec![false; 512 * 512];
        for tile in plan.tiles() {
            for y in 0..tile.extent[1] {
                for x in 0..tile.extent[0] {
                    covered[((tile.atlas[1] + y) * 512 + tile.atlas[0] + x) as usize] = true;
                    for sample in 0..8 {
                        let a =
                            (((tile.atlas[1] + y) * 512 + tile.atlas[0] + x) * 8 + sample) as usize;
                        let r = (((tile.source[1] + y) * extent[0] + tile.source[0] + x) * 8
                            + sample) as usize;
                        assert_eq!(
                            actual[a], reference[r],
                            "{label}: source={:?} atlas={:?} local=({x},{y}) sample={sample}",
                            tile.source, tile.atlas
                        );
                        checked += 1;
                    }
                }
            }
        }
        for (pixel, is_covered) in covered.into_iter().enumerate() {
            if !is_covered {
                assert_eq!(
                    &actual[pixel * 8..pixel * 8 + 8],
                    &[0; 8],
                    "{label}: untouched/padded atlas pixel {pixel}"
                );
            }
        }
        eprintln!(
            "r4 exact samples: {label}, {checked} samples, {} tiles",
            plan.tiles().count()
        );
    }
}
