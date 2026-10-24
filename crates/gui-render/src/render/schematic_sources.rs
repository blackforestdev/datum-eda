//! Immutable qualified schematic sources shared by a session and its in-flight frames.
use crate::cpu_alloc::heap;
use crate::{RetainedScene, ReviewWorkspaceState, ShellLayout};
use datum_gui_protocol::{PaneContent, schematic_documents::SchematicDocumentKey};
use std::sync::Arc;

#[derive(Clone, Default)]
pub(crate) struct SchematicSources(Option<Arc<Sources>>);
struct Sources {
    entries: Vec<Entry>,
    _charge: Option<crate::DocumentCpuCharge>,
}
struct Entry {
    document: Option<SchematicDocumentKey>,
    viewport: crate::RectPx,
    retained: RetainedScene,
}
impl std::fmt::Debug for SchematicSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter().map(|(key, viewport, _)| (key, viewport)))
            .finish()
    }
}
impl PartialEq for SchematicSources {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl SchematicSources {
    pub(crate) fn scene(
        &self,
        document: Option<&SchematicDocumentKey>,
        viewport: crate::RectPx,
    ) -> Option<&RetainedScene> {
        self.0
            .as_ref()?
            .entries
            .iter()
            .find(|e| {
                e.document.as_ref() == document
                    && (e.viewport == viewport || e.retained.can_reuse_for_surface_resize())
            })
            .map(|e| &e.retained)
    }
    pub(crate) fn iter(
        &self,
    ) -> impl Iterator<Item = (Option<&SchematicDocumentKey>, crate::RectPx, &RetainedScene)> {
        self.0
            .iter()
            .flat_map(|s| s.entries.iter())
            .map(|e| (e.document.as_ref(), e.viewport, &e.retained))
    }
    pub(crate) fn capture(
        state: &ReviewWorkspaceState,
        primary: Option<&RetainedScene>,
        previous: &Self,
        width: u32,
        height: u32,
        scale: f32,
    ) -> anyhow::Result<Self> {
        let shell = ShellLayout::for_surface(
            width,
            height,
            scale,
            state
                .ui
                .active_dock_tab
                .map(|_| state.ui.effective_dock_height_px()),
        );
        let panes = shell.viewport_panes(&state.ui.layout);
        let count = panes
            .panes
            .iter()
            .filter(|p| {
                p.content == PaneContent::Schematic
                    && datum_gui_protocol::camera_scene_for_pane(state, p.id).is_some()
            })
            .count();
        if count == 0 {
            return Ok(Self::default());
        }
        let primary_scene = state
            .schematic_scene
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Schematic allowance source unavailable"))?;
        let first = panes
            .panes
            .iter()
            .find(|pane| {
                pane.content == PaneContent::Schematic
                    && datum_gui_protocol::camera_scene_for_pane(state, pane.id).is_some()
            })
            .expect("counted visible source");
        let first_scene =
            datum_gui_protocol::camera_scene_for_pane(state, first.id).expect("qualified source");
        let retained_first =
            previous.scene(first_scene.schematic_document.as_ref(), first.rect.scene);
        let constructed_first = if primary.is_none() && retained_first.is_none() {
            Some(RetainedScene::schematic_source_bounded(
                state,
                first_scene,
                first.rect.scene,
                &primary_scene.scene_id,
                crate::retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
            )?)
        } else {
            None
        };
        let anchor = primary
            .or(retained_first)
            .or(constructed_first.as_ref())
            .expect("retained source allowance");
        let anchor_scene = if primary.is_some() {
            primary_scene
        } else {
            first_scene
        };
        let anchor_viewport = if primary.is_some() {
            previous
                .iter()
                .find_map(|(_, viewport, retained)| {
                    retained
                        .world_vertices
                        .ptr_eq(&anchor.world_vertices)
                        .then_some(viewport)
                })
                .or_else(|| shell.schematic_scene_viewport(&state.ui.layout))
        } else {
            Some(first.rect.scene)
        };
        // Charge the complete set container and its keys before copying them.
        // Every retained payload below uses this same existing document budget.
        let mut bytes = container_bytes()
            .checked_add(heap::capacity_bytes::<Entry>(count))
            .ok_or_else(|| anyhow::anyhow!("schematic source set layout overflow"))?;
        for pane in &panes.panes {
            if pane.content != PaneContent::Schematic {
                continue;
            }
            let Some(scene) = datum_gui_protocol::camera_scene_for_pane(state, pane.id) else {
                continue;
            };
            if let Some(key) = &scene.schematic_document {
                bytes = bytes
                    .checked_add(heap::capacity_bytes::<uuid::Uuid>(key.instance_path.len()))
                    .ok_or_else(|| anyhow::anyhow!("schematic occurrence key overflow"))?;
            }
        }
        let charge = anchor
            .geometry_observer()
            .try_charge_document_storage(bytes)
            .ok_or_else(|| anyhow::anyhow!("schematic source set exceeds document CPU budget"))?;
        let mut entries: Vec<Entry> = Vec::new();
        entries.try_reserve_exact(count)?;
        for pane in &panes.panes {
            if pane.content != PaneContent::Schematic {
                continue;
            }
            let Some(scene) = datum_gui_protocol::camera_scene_for_pane(state, pane.id) else {
                continue;
            };
            if entries.iter().any(|e| {
                e.document == scene.schematic_document
                    && (e.viewport == pane.rect.scene || e.retained.can_reuse_for_surface_resize())
            }) {
                continue;
            }
            let old = previous.scene(scene.schematic_document.as_ref(), pane.rect.scene);
            let retained = if let Some(old) = old {
                old.clone()
            } else if scene.schematic_document == anchor_scene.schematic_document
                && (anchor.can_reuse_for_surface_resize()
                    || anchor_viewport == Some(pane.rect.scene))
            {
                anchor.clone()
            } else {
                RetainedScene::schematic_source_bounded(
                    state,
                    scene,
                    pane.rect.scene,
                    &primary_scene.scene_id,
                    crate::retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
                )?
            };
            entries.push(Entry {
                document: scene.schematic_document.clone(),
                viewport: pane.rect.scene,
                retained,
            });
        }
        Ok(Self(Some(Arc::new(Sources {
            entries,
            _charge: Some(charge),
        }))))
    }
}
fn container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        return std::mem::size_of::<Sources>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        heap::arc_bytes(Sources {
            entries: Vec::new(),
            _charge: None,
        })
    })
}

#[cfg(test)]
#[path = "schematic_sources_tests.rs"]
mod tests;
