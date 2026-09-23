//! Original instance-based glyph drawing over Datum's texture-page owner.
use std::ops::Range;

use glyphon::{Color, FontSystem, LayoutRun, SwashCache, TextBounds};

use super::atlas::Atlas;
use super::lifetime::{Kind, SubmissionRef, Tracked};

pub(crate) struct Area<'a, R> {
    pub rich_spans: &'a [crate::TextRunSpan],
    pub rows: R,
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub bounds: TextBounds,
    pub default_color: Color,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    rect: [f32; 4],
    tex: [u32; 4],
    color: u32,
    is_color: u32,
}

pub(crate) struct Draw {
    screen_budget: std::sync::Arc<super::budget::Budget>,
    generation_budget: std::sync::Arc<super::budget::Budget>,
    pipeline: wgpu::RenderPipeline,
    instances: Option<Tracked<wgpu::Buffer>>,
    batches: Vec<(usize, Range<u32>)>,
    snapshot: Box<[Instance]>,
    pending_instances: Option<Vec<Instance>>,
    generation: Option<u64>,
    pub upload_bytes: u64,
}

impl Draw {
    pub fn new(
        device: &wgpu::Device,
        atlas: &Atlas,
        format: wgpu::TextureFormat,
        samples: u32,
        screen_budget: std::sync::Arc<super::budget::Budget>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("datum-glyph-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glyph.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("datum-glyph-pipeline-layout"),
            bind_group_layouts: &[&atlas.layout],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-glyph-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Uint32x4, 2 => Uint32, 3 => Uint32],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
            multiview_mask: None,
            cache: None,
        });
        Self {
            screen_budget,
            generation_budget: super::budget::Budget::new(2),
            pipeline,
            instances: None,
            batches: Vec::new(),
            snapshot: Box::default(),
            pending_instances: None,
            generation: None,
            upload_bytes: 0,
        }
    }

    pub fn replacement(
        &self,
        device: &wgpu::Device,
        atlas: &Atlas,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> Self {
        let mut replacement = Self::new(device, atlas, format, samples, self.screen_budget.clone());
        replacement.generation_budget = self.generation_budget.clone();
        replacement
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare<'a, R: IntoIterator<Item = LayoutRun<'a>>>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut Atlas,
        fonts: &mut FontSystem,
        raster: &mut SwashCache,
        resolution: [u32; 2],
        areas: impl IntoIterator<Item = Area<'a, R>>,
    ) -> anyhow::Result<()> {
        self.generation = None;
        self.pending_instances = None;
        self.batches.clear();
        self.upload_bytes = 0;
        anyhow::ensure!(!resolution.contains(&0), "zero text target extent");
        let mut instances = Vec::new();
        for area in areas {
            let bounds = [
                area.bounds.left.max(0),
                area.bounds.top.max(0),
                area.bounds.right.min(resolution[0] as i32),
                area.bounds.bottom.min(resolution[1] as i32),
            ];
            for row in area.rows {
                let row_top = (area.top + row.line_top * area.scale) as i32;
                if row_top > bounds[3]
                    || row_top + ((row.line_height * area.scale) as i32) < bounds[1]
                {
                    continue;
                }
                for glyph in row.glyphs {
                    let physical = glyph.physical((area.left, area.top), area.scale);
                    let Some(location) =
                        atlas.glyph(device, queue, fonts, raster, physical.cache_key)?
                    else {
                        continue;
                    };
                    let x = physical.x + location.bearing[0];
                    let y =
                        physical.y + (row.line_y * area.scale).round() as i32 - location.bearing[1];
                    let left = x.max(bounds[0]);
                    let top = y.max(bounds[1]);
                    let right = (x + location.size[0] as i32).min(bounds[2]);
                    let bottom = (y + location.size[1] as i32).min(bounds[3]);
                    if left >= right || top >= bottom {
                        continue;
                    }
                    let index = instances.len() as u32;
                    instances.push(Instance {
                        rect: [
                            2.0 * left as f32 / resolution[0] as f32 - 1.0,
                            1.0 - 2.0 * top as f32 / resolution[1] as f32,
                            2.0 * (right - left) as f32 / resolution[0] as f32,
                            -2.0 * (bottom - top) as f32 / resolution[1] as f32,
                        ],
                        tex: [
                            location.origin[0] + (left - x) as u32,
                            location.origin[1] + (top - y) as u32,
                            (right - left) as u32,
                            (bottom - top) as u32,
                        ],
                        color: glyph
                            .metadata
                            .checked_sub(1)
                            .and_then(|index| area.rich_spans.get(index))
                            .map(|span| crate::text_color(span.color))
                            .or(glyph.color_opt)
                            .unwrap_or(area.default_color)
                            .0,
                        is_color: u32::from(location.color),
                    });
                    // Only adjacent glyphs are batched; overlapping text retains
                    // the caller's painter order even across mask/color pages.
                    if let Some((_, range)) = self
                        .batches
                        .last_mut()
                        .filter(|(page, _)| *page == location.page)
                    {
                        range.end = index + 1;
                    } else {
                        self.batches.push((location.page, index..index + 1));
                    }
                }
            }
        }
        let required = (instances.len() * std::mem::size_of::<Instance>()) as u64;
        if required == 0 {
            self.instances = None;
            self.snapshot = Box::default();
        } else if self.instances.as_ref().is_none_or(|buffer| {
            required > buffer.size() || buffer.size() > required.saturating_mul(4)
        }) {
            let capacity = required.next_power_of_two();
            let generation_permit = self.generation_budget.reserve(1).map_err(|_| {
                anyhow::anyhow!(
                    "glyph instances have two live GPU allocations; wait for retirement"
                )
            })?;
            let screen_permit = self.screen_budget.reserve(capacity)?;
            let reservation = super::budget::GpuReservation::new(
                capacity,
                vec![generation_permit, screen_permit],
            )?;
            self.instances = Some(atlas.owner.track_reserved(
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("datum-glyph-instances"),
                    size: capacity,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                }),
                atlas.generation,
                Kind::Instances,
                reservation,
            ));
            self.snapshot = Box::default();
        }
        self.pending_instances = Some(instances);
        self.generation = Some(atlas.generation);
        Ok(())
    }

    /// Share the screen-stream changed-range writer. Updates reach the queue
    /// only at the caller's successful frame submission boundary.
    pub fn has_pending_uploads(&self) -> bool {
        self.pending_instances.is_some()
    }

    pub fn cancel_preparation(&mut self) {
        self.generation = None;
        self.pending_instances = None;
        self.batches.clear();
    }

    pub fn append_uploads<'a>(&'a self, out: &mut Vec<super::upload::BufferUpload<'a>>) -> u64 {
        match (&self.pending_instances, &self.instances) {
            (Some(instances), Some(buffer)) => crate::gpu_data::screen_buffer::dirty_ranges(
                bytemuck::cast_slice(&self.snapshot),
                bytemuck::cast_slice(instances),
                std::mem::size_of::<Instance>(),
                |offset, bytes| {
                    out.push(super::upload::BufferUpload {
                        buffer,
                        offset,
                        bytes,
                    })
                },
            ) as u64,
            _ => 0,
        }
    }

    pub fn finish_uploads(&mut self, bytes: u64) {
        self.upload_bytes = bytes;
        if let Some(instances) = self.pending_instances.take() {
            // Every prepared payload has passed screen/process GPU admission.
            // Retain its exact length for dirty comparisons, without spare Vec
            // capacity or a size cutoff that forces subsequent full transfers.
            self.snapshot = instances.into_boxed_slice();
        }
    }

    #[cfg(all(test, feature = "visual"))]
    pub fn flush_uploads(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut uploads = Vec::new();
        let bytes = self.append_uploads(&mut uploads);
        super::upload::submit_buffers_for_test(device, queue, &uploads);
        self.finish_uploads(bytes);
    }

    pub fn submission_ref(&self) -> Option<SubmissionRef> {
        self.instances.as_ref().map(Tracked::submission_ref)
    }

    pub fn render(&self, atlas: &Atlas, pass: &mut wgpu::RenderPass<'_>) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.generation == Some(atlas.generation),
            "stale or failed glyph preparation"
        );
        if let Some(instances) = &self.instances {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, instances.slice(..));
            for (page, range) in &self.batches {
                pass.set_bind_group(0, &atlas.pages[*page].bind_group, &[]);
                pass.draw(0..6, range.clone());
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "draw_generation_tests.rs"]
mod generation_tests;
