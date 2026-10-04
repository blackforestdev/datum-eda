//! Source-bound eligibility at the shared topmost hit query, before choosing a winner.
use super::*;
use datum_gui_protocol::selection_scene::SceneSelectionBindings;
use std::sync::Arc;
pub(super) struct BindingBasis {
    scene_id: String,
    revision: String,
    pub(super) bindings: SceneSelectionBindings,
}
impl Runtime {
    pub(super) fn native_binding_basis(
        &mut self,
        surface: SceneSurface,
    ) -> anyhow::Result<Option<Arc<BindingBasis>>> {
        if !self.ensure_selection_authority()? {
            return Ok(None);
        }
        let native = self.native_selection.as_ref().unwrap();
        let (key, scene) = match surface {
            SceneSurface::Board => ("board", &self.workspace().scene),
            SceneSurface::Schematic => (
                "schematic",
                self.workspace()
                    .schematic_scene
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Schematic source unavailable"))?,
            ),
        };
        if scene.project_uuid != native.model.project.project_id.to_string()
            || scene.source_revision != native.model.model_revision.0
        {
            anyhow::bail!("Selection scene does not match the resolved source revision");
        }
        if let Some(basis) = native.binding_bases.get(key).filter(|basis| {
            basis.scene_id == scene.scene_id && basis.revision == scene.source_revision
        }) {
            return Ok(Some(basis.clone()));
        }
        let basis = Arc::new(BindingBasis {
            scene_id: scene.scene_id.clone(),
            revision: scene.source_revision.clone(),
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
    ) -> Result<Option<HitTarget>, ()> {
        let basis = match self.native_binding_basis(surface) {
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
            SceneSurface::Schematic => self.renderer.render_session().schematic(),
        };
        let Some(retained) = retained else {
            return Ok(None);
        };
        let state = self.workspace();
        Ok(if let Some(basis) = basis {
            retained.hit_test_authored_world_eligible(point, state, |target| {
                let HitTarget::AuthoredObject(key) = target else {
                    return false;
                };
                basis.bindings.get(key).is_some_and(|binding| {
                    !state
                        .ui
                        .filters
                        .excluded_selection_classes
                        .contains(&binding.acquired.class)
                        && binding.visible(|layer| {
                            state
                                .ui
                                .filters
                                .layer_visibility
                                .get(layer)
                                .copied()
                                .unwrap_or(true)
                        })
                })
            })
        } else {
            match surface {
                SceneSurface::Board => retained.hit_test_authored_world(point, state),
                SceneSurface::Schematic => retained.hit_test_world(point),
            }
        })
    }
}
