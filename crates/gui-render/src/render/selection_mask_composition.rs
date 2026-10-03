//! The existing prefix/suffix graph with one bounded guarded mask stripe.
//! Only complete encoded frames publish; source geometry and resize buffers stay retained.
use super::*;
use crate::selection_stroke::mask::BAND;
impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_mask_composition(
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
        let damage = &self.damage_masks.unrestricted;
        damage.set_consumers(self.frame_consumers.all());
        let mut elapsed = std::time::Duration::ZERO;
        if let (Some(images), Some(plan)) = (images, plan) {
            if !plan.is_empty() {
                let (source, destination) =
                    images.restoration_views().expect("admitted prefix aliases");
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
                let span = measurement.as_mut().map(|m| m.pass("suffix")).transpose()?;
                let active = |band: usize| {
                    let top = band as u32 * BAND;
                    plan.tiles()
                        .any(|tile| tile.source[1] >= top && tile.source[1] < top + BAND)
                };
                let first = (0..self.selection_mask.bands()).find(|&b| active(b));
                let last = (0..self.selection_mask.bands()).rev().find(|&b| active(b));
                for band in 0..self.selection_mask.bands() {
                    let top = band as u32 * BAND;
                    if !plan
                        .tiles()
                        .any(|tile| tile.source[1] >= top && tile.source[1] < top + BAND)
                    {
                        continue;
                    }
                    self.selection_mask.begin_band(band);
                    self.encode_selection_mask(
                        encoder,
                        prepared,
                        [width, height],
                        span.as_ref().filter(|_| first == Some(band)).map(|s| {
                            wgpu::RenderPassTimestampWrites {
                                query_set: s.query_set,
                                beginning_of_pass_write_index: s.beginning_of_pass_write_index,
                                end_of_pass_write_index: None,
                            }
                        }),
                    )?;
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-mask-regional-suffix"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: images.atlas_view(),
                            resolve_target: Some(images.resolved_atlas_view()),
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        timestamp_writes: span.as_ref().filter(|_| last == Some(band)).map(|s| {
                            wgpu::RenderPassTimestampWrites {
                                query_set: s.query_set,
                                beginning_of_pass_write_index: None,
                                end_of_pass_write_index: s.end_of_pass_write_index,
                            }
                        }),
                        ..Default::default()
                    });
                    for tile in plan
                        .tiles()
                        .filter(|tile| tile.source[1] >= top && tile.source[1] < top + BAND)
                    {
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
                            &damage.group,
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
                                &damage.group,
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
                        label: Some("datum-mask-retained-prefix"),
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
                    drop(pass);
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
            } else {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("datum-mask-full-prefix"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: working,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: clear,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    timestamp_writes: measurement.as_mut().map(|m| m.pass("frame")).transpose()?,
                    ..Default::default()
                });
                self.draw_frame_prefix(
                    &mut pass,
                    prepared,
                    width,
                    height,
                    Clip::resampled([width, height], target.raster_extent([width, height])),
                    measurement,
                )?;
            }
            let span = measurement.as_mut().map(|m| m.pass("suffix")).transpose()?;
            for band in 0..self.selection_mask.bands() {
                self.selection_mask.begin_band(band);
                self.encode_selection_mask(
                    encoder,
                    prepared,
                    [width, height],
                    span.as_ref()
                        .filter(|_| band == 0)
                        .map(|s| wgpu::RenderPassTimestampWrites {
                            query_set: s.query_set,
                            beginning_of_pass_write_index: s.beginning_of_pass_write_index,
                            end_of_pass_write_index: None,
                        }),
                )?;
                let last = band + 1 == self.selection_mask.bands();
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("datum-mask-full-suffix"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: working,
                        resolve_target: last
                            .then(|| images.map_or(target.view(), PrefixImages::composed_view)),
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: if last {
                                wgpu::StoreOp::Discard
                            } else {
                                wgpu::StoreOp::Store
                            },
                        },
                    })],
                    timestamp_writes: span.as_ref().filter(|_| last).map(|s| {
                        wgpu::RenderPassTimestampWrites {
                            query_set: s.query_set,
                            beginning_of_pass_write_index: None,
                            end_of_pass_write_index: s.end_of_pass_write_index,
                        }
                    }),
                    ..Default::default()
                });
                let top = band as u32 * BAND;
                let clip = Clip::resampled_band(
                    [width, height],
                    target.raster_extent([width, height]),
                    top,
                    (top + BAND).min(height),
                );
                elapsed += self.draw_frame_suffix(
                    &mut pass,
                    prepared,
                    width,
                    height,
                    &damage.group,
                    clip,
                )?;
            }
        }
        if let Some(images) = images {
            images.present(
                encoder,
                target
                    .copy_destination([width, height], self.msaa_format)
                    .expect("admitted copy target"),
            );
        }
        Ok(elapsed)
    }
}
