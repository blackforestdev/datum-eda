//! Exact junction anchors acquired through the shared visible-ink hit query.
use super::*;
use datum_gui_protocol::schematic_authoring::MarkerKind;
use eda_engine::connectivity::exact_segment_intersection;
use std::collections::{BTreeMap, BTreeSet};

impl Runtime {
    pub(super) fn schematic_command_point(
        &mut self,
        world: PointNm,
        screen: (f32, f32),
    ) -> Option<PointNm> {
        if self.workspace().schematic_authoring.marker != Some(MarkerKind::Junction) {
            return self.schematic_snap_point(world, screen);
        }
        match self.junction_anchor(world, screen) {
            Ok(anchor) => {
                self.session.workspace_mut().schematic_authoring.refusal = None;
                anchor.or_else(|| self.schematic_snap_point(world, screen))
            }
            Err(reason) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.refusal = Some(reason);
                e.position = None;
                e.preview_dirty = true;
                self.invalidate_frame();
                None
            }
        }
    }

    fn junction_anchor(
        &self,
        world: PointNm,
        screen: (f32, f32),
    ) -> Result<Option<PointNm>, String> {
        let pane = self
            .pane_at_screen(screen.0, screen.1)
            .ok_or("Focus the schematic")?;
        let state = self.workspace();
        let scene = datum_gui_protocol::camera_scene_for_pane(state, pane)
            .ok_or("Schematic source is unavailable")?;
        let authoring = &state.schematic_authoring;
        let sheet = authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&authoring.sheet?))
            .ok_or("Schematic source is unavailable")?;
        let retained = self
            .renderer
            .render_session()
            .schematic_for_pane(pane)
            .ok_or("Wait for the current schematic frame")?;
        let source_wire = |target: &HitTarget| {
            let HitTarget::AuthoredObject(id) = target else {
                return None;
            };
            let graphic = scene.board_graphics.iter().find(|g| &g.object_id == id)?;
            let uuid = graphic
                .source_object_uuid
                .parse::<eda_engine::substrate::ObjectId>()
                .ok()?;
            let wire = sheet.wires.get(&uuid)?;
            Some((wire, graphic.width_nm))
        };
        let mut wires = BTreeMap::new();
        for hit in retained.authored_overlap_world(world, state, |hit| source_wire(hit).is_some()) {
            if let Some((wire, width)) = source_wire(&hit) {
                wires.insert(wire.uuid, (wire, width));
            }
        }
        let wires: Vec<_> = wires.into_values().collect();
        let nearby = |point: eda_engine::ir::geometry::Point, width| {
            datum_gui_render::schematic_anchor_contains(
                width,
                PointNm {
                    x: point.x,
                    y: point.y,
                },
                world,
            )
        };
        let mut anchors = BTreeSet::new();
        for (index, (wire, width)) in wires.iter().enumerate() {
            for point in [wire.from, wire.to] {
                if nearby(point, *width) {
                    anchors.insert((point.x, point.y));
                }
            }
            for (other, other_width) in &wires[index + 1..] {
                if let Some(point) =
                    exact_segment_intersection(wire.from, wire.to, other.from, other.to)
                        .map_err(|e| format!("Cannot acquire this junction: {e}"))?
                    && nearby(point, *width)
                    && nearby(point, *other_width)
                {
                    anchors.insert((point.x, point.y));
                }
            }
        }
        match anchors.len() {
            0 if wires.len() > 1 => Err("Point at the wire crossing to place a junction".into()),
            0 => Ok(None),
            1 => {
                let (x, y) = *anchors.first().unwrap();
                Ok(Some(PointNm { x, y }))
            }
            _ => Err("Several wire contacts are nearby; zoom in to choose one".into()),
        }
    }
}
