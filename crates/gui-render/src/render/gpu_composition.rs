//! Shared cold, regional, empty and full-fallback image graphs.
use super::*;
use crate::gpu_surface::PrefixImages;
use crate::renderer_state::damage::{clip::Clip, regional::Plan};

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_composition(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &crate::render_input::FrameTarget,
        working: &wgpu::TextureView,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        images: Option<&PrefixImages>,
        plan: Option<&Plan>,
        measurement: &mut Option<gpu_measurements::FrameQueries>,
        #[cfg(all(test, feature = "visual"))]
        fault: crate::gpu_surface::prefix_negative_control::Fault,
    ) -> anyhow::Result<std::time::Duration> {
        let clear = wgpu::LoadOp::Clear(wgpu::Color {
            r: APP_BG[0] as f64,
            g: APP_BG[1] as f64,
            b: APP_BG[2] as f64,
            a: 1.0,
        });
        let mask = &self.damage_masks.unrestricted;
        mask.set_consumers(self.frame_consumers.all());
        let mut elapsed = std::time::Duration::ZERO;
        if let (Some(images), Some(plan)) = (images, plan) {
            if !plan.is_empty() {
                let (source, destination) = images.restoration_views().expect("admitted aliases");
                #[cfg(all(test, feature = "visual"))]
                let restore =
                    fault != crate::gpu_surface::prefix_negative_control::Fault::StaleWorking;
                #[cfg(not(all(test, feature = "visual")))]
                let restore = true;
                if restore {
                    self.damage_masks
                        .restoration
                        .as_ref()
                        .expect("admitted restoration")
                        .encode_regional(
                            device,
                            encoder,
                            source,
                            destination,
                            plan,
                            measurement
                                .as_mut()
                                .map(|m| m.pass("restore"))
                                .transpose()?,
                        );
                }
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-regional-suffix-resolve"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: images.atlas_view(),
                            resolve_target: Some(images.resolved_atlas_view()),
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: self.surface_attachments.regional_store(),
                            },
                        })],
                        timestamp_writes: measurement
                            .as_mut()
                            .map(|m| m.pass("suffix"))
                            .transpose()?,
                        ..Default::default()
                    });
                    for tile in plan.tiles() {
                        tile.viewport(&mut pass, width, height);
                        let clip = tile.clip();
                        #[cfg(all(test, feature = "visual"))]
                        let clip = if fault
                            == crate::gpu_surface::prefix_negative_control::Fault::MissingSuffixMask
                        {
                            Clip::full(
                                crate::renderer_state::damage::regional::ATLAS,
                                crate::renderer_state::damage::regional::ATLAS,
                            )
                        } else {
                            clip
                        };
                        elapsed += self.draw_frame_suffix(
                            &mut pass,
                            prepared,
                            width,
                            height,
                            &mask.group,
                            clip,
                        )?;
                        #[cfg(all(test, feature = "visual"))]
                        if fault
                            == crate::gpu_surface::prefix_negative_control::Fault::OverlappingSuffix
                        {
                            elapsed += self.draw_frame_suffix(
                                &mut pass,
                                prepared,
                                width,
                                height,
                                &mask.group,
                                clip,
                            )?;
                        }
                    }
                }
                images.compose_tiles(encoder, plan);
            }
        } else {
            if let Some(images) = images {
                #[cfg(all(test, feature = "visual"))]
                let draw_prefix =
                    fault != crate::gpu_surface::prefix_negative_control::Fault::StaleKey;
                #[cfg(not(all(test, feature = "visual")))]
                let draw_prefix = true;
                if draw_prefix {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-retained-prefix"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: images.prefix_view(),
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: clear,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        timestamp_writes: measurement
                            .as_mut()
                            .map(|m| m.pass("frame"))
                            .transpose()?,
                        ..Default::default()
                    });
                    self.draw_frame_prefix(
                        &mut pass,
                        prepared,
                        width,
                        height,
                        Clip::full(width, height),
                        measurement,
                    )?;
                }
                #[cfg(not(all(test, feature = "visual")))]
                images.copy(encoder);
                #[cfg(all(test, feature = "visual"))]
                fault.copy(
                    device,
                    encoder,
                    images,
                    working,
                    width,
                    height,
                    self.msaa_format,
                );
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-full-composition-resolve"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: working,
                    resolve_target: Some(images.map_or(target.view(), PrefixImages::composed_view)),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: if images.is_some() {
                            wgpu::LoadOp::Load
                        } else {
                            clear
                        },
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                timestamp_writes: measurement
                    .as_mut()
                    .map(|m| m.pass(if images.is_some() { "suffix" } else { "frame" }))
                    .transpose()?,
                ..Default::default()
            });
            if images.is_none() {
                self.draw_frame_prefix(
                    &mut pass,
                    prepared,
                    width,
                    height,
                    Clip::resampled([width, height], target.raster_extent([width, height])),
                    measurement,
                )?;
            }
            elapsed += self.draw_frame_suffix(
                &mut pass,
                prepared,
                width,
                height,
                &mask.group,
                Clip::resampled([width, height], target.raster_extent([width, height])),
            )?;
        }
        if let Some(images) = images {
            images.present(
                encoder,
                target
                    .copy_destination([width, height], self.msaa_format)
                    .expect("admitted full copy target"),
            );
        }
        Ok(elapsed)
    }
}
