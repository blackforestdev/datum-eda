use super::*;
use crate::board::{Track, nominal_geometry::Rational as R};
use crate::ir::geometry::Point;
use uuid::Uuid;

fn arc(from: Point, midpoint: Point, to: Point, width: i64) -> Track {
    let mut t = Track::straight(
        Uuid::from_u128(123),
        Uuid::from_u128(99),
        from,
        to,
        width,
        1,
    );
    t.midpoint = Some(midpoint);
    t
}
fn t04() -> Track {
    arc(
        Point::new(0, 0),
        Point::new(1_000_000, 3_000_000),
        Point::new(3_000_000, 0),
        200_001,
    )
}
#[test]
fn exact_center_output_retains_anchors_aperture_and_directed_major_sweep() {
    let t = arc(
        Point::new(-3_000_000, 4_000_000),
        Point::new(0, -5_000_000),
        Point::new(3_000_000, 4_000_000),
        1,
    );
    let original = serde_json::to_vec(&t).unwrap();
    let w = arc_output_witness(&t).unwrap();
    assert_eq!(w.disposition(), ArcOutputDisposition::Exact);
    assert_eq!(
        w.exact_center_offset(),
        [R::integer(3_000_000), R::integer(-4_000_000)]
    );
    assert_eq!(
        w.source_radius_squared_nm2(),
        R::integer(25_000_000_000_000)
    );
    assert_eq!(w.occupied_deviation_upper_nm(), Some(R::integer(0)));
    assert!(w.primitive().counterclockwise);
    let straight = Track::straight(
        Uuid::from_u128(124),
        t.net,
        t.to,
        Point::new(10_000_000, 4_000_000),
        1,
        1,
    );
    let text = render_rs274x_copper_layer(1, &[], &[t.clone(), straight], &[], &[]).unwrap();
    assert!(text.contains("%ADD10C,0.000001*%"));
    assert!(text.contains("X-3000000Y4000000D02*"));
    assert!(text.contains("G75*\nG03*\nX3000000Y-4000000I3000000J-4000000D01*\nG03*\nX3000000Y4000000I-3000000J4000000D01*\nG01*"));
    assert!(text.contains("X10000000Y4000000D01*"));
    assert_eq!(serde_json::to_vec(&t).unwrap(), original);
}
#[test]
fn t04_rational_center_has_exact_error_witness_but_output_remains_refused() {
    let t = t04();
    let w = arc_output_witness(&t).unwrap();
    // Independently derived perpendicular bisectors: cx=3mm/2,
    // cy=((1^2+3^2)-2*1*(3/2))/(2*3)=7mm/6.
    assert_eq!(
        w.exact_center_offset(),
        [R::integer(1_500_000), R::new(3_500_000, 3).unwrap()]
    );
    assert_eq!(
        w.source_radius_squared_nm2(),
        R::new(32_500_000_000_000, 9).unwrap()
    );
    assert_eq!(
        w.primitive().center_offset,
        Point::new(1_500_000, 1_166_667)
    );
    assert_eq!(
        w.output_radius_squared_nm2(),
        [R::integer(3_611_111_888_889); 2]
    );
    assert_eq!(w.center_error_l1_nm(), R::new(1, 3).unwrap());
    assert_eq!(w.endpoint_radius_mismatch_upper_nm(), R::integer(0));
    assert_eq!(w.occupied_deviation_upper_nm(), Some(R::new(2, 3).unwrap()));
    assert!(w.sweep_preserved());
    assert_eq!(w.disposition(), ArcOutputDisposition::AllowanceRequired);
    match render_rs274x_copper_layer(1, &[], &[t], &[], &[]).unwrap_err() {
        ExportError::DeferredArcManufacturing { witness, .. } => assert_eq!(*witness, w),
        e => panic!("{e:?}"),
    }
    // This is positive witness proof, NOT positive rational-center CAM/T04.
}
#[test]
fn reverse_mirror_and_translation_preserve_certified_bound() {
    let t = t04();
    let variants = [
        arc(t.to, t.midpoint.unwrap(), t.from, t.width),
        arc(
            Point::new(0, 0),
            Point::new(1_000_000, -3_000_000),
            Point::new(3_000_000, 0),
            t.width,
        ),
        arc(
            Point::new(8, 9),
            Point::new(1_000_008, 3_000_009),
            Point::new(3_000_008, 9),
            t.width,
        ),
    ];
    for v in variants {
        let w = arc_output_witness(&v).unwrap();
        assert_eq!(w.occupied_deviation_upper_nm(), Some(R::new(2, 3).unwrap()));
        assert_eq!(
            w.output_radius_squared_nm2()[0],
            w.output_radius_squared_nm2()[1]
        );
        assert_eq!(w.disposition(), ArcOutputDisposition::AllowanceRequired);
    }
}
#[test]
fn primitive_changes_invalid_source_range_and_unequal_radii_refuse_honestly() {
    let t = t04();
    let p = arc_output_witness(&t).unwrap().primitive();
    assert_eq!(
        validate_arc_output_primitive(
            &t,
            ArcOutputPrimitive {
                to: Point::new(3_000_001, 0),
                ..p
            }
        ),
        Err(ArcWitnessError::EndpointChanged)
    );
    assert_eq!(
        validate_arc_output_primitive(
            &t,
            ArcOutputPrimitive {
                width_nm: 200_000,
                ..p
            }
        ),
        Err(ArcWitnessError::WidthChanged)
    );
    assert_eq!(
        validate_arc_output_primitive(
            &t,
            ArcOutputPrimitive {
                counterclockwise: !p.counterclockwise,
                ..p
            }
        ),
        Err(ArcWitnessError::DirectionChanged)
    );
    let invalid = arc(Point::new(0, 0), Point::new(1, 0), Point::new(3, 0), 1);
    assert!(matches!(
        arc_output_witness(&invalid),
        Err(ArcWitnessError::Geometry(_))
    ));
    let off_grid = arc(
        Point::new(0, 0),
        Point::new(1_000_000, 3_000_000),
        Point::new(3_000_001, 0),
        200_001,
    );
    let w = arc_output_witness(&off_grid).unwrap();
    assert_ne!(
        w.output_radius_squared_nm2()[0],
        w.output_radius_squared_nm2()[1]
    );
    assert_eq!(
        w.disposition(),
        ArcOutputDisposition::UncertifiedConstruction
    );
    assert_eq!(w.occupied_deviation_upper_nm(), None);
    let out_of_range = arc(
        Point::new(10_000_000_000, 0),
        Point::new(10_000_000_005, 5),
        Point::new(10_000_000_010, 0),
        1,
    );
    assert_eq!(
        arc_output_witness(&out_of_range),
        Err(ArcWitnessError::OutputRange)
    );
}
#[test]
fn width_clearance_and_topology_diagnostics_use_computed_error_without_waiver() {
    let t = t04();
    let w = arc_output_witness(&t).unwrap();
    let distant = Track::straight(
        Uuid::from_u128(200),
        Uuid::from_u128(201),
        Point::new(0, 4_000_000),
        Point::new(3_000_000, 4_000_000),
        1,
        1,
    );
    let connected = Track::straight(
        Uuid::from_u128(202),
        t.net,
        t.from,
        Point::new(-1_000_000, 0),
        1,
        1,
    );
    let r = check_arc_output_constraints(&w, context(t.width, 100, &[distant, connected])).unwrap();
    assert_eq!(r.minimum_width, OutputMarginResult::AtRisk);
    assert_eq!(r.pairs[0].clearance, OutputMarginResult::Preserved);
    assert_eq!(r.pairs[0].topology, OutputMarginResult::Preserved);
    assert!(r.pairs[1].nominal_contact);
    assert_eq!(r.pairs[1].topology, OutputMarginResult::AtRisk);
    assert_eq!(r.pairs[1].clearance, OutputMarginResult::AtRisk);
    assert!(r.requires_model_bound_fill_and_complete_inventory);
    let r = check_arc_output_constraints(&w, context(t.width - 2, 0, &[])).unwrap();
    assert_eq!(r.minimum_width, OutputMarginResult::Preserved);
    let exact = arc(Point::new(-5, 0), Point::new(0, 5), Point::new(5, 0), 1);
    let same = Track::straight(
        Uuid::from_u128(300),
        exact.net,
        exact.from,
        Point::new(-10, 0),
        1,
        1,
    );
    let r =
        check_arc_output_constraints(&arc_output_witness(&exact).unwrap(), context(1, 0, &[same]))
            .unwrap();
    assert_eq!(r.minimum_width, OutputMarginResult::Preserved);
    assert_eq!(r.pairs[0].topology, OutputMarginResult::Preserved);
}

fn context(width: i64, clearance: i64, tracks: &[Track]) -> ArcOutputConstraintContext<'_> {
    static STACK: std::sync::LazyLock<crate::board::Stackup> =
        std::sync::LazyLock::new(|| crate::board::Stackup {
            layers: vec![crate::board::StackupLayer::new(
                1,
                "Cu",
                crate::board::StackupLayerType::Copper,
                35_000,
            )],
        });
    ArcOutputConstraintContext {
        stackup: &STACK,
        minimum_width_nm: width,
        clearance_nm: clearance,
        tracks,
        pads: &[],
        vias: &[],
    }
}

#[test]
fn exact_but_submicrometre_endpoint_arcs_are_not_format_ready() {
    let t = arc(Point::new(-3, 4), Point::new(0, -5), Point::new(3, 4), 1);
    let w = arc_output_witness(&t).unwrap();
    assert_eq!(w.occupied_deviation_upper_nm(), Some(R::integer(0)));
    assert!(!w.endpoints_stable());
    assert_eq!(
        w.disposition(),
        ArcOutputDisposition::UncertifiedConstruction
    );
    assert!(render_rs274x_copper_layer(1, &[], &[t], &[], &[]).is_err());
}

#[test]
fn changed_source_under_same_uuid_cannot_be_ignored_as_self() {
    let t = t04();
    let w = arc_output_witness(&t).unwrap();
    let mut different = t.clone();
    different.width += 1;
    assert_eq!(
        check_arc_output_constraints(&w, context(1, 0, &[different])),
        Err(ArcWitnessError::SourceConflict)
    );
}
