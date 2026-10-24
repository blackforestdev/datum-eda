//! Source-bound eligibility at the shared topmost hit query, before choosing a winner.
use super::*;
use datum_gui_protocol::selection_scene::SceneSelectionBindings;
use std::sync::Arc;
pub(super) struct BindingBasis {
    scene_id: String,
    revision: String,
    document: Option<datum_gui_protocol::schematic_documents::SchematicDocumentKey>,
    pub(super) bindings: SceneSelectionBindings,
}
pub(super) struct SelectionWorldHit {
    pub target: Option<HitTarget>,
    pub basis: Option<Arc<BindingBasis>>,
}
impl Runtime {
    pub(super) fn native_binding_basis(
        &mut self,
        surface: SceneSurface,
        pane: Option<datum_gui_protocol::PaneId>,
    ) -> anyhow::Result<Option<Arc<BindingBasis>>> {
        if !self.ensure_selection_authority()? {
            return Ok(None);
        }
        if surface == SceneSurface::Schematic
            && pane.is_none_or(|pane| {
                self.workspace().ui.layout.content_for(pane)
                    != Some(datum_gui_protocol::PaneContent::Schematic)
            })
        {
            anyhow::bail!("Selection requires the actual schematic pane");
        }
        let native = self.native_selection.as_ref().unwrap();
        let (key, scene) = match surface {
            SceneSurface::Board => ("board", &self.workspace().scene),
            SceneSurface::Schematic => (
                "schematic",
                datum_gui_protocol::camera_scene_for_pane(
                    self.workspace(),
                    pane.ok_or_else(|| {
                        anyhow::anyhow!("Selection requires the actual schematic pane")
                    })?,
                )
                .ok_or_else(|| anyhow::anyhow!("Qualified schematic source unavailable"))?,
            ),
        };
        if scene.project_uuid != native.model.project.project_id.to_string()
            || scene.source_revision != native.model.model_revision.0
        {
            anyhow::bail!("Selection scene does not match the resolved source revision");
        }
        if let Some(basis) = native.binding_bases.get(key).filter(|basis| {
            basis.scene_id == scene.scene_id
                && basis.revision == scene.source_revision
                && basis.document == scene.schematic_document
        }) {
            return Ok(Some(basis.clone()));
        }
        let basis = Arc::new(BindingBasis {
            scene_id: scene.scene_id.clone(),
            revision: scene.source_revision.clone(),
            document: scene.schematic_document.clone(),
            bindings: SceneSelectionBindings::capture(scene, &native.authority),
        });
        self.native_selection
            .as_mut()
            .unwrap()
            .binding_bases
            .insert(key.into(), basis.clone());
        Ok(Some(basis))
    }
    pub(super) fn selection_world_hit(
        &mut self,
        point: PointNm,
        surface: SceneSurface,
        pane: Option<datum_gui_protocol::PaneId>,
    ) -> Result<SelectionWorldHit, ()> {
        let basis = match self.native_binding_basis(surface, pane) {
            Ok(basis) => basis,
            Err(error) => {
                self.selection_notice(error.to_string());
                return Err(());
            }
        };
        if !self.ensure_retained_scene() {
            return Err(());
        }
        let retained = match surface {
            SceneSurface::Board => self.renderer.render_session().board(),
            SceneSurface::Schematic => {
                pane.and_then(|pane| self.renderer.render_session().schematic_for_pane(pane))
            }
        };
        let Some(retained) = retained else {
            return Ok(SelectionWorldHit {
                target: None,
                basis,
            });
        };
        let state = self.workspace();
        let target = if let Some(basis) = &basis {
            retained.hit_test_authored_world_eligible(point, state, |target| {
                let HitTarget::AuthoredObject(key) = target else {
                    return false;
                };
                basis
                    .bindings
                    .get(key)
                    .is_some_and(|binding| binding.eligible(&state.ui.filters))
            })
        } else {
            match surface {
                SceneSurface::Board => retained.hit_test_authored_world(point, state),
                SceneSurface::Schematic => retained.hit_test_world(point),
            }
        };
        Ok(SelectionWorldHit { target, basis })
    }
}
