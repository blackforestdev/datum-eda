//! Immutable scene ownership and weak CPU lifetime accounting.
use super::gpu_data::shared_geometry::GeometryElement;
use super::*;
use crate::cpu_alloc::heap::capacity_bytes;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct RetainedScene {
    pub(crate) surface_size_independent: bool,
    pub(crate) world_vertices: gpu_data::shared_geometry::SharedGeometry<Vertex>,
    pub(crate) world_strokes: gpu_data::shared_geometry::SharedGeometry<WorldStrokeInstance>,
    pub(crate) draw_commands: Arc<Vec<RetainedDrawCommand>>,
    pub(crate) world_hit_index: Arc<datum_gui_viewport::SpatialHitIndex<HitTarget>>,
}
/// Non-owning retained CPU scene lifetime observation. Weak references retain only
/// the small shared owner, never the separately boxed vertex/stroke payload.
#[derive(Clone)]
pub struct RetainedGeometryObserver {
    vertices: std::sync::Weak<Box<[Vertex]>>,
    strokes: std::sync::Weak<Box<[WorldStrokeInstance]>>,
    vertex_bytes: usize,
    stroke_bytes: usize,
    commands: std::sync::Weak<Vec<RetainedDrawCommand>>,
    hits: std::sync::Weak<datum_gui_viewport::SpatialHitIndex<HitTarget>>,
    metadata_bytes: [usize; 2],
}

impl RetainedGeometryObserver {
    pub fn is_live(&self) -> bool {
        self.vertices.strong_count() != 0
            || self.strokes.strong_count() != 0
            || self.commands.strong_count() != 0
            || self.hits.strong_count() != 0
    }

    /// Deduplicate each allocation independently. Measured Arc containers and
    /// allocator headers remain charged until weak observers also drop.
    pub fn heap_bytes_excluding<'a>(&self, others: impl IntoIterator<Item = &'a Self>) -> usize {
        let mut vertices = Vertex::container_bytes()
            + usize::from(self.vertices.strong_count() != 0) * self.vertex_bytes;
        let mut strokes = WorldStrokeInstance::container_bytes()
            + usize::from(self.strokes.strong_count() != 0) * self.stroke_bytes;
        let mut commands =
            commands_container_bytes().saturating_add(if self.commands.strong_count() != 0 {
                self.metadata_bytes[0]
            } else {
                0
            });
        let mut hits = hits_container_bytes().saturating_add(if self.hits.strong_count() != 0 {
            self.metadata_bytes[1]
        } else {
            0
        });
        for other in others {
            if self.vertices.ptr_eq(&other.vertices) {
                vertices = 0;
            }
            if self.strokes.ptr_eq(&other.strokes) {
                strokes = 0;
            }
            if self.commands.ptr_eq(&other.commands) {
                commands = 0;
            }
            if self.hits.ptr_eq(&other.hits) {
                hits = 0;
            }
        }
        vertices
            .saturating_add(strokes)
            .saturating_add(commands)
            .saturating_add(hits)
    }
}

impl RetainedScene {
    /// All live and submitted-retiring GPU world buffers for this scene/document,
    /// including other retained revisions and renderers sharing its scene ID.
    pub fn world_gpu_reserved_bytes(&self) -> u64 {
        self.world_vertices
            .document_budget()
            .map_or(0, |budget| budget.used())
    }

    pub fn geometry_observer(&self) -> RetainedGeometryObserver {
        RetainedGeometryObserver {
            vertices: self.world_vertices.downgrade(),
            strokes: self.world_strokes.downgrade(),
            vertex_bytes: capacity_bytes::<Vertex>(self.world_vertices.len()),
            stroke_bytes: capacity_bytes::<WorldStrokeInstance>(self.world_strokes.len()),
            commands: Arc::downgrade(&self.draw_commands),
            hits: Arc::downgrade(&self.world_hit_index),
            metadata_bytes: [
                self.command_bytes().unwrap_or(usize::MAX),
                self.hit_bytes().unwrap_or(usize::MAX),
            ],
        }
    }
}

impl RetainedScene {
    fn command_bytes(&self) -> Option<usize> {
        let mut bytes = capacity_bytes::<RetainedDrawCommand>(self.draw_commands.capacity());
        for command in self.draw_commands.iter() {
            let layer = match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id,
            };
            bytes = bytes.checked_add(capacity_bytes::<u8>(
                layer.as_ref().map_or(0, String::capacity),
            ))?;
        }
        Some(bytes)
    }
    fn hit_bytes(&self) -> Option<usize> {
        self.world_hit_index.heap_bytes_with(
            |target| match target {
                HitTarget::AuthoredObject(id) => Some(capacity_bytes::<u8>(id.capacity())),
                _ => None,
            },
            |layout| Some(crate::cpu_alloc::heap::allocation_bytes(layout)),
        )
    }
    /// Owned capacities and measured Arc/Datum headers; shared allocations are
    /// deduplicated by the observer. Excludes inline handles and document registry.
    pub fn heap_payload_bytes(&self) -> Option<usize> {
        self.world_vertices
            .heap_bytes()
            .checked_add(self.world_strokes.heap_bytes())?
            .checked_add(self.command_bytes()?)?
            .checked_add(self.hit_bytes()?)?
            .checked_add(commands_container_bytes())?
            .checked_add(hits_container_bytes())
    }
}
fn commands_container_bytes() -> usize {
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| crate::cpu_alloc::heap::arc_bytes(Vec::<RetainedDrawCommand>::new()))
}
fn hits_container_bytes() -> usize {
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        crate::cpu_alloc::heap::arc_bytes(datum_gui_viewport::SpatialHitIndex::<HitTarget>::new(
            Vec::new(),
        ))
    })
}
