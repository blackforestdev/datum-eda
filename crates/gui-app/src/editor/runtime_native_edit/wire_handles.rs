//! Wire handles configure the existing qualified Move and contact services.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Identity};
use eda_engine::api::native_write::{
    schematic_wire_edit::{WireEndpoint, WireHandle, available_handles},
    schematic_wire_edit_adapter::WireEditParams,
};
use eda_engine::substrate::ObjectId as Uuid;
use eda_engine::{ir::geometry::Point, schematic::Sheet};

fn view_point(point: Point) -> PointNm {
    PointNm {
        x: point.x,
        y: point.y,
    }
}

impl Runtime {
    fn wire_handle_sheet<'a>(
        &'a self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
    ) -> Result<&'a Sheet> {
        let document = scene
            .schematic_document
            .as_ref()
            .context("Choose a schematic source")?;
        anyhow::ensure!(
            document.instance_path == id.instance_path,
            "Wire occurrence changed"
        );
        self.workspace()
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|source| source.sheets.get(&document.source_sheet))
            .context("Wire source is unavailable")
    }

    fn wire_cached_source<'a>(
        &'a self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
    ) -> Result<&'a serde_json::Value> {
        let document = scene
            .schematic_document
            .as_ref()
            .context("Choose a schematic source")?;
        let native = self
            .native_selection
            .as_ref()
            .context("Wire source authority is unavailable")?;
        anyhow::ensure!(
            native.model.project.project_id.to_string() == scene.project_uuid
                && native.model.model_revision.0 == scene.source_revision,
            "Wire source revision changed"
        );
        let object = native
            .model
            .objects
            .get(&id.id)
            .context("Wire source object is unavailable")?;
        let raw = native
            .authority
            .snapshot()
            .source_shard_value(&native.model.model_revision, object.source_shard_id)
            .map_err(|error| anyhow::anyhow!("{error:?}"))?;
        anyhow::ensure!(
            raw.get("uuid") == Some(&serde_json::json!(document.source_sheet)),
            "Wire source sheet changed"
        );
        Ok(raw)
    }

    fn wire_primary_unlocked(
        &self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
    ) -> Result<()> {
        let class = match id.class {
            Class::Wire => "wires",
            Class::Junction => "junctions",
            _ => return Ok(()),
        };
        let raw = self.wire_cached_source(scene, id)?;
        let member = raw
            .get(class)
            .and_then(|members| members.get(id.id.to_string()))
            .context("Wire source member is unavailable")?;
        anyhow::ensure!(
            member.get("locked").and_then(serde_json::Value::as_bool) != Some(true),
            "Selected wire or junction is locked"
        );
        Ok(())
    }

    pub(crate) fn wire_locked_source_members(
        &self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
    ) -> Result<std::collections::BTreeSet<Uuid>> {
        let raw = self.wire_cached_source(scene, id)?;
        let mut locked = std::collections::BTreeSet::new();
        for class in ["wires", "junctions"] {
            if let Some(members) = raw.get(class).and_then(serde_json::Value::as_object) {
                for (id, member) in members {
                    if member.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
                        locked.insert(id.parse().context("Locked source identity is invalid")?);
                    }
                }
            }
        }
        Ok(locked)
    }

    pub(crate) fn wire_pickup_handle(
        &self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
        world: PointNm,
        allow_segment: bool,
    ) -> Result<WireHandle> {
        let sheet = self.wire_handle_sheet(scene, id)?;
        self.wire_primary_unlocked(scene, id)?;
        let width = scene
            .board_graphics
            .iter()
            .find(|graphic| graphic.source_object_uuid == id.id.to_string())
            .context("Wire ink is unavailable")?
            .width_nm;
        source_wire_pickup(
            &self.workspace().schematic_authoring,
            sheet,
            id.id,
            width,
            world,
            allow_segment,
        )
    }

    /// Explicit reconnect intent chooses a physical end even when a shared
    /// node occupies the same point. Ordinary drag retains node precedence.
    pub(crate) fn wire_reconnect_handle(
        &self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
        world: PointNm,
    ) -> Result<WireHandle> {
        let sheet = self.wire_handle_sheet(scene, id)?;
        self.wire_primary_unlocked(scene, id)?;
        let wire = sheet.wires.get(&id.id).context("Wire source unavailable")?;
        let width = scene
            .board_graphics
            .iter()
            .find(|g| g.source_object_uuid == id.id.to_string())
            .context("Wire ink is unavailable")?
            .width_nm;
        nearest_wire_end(wire, width, world)
            .context("Point at an end of the selected wire to reconnect it")
    }

    /// Reuse the captured hit/binding basis. An endpoint on a pin can belong
    /// to its explicitly selected wire; an unambiguous sole wire is also safe.
    /// A near-dot wire hit resolves to the real junction when eligible.
    pub(crate) fn wire_handle_pickup_key(
        &self,
        key: String,
        basis: &crate::runtime_selection_hit::BindingBasis,
        world: PointNm,
        pane: datum_gui_protocol::PaneId,
    ) -> String {
        let Some(scene) = datum_gui_protocol::camera_scene_for_pane(self.workspace(), pane) else {
            return key;
        };
        let Some(binding) = basis.bindings.get(&key) else {
            return key;
        };
        let selected = match &self.workspace().selection {
            datum_gui_protocol::SelectionTarget::Native(view) if !view.stale => match &view.subject
            {
                datum_gui_protocol::SelectionSubject::Object(id) if id.class == Class::Wire => {
                    Some(id)
                }
                _ => None,
            },
            _ => None,
        };
        let mut key = key;
        if binding.projected.class == Class::Pin {
            let Some(retained) = self.renderer.render_session().schematic_for_pane(pane) else {
                return key;
            };
            let candidates = retained.authored_overlap_world(world, self.workspace(), |hit| {
                let HitTarget::AuthoredObject(candidate) = hit else {
                    return false;
                };
                basis.bindings.get(candidate).is_some_and(|entry| {
                    entry.acquired.class == Class::Wire
                        && entry.eligible(&self.workspace().ui.filters)
                })
            });
            let candidates: Vec<_> = candidates
                .into_iter()
                .filter_map(|hit| {
                    let HitTarget::AuthoredObject(candidate) = hit else {
                        return None;
                    };
                    let entry = basis.bindings.get(&candidate)?;
                    self.wire_pickup_handle(scene, &entry.acquired, world, false)
                        .ok()?;
                    Some((candidate, &entry.acquired))
                })
                .collect();
            if let Some((candidate, _)) = candidates.iter().find(|(_, id)| selected == Some(*id)) {
                key = candidate.clone();
            } else if candidates.len() == 1 {
                key = candidates[0].0.clone();
            }
        }
        let Some(binding) = basis.bindings.get(&key) else {
            return key;
        };
        if binding.acquired.class == Class::Wire {
            if let Ok(WireHandle::Vertex { point }) =
                self.wire_pickup_handle(scene, &binding.acquired, world, false)
            {
                if let Ok(sheet) = self.wire_handle_sheet(scene, &binding.acquired) {
                    if let Some(junction) = sheet
                        .junctions
                        .values()
                        .find(|junction| junction.position == point)
                    {
                        if let Some((junction_key, _)) = basis.bindings.iter().find(|(_, entry)| {
                            entry.acquired.class == Class::Junction
                                && entry.acquired.id == junction.uuid
                                && entry.acquired.instance_path == binding.acquired.instance_path
                                && entry.eligible(&self.workspace().ui.filters)
                        }) {
                            return junction_key.to_owned();
                        }
                    }
                }
            }
        }
        key
    }

    pub(crate) fn wire_edit_configuration(
        &self,
        scene: &datum_gui_protocol::BoardReviewSceneV1,
        id: &Identity,
        world: PointNm,
    ) -> Result<Option<WireEditParams>> {
        if !matches!(id.class, Class::Wire | Class::Junction) {
            return Ok(None);
        }
        let sheet = self.wire_handle_sheet(scene, id)?;
        self.wire_primary_unlocked(scene, id)?;
        let handle = if id.class == Class::Wire {
            self.wire_pickup_handle(scene, id, world, true)?
        } else {
            let junction = sheet
                .junctions
                .get(&id.id)
                .context("Junction source is unavailable")?;
            let mut incident = false;
            for wire in sheet.wires.values() {
                incident |= eda_engine::connectivity::exact_segment_intersection(
                    junction.position,
                    junction.position,
                    wire.from,
                    wire.to,
                )
                .map_err(anyhow::Error::msg)?
                .is_some();
            }
            if !incident {
                return Ok(None);
            }
            WireHandle::Vertex {
                point: junction.position,
            }
        };
        Ok(Some(WireEditParams {
            action: eda_engine::api::native_write::schematic_wire_edit::WireEditAction::Move,
            handle,
            routing: self.workspace().schematic_authoring.wire_route.routing,
            edit_seed: Uuid::new_v4(),
        }))
    }

    /// Preview and release share the same exact electrical contact result.
    pub(crate) fn native_move_destination(
        &self,
        preview: &NativeMovePreview,
        world: PointNm,
    ) -> Result<PointNm> {
        let snap = self.workspace().authoring.snap;
        if preview.uses_electrical_pin_snap() {
            let screen = self
                .last_cursor_pos
                .context("Choose the endpoint destination in its source pane")?;
            let identity = preview.identity()?;
            let point = self
                .schematic_wire_endpoint_snap_point(world, screen, identity.id)
                .map_err(anyhow::Error::msg)?;
            return preview.destination(
                point,
                datum_gui_protocol::AuthoringSnapState {
                    enabled: false,
                    ..snap
                },
            );
        }
        preview.destination(world, snap)
    }
}

fn nearest_wire_end(
    wire: &eda_engine::schematic::SchematicWire,
    width: Option<i64>,
    world: PointNm,
) -> Option<WireHandle> {
    Runtime::nearest_schematic_contact(
        world,
        [
            (wire.from, WireEndpoint::From, "from"),
            (wire.to, WireEndpoint::To, "to"),
        ]
        .into_iter()
        .map(|(point, end, key)| {
            (
                view_point(point),
                width,
                key.to_owned(),
                WireHandle::Endpoint { end },
            )
        }),
    )
}

fn source_wire_pickup(
    authoring: &datum_gui_protocol::schematic_authoring::SchematicAuthoring,
    sheet: &Sheet,
    wire_id: Uuid,
    width_nm: Option<i64>,
    world: PointNm,
    allow_segment: bool,
) -> Result<WireHandle> {
    let wire = sheet
        .wires
        .get(&wire_id)
        .context("Wire source is unavailable")?;
    let handles = available_handles(sheet, wire_id)?;
    let anchors = handles.iter().filter_map(|handle| {
        let point = match handle {
            WireHandle::Endpoint {
                end: WireEndpoint::From,
            } => wire.from,
            WireHandle::Endpoint {
                end: WireEndpoint::To,
            } => wire.to,
            WireHandle::Vertex { point } => *point,
            _ => return None,
        };
        // A real branch node wins over its overlapping wire endpoint.
        let priority = match handle {
            WireHandle::Vertex { .. } => 0,
            _ => 1,
        };
        Some((
            view_point(point),
            width_nm,
            format!("{priority}:{handle:?}"),
            *handle,
        ))
    });
    if let Some(handle) = Runtime::nearest_schematic_contact(world, anchors) {
        return Ok(handle);
    }
    anyhow::ensure!(
        allow_segment,
        "Select the wire before dragging a segment; grab an endpoint to reconnect"
    );
    let point = authoring
        .wire_point_for_sheet(sheet.uuid, &wire_id.to_string(), world)
        .map_err(anyhow::Error::msg)?
        .context("Wire contact is unavailable")?;
    anyhow::ensure!(
        datum_gui_render::schematic_anchor_contains(width_nm, point, world),
        "Choose a visible wire segment"
    );
    let point = Point::new(point.x, point.y);
    handles
        .into_iter()
        .find(|handle| match handle {
            WireHandle::Segment { from, to } => {
                eda_engine::connectivity::exact_segment_intersection(point, point, *from, *to)
                    .is_ok_and(|intersection| intersection.is_some())
            }
            _ => false,
        })
        .context("Choose a wire segment between its contact nodes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::schematic_authoring::SchematicAuthoring;
    use eda_engine::schematic::{Junction, Schematic, SchematicWire};
    use std::collections::HashMap;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }
    fn owner_geometry() -> Sheet {
        let mut sheet: Sheet = serde_json::from_value(serde_json::json!({"uuid":id(1),"name":"Wire handles","frame":null,"symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
        sheet.wires.insert(
            id(2),
            SchematicWire {
                uuid: id(2),
                from: Point::new(-2_400_000, -3_290_000),
                to: Point::new(-2_400_000, 2_690_000),
            },
        );
        sheet.wires.insert(
            id(3),
            SchematicWire {
                uuid: id(3),
                from: Point::new(1_720_000, 700_000),
                to: Point::new(-2_400_000, 1_200_000),
            },
        );
        sheet.junctions.insert(
            id(4),
            Junction {
                uuid: id(4),
                position: Point::new(-2_400_000, 1_200_000),
            },
        );
        sheet
    }
    fn authoring(sheet: &Sheet) -> SchematicAuthoring {
        let mut authoring = SchematicAuthoring::default();
        authoring.sheet = Some(sheet.uuid);
        authoring.source = Some(Schematic {
            uuid: id(90),
            sheets: HashMap::from([(sheet.uuid, sheet.clone())]),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: vec![],
        });
        authoring
    }
    #[test]
    fn near_dot_pickup_is_a_vertex_and_unselected_body_is_not_a_rigid_move() {
        let sheet = owner_geometry();
        let authoring = authoring(&sheet);
        let before = sheet.clone();
        assert_eq!(
            source_wire_pickup(
                &authoring,
                &sheet,
                id(2),
                None,
                PointNm {
                    x: -2_300_000,
                    y: 1_200_000
                },
                false
            )
            .unwrap(),
            WireHandle::Vertex {
                point: Point::new(-2_400_000, 1_200_000)
            }
        );
        let body = PointNm {
            x: -2_400_000,
            y: 0,
        };
        assert!(source_wire_pickup(&authoring, &sheet, id(2), None, body, false).is_err());
        assert_eq!(
            source_wire_pickup(&authoring, &sheet, id(2), None, body, true).unwrap(),
            WireHandle::Segment {
                from: Point::new(-2_400_000, -3_290_000),
                to: Point::new(-2_400_000, 1_200_000)
            }
        );
        assert_eq!(
            source_wire_pickup(
                &authoring,
                &sheet,
                id(2),
                None,
                PointNm {
                    x: -2_390_000,
                    y: -3_290_000
                },
                false
            )
            .unwrap(),
            WireHandle::Endpoint {
                end: WireEndpoint::From
            }
        );
        assert_eq!(sheet, before);
        assert_eq!(authoring.source.unwrap().sheets[&sheet.uuid], before);
    }
    #[test]
    fn explicit_reconnect_can_choose_shared_end_while_drag_keeps_vertex_priority() {
        let sheet = owner_geometry();
        let editor = authoring(&sheet);
        let node = PointNm {
            x: -2_400_000,
            y: 1_200_000,
        };
        assert!(matches!(
            source_wire_pickup(&editor, &sheet, id(3), None, node, false).unwrap(),
            WireHandle::Vertex { .. }
        ));
        assert_eq!(
            nearest_wire_end(&sheet.wires[&id(3)], None, node),
            Some(WireHandle::Endpoint {
                end: WireEndpoint::To
            })
        );
        assert!(
            nearest_wire_end(&sheet.wires[&id(3)], None, PointNm { x: 0, y: 950_000 }).is_none()
        );
    }

    #[test]
    fn segmented_and_unsplit_junctions_have_same_pickup_and_explicit_sheet_beats_primary() {
        let mut sheet = owner_geometry();
        let original = authoring(&sheet);
        let near = PointNm {
            x: -2_300_000,
            y: 1_200_000,
        };
        let expected = source_wire_pickup(&original, &sheet, id(2), None, near, false).unwrap();
        sheet.wires.get_mut(&id(2)).unwrap().to = Point::new(-2_400_000, 1_200_000);
        sheet.wires.insert(
            id(5),
            SchematicWire {
                uuid: id(5),
                from: Point::new(-2_400_000, 1_200_000),
                to: Point::new(-2_400_000, 2_690_000),
            },
        );
        let mut editor = authoring(&sheet);
        editor.sheet = Some(id(99));
        assert_eq!(
            source_wire_pickup(&editor, &sheet, id(2), None, near, false).unwrap(),
            expected
        );
        assert!(matches!(
            source_wire_pickup(
                &editor,
                &sheet,
                id(2),
                None,
                PointNm {
                    x: -2_400_000,
                    y: 0
                },
                true
            )
            .unwrap(),
            WireHandle::Segment { .. }
        ));
    }
    #[test]
    #[ignore = "requires DATUM_JUNCTION_REPRO_ROOT frozen real owner project; read-only CPU"]
    fn owner_unsplit_wire_pickup_preserves_junction_and_refuses_unselected_body() {
        let root = std::path::PathBuf::from(std::env::var_os("DATUM_JUNCTION_REPRO_ROOT").unwrap());
        let model = eda_engine::substrate::ProjectResolver::new(root)
            .resolve()
            .unwrap();
        let before: Vec<_> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let editor = SchematicAuthoring::from_model(&model).unwrap();
        let sheet = &editor.source.as_ref().unwrap().sheets
            [&Uuid::parse_str("ef48f5e2-eec6-4345-bc2b-0f3fffcb8c28").unwrap()];
        let wire = Uuid::parse_str("f067fa70-4134-49d2-8f64-ef0bde5137f9").unwrap();
        assert_eq!(
            source_wire_pickup(
                &editor,
                sheet,
                wire,
                None,
                PointNm {
                    x: -2_300_000,
                    y: 1_200_000
                },
                false
            )
            .unwrap(),
            WireHandle::Vertex {
                point: Point::new(-2_400_000, 1_200_000)
            }
        );
        assert!(
            source_wire_pickup(
                &editor,
                sheet,
                wire,
                None,
                PointNm {
                    x: -2_400_000,
                    y: 0
                },
                false
            )
            .is_err()
        );
        assert_eq!(
            source_wire_pickup(
                &editor,
                sheet,
                wire,
                None,
                PointNm {
                    x: -2_400_000,
                    y: -3_290_000
                },
                false
            )
            .unwrap(),
            WireHandle::Endpoint {
                end: WireEndpoint::From
            }
        );
        for (path, bytes) in before {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
}
