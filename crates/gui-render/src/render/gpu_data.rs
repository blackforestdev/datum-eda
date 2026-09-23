#[path = "document_gpu_budget.rs"]
mod document_gpu_budget;
#[path = "retained_buffer.rs"]
pub(crate) mod retained_buffer;
#[path = "shared_geometry.rs"]
pub(crate) mod shared_geometry;
#[path = "uniform_buffer.rs"]
pub(crate) mod uniform_buffer;
#[path = "vertex_allocation.rs"]
mod vertex_allocation;

#[path = "screen_buffer.rs"]
pub(crate) mod screen_buffer;

use super::Quad;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ScreenUniform {
    pub(crate) resolution: [f32; 2],
    pub(crate) _pad: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SceneUniform {
    pub(crate) resolution: [f32; 4],
    pub(crate) viewport_origin: [f32; 4],
    pub(crate) viewport_size: [f32; 4],
    pub(crate) camera_center_scale: [f32; 4],
}

impl Vertex {
    pub(crate) fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Quad colors stay in sRGB *token* space on the CPU (so vertex-color contract
/// tests compare against the design tokens directly). The sRGB->linear
/// conversion happens in the fragment shader, at the GPU boundary, so the sRGB
/// surface's encode round-trips to the authored token instead of washing
/// near-blacks up to grey. Text goes through glyphon, which is already sRGB-aware.
fn quad_to_vertices(out: &mut Vec<Vertex>, quad: Quad) {
    let [a, b, c, d] = quad.points;
    out.extend_from_slice(&[
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [b.0, b.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [a.0, a.1],
            color: quad.color,
        },
        Vertex {
            pos: [c.0, c.1],
            color: quad.color,
        },
        Vertex {
            pos: [d.0, d.1],
            color: quad.color,
        },
    ]);
}

pub(crate) fn quads_to_vertices(quads: &[Quad]) -> Vec<Vertex> {
    let mut out = Vec::with_capacity(quads.len() * 6);
    for quad in quads {
        quad_to_vertices(&mut out, *quad);
    }
    out
}
