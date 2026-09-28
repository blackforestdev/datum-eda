//! Read-only admission views over production prepared/retained geometry.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSourceIdentity {
    pub scene_id: String,
    /// Native projections carry the engine model revision; legacy projections
    /// carry their source revision. The observation does not relabel one as the other.
    pub source_revision: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSourceIdentities {
    pub terminal_sessions: Vec<String>,
    pub board: PreparedSourceIdentity,
    pub schematic: Option<PreparedSourceIdentity>,
}
impl PreparedSourceIdentities {
    pub(crate) fn observe(
        state: &ReviewWorkspaceState,
        terminal_panes: &[crate::TerminalPaneRenderState<'_>],
    ) -> Option<std::sync::Arc<Self>> {
        std::env::var_os("DATUM_RESOURCE_TRACE").map(|_| Self::capture(state, terminal_panes))
    }
    fn capture(
        state: &ReviewWorkspaceState,
        terminal_panes: &[crate::TerminalPaneRenderState<'_>],
    ) -> std::sync::Arc<Self> {
        let scope = crate::cpu_alloc::Scope::new("prepared-source-observer");
        scope.with(|| {
            let identity = |s: &BoardReviewSceneV1| PreparedSourceIdentity {
                scene_id: s.scene_id.clone(),
                source_revision: s.source_revision.clone(),
            };
            std::sync::Arc::new(Self {
                terminal_sessions: terminal_panes
                    .iter()
                    .map(|p| p.session_id.clone())
                    .collect(),
                board: identity(&state.scene),
                schematic: state.schematic_scene.as_ref().map(identity),
            })
        })
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GeometryAdmissionCounts {
    pub retained_vertices: usize,
    pub retained_stroke_instances: usize,
    pub prepared_commands: usize,
    pub prepared_vertices: u64,
    pub prepared_stroke_instances: u64,
    pub prepared_triangles: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryAdmissionRange {
    Vertices(std::ops::Range<u32>),
    StrokeInstances(std::ops::Range<u32>),
}
#[derive(Clone, Copy)]
pub struct PreparedGeometryAdmission<'a> {
    pub pane_id: datum_gui_protocol::PaneId,
    pub surface: SceneSurface,
    pub viewport: RectPx,
    pub source: Option<&'a PreparedSourceIdentity>,
    retained: Option<&'a RetainedScene>,
    commands: &'a [RetainedDrawCommand],
}
impl PreparedGeometryAdmission<'_> {
    pub fn ranges(&self) -> impl Iterator<Item = GeometryAdmissionRange> + '_ {
        self.commands.iter().map(|c| match c {
            RetainedDrawCommand::Quads { range, .. } => {
                GeometryAdmissionRange::Vertices(range.clone())
            }
            RetainedDrawCommand::Strokes { range, .. } => {
                GeometryAdmissionRange::StrokeInstances(range.clone())
            }
        })
    }
    /// Prepared ranges have survived production visibility selection. They are
    /// not raster-visible pixels, and no submission is asserted by this view.
    pub fn counts(&self) -> anyhow::Result<GeometryAdmissionCounts> {
        let retained = self
            .retained
            .ok_or_else(|| anyhow::anyhow!("missing retained scene for prepared pane"))?;
        let mut counts = GeometryAdmissionCounts {
            retained_vertices: retained.world_vertices.len(),
            retained_stroke_instances: retained.world_strokes.len(),
            prepared_commands: self.commands.len(),
            ..Default::default()
        };
        for command in self.ranges() {
            let (range, capacity, strokes) = match command {
                GeometryAdmissionRange::Vertices(r) => (r, counts.retained_vertices, false),
                GeometryAdmissionRange::StrokeInstances(r) => {
                    (r, counts.retained_stroke_instances, true)
                }
            };
            anyhow::ensure!(
                range.start <= range.end && range.end as usize <= capacity,
                "prepared range exceeds retained geometry"
            );
            let n = u64::from(range.end - range.start);
            if strokes {
                counts.prepared_stroke_instances = counts
                    .prepared_stroke_instances
                    .checked_add(n)
                    .ok_or_else(|| anyhow::anyhow!("stroke count overflow"))?;
                counts.prepared_triangles = counts
                    .prepared_triangles
                    .checked_add(n * 2)
                    .ok_or_else(|| anyhow::anyhow!("triangle count overflow"))?;
            } else {
                counts.prepared_vertices = counts
                    .prepared_vertices
                    .checked_add(n)
                    .ok_or_else(|| anyhow::anyhow!("vertex count overflow"))?;
                // Triangle-list assembly restarts per draw; do not join remainders.
                counts.prepared_triangles = counts
                    .prepared_triangles
                    .checked_add(n / 3)
                    .ok_or_else(|| anyhow::anyhow!("triangle count overflow"))?;
            }
        }
        Ok(counts)
    }
}
impl PreparedScene {
    pub fn admission_sources(&self) -> Option<&PreparedSourceIdentities> {
        self.admission_sources.as_deref()
    }
    /// One view per production world pass. Shared retained storage is repeated
    /// incidence, never additional allocation. Native dialogs have no world pass.
    pub fn geometry_admission<'a>(
        &'a self,
        board: &'a RetainedScene,
        schematic: Option<&'a RetainedScene>,
    ) -> impl Iterator<Item = PreparedGeometryAdmission<'a>> + 'a {
        self.surface_passes
            .iter()
            .map(|pass| (pass.pane_id, pass.surface, pass.scene_viewport))
            .map(move |(pane_id, surface, viewport)| {
                let (retained, commands, source) = match surface {
                    SceneSurface::Board => (
                        Some(board),
                        self.visible_draw_commands.as_slice(),
                        self.admission_sources.as_ref().map(|s| &s.board),
                    ),
                    SceneSurface::Schematic => (
                        schematic,
                        schematic.map_or(&[][..], RetainedScene::all_draw_commands),
                        self.admission_sources
                            .as_ref()
                            .and_then(|s| s.schematic.as_ref()),
                    ),
                };
                PreparedGeometryAdmission {
                    pane_id,
                    surface,
                    viewport,
                    source,
                    retained,
                    commands,
                }
            })
    }
}

#[cfg(test)]
#[path = "geometry_admission_tests.rs"]
mod tests;
