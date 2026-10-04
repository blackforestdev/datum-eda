//! Canonical source angles and certified anchors must agree with native projection.
#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, PointNm,
    selection_geometry_catalog::{QualificationGeometry, qualification_catalog},
    selection_resolution::NativeSelectionResolution,
};
use eda_engine::{
    ir::geometry::{Arc, Point},
    schematic::SchematicPrimitive,
    substrate::Operation,
};
use fixture::{Fixture, id};
fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
#[test]
fn native_arc_qualification_uses_authored_tenths_and_positive_wrapped_sweep() {
    let mut f = Fixture::new("native-arc-qualification");
    let cases = [
        (1, 0, 1800, vec![p(10, 0), p(0, 10), p(-10, 0)]),
        (2, 2700, 900, vec![p(0, -10), p(10, 0), p(0, 10)]),
        (3, 900, 2700, vec![p(0, 10), p(-10, 0), p(0, -10)]),
        (4, 0, 0, vec![p(10, 0), p(-10, 0), p(10, 0)]),
    ];
    for (n, start_angle, end_angle, _) in &cases {
        let drawing = SchematicPrimitive::Arc {
            uuid: id(*n),
            arc: Arc {
                center: Point::new(0, 0),
                radius: 10,
                start_angle: *start_angle,
                end_angle: *end_angle,
            },
        };
        f.commit([Operation::CreateSchematicDrawing {
            sheet_id: f.sheet,
            drawing_id: id(*n),
            drawing: serde_json::to_value(drawing).unwrap(),
        }]);
    }
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let scene = datum_gui_protocol::project_native_schematic_scene(&f.model, &f.root)
        .unwrap()
        .unwrap();
    let catalog = qualification_catalog(&f.model, &authority, &scene);
    for (n, _, _, expected) in &cases {
        let identity = Id {
            class: Class::SchematicGraphic,
            id: id(*n),
            instance_path: vec![],
        };
        let QualificationGeometry::Anchors(actual) = catalog[&identity].as_ref().unwrap() else {
            panic!("three certified authored anchors")
        };
        assert_eq!(
            actual, expected,
            "engine tenths-of-degree source, independent cardinal oracle"
        );
    }
    // The positive half-circle includes its right endpoint and upper midpoint;
    // its left endpoint alone does not reach the 2/3 majority requirement.
    let QualificationGeometry::Anchors(half) = catalog[&Id {
        class: Class::SchematicGraphic,
        id: id(1),
        instance_path: vec![],
    }]
        .as_ref()
        .unwrap()
    else {
        unreachable!()
    };
    assert_eq!(half.iter().filter(|p| p.x >= 0 && p.y >= 0).count(), 2);
    assert_eq!(half.iter().filter(|p| p.x < 0).count(), 1);
    Fixture::assert_bytes(&before);
    assert_eq!(f.reopen().model_revision, f.model.model_revision);
}
#[test]
fn noncertified_arc_anchor_or_overflow_refuses_without_source_mutation() {
    let mut f = Fixture::new("native-arc-refusal");
    let mut safe_scene = None;
    for (n, center, start_angle, end_angle) in [
        (1, Point::new(0, 0), 0, 900),
        (2, Point::new(i64::MAX, 0), 0, 1800),
    ] {
        let drawing = SchematicPrimitive::Arc {
            uuid: id(n),
            arc: Arc {
                center,
                radius: 10,
                start_angle,
                end_angle,
            },
        };
        f.commit([Operation::CreateSchematicDrawing {
            sheet_id: f.sheet,
            drawing_id: id(n),
            drawing: serde_json::to_value(drawing).unwrap(),
        }]);
        if n == 1 {
            safe_scene =
                datum_gui_protocol::project_native_schematic_scene(&f.model, &f.root).unwrap();
        }
    }
    let before = f.source_bytes();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    // Projection's legacy floating display path cannot represent overflow; use
    // actual safe source projection then bind the real second source identity.
    let mut scene = safe_scene.unwrap();
    scene.source_revision = f.model.model_revision.0.clone();
    scene
        .board_graphics
        .retain(|g| g.source_object_uuid == id(1).to_string());
    let mut overflow = scene.board_graphics[0].clone();
    overflow.object_id = "overflow-source".into();
    overflow.source_object_uuid = id(2).to_string();
    scene.board_graphics.push(overflow);
    let catalog = qualification_catalog(&f.model, &authority, &scene);
    for n in [1, 2] {
        assert!(
            catalog[&Id {
                class: Class::SchematicGraphic,
                id: id(n),
                instance_path: vec![]
            }]
                .is_err()
        );
    }
    Fixture::assert_bytes(&before);
}
