#[path = "document_gpu_budget.rs"]
mod document_gpu_budget;
pub use document_gpu_budget::DocumentGpuUsage;
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
    try_quads_to_vertices(quads).expect("quad vertex allocation succeeds")
}

fn reserve_quad_vertices(quads: usize) -> anyhow::Result<Vec<Vertex>> {
    let count = quads
        .checked_mul(6)
        .ok_or_else(|| anyhow::anyhow!("quad vertex count overflow"))?;
    let mut out = Vec::new();
    out.try_reserve_exact(count)
        .map_err(|error| anyhow::anyhow!("quad vertex allocation refused: {error}"))?;
    Ok(out)
}

/// Reserve the admitted capacity fallibly before writing any expanded vertices.
pub(crate) fn try_quads_to_vertices(quads: &[Quad]) -> anyhow::Result<Vec<Vertex>> {
    let mut out = reserve_quad_vertices(quads.len())?;
    for quad in quads {
        quad_to_vertices(&mut out, *quad);
    }
    Ok(out)
}

#[cfg(test)]
mod quad_allocation_tests {
    use super::*;

    #[test]
    fn fallible_expansion_uses_one_exact_allocation_and_preserves_winding_color() {
        let scope = crate::cpu_alloc::Scope::new("quad-allocation-proof");
        let quad = Quad {
            points: [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0)],
            color: [0.1, 0.2, 0.3],
        };
        let vertices = scope.with(|| try_quads_to_vertices(&[quad])).unwrap();
        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices.capacity(), 6);
        for (vertex, index) in vertices.iter().zip([0, 1, 2, 0, 2, 3]) {
            let point = quad.points[index];
            assert_eq!(vertex.pos, [point.0, point.1]);
            assert_eq!(vertex.color, quad.color);
        }
        let usage = scope.usage();
        assert_eq!(usage.allocations, 1);
        assert_eq!(
            usage.payload_bytes + usage.tracking_bytes,
            crate::cpu_alloc::heap::capacity_bytes::<Vertex>(6) as u64
        );
        drop(vertices);
        assert_eq!(scope.usage().allocations, 0);
    }

    #[test]
    fn impossible_capacity_is_refused_without_publishing_vertices() {
        assert!(reserve_quad_vertices(usize::MAX).is_err());
        // Multiplication fits, but the element layout cannot fit a Rust allocation.
        assert!(reserve_quad_vertices(usize::MAX / 6).is_err());
        let empty = try_quads_to_vertices(&[]).unwrap();
        assert_eq!(empty.capacity(), 0);
    }
}
