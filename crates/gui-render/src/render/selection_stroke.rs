//! Post-world selection of the existing authored capsule presentation.
//! Screen positions are disposable projections, never acquisition or identities.
use super::*;
#[path = "selection_stroke/gpu.rs"]
pub(super) mod gpu;
#[path = "selection_mask.rs"]
pub(crate) mod mask;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Stroke {
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub material_width: [f32; 4],
    pub clip: [f32; 4],
    pub high_contrast: [f32; 4],
}
impl Stroke {
    /// Disposable minimum-cue presentation mode: 1 = open ring, 2 = X segment.
    /// No acquisition, source identity, or authored material is encoded here.
    pub(super) fn minimum(from: [f32; 2], to: [f32; 2], viewport: RectPx, mode: f32) -> Self {
        Self {
            from,
            to,
            material_width: [0.0, 0.0, 0.0, if mode == 1.0 { 6.0 } else { 2.0 }],
            clip: [
                viewport.x,
                viewport.y,
                viewport.x + viewport.width,
                viewport.y + viewport.height,
            ],
            high_contrast: [1.0, mode, 0.0, 0.0],
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn push(
    out: &mut Vec<Stroke>,
    path: &[PointNm],
    width_nm: i64,
    min_px: f32,
    material: [f32; 3],
    projection: &Projection,
    viewport: RectPx,
    high_contrast: bool,
) {
    let width = (width_nm.max(0) as f32 * projection.scale).max(min_px);
    for pair in path.windows(2).filter(|p| p[0] != p[1]) {
        let a = projection.project_point(pair[0]);
        let b = projection.project_point(pair[1]);
        let radius = width * 0.5 + 2.0;
        if a.0.min(b.0) - radius > viewport.x + viewport.width
            || a.0.max(b.0) + radius < viewport.x
            || a.1.min(b.1) - radius > viewport.y + viewport.height
            || a.1.max(b.1) + radius < viewport.y
        {
            continue;
        }
        out.push(Stroke {
            from: [a.0, a.1],
            to: [b.0, b.1],
            material_width: [material[0], material[1], material[2], width],
            clip: [
                viewport.x,
                viewport.y,
                viewport.x + viewport.width,
                viewport.y + viewport.height,
            ],
            high_contrast: [if high_contrast { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
        });
    }
}
pub(super) struct Pipelines {
    pub lift: wgpu::RenderPipeline,
    pub cue: wgpu::RenderPipeline,
}
pub(super) fn pipelines(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    samples: u32,
) -> Pipelines {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("datum-selection-capsules"),
        source: wgpu::ShaderSource::Wgsl(
            crate::renderer_state::damage::shader(include_str!("selection_stroke.wgsl"), 1).into(),
        ),
    });
    let build = |entry, blend| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("datum-selection-capsules"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[gpu::Instance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    };
    Pipelines {
        lift: build("fs_lift", wgpu::BlendState::REPLACE),
        cue: build("fs_cue", wgpu::BlendState::ALPHA_BLENDING),
    }
}

#[cfg(test)]
#[path = "selection_stroke_tests.rs"]
mod tests;
