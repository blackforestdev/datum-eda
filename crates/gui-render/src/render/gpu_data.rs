use wgpu::util::DeviceExt;

use super::{Quad, Renderer};

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

impl Renderer {
    pub(crate) fn upload_vertices(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &mut Option<wgpu::Buffer>,
        capacity: &mut usize,
        label: &str,
        vertices: &[Vertex],
    ) {
        let bytes = bytemuck::cast_slice(vertices);
        if buffer.is_none() || *capacity < bytes.len() {
            *buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytes,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
            );
            *capacity = bytes.len();
            return;
        }
        if let Some(buffer) = buffer.as_ref() {
            queue.write_buffer(buffer, 0, bytes);
        }
    }

    pub(crate) fn sync_world_vertices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &std::sync::Arc<[Vertex]>,
    ) {
        if self.world_vertex_buffer.is_some()
            && same_vertex_source(&self.world_vertex_source, vertices)
        {
            return;
        }
        Self::upload_vertices(
            device,
            queue,
            &mut self.world_vertex_buffer,
            &mut self.world_vertex_capacity,
            "datum-gui-render-world-vertex-buffer",
            vertices,
        );
        self.world_vertex_source = Some(vertices.clone());
    }
}

/// Holding the immutable allocation prevents an old address from being reused
/// for a different scene. A raw pointer/length pair alone cannot establish this.
pub(crate) fn same_vertex_source(
    cached: &Option<std::sync::Arc<[Vertex]>>,
    current: &std::sync::Arc<[Vertex]>,
) -> bool {
    cached
        .as_ref()
        .is_some_and(|cached| std::sync::Arc::ptr_eq(cached, current))
}

#[cfg(test)]
mod vertex_source_tests {
    use super::*;

    #[test]
    fn cloned_scene_reuses_storage_but_equal_sized_replacement_requires_upload() {
        let state = crate::gpu_surface_pass::board_fixture_state();
        let scene = crate::RetainedScene::from_workspace(&state, 1280, 800);
        assert!(!scene.world_vertices.is_empty());
        let clone = scene.clone();
        let cached = Some(scene.world_vertices.clone());
        assert!(same_vertex_source(&cached, &clone.world_vertices));
        let replacement = std::sync::Arc::from(scene.world_vertices.to_vec());
        assert_eq!(&*scene.world_vertices, &*replacement);
        assert!(!same_vertex_source(&cached, &replacement));
        assert!(!same_vertex_source(&None, &replacement));
        drop(scene);
        drop(clone);
        // The upload cache owns the old allocation until replacement, so a new
        // allocation can never masquerade as it through allocator address reuse.
        assert_eq!(std::sync::Arc::strong_count(cached.as_ref().unwrap()), 1);
        assert!(!same_vertex_source(&cached, &replacement));
    }
}
