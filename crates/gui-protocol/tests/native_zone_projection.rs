#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject,
    ZoneFillProjection,
    selection_resolution::NativeSelectionResolution,
    selection_view::{InspectionValue, build_selection_view},
};
use eda_engine::substrate::ZoneFillState;
use fixture::{Fixture, id};

#[test]
fn certified_current_fill_distinguishes_complete_empty_and_unavailable() {
    let mut f = Fixture::new("zone-projection");
    let zone = f.zone(id(10));
    for (state, polygons, expected) in [
        (
            ZoneFillState::Filled,
            vec![
                fixture::rectangle(0, 0, 4, 4),
                fixture::rectangle(20, 0, 4, 4),
            ],
            Some(2),
        ),
        (ZoneFillState::Filled, vec![], Some(0)),
        (ZoneFillState::Unsupported, vec![], None),
        (ZoneFillState::Unfilled, vec![], None),
    ] {
        f.fill(&zone, state, polygons);
        let before = f.source_bytes();
        let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
        scene.project_uuid = f.model.project.project_id.to_string();
        scene.source_revision = f.model.model_revision.0.clone();
        scene.board_uuid = f.board.to_string();
        let mut primitive = scene.zones[0].clone();
        primitive.source_object_uuid = zone.uuid.to_string();
        primitive.zone_uuid = zone.uuid.to_string();
        primitive.object_id = "native-zone".into();
        primitive.fill_projection = ZoneFillProjection::capture(&f.model, zone.uuid);
        primitive.polygon = zone
            .polygon
            .vertices
            .iter()
            .map(|p| datum_gui_protocol::PointNm { x: p.x, y: p.y })
            .collect();
        match (&primitive.fill_projection, expected) {
            (ZoneFillProjection::Current(p), Some(count)) => assert_eq!(p.len(), count),
            (ZoneFillProjection::Unavailable(_), None) => {}
            _ => panic!("fill state must not be conflated"),
        }
        assert_eq!(primitive.occupied_polygons().count(), expected.unwrap_or(0));
        scene.zones = vec![primitive];
        let authority = NativeSelectionResolution::capture(&f.model).unwrap();
        let identity = Id {
            class: Class::Zone,
            id: zone.uuid,
            instance_path: vec![],
        };
        let mut view = build_selection_view(
            &f.model,
            &authority,
            SelectionSubject::Object(identity.clone()),
            &scene,
            None,
            |_| true,
        )
        .unwrap();
        assert_eq!(view.members, std::collections::BTreeSet::from([identity]));
        assert_eq!(view.zone_regions.get("native-zone").map(Vec::len), expected);
        view.scope = Some(Class::Zone);
        match expected {
            Some(count) => assert_eq!(
                view.field("occupied_fill_status"),
                InspectionValue::Common(serde_json::json!(if count == 0 {
                    "Current: no occupied copper"
                } else {
                    "Current occupied copper"
                }))
            ),
            None => assert!(matches!(
                view.field("occupied_fill_status"),
                InspectionValue::Unavailable(_)
            )),
        }
        // The authored boundary remains inspectable; it is never occupied copper.
        assert!(matches!(
            view.field("combined_bounds_nm"),
            InspectionValue::Common(_)
        ));
        Fixture::assert_bytes(&before);
        assert_eq!(
            ZoneFillProjection::capture(&f.reopen(), zone.uuid),
            scene.zones[0].fill_projection
        );
    }
    f.fill(
        &zone,
        ZoneFillState::Filled,
        vec![fixture::rectangle(0, 0, 4, 4)],
    );
    f.model.zone_fills.get_mut(&zone.uuid).unwrap().zone_id = id(999);
    assert!(
        matches!(
            ZoneFillProjection::capture(&f.model, zone.uuid),
            ZoneFillProjection::Unavailable(_)
        ),
        "a foreign record cannot become current copper"
    );
}
