//! Shared source-qualified pin and wire contacts before background grid snapping.
use super::*;

impl Runtime {
    pub(super) fn schematic_snap_point(
        &self,
        world: PointNm,
        screen: (f32, f32),
    ) -> Result<PointNm, String> {
        let pane = self
            .pane_at_screen(screen.0, screen.1)
            .ok_or("Focus the schematic")?;
        let state = self.workspace();
        let scene = datum_gui_protocol::camera_scene_for_pane(state, pane)
            .ok_or("Schematic source is unavailable")?;
        let authoring = &state.schematic_authoring;
        if let Some(HitTarget::AuthoredObject(id)) =
            self.presented_hits.hit_test(screen.0, screen.1)
            && let Some(graphic) = scene.board_graphics.iter().find(|g| &g.object_id == id)
            && let Some(point) = self
                .workspace()
                .schematic_authoring
                .pin_point(&graphic.source_object_uuid)
        {
            return Ok(point);
        }
        // Authored world ink is retained, not a screen-chrome hit region. Reuse
        // its existing geometry predicate, admitting only complete committed
        // source pins from the gesture's revision-qualified immutable basis.
        if let Some(retained) = self.renderer.render_session().schematic_for_pane(pane)
            && let Some(HitTarget::AuthoredObject(id)) =
                retained.hit_test_authored_world_eligible(world, self.workspace(), |target| {
                    let HitTarget::AuthoredObject(id) = target else {
                        return false;
                    };
                    scene
                        .board_graphics
                        .iter()
                        .find(|g| &g.object_id == id)
                        .and_then(|graphic| {
                            self.workspace()
                                .schematic_authoring
                                .pin_point(&graphic.source_object_uuid)
                        })
                        .is_some()
                })
            && let Some(point) = scene
                .board_graphics
                .iter()
                .find(|g| g.object_id == id)
                .and_then(|graphic| {
                    self.workspace()
                        .schematic_authoring
                        .pin_point(&graphic.source_object_uuid)
                })
        {
            return Ok(point);
        }
        let grid = datum_gui_protocol::checked_snap_authoring_point(world, state.authoring.snap)
            .map_err(|error| error.to_string())?;
        let electrical = (authoring.wire && !authoring.bus)
            || authoring.marker
                == Some(datum_gui_protocol::schematic_authoring::MarkerKind::Junction)
            || authoring
                .label
                .as_ref()
                .is_some_and(|label| label.text_rotation.is_none());
        if !electrical {
            return Ok(grid);
        }
        if scene.schematic_document.as_ref().is_none_or(|document| {
            Some(document.source_sheet) != authoring.placement_sheet()
                || document.instance_path != authoring.placement_path()
        }) {
            return Err("Finish the electrical gesture in its captured source pane".into());
        }
        let retained = self
            .renderer
            .render_session()
            .schematic_for_pane(pane)
            .ok_or("Wait for the current schematic frame")?;
        // Acquire existing visible wire ink before applying its exact source
        // contact. Grid rounding alone can leave an off-grid pin wire and its
        // apparently coincident Label electrically disconnected.
        let hit = retained.hit_test_authored_world_eligible(world, state, |target| {
            let HitTarget::AuthoredObject(id) = target else {
                return false;
            };
            scene
                .board_graphics
                .iter()
                .find(|graphic| &graphic.object_id == id)
                .is_some_and(|graphic| {
                    !matches!(
                        authoring.wire_point(&graphic.source_object_uuid, grid),
                        Ok(None)
                    )
                })
        });
        let Some(HitTarget::AuthoredObject(id)) = hit else {
            return Ok(grid);
        };
        let graphic = scene
            .board_graphics
            .iter()
            .find(|graphic| graphic.object_id == id)
            .ok_or("The captured wire is unavailable")?;
        let point = authoring
            .wire_point(&graphic.source_object_uuid, grid)?
            .ok_or("The captured wire is unavailable")?;
        if !datum_gui_render::schematic_anchor_contains(graphic.width_nm, point, world) {
            return Err("No exact wire contact is available here; choose a wire endpoint".into());
        }
        Ok(point)
    }
}
