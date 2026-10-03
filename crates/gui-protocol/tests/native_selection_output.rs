#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
    DatumSelectionContext, SelectionSubject, SelectionTarget,
    selection_resolution::NativeSelectionResolution,
    selection_view::{InspectionValue, build_selection_view},
};
use eda_engine::{ir::geometry::Point, substrate::Operation};
use fixture::{Fixture, id};
fn setup() -> (Fixture, datum_gui_protocol::BoardReviewSceneV1, Vec<Id>) {
    let mut f = Fixture::new("complete-output");
    let mut tracks = vec![
        f.line(id(1), Point::new(0, 0), Point::new(10_000_000, 0)),
        f.line(id(2), Point::new(10_000_000, 0), Point::new(20_000_000, 0)),
    ];
    tracks[0].width = 5_080_000;
    tracks[1].width = 5_080_001;
    for track in &tracks {
        f.commit([Operation::SetBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track).unwrap(),
        }]);
    }
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    scene.components.clear();
    scene.pads.clear();
    scene.vias.clear();
    scene.zones.clear();
    scene.board_graphics.clear();
    scene.board_texts.clear();
    scene.outline.clear();
    let template = scene.tracks[0].clone();
    scene.tracks = tracks
        .iter()
        .map(|track| {
            let mut t = template.clone();
            t.object_id = format!("display-{}", track.uuid);
            t.source_object_uuid = track.uuid.to_string();
            t.track_uuid = t.source_object_uuid.clone();
            t.width_nm = track.width;
            t.path = vec![
                datum_gui_protocol::PointNm {
                    x: track.from.x,
                    y: track.from.y,
                },
                datum_gui_protocol::PointNm {
                    x: track.to.x,
                    y: track.to.y,
                },
            ];
            t
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
fn inspection_compares_canonical_values_and_scopes_never_change_membership() {
    let (f, scene, ids) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Compound(
        CompoundSelection::new(ids.clone(), Some(ids[1].clone())).unwrap(),
    );
    let mut view =
        build_selection_view(&f.model, &authority, subject.clone(), &scene, None, |_| {
            true
        })
        .unwrap();
    assert_eq!(view.members.len(), 2);
    assert_eq!(view.field("copper_width_nm"), InspectionValue::Mixed);
    assert_eq!(
        view.field("board_net_id"),
        InspectionValue::Common(serde_json::json!(f.net))
    );
    assert!(matches!(
        view.field("placement_position_nm"),
        InspectionValue::Unavailable(_)
    ));
    let original = view.members.clone();
    view.scope = Some(Class::Track);
    assert_eq!(view.members, original);
    assert_eq!(view.subject, subject);
    assert_eq!(view.focus(), Some(&ids[1]));
    assert_eq!(view.scoped_members().count(), 2);
    let context = DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(view)));
    assert_eq!(context.kind, "compound");
    let details = context.details.unwrap();
    assert_eq!(details.total, 2);
    assert_eq!(details.per_type[&Class::Track], 2);
    assert_eq!(
        details.inspection["copper_width_nm"],
        InspectionValue::Mixed
    );
    assert_eq!(details.scope, Some(Class::Track));
    assert_eq!(details.focus, Some(ids[1].clone()));
    assert_eq!(details.members.unwrap().len(), 2);
    Fixture::assert_bytes(&before);
}
#[test]
fn hidden_projection_does_not_prune_members_and_uncapped_context_is_not_authority() {
    let (f, scene, ids) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = authority.acquire_run(ids[0].clone(), None).unwrap();
    let view = build_selection_view(&f.model, &authority, subject.clone(), &scene, None, |_| {
        false
    })
    .unwrap();
    assert_eq!(view.members.len(), 2);
    assert_eq!(view.hidden.len(), 2);
    assert_eq!(view.unprojected.len(), 0);
    assert_eq!(view.board_keys.len(), 2);
    let mut dense = view;
    for n in 3..=257 {
        dense.members.insert(Id {
            class: Class::Track,
            id: id(n),
            instance_path: vec![],
        });
    }
    let original = dense.members.clone();
    let context = DatumSelectionContext::from_selection(&SelectionTarget::Native(Box::new(dense)));
    assert_eq!(context.kind, "run");
    let details = context.details.unwrap();
    assert_eq!(details.total, 257);
    assert!(details.members.is_none());
    assert_eq!(details.subject, Some(subject));
    assert_eq!(original.len(), 257);
    Fixture::assert_bytes(&before);
}
#[test]
fn source_catalog_retains_authored_arc_midpoint_independent_of_flattening() {
    let (mut f, mut scene, ids) = setup();
    let mut track: eda_engine::board::Track = serde_json::from_value(
        f.model
            .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
            .unwrap()["tracks"][id(1).to_string()]
        .clone(),
    )
    .unwrap();
    track.midpoint = Some(Point::new(5_000_000, 5_000_000));
    f.commit([Operation::SetBoardTrack {
        track_id: id(1),
        track: serde_json::to_value(track).unwrap(),
    }]);
    // Display flattening is deliberately irrelevant to source anchor qualification.
    scene.tracks[0].path = vec![
        datum_gui_protocol::PointNm { x: 99, y: 99 },
        datum_gui_protocol::PointNm { x: 100, y: 100 },
    ];
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let catalog = datum_gui_protocol::selection_geometry_catalog::qualification_catalog(
        &f.model, &authority, &scene,
    );
    let datum_gui_protocol::selection_geometry_catalog::QualificationGeometry::Anchors(anchors) =
        catalog[&ids[0]].as_ref().unwrap()
    else {
        panic!("source anchors")
    };
    assert_eq!(anchors.len(), 3);
    assert!(anchors.contains(&datum_gui_protocol::PointNm {
        x: 5_000_000,
        y: 5_000_000
    }));
}

#[test]
fn current_zone_islands_are_complete_for_object_and_qualified_for_run() {
    let mut f = Fixture::new("zone-output");
    let zone = f.zone(id(10));
    f.fill(
        &zone,
        eda_engine::substrate::ZoneFillState::Filled,
        vec![
            fixture::rectangle(0, 0, 4, 4),
            fixture::rectangle(20, 0, 4, 4),
        ],
    );
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    scene.project_uuid = f.model.project.project_id.to_string();
    scene.board_uuid = f.board.to_string();
    let mut primitive = scene.zones[0].clone();
    primitive.source_object_uuid = zone.uuid.to_string();
    primitive.object_id = "zone-source-display".into();
    scene.zones = vec![primitive];
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let zone_id = Id {
        class: Class::Zone,
        id: zone.uuid,
        instance_path: vec![],
    };
    let catalog = datum_gui_protocol::selection_geometry_catalog::qualification_catalog(
        &f.model, &authority, &scene,
    );
    let datum_gui_protocol::selection_geometry_catalog::QualificationGeometry::Filled(polygons) =
        catalog[&zone_id].as_ref().unwrap()
    else {
        panic!("complete fill");
    };
    assert_eq!(polygons.len(), 2);
    let object = build_selection_view(
        &f.model,
        &authority,
        SelectionSubject::Object(zone_id.clone()),
        &scene,
        None,
        |_| true,
    )
    .unwrap();
    assert_eq!(object.zone_regions["zone-source-display"].len(), 2);
    let run = authority
        .acquire_run(zone_id.clone(), Some(Point::new(1, 1)))
        .unwrap();
    let projection =
        build_selection_view(&f.model, &authority, run, &scene, None, |_| true).unwrap();
    assert_eq!(projection.zone_regions["zone-source-display"].len(), 1);
    assert_eq!(
        projection.members,
        std::collections::BTreeSet::from([zone_id.clone()])
    );
    Fixture::assert_bytes(&before);
    f.fill(&zone, eda_engine::substrate::ZoneFillState::Filled, vec![]);
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let catalog = datum_gui_protocol::selection_geometry_catalog::qualification_catalog(
        &f.model, &authority, &scene,
    );
    let datum_gui_protocol::selection_geometry_catalog::QualificationGeometry::Filled(polygons) =
        catalog[&zone_id].as_ref().unwrap()
    else {
        panic!("successful empty fill");
    };
    assert!(polygons.is_empty());
}

#[test]
fn indexed_bindings_validate_sources_and_refuse_ambiguous_display_keys() {
    use datum_gui_protocol::selection_scene::{
        SceneSelectionBindings, authored_candidate, projection_candidate,
    };
    let (f, mut scene, _) = setup();
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let bindings = SceneSelectionBindings::capture(&scene, &authority);
    for track in &scene.tracks {
        let binding = bindings.get(&track.object_id).unwrap();
        assert_eq!(
            Some(binding.projected.clone()),
            projection_candidate(&scene, &track.object_id, &authority)
        );
        assert_eq!(
            Some(binding.acquired.clone()),
            authored_candidate(&scene, &track.object_id, &authority)
        );
        assert!(binding.visible(|_| true));
        assert!(!binding.visible(|_| false));
    }
    scene.tracks[1].object_id = scene.tracks[0].object_id.clone();
    let bindings = SceneSelectionBindings::capture(&scene, &authority);
    assert!(bindings.get(&scene.tracks[0].object_id).is_none());
    scene.tracks[1].object_id = "invalid-source".into();
    scene.tracks[1].source_object_uuid = id(999).to_string();
    assert!(
        SceneSelectionBindings::capture(&scene, &authority)
            .get("invalid-source")
            .is_none()
    );
    Fixture::assert_bytes(&before);
}
