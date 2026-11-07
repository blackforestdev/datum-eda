//! Shared source-qualified pin and wire contacts before background grid snapping.
use super::*;
use eda_engine::substrate::ObjectId as Uuid;

impl Runtime {
    /// Warm the shared binding cache once, including first-use authoring when
    /// no selection has initialized native authority yet. Pointer snap reads it.
    pub(crate) fn prepare_schematic_contact_bindings(
        &mut self,
        pane: datum_gui_protocol::PaneId,
    ) -> Result<(), String> {
        let basis = match self.read_native_binding_basis(SceneSurface::Schematic, Some(pane)) {
            Ok(Some(basis)) => basis,
            _ => self
                .native_binding_basis(SceneSurface::Schematic, Some(pane))
                .map_err(|error| error.to_string())?
                .ok_or("Electrical source authority is unavailable")?,
        };
        self.native_selection
            .as_mut()
            .ok_or("Electrical source authority is unavailable")?
            .binding_bases
            .insert("schematic".into(), basis);
        // Authority setup may invalidate the projection. Retained pin/wire
        // contacts require the current shared preparation, including first use.
        self.prepared_scene()
            .ok_or("Wait for the current schematic frame")?;
        Ok(())
    }
    /// Pin contact selection is shared by placement and wire handle editing.
    /// Candidate ink comes from the current pane; the returned point is the
    /// exact captured electrical anchor, including off-grid pins.
    pub(crate) fn schematic_pin_snap_point(
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
        let document = scene
            .schematic_document
            .as_ref()
            .ok_or("Schematic source is unavailable")?;
        let authoring = &state.schematic_authoring;
        if authoring.placement_sheet() != Some(document.source_sheet)
            || authoring.placement_path() != document.instance_path
        {
            return Err("Finish the electrical gesture in its captured source pane".into());
        }
        let Some(retained) = self.renderer.render_session().schematic_for_pane(pane) else {
            return Ok(None);
        };
        let bindings = self
            .read_native_binding_basis(SceneSurface::Schematic, Some(pane))
            .map_err(|error| error.to_string())?
            .ok_or("Electrical source authority is unavailable")?;
        let source_pin = |hit: &HitTarget| {
            let HitTarget::AuthoredObject(id) = hit else {
                return None;
            };
            let graphic = scene.board_graphics.iter().find(|g| &g.object_id == id)?;
            source_pin_contact(
                authoring,
                bindings.bindings.get(id)?,
                &state.ui.filters,
                &graphic.source_object_uuid,
                graphic.width_nm,
            )
        };
        let hits = retained.authored_overlap_world(world, state, |hit| source_pin(hit).is_some());
        Ok(nearest_pin_contact(
            world,
            hits.iter().filter_map(source_pin),
        ))
    }

    pub(super) fn schematic_snap_point(
        &self,
        world: PointNm,
        screen: (f32, f32),
    ) -> Result<PointNm, String> {
        self.schematic_contact_snap_point(world, screen, None)
    }
    pub(crate) fn schematic_wire_endpoint_snap_point(
        &self,
        world: PointNm,
        screen: (f32, f32),
        source_wire: Uuid,
    ) -> Result<PointNm, String> {
        self.schematic_contact_snap_point(world, screen, Some(source_wire))
    }
    fn schematic_contact_snap_point(
        &self,
        world: PointNm,
        screen: (f32, f32),
        endpoint_source: Option<Uuid>,
    ) -> Result<PointNm, String> {
        let pane = self
            .pane_at_screen(screen.0, screen.1)
            .ok_or("Focus the schematic")?;
        let state = self.workspace();
        let scene = datum_gui_protocol::camera_scene_for_pane(state, pane)
            .ok_or("Schematic source is unavailable")?;
        let authoring = &state.schematic_authoring;
        if let Some(point) = self.schematic_pin_snap_point(world, screen)? {
            return Ok(point);
        }
        let grid = datum_gui_protocol::checked_snap_authoring_point(world, state.authoring.snap)
            .map_err(|error| error.to_string())?;
        let electrical = endpoint_source.is_some()
            || authoring.design_placement.is_some()
            || (authoring.wire && !authoring.bus)
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
        let bindings = self
            .read_native_binding_basis(SceneSurface::Schematic, Some(pane))
            .map_err(|error| error.to_string())?
            .ok_or("Electrical source authority is unavailable")?;
        let hit = retained.hit_test_authored_world_eligible(world, state, |target| {
            let HitTarget::AuthoredObject(id) = target else {
                return false;
            };
            if !bindings
                .bindings
                .get(id)
                .is_some_and(|binding| binding.eligible(&state.ui.filters))
            {
                return false;
            }
            scene
                .board_graphics
                .iter()
                .find(|graphic| &graphic.object_id == id)
                .is_some_and(|graphic| {
                    if endpoint_source
                        .is_some_and(|id| graphic.source_object_uuid == id.to_string())
                    {
                        return false;
                    }
                    !matches!(
                        authoring.wire_point_for_sheet(
                            scene.schematic_document.as_ref().unwrap().source_sheet,
                            &graphic.source_object_uuid,
                            grid
                        ),
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
            .wire_point_for_sheet(
                scene.schematic_document.as_ref().unwrap().source_sheet,
                &graphic.source_object_uuid,
                grid,
            )?
            .ok_or("The captured wire is unavailable")?;
        if !datum_gui_render::schematic_anchor_contains(graphic.width_nm, point, world) {
            return Err("No exact wire contact is available here; choose a wire endpoint".into());
        }
        Ok(point)
    }
}

/// Rank only anchors inside the shared contact envelope. The payload can be
/// a pin point or an editing handle; neither consumer defines another radius.
impl Runtime {
    pub(crate) fn nearest_schematic_contact<T>(
        world: PointNm,
        candidates: impl IntoIterator<Item = (PointNm, Option<i64>, String, T)>,
    ) -> Option<T> {
        let distance = |point: PointNm| {
            let x = (i128::from(point.x) - i128::from(world.x)).unsigned_abs();
            let y = (i128::from(point.y) - i128::from(world.y)).unsigned_abs();
            x.saturating_mul(x).saturating_add(y.saturating_mul(y))
        };
        candidates
            .into_iter()
            .filter(|(point, width, _, _)| {
                datum_gui_render::schematic_anchor_contains(*width, *point, world)
            })
            .min_by(|a, b| {
                distance(a.0)
                    .cmp(&distance(b.0))
                    .then_with(|| a.2.cmp(&b.2))
            })
            .map(|candidate| candidate.3)
    }
}

fn source_pin_contact(
    authoring: &datum_gui_protocol::schematic_authoring::SchematicAuthoring,
    binding: &datum_gui_protocol::selection_scene::SceneSelectionBinding,
    filters: &datum_gui_protocol::WorkspaceFilterState,
    source_uuid: &str,
    width: Option<i64>,
) -> Option<(PointNm, Option<i64>, String)> {
    if !binding.eligible(filters) {
        return None;
    }
    Some((
        authoring.pin_point(source_uuid)?,
        width,
        source_uuid.to_owned(),
    ))
}

fn nearest_pin_contact(
    world: PointNm,
    candidates: impl IntoIterator<Item = (PointNm, Option<i64>, String)>,
) -> Option<PointNm> {
    Runtime::nearest_schematic_contact(
        world,
        candidates
            .into_iter()
            .map(|(point, width, id)| (point, width, id, point)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excluded_source_origin_does_not_attract_snap_and_tool_guards_remain() {
        use datum_gui_protocol::{
            AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity,
            schematic_authoring::{LabelDraft, MarkerKind, SchematicAuthoring},
            selection_scene::SceneSelectionBinding,
        };
        use eda_engine::schematic::{LabelKind, Schematic, Sheet};
        use std::collections::HashMap;
        let id = |n| Uuid::from_u128(n);
        let mut sheet:Sheet=serde_json::from_value(serde_json::json!({"uuid":id(1),"name":"Snap","frame":null,"symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
        let symbol=serde_json::from_value(serde_json::json!({"uuid":id(2),"part":null,"entity":null,"gate":null,"lib_id":null,"reference":"R1","value":"","fields":[],"pins":[{"uuid":id(3),"number":"1","name":"","electrical_type":"Passive","position":{"x":20_000,"y":0}}],"position":{"x":0,"y":0},"rotation":0,"mirrored":false,"unit_selection":null,"display_mode":"LibraryDefault","pin_overrides":[],"hidden_power_behavior":"SourceDefinedImplicit"})).unwrap();
        sheet.symbols.insert(id(2), symbol);
        sheet.labels.insert(
            id(4),
            eda_engine::schematic::NetLabel {
                uuid: id(4),
                kind: eda_engine::schematic::LabelKind::Power,
                name: "VCC".into(),
                position: eda_engine::ir::geometry::Point::new(100_000, 0),
            },
        );
        let mut authoring = SchematicAuthoring::default();
        authoring.sheet = Some(id(1));
        authoring.source = Some(Schematic {
            uuid: id(90),
            sheets: HashMap::from([(id(1), sheet)]),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: vec![],
        });
        let binding =
            |projected_class, projected, acquired_class, acquired| SceneSelectionBinding {
                projected: Identity {
                    class: projected_class,
                    id: id(projected),
                    instance_path: vec![],
                },
                acquired: Identity {
                    class: acquired_class,
                    id: id(acquired),
                    instance_path: vec![],
                },
                layers: vec!["electrical".into()],
            };
        let pin = binding(Class::Pin, 3, Class::Symbol, 2);
        let power = binding(Class::Label, 4, Class::Label, 4);
        let mut filters = datum_gui_protocol::WorkspaceFilterState {
            show_authored: true,
            excluded_selection_classes: Default::default(),
            show_proposed: true,
            show_unrouted: true,
            dim_unrelated: false,
            active_layer_id: None,
            layer_visibility: Default::default(),
            layer_scroll_offset: 0,
        };
        let cursor = PointNm { x: 0, y: 0 };
        let choose = |authoring: &SchematicAuthoring,
                      filters: &datum_gui_protocol::WorkspaceFilterState| {
            nearest_pin_contact(
                cursor,
                [(id(3), &pin), (id(4), &power)]
                    .into_iter()
                    .filter_map(|(id, binding)| {
                        source_pin_contact(authoring, binding, filters, &id.to_string(), None)
                    }),
            )
        };
        assert_eq!(
            choose(&authoring, &filters),
            Some(PointNm { x: 20_000, y: 0 })
        );
        filters.excluded_selection_classes.insert(Class::Symbol);
        assert_eq!(
            choose(&authoring, &filters),
            Some(PointNm { x: 100_000, y: 0 })
        );
        filters.excluded_selection_classes.insert(Class::Label);
        assert_eq!(choose(&authoring, &filters), None);
        filters.excluded_selection_classes.clear();
        authoring.marker = Some(MarkerKind::NoConnect);
        assert!(
            source_pin_contact(&authoring, &power, &filters, &id(4).to_string(), None).is_none()
        );
        assert_eq!(
            choose(&authoring, &filters),
            Some(PointNm { x: 20_000, y: 0 })
        );
        authoring.marker = None;
        let mut label = LabelDraft::new(LabelKind::Local);
        label.text_rotation = Some(0);
        authoring.label = Some(label);
        assert_eq!(choose(&authoring, &filters), None);
    }

    #[test]
    fn nearest_pin_uses_exact_off_grid_contact_and_not_painter_order() {
        let cursor = PointNm {
            x: 1_700_000,
            y: 700_000,
        };
        let near = PointNm {
            x: 1_720_000,
            y: 700_000,
        };
        let far = PointNm {
            x: 1_800_000,
            y: 700_000,
        };
        for candidates in [
            vec![(far, None, "a".into()), (near, None, "z".into())],
            vec![(near, None, "z".into()), (far, None, "a".into())],
        ] {
            assert_eq!(nearest_pin_contact(cursor, candidates), Some(near));
        }
        assert_eq!(
            nearest_pin_contact(
                cursor,
                [(
                    PointNm {
                        x: 2_400_000,
                        y: 700_000
                    },
                    None,
                    "a".into()
                )]
            ),
            None
        );
    }

    #[test]
    fn equal_distance_pin_choice_is_stable_by_source_identity() {
        let cursor = PointNm { x: 0, y: 0 };
        let left = PointNm { x: -10_000, y: 0 };
        let right = PointNm { x: 10_000, y: 0 };
        assert_eq!(
            nearest_pin_contact(
                cursor,
                [(left, None, "z".into()), (right, None, "a".into())]
            ),
            Some(right)
        );
        assert_eq!(
            nearest_pin_contact(
                cursor,
                [(right, None, "a".into()), (left, None, "z".into())]
            ),
            Some(right)
        );
    }
}
