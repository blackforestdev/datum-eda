use super::*;
use std::ops::Range;

pub(crate) struct SurfaceGridBatch {
    pub pane_id: datum_gui_protocol::PaneId,
    pub viewport: RectPx,
    pub vertices: Range<u32>,
}

#[cfg(test)]
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
                grid::push_schematic_grid_with_lod(&mut quads, &projection, pass.grid_lod_resolved);
            }
        }
        let start = vertices.len() as u32;
        vertices.extend(quads_to_vertices(&quads));
        let end = vertices.len() as u32;
        if start != end {
            batches.push(SurfaceGridBatch {
                pane_id: pass.pane_id,
                viewport: pass.scene_viewport,
                vertices: start..end,
            });
        }
    }
    (vertices, batches)
}

use crate::text_gpu::{budget::Budget, staging_vec::StagingVec};
use std::sync::Arc;

/// One current geometry generation; replacement is admitted while the old one
/// remains live. Failed preparation never publishes a partial cache entry.
pub(super) struct GridCache {
    keys: StagingVec<PreparedSurfacePass>,
    pub(super) vertices: StagingVec<Vertex>,
    pub(super) batches: StagingVec<SurfaceGridBatch>,
    scope: crate::cpu_alloc::Scope,
}
impl Default for GridCache {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            vertices: Default::default(),
            batches: Default::default(),
            scope: crate::cpu_alloc::Scope::new("surface-grid-cache"),
        }
    }
}
impl GridCache {
    pub(super) fn allocated_bytes(&self) -> u64 {
        self.keys.allocated_bytes()
            + self.vertices.allocated_bytes()
            + self.batches.allocated_bytes()
    }

    pub(super) fn prepare(
        &mut self,
        passes: &[PreparedSurfacePass],
        budget: &Arc<Budget>,
    ) -> anyhow::Result<bool> {
        if self.keys.len() == passes.len()
            && self.keys.iter().zip(passes).all(|(a, b)| {
                a.pane_id == b.pane_id
                    && a.surface == b.surface
                    && a.scene_viewport == b.scene_viewport
                    && a.bounds == b.bounds
                    && a.camera == b.camera
                    && a.grid_lod_resolved == b.grid_lod_resolved
            })
        {
            return Ok(false);
        }
        let (keys, vertices, batches) = self.scope.with(|| -> anyhow::Result<_> {
            let mut keys = StagingVec::new(passes.len(), budget)?;
            let mut count = 0usize;
            for pass in passes {
                grid::visit_surface_grid(pass, |_| {
                    count += 1;
                    true
                });
                keys.push(pass.clone());
            }
            let mut vertices = StagingVec::new(
                count
                    .checked_mul(6)
                    .ok_or_else(|| anyhow::anyhow!("surface grid vertex count overflow"))?,
                budget,
            )?;
            let mut batches = StagingVec::new(passes.len(), budget)?;
            for pass in passes {
                let start = u32::try_from(vertices.len())?;
                grid::visit_surface_grid(pass, |line| {
                    let quad = Quad::from_rect(
                        RectPx {
                            x: line.x,
                            y: line.y,
                            width: line.width,
                            height: line.height,
                        },
                        line.color,
                    );
                    vertices.extend_from_slice(&gpu_data::quad_vertices(quad));
                    true
                });
                let end = u32::try_from(vertices.len())?;
                if start != end {
                    batches.push(SurfaceGridBatch {
                        pane_id: pass.pane_id,
                        viewport: pass.scene_viewport,
                        vertices: start..end,
                    });
                }
            }
            Ok((keys, vertices, batches))
        })?;
        self.keys = keys;
        self.vertices = vertices;
        self.batches = batches;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "surface_grid_cache_tests.rs"]
mod tests;
