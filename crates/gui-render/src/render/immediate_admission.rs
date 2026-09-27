//! Admission metadata from existing screen geometry and grid preparation.
use super::*;

#[derive(Clone, Copy, Debug)]
pub struct PreparedScreenGeometry {
    pub group: &'static str,
    pub vertices: usize,
    pub payload_bytes: usize,
}
impl PreparedScene {
    /// CPU-prepared streams, not a claim that each stream was submitted.
    /// Control meshes are already expanded into these production vertices.
    pub fn screen_geometry_admission(&self) -> [PreparedScreenGeometry; 8] {
        [
            ("panel", &self.panel_vertices),
            ("menu_overlay", &self.menu_overlay_vertices),
            ("viewport_underlay", &self.viewport_underlay_vertices),
            ("viewport_overlay", &self.viewport_overlay_vertices),
            ("board_interaction", &self.board_interaction_vertices),
            ("console_overlay", &self.console_overlay_vertices),
            ("schematic_underlay", &self.schematic_underlay_vertices),
            ("schematic_overlay", &self.schematic_overlay_vertices),
        ]
        .map(|(group, vertices)| PreparedScreenGeometry {
            group,
            vertices: vertices.len(),
            payload_bytes: std::mem::size_of_val(vertices.as_slice()),
        })
    }
    pub fn prepared_terminal_graphic_count(&self) -> usize {
        self.terminal_graphics.len()
    }
}

#[derive(Clone, Debug)]
pub struct PreparedGridAdmission {
    pub pane_id: datum_gui_protocol::PaneId,
    pub viewport: RectPx,
    pub vertices: std::ops::Range<u32>,
    pub generated_vertices: usize,
}
impl Renderer {
    pub(crate) fn observe_surface_grids(
        &mut self,
        vertices: &[Vertex],
        batches: &[surface_grid_pass::SurfaceGridBatch],
    ) -> anyhow::Result<()> {
        if self.frame_observer.is_none() {
            return Ok(());
        }
        let scope = crate::cpu_alloc::Scope::new("grid-admission-observer");
        self.grid_admission = Some(scope.with(|| -> anyhow::Result<_> {
            let mut observations = crate::text_gpu::staging_vec::StagingVec::new(
                batches.len(),
                &self.atlas.staging_budget,
            )?;
            for batch in batches {
                observations.push(PreparedGridAdmission {
                    pane_id: batch.pane_id,
                    viewport: batch.viewport,
                    vertices: batch.vertices.clone(),
                    generated_vertices: vertices.len(),
                });
            }
            Ok(observations)
        })?);
        Ok(())
    }
    /// Absent before grid generation in this attempt; empty means no batches.
    /// Generated ranges do not establish submission or raster coverage.
    pub fn grid_geometry_admission(&self) -> Option<&[PreparedGridAdmission]> {
        self.grid_admission.as_deref()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TerminalGeometryAdmission {
    pub graphic_id: u64,
    pub foreground: bool,
    pub scissor: [u32; 4],
    pub vertices: u32,
    pub payload_bytes: usize,
}
impl Renderer {
    /// Current uploaded terminal draws; callers must establish current submission.
    pub fn terminal_geometry_admission(
        &self,
    ) -> impl Iterator<Item = TerminalGeometryAdmission> + '_ {
        self.terminal_graphics.geometry_admission()
    }
}
