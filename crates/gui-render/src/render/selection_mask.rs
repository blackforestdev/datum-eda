//! Disposable mask resources under the existing screen/staging/attachment owners.
//! Tables index retained painter spans, never authored objects or acquisition.
use super::*;
use crate::presentation_ranges::{Presentation, Role};
use crate::text_gpu::{budget::Budget, staging_vec::StagingVec};
use std::sync::Arc;
#[path = "selection_mask_painter.rs"]
mod painter;
pub(crate) const BAND: u32 = 96;
pub(crate) const GUARD: u32 = 2;
const HEADER_ROWS: usize = 3;
#[path = "selection_mask_pipelines.rs"]
mod pipelines;
pub(crate) struct Owner {
    pub(crate) table_gpu: gpu_data::screen_buffer::ScreenBuffer,
    table: StagingVec<[u32; 4]>,
    bindings: StagingVec<Option<Binding>>,
    images: gpu_surface::SurfaceAttachments,
    colors: gpu_surface::SurfaceAttachments,
    pipelines: std::sync::OnceLock<pipelines::Pipelines>,
    staging: Arc<Budget>,
    screen_layout: wgpu::BindGroupLayout,
    bands: usize,
    active_band: std::cell::Cell<usize>,
}
pub(super) struct Binding {
    pub table: wgpu::BindGroup,
    pub images: StagingVec<wgpu::BindGroup>,
}
impl Owner {
    pub(crate) fn new(
        screen: Arc<Budget>,
        staging: Arc<Budget>,
        screen_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            table_gpu: gpu_data::screen_buffer::ScreenBuffer::with_budget(screen)
                .with_staging_budget(staging.clone()),
            table: Default::default(),
            bindings: Default::default(),
            images: gpu_surface::SurfaceAttachments::sampled(),
            colors: gpu_surface::SurfaceAttachments::sampled(),
            pipelines: Default::default(),
            staging,
            screen_layout,
            bands: 0,
            active_band: std::cell::Cell::new(0),
        }
    }
    pub(crate) fn preserve_allowances(&mut self, old: &Self) {
        self.table_gpu = old.table_gpu.replacement();
        self.images = old.images.replacement();
        self.colors = old.colors.replacement();
    }
    pub(crate) fn submission_refs(
        &self,
    ) -> impl Iterator<Item = crate::text_gpu::lifetime::SubmissionRef> {
        self.images
            .submission_refs()
            .chain(self.colors.submission_refs())
    }
    pub(crate) fn bands(&self) -> usize {
        self.bands
    }
    pub(crate) fn begin_band(&self, index: usize) {
        assert!(index < self.bands);
        self.active_band.set(index);
    }
    pub(crate) fn active(&self) -> bool {
        self.bindings.iter().any(Option::is_some)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_layout: &wgpu::BindGroupLayout,
        damage_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        samples: u32,
        prepared: &PreparedScene,
        board: &RetainedScene,
        schematic: Option<&RetainedScene>,
        extent: [u32; 2],
        _raster: [u32; 2],
    ) -> anyhow::Result<()> {
        self.bands = extent[1].div_ceil(BAND) as usize;
        self.bindings.clear();
        self.table.clear();
        self.bindings
            .ensure_capacity(prepared.surface_passes().len(), &self.staging)?;
        let mut count = 0_usize;
        for pass in prepared.surface_passes() {
            if let Some((_, _, request)) = prepared
                .selection_masks
                .iter()
                .find(|(pane, surface, _)| *pane == pass.pane_id && *surface == pass.surface)
            {
                let retained = match pass.surface {
                    SceneSurface::Board => Some(board),
                    SceneSurface::Schematic => schematic,
                };
                if let Some(retained) = retained {
                    anyhow::ensure!(
                        request.matches(&retained.presentation),
                        "selection mask source differs from retained painter"
                    );
                    count = count
                        .checked_add(
                            HEADER_ROWS
                                + 15
                                + retained.presentation.quads.len()
                                + retained.presentation.strokes.len(),
                        )
                        .ok_or_else(|| anyhow::anyhow!("mask table overflow"))?;
                }
            }
        }
        if count == 0 {
            self.table_gpu
                .sync::<[u32; 4]>(device, queue, "datum-selection-mask-table", &[])?;
            self.images.clear_sampled();
            self.colors.clear_sampled();
            return Ok(());
        }
        self.table.ensure_capacity(
            count
                .checked_add(
                    self.bands
                        .checked_mul(16)
                        .ok_or_else(|| anyhow::anyhow!("mask band metadata overflow"))?,
                )
                .ok_or_else(|| anyhow::anyhow!("mask table overflow"))?,
            &self.staging,
        )?;
        // Every storage binding begins at the device's actual required alignment.
        let alignment = device.limits().min_storage_buffer_offset_alignment as usize;
        anyhow::ensure!(
            alignment <= 256 && 256 % alignment == 0,
            "mask table storage alignment unavailable"
        );
        for pass in prepared.surface_passes() {
            let retained = match pass.surface {
                SceneSurface::Board => Some(board),
                SceneSurface::Schematic => schematic,
            };
            let request = prepared
                .selection_masks
                .iter()
                .find(|(pane, surface, _)| *pane == pass.pane_id && *surface == pass.surface)
                .map(|(_, _, request)| request);
            if let (Some(retained), Some(request)) = (retained, request) {
                while !self.table.len().is_multiple_of(16) {
                    self.table.push([0; 4]);
                }
                let begin = self.table.len();
                let presentation = &retained.presentation;
                self.table.push([
                    u32::try_from(presentation.quads.len())?,
                    u32::try_from(presentation.strokes.len())?,
                    u32::try_from(presentation.quads.len() + 3)?,
                    u32::from(request.high_contrast()) | (u32::from(request.dim_unrelated()) << 1),
                ]);
                self.table.push([
                    BOARD_INNER_FIELD[0].to_bits(),
                    BOARD_INNER_FIELD[1].to_bits(),
                    BOARD_INNER_FIELD[2].to_bits(),
                    CONTEXT_DIM_FACTOR.to_bits(),
                ]);
                self.table.push([
                    crate::dim_policy::CONTEXT_FLOOR.to_bits(),
                    crate::dim_policy::CONTEXT_MIX.to_bits(),
                    0,
                    0,
                ]);
                append_spans(&mut self.table, presentation, request, false)?;
                append_spans(&mut self.table, presentation, request, true)?;
                // Temporary offsets occupy admitted binding slots until the GPU
                // buffer is ready. No partial binding is published on refusal.
                self.bindings.push(None);
                debug_assert!(self.table.len() > begin);
            } else {
                self.bindings.push(None);
            }
        }
        while !self.table.len().is_multiple_of(16) {
            self.table.push([0; 4]);
        }
        let band_headers = self.table.len();
        for band in 0..self.bands {
            self.table.push([
                u32::try_from(band)?
                    .checked_mul(BAND)
                    .ok_or_else(|| anyhow::anyhow!("mask band origin overflow"))?,
                BAND,
                extent[1],
                0,
            ]);
            if band + 1 < self.bands {
                while !self.table.len().is_multiple_of(16) {
                    self.table.push([0; 4]);
                }
            }
        }
        self.table_gpu
            .sync(device, queue, "datum-selection-mask-table", &self.table)?;
        let pipelines = self.pipelines.get_or_init(|| {
            pipelines::Pipelines::new(
                device,
                scene_layout,
                &self.screen_layout,
                damage_layout,
                format,
                samples,
            )
        });
        let view = self.images.prepare_sampled(
            device,
            extent[0],
            BAND + 2 * GUARD,
            format.remove_srgb_suffix(),
            samples,
        )?;
        let color_view =
            self.colors
                .prepare_sampled(device, extent[0], BAND + 2 * GUARD, format, samples)?;
        let buffer = self.table_gpu.buffer().expect("admitted mask table");
        let mut begin: usize = 0;
        for (index, pass) in prepared.surface_passes().iter().enumerate() {
            let retained = match pass.surface {
                SceneSurface::Board => Some(board),
                SceneSurface::Schematic => schematic,
            };
            if let Some(retained) = retained.filter(|_| {
                prepared
                    .selection_masks
                    .iter()
                    .any(|(pane, surface, _)| *pane == pass.pane_id && *surface == pass.surface)
            }) {
                begin = begin.div_ceil(16) * 16;
                let entries = HEADER_ROWS
                    + retained.presentation.quads.len()
                    + retained.presentation.strokes.len();
                let resource = || {
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: (begin * 16) as u64,
                        size: std::num::NonZeroU64::new((entries * 16) as u64),
                    })
                };
                let table = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("datum-selection-mask-table"),
                    layout: &pipelines.table_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: resource(),
                    }],
                });
                let mut images = StagingVec::new(self.bands, &self.staging)?;
                for band in 0..self.bands {
                    let image = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("datum-selection-mask-image"),
                        layout: &pipelines.image_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: resource(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::TextureView(color_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer,
                                    offset: ((band_headers + band * 16) * 16) as u64,
                                    size: std::num::NonZeroU64::new(16),
                                }),
                            },
                        ],
                    });
                    images.push(image);
                }
                self.bindings[index] = Some(Binding { table, images });
                begin += entries;
            }
        }
        let consumers = prepared
            .consumers
            .get(crate::resource_consumers::Stream::Selection);
        self.table_gpu.set_consumers(consumers);
        self.images.set_consumers(consumers);
        self.colors.set_consumers(consumers);
        Ok(())
    }
}
fn append_spans(
    table: &mut StagingVec<[u32; 4]>,
    presentation: &Presentation,
    request: &crate::native_selection_overlay::mask::Request,
    stroke: bool,
) -> anyhow::Result<()> {
    for span in if stroke {
        &presentation.strokes
    } else {
        &presentation.quads
    } {
        let label = if span.role == Role::Void {
            u32::MAX
        } else {
            request.label(span)
        };
        table.push([
            u32::try_from(span.range.start)?,
            u32::try_from(span.range.end)?,
            label,
            request.material_role(span),
        ]);
    }
    Ok(())
}
impl Renderer {
    pub fn selection_mask_attachment_observer(&self) -> crate::text_gpu::lifetime::Observer {
        self.selection_mask.images.sampled_observer()
    }
    pub fn selection_mask_attachment_snapshot(
        &self,
    ) -> Option<gpu_surface::SurfaceAttachmentSnapshot> {
        self.selection_mask.images.sampled_snapshot()
    }
    pub fn selection_mask_color_attachment_observer(&self) -> crate::text_gpu::lifetime::Observer {
        self.selection_mask.colors.sampled_observer()
    }
    pub fn selection_mask_color_attachment_snapshot(
        &self,
    ) -> Option<gpu_surface::SurfaceAttachmentSnapshot> {
        self.selection_mask.colors.sampled_snapshot()
    }
    pub fn selection_mask_cpu_storage_bytes(&self) -> u64 {
        self.selection_mask.table.allocated_bytes()
            + self.selection_mask.bindings.allocated_bytes()
            + self
                .selection_mask
                .bindings
                .iter()
                .flatten()
                .map(|binding| binding.images.allocated_bytes())
                .sum::<u64>()
    }
}
