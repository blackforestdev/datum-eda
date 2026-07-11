use super::*;
use std::ops::Range;

pub(crate) struct SurfaceGridBatch {
    pub viewport: RectPx,
    pub vertices: Range<u32>,
}

pub(crate) fn build_surface_grids(
    prepared: &PreparedScene,
) -> (Vec<Vertex>, Vec<SurfaceGridBatch>) {
    let mut vertices = Vec::new();
    let mut batches = Vec::new();
    for pass in prepared.surface_passes() {
        let field = inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let projection = Projection::new(field, &pass.bounds, pass.camera);
        let mut quads = Vec::new();
        match pass.surface {
            SceneSurface::Board => {
                grid::push_scene_grid_with_lod(&mut quads, &projection, pass.grid_lod_resolved);
            }
            SceneSurface::Schematic => {
                grid::push_schematic_grid_with_lod(
                    &mut quads,
                    &projection,
                    pass.grid_lod_resolved,
                );
            }
        }
        let start = vertices.len() as u32;
        vertices.extend(quads_to_vertices(&quads));
        let end = vertices.len() as u32;
        if start != end {
            batches.push(SurfaceGridBatch {
                viewport: pass.scene_viewport,
                vertices: start..end,
            });
        }
    }
    (vertices, batches)
}
