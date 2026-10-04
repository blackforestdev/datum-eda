//! Exact CPU-transformed glyph triangles with one color per painter batch.
//! Geometry ordinals are disposable; source spans still resolve through selection authority.
use crate::presentation_ranges::Role;
use crate::*;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Point {
    pub pos: [f32; 2],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Material {
    pub color: [f32; 3],
    pub pad: f32,
}
impl Point {
    pub(crate) fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}
impl Material {
    pub(crate) fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 16,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn push(
    quads: &mut impl Output<Quad>,
    points: &mut impl Output<Point>,
    materials: &mut impl Output<Material>,
    commands: &mut impl Output<RetainedDrawCommand>,
    scene: &BoardReviewSceneV1,
    projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    let Some(assets) = glyph_mesh_lookup::MeshIndex::new(&scene.glyph_mesh_assets, points) else {
        return;
    };
    let meshes = |text: &BoardTextGeometryPrimitive| {
        text.world_transform_nm.is_some() && !text.glyphs.is_empty()
    };
    let mut count = 0usize;
    for text in scene
        .board_text_geometries
        .iter()
        .filter(|t| layer_visible(state, &t.layer_id) && meshes(t))
    {
        for glyph in &text.glyphs {
            use glyph_mesh_lookup::GlyphMeshLookup;
            if let Some(asset) = assets.mesh(&glyph.glyph_handle) {
                for triangle in asset.indices.as_chunks::<3>().0 {
                    if triangle
                        .iter()
                        .all(|i| (*i as usize) < asset.vertices.len())
                    {
                        let Some(next) = count.checked_add(3) else {
                            points
                                .allocation_failed(anyhow::anyhow!("glyph vertex count overflow"));
                            return;
                        };
                        count = next;
                    }
                }
            }
        }
    }
    points.reserve_exact(count);
    for stage in POST_COPPER_STAGES {
        for text in scene.board_text_geometries.iter().filter(|t| {
            render_stage_for_layer(&t.layer_id, &scene.layers) == stage
                && layer_visible(state, &t.layer_id)
        }) {
            let color = board_graphic_world_color(
                &text.layer_id,
                &scene.layers,
                dim_unrelated_active(state),
            );
            if !meshes(text) {
                quads.source(Some(&text.object_id), Role::Material);
                let start = quads.len();
                push_board_text_geometry_world(quads, text, &assets, color, projection);
                scene_retained_access::finish_retained_draw_commands(
                    commands,
                    Some(&text.layer_id),
                    start,
                    quads.len(),
                    0,
                    0,
                );
                continue;
            }
            points.source(Some(&text.object_id), Role::Material);
            let start = points.len();
            board_text_geometry::for_each_mesh_triangle(
                text,
                &assets,
                text.world_transform_nm.unwrap(),
                |triangle| {
                    points.extend(triangle.map(|p| Point { pos: [p.0, p.1] }));
                },
            );
            if start == points.len() {
                continue;
            }
            let material = Material { color, pad: 0.0 };
            if materials.last() != Some(&material) {
                materials.push(material);
            }
            if materials.last() != Some(&material) {
                return;
            }
            let Ok(material) = u32::try_from(materials.len() - 1) else {
                materials.allocation_failed(anyhow::anyhow!("glyph material index overflow"));
                return;
            };
            let (Ok(start), Ok(end)) = (u32::try_from(start), u32::try_from(points.len())) else {
                points.allocation_failed(anyhow::anyhow!("glyph draw index overflow"));
                return;
            };
            if let Some(RetainedDrawCommand::Glyphs {
                layer_id,
                range,
                material: prior,
            }) = commands.last_mut()
                && layer_id.as_deref() == Some(text.layer_id.as_str())
                && *prior == material
                && range.end == start
            {
                range.end = end;
            } else {
                commands.push(RetainedDrawCommand::Glyphs {
                    layer_id: Some(text.layer_id.clone()),
                    range: start..end,
                    material,
                });
            }
        }
    }
}

use gpu_data::shared_geometry::{GeometryElement, SharedGeometry};
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Geometry {
    pub points: SharedGeometry<Point>,
    pub materials: SharedGeometry<Material>,
}
impl Default for Geometry {
    fn default() -> Self {
        Self {
            points: Vec::new().into(),
            materials: Vec::new().into(),
        }
    }
}
impl Geometry {
    pub(crate) fn from_document(points: Vec<Point>, materials: Vec<Material>, scene: &str) -> Self {
        Self {
            points: SharedGeometry::for_document(points, scene),
            materials: SharedGeometry::for_document(materials, scene),
        }
    }
    pub(crate) fn admit(
        points: &Vec<Point>,
        materials: &Vec<Material>,
        mut admit: impl FnMut(usize) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut bytes = Point::container_bytes() + Material::container_bytes();
        if points.capacity() != points.len() {
            bytes = bytes
                .checked_add(crate::cpu_alloc::heap::capacity_bytes::<Point>(
                    points.len(),
                ))
                .ok_or_else(|| anyhow::anyhow!("glyph payload overflow"))?;
        }
        if materials.capacity() != materials.len() {
            bytes = bytes
                .checked_add(crate::cpu_alloc::heap::capacity_bytes::<Material>(
                    materials.len(),
                ))
                .ok_or_else(|| anyhow::anyhow!("glyph payload overflow"))?;
        }
        admit(bytes)
    }
    pub(crate) fn bytes(&self) -> usize {
        self.points.heap_bytes() + self.materials.heap_bytes()
    }
    pub(crate) fn observe(&self) -> Observer {
        Observer {
            points: self.points.downgrade(),
            materials: self.materials.downgrade(),
            bytes: [
                crate::cpu_alloc::heap::capacity_bytes::<Point>(self.points.len()),
                crate::cpu_alloc::heap::capacity_bytes::<Material>(self.materials.len()),
            ],
        }
    }
}
#[derive(Clone)]
pub(crate) struct Observer {
    points: std::sync::Weak<Box<[Point]>>,
    materials: std::sync::Weak<Box<[Material]>>,
    bytes: [usize; 2],
}
impl Observer {
    pub(crate) fn shares(&self, other: &Self) -> [bool; 2] {
        [
            self.points.ptr_eq(&other.points),
            self.materials.ptr_eq(&other.materials),
        ]
    }
    pub(crate) fn live(&self) -> bool {
        self.points.strong_count() != 0 || self.materials.strong_count() != 0
    }
    pub(crate) fn bytes_mask(&self, excluded: [bool; 2]) -> usize {
        let mut points =
            Point::container_bytes() + usize::from(self.points.strong_count() != 0) * self.bytes[0];
        let mut materials = Material::container_bytes()
            + usize::from(self.materials.strong_count() != 0) * self.bytes[1];
        if excluded[0] {
            points = 0;
        }
        if excluded[1] {
            materials = 0;
        }
        points + materials
    }
}

pub(crate) struct Builders<F> {
    pub points: crate::geometry_output::Admitted<Point, F>,
    pub materials: crate::geometry_output::Admitted<Material, F>,
}
pub(crate) fn builders<'a>(
    budget: &'a std::sync::Arc<crate::text_gpu::budget::Budget>,
    scope: &'a crate::cpu_alloc::Scope,
    limit: usize,
) -> Builders<impl FnMut(usize) -> anyhow::Result<()> + 'a> {
    let admit = move |bytes| {
        retained_scene_owner::document_cpu::admit_constructor_allocation(
            budget,
            scope,
            bytes,
            limit,
            "geometry emission",
        )
    };
    Builders {
        points: crate::geometry_output::Admitted::new(admit).track_sources(),
        materials: crate::geometry_output::Admitted::new(admit),
    }
}

#[cfg(test)]
#[path = "retained_glyph_tests.rs"]
mod tests;
