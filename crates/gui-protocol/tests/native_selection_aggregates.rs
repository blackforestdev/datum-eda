#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection, PointNm,
    SelectionSubject,
    selection_resolution::NativeSelectionResolution,
    selection_view::{InspectionValue, build_selection_view},
};
use eda_engine::{ir::geometry::Point, substrate::Operation};
use fixture::{Fixture, id};
fn tracks() -> (Fixture, datum_gui_protocol::BoardReviewSceneV1, Vec<Id>) {
    let mut f = Fixture::new("projection-aggregates");
    let tracks = [
        f.line(id(1), Point::new(0, 0), Point::new(10, 0)),
        f.line(id(2), Point::new(10, 0), Point::new(20, 0)),
    ];
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.source_revision = f.model.model_revision.0.clone();
    scene.board_uuid = f.board.to_string();
    scene.components.clear();
    scene.pads.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.outline.clear();
    scene.board_graphics.clear();
    scene.board_texts.clear();
    scene.board_text_geometries.clear();
    let template = scene.tracks[0].clone();
    scene.tracks = tracks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut p = template.clone();
            p.object_id = format!("source-{i}");
            p.source_object_uuid = t.uuid.to_string();
            p.track_uuid = t.uuid.to_string();
            p.layer_id = "L0".into();
            p.width_nm = 3;
            p.path = vec![
                PointNm {
                    x: t.from.x,
                    y: t.from.y,
                },
                PointNm {
                    x: t.to.x,
                    y: t.to.y,
                },
            ];
            p
        })
        .collect();
    let ids = tracks
        .iter()
        .map(|t| Id {
            class: Class::Track,
            id: t.uuid,
            instance_path: vec![],
        })
        .collect();
    (f, scene, ids)
}
#[test]
fn complete_hidden_projection_bounds_are_exact_and_not_qualification_anchors() {
    let (f, scene, ids) = tracks();
    let before = f.source_bytes();
    let a = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap());
    let mut view = build_selection_view(&f.model, &a, subject, &scene, None, |_| false).unwrap();
    assert_eq!(view.hidden.len(), 2);
    let InspectionValue::Common(bounds) = view.field("combined_bounds_nm") else {
        panic!("complete projection");
    };
    let b = &bounds[format!("board:{}", f.board)];
    assert_eq!(
        b["min_x"],
        serde_json::json!({"numerator":"-3","denominator":"2"})
    );
    assert_eq!(
        b["max_x"],
        serde_json::json!({"numerator":"43","denominator":"2"})
    );
    assert_eq!(
        b["max_y"],
        serde_json::json!({"numerator":"3","denominator":"2"})
    );
    let original = view.members.clone();
    view.scope = Some(Class::Track);
    assert_eq!(
        view.field("combined_bounds_nm"),
        InspectionValue::Common(bounds)
    );
    assert_eq!(view.members, original);
    let InspectionValue::Common(layers) = view.field("layer_coverage") else {
        panic!("complete layers");
    };
    assert_eq!(
        layers[format!("board:{}:L0", f.board)]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn absent_member_projection_and_pending_logical_correspondence_are_disclosed() {
    let (f, mut scene, ids) = tracks();
    let before = f.source_bytes();
    scene.tracks.pop();
    let a = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap());
    let view = build_selection_view(&f.model, &a, subject, &scene, None, |_| true).unwrap();
    assert_eq!(
        view.unprojected,
        std::collections::BTreeSet::from([ids[1].clone()])
    );
    assert!(
        matches!(view.field("combined_bounds_nm"),InspectionValue::Unavailable(reason) if reason.contains(&ids[1].id.to_string()))
    );
    assert!(matches!(
        view.field("layer_coverage"),
        InspectionValue::Unavailable(_)
    ));
    assert!(matches!(
        view.field("projected_bounds_nm"),
        InspectionValue::Common(_)
    ));
    assert!(
        matches!(view.field("net_coverage"),InspectionValue::Unavailable(reason) if reason.contains("UnavailableAssignment"))
    );
    assert_eq!(
        view.field("resolved_net_coverage"),
        InspectionValue::Common(serde_json::json!({}))
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn board_and_schematic_frames_are_never_silently_unioned() {
    let (mut f, mut scene, mut ids) = tracks();
    f.commit([Operation::CreateSchematicJunction {
        sheet_id: f.sheet,
        junction_id: id(3),
        junction: serde_json::json!({"uuid":id(3),"position":{"x":100,"y":100}}),
    }]);
    let before = f.source_bytes();
    ids.push(Id {
        class: Class::Junction,
        id: id(3),
        instance_path: vec![],
    });
    let a = NativeSelectionResolution::capture(&f.model).unwrap();
    scene.source_revision = f.model.model_revision.0.clone();
    scene.scene_id = "schematic-misleading-name".into();
    let mut schematic = scene.clone();
    schematic.scene_id = "board-misleading-name".into();
    schematic.board_uuid = f.sheet.to_string();
    schematic.tracks.clear();
    schematic
        .board_graphics
        .push(datum_gui_protocol::BoardGraphicPrimitive {
            object_id: "junction-display".into(),
            object_kind: "schematic_junction".into(),
            primitive_kind: "junction".into(),
            source_object_uuid: id(3).to_string(),
            layer_id: "schematic".into(),
            path: vec![PointNm { x: 100, y: 100 }],
            holes: vec![],
            width_nm: Some(2),
        });
    let subject = SelectionSubject::Compound(CompoundSelection::new(ids, None).unwrap());
    let view =
        build_selection_view(&f.model, &a, subject, &scene, Some(&schematic), |_| true).unwrap();
    let InspectionValue::Common(bounds) = view.field("combined_bounds_nm") else {
        panic!("separate frame bounds");
    };
    assert_eq!(bounds.as_object().unwrap().len(), 2);
    assert_eq!(
        bounds[format!("schematic:{}", f.sheet)]["min_x"]["numerator"],
        "99"
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn run_discloses_assignment_and_contact_evidence_without_widening_membership() {
    let (f, scene, ids) = tracks();
    let before = f.source_bytes();
    let a = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = a.acquire_run(ids[0].clone(), None).unwrap();
    let view = build_selection_view(&f.model, &a, subject, &scene, None, |_| true).unwrap();
    assert_eq!(view.members, ids.into_iter().collect());
    assert_eq!(
        view.field("board_assignment_id"),
        InspectionValue::Common(serde_json::json!(f.net))
    );
    assert_eq!(
        view.field("foreign_net_contacts"),
        InspectionValue::Common(serde_json::json!([]))
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn logical_coverage_uses_canonical_binding_and_survives_reopen_without_selection_writes() {
    use eda_engine::{
        api::native_write::{self, electrical},
        substrate::{
            ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence, NetAnchorReason,
            NetRelationshipIntent, ObjectRevision, RevisionedRef,
        },
    };
    let (mut f, scene, ids) = tracks();
    for record in [
        ElectricalIdentityRecord {
            id: id(800),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::Net {
                anchor: ElectricalOccurrence {
                    class: "tracks".into(),
                    source_id: ids[0].id,
                    instance_path: vec![],
                },
                anchor_reason: NetAnchorReason::Authored,
                retired: false,
                predecessors: Default::default(),
            },
        },
        ElectricalIdentityRecord {
            id: id(801),
            object_revision: ObjectRevision(0),
            identity: ElectricalIdentity::NetRelationship {
                logical_net: id(800),
                board_net: Some(RevisionedRef {
                    object_id: f.net,
                    object_revision: f.model.objects[&f.net].object_revision,
                }),
                intent: NetRelationshipIntent::BoardOnly,
                evidence: vec![],
            },
        },
    ] {
        let write =
            electrical::build_create_electrical_identity(&f.model, fixture::provenance(), record)
                .unwrap();
        native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let before = f.source_bytes();
    let subject = SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap());
    for model in [&f.model, &f.reopen()] {
        let a = NativeSelectionResolution::capture(model).unwrap();
        let view =
            build_selection_view(model, &a, subject.clone(), &scene, None, |_| true).unwrap();
        let InspectionValue::Common(nets) = view.field("net_coverage") else {
            panic!("canonical correspondence");
        };
        assert_eq!(nets.as_object().unwrap().len(), 1);
        assert_eq!(nets[id(800).to_string()].as_array().unwrap().len(), 2);
        assert!(nets.get(f.net.to_string()).is_none());
    }
    Fixture::assert_bytes(&before);
}

#[test]
fn stale_or_foreign_scene_discloses_complete_members_as_unprojected_without_fabricated_bounds() {
    let (f, mut scene, ids) = tracks();
    let before = f.source_bytes();
    let a = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Compound(CompoundSelection::new(ids.clone(), None).unwrap());
    let revision = scene.source_revision.clone();
    for foreign_project in [false, true] {
        if foreign_project {
            scene.source_revision = revision.clone();
            scene.project_uuid = uuid::Uuid::new_v4().to_string();
        } else {
            scene.source_revision = "older-canonical-basis".into();
        }
        let view =
            build_selection_view(&f.model, &a, subject.clone(), &scene, None, |_| true).unwrap();
        assert_eq!(view.members, ids.iter().cloned().collect());
        assert_eq!(view.unprojected, view.members);
        assert!(view.hidden.is_empty());
        assert!(view.board_keys.is_empty());
        assert!(matches!(
            view.field("combined_bounds_nm"),
            InspectionValue::Unavailable(_)
        ));
        assert_eq!(
            view.field("projected_bounds_nm"),
            InspectionValue::Common(serde_json::json!({}))
        );
        assert!(matches!(
            view.field("layer_coverage"),
            InspectionValue::Unavailable(_)
        ));
    }
    Fixture::assert_bytes(&before);
}
