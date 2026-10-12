//! Lossless GPU-only quad input; CPU expanded vertices remain the painter authority.
use super::super::*;
use crate::text_gpu::{budget::Budget, staging_vec::StagingVec};
use std::sync::Arc;

// One owner stores exact X, Y and RGB planes; pan updates stay axis-local.
pub(crate) fn layouts() -> [wgpu::VertexBufferLayout<'static>; 3] {
    const X: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0=>Float32x4];
    const Y: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![1=>Float32x4];
    const COLOR: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![4=>Float32x3];
    [
        wgpu::VertexBufferLayout {
            array_stride: 16,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &X,
        },
        wgpu::VertexBufferLayout {
            array_stride: 16,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Y,
        },
        wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &COLOR,
        },
    ]
}
pub(crate) fn bind<'a>(pass: &mut wgpu::RenderPass<'a>, buffer: &'a wgpu::Buffer, count: usize) {
    let axis = count as u64 * 16;
    let coordinates = axis * 2;
    let total = count as u64 * 44;
    assert!(
        count > 0 && total <= buffer.size(),
        "current quad planes must fit admitted storage"
    );
    pass.set_vertex_buffer(0, buffer.slice(..axis));
    pass.set_vertex_buffer(1, buffer.slice(axis..coordinates));
    pass.set_vertex_buffer(2, buffer.slice(coordinates..total));
}
pub(crate) fn encode(vertices: &[Vertex], staging: &Arc<Budget>) -> anyhow::Result<StagingVec<u8>> {
    anyhow::ensure!(vertices.len().is_multiple_of(6), "incomplete viewport quad");
    // Validate all source topology before allocation or producer publication.
    for quad in vertices.chunks_exact(6) {
        anyhow::ensure!(
            bytemuck::bytes_of(&quad[0].pos) == bytemuck::bytes_of(&quad[3].pos)
                && bytemuck::bytes_of(&quad[2].pos) == bytemuck::bytes_of(&quad[4].pos)
                && quad
                    .iter()
                    .all(|v| bytemuck::bytes_of(&v.color) == bytemuck::bytes_of(&quad[0].color)),
            "non-quad viewport vertices"
        );
    }
    let count = vertices.len() / 6;
    let mut bytes = StagingVec::new(
        count
            .checked_mul(44)
            .ok_or_else(|| anyhow::anyhow!("quad bytes overflow"))?,
        staging,
    )?;
    for axis in 0..2 {
        for quad in vertices.chunks_exact(6) {
            bytes.extend_from_slice(bytemuck::bytes_of(&[
                quad[0].pos[axis],
                quad[1].pos[axis],
                quad[2].pos[axis],
                quad[5].pos[axis],
            ]));
        }
    }
    for quad in vertices.chunks_exact(6) {
        bytes.extend_from_slice(bytemuck::bytes_of(&quad[0].color));
    }
    Ok(bytes)
}
pub(crate) fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    msaa_samples: u32,
    compact: bool,
) -> wgpu::RenderPipeline {
    let compact_layouts = layouts();
    let expanded_layouts = [Vertex::layout()];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("datum-gui-render-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(if compact { "vs_compact" } else { "vs_main" }),
            buffers: if compact {
                &compact_layouts
            } else {
                &expanded_layouts
            },
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
pub(crate) const VERTEX_SHADER: &str = r#"
struct CompactIn {
 @builtin(vertex_index) vertex:u32,
 @location(0) x:vec4<f32>, @location(1) y:vec4<f32>,
 @location(4) color:vec3<f32>,
};
@vertex fn vs_compact(in:CompactIn)->VsOut {
 let a=vec2<f32>(in.x.x,in.y.x); let b=vec2<f32>(in.x.y,in.y.y);
 let c=vec2<f32>(in.x.z,in.y.z); let d=vec2<f32>(in.x.w,in.y.w);
 let points=array<vec2<f32>,6>(a,b,c,a,c,d);
 let p=points[in.vertex]; var out:VsOut;
 out.position=vec4<f32>(p.x/screen.resolution.x*2.0-1.0,1.0-p.y/screen.resolution.y*2.0,0.0,1.0);
 out.color=in.color; return out;
}
"#;

#[cfg(test)]
#[path = "screen_quad_gpu/tests.rs"]
mod tests;
