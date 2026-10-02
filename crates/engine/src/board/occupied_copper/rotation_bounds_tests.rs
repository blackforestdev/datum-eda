//! Certificates settle only mathematically proved subset/superset cases.
use super::*;
use uuid::Uuid;

fn pad(shape: PadShape, drill: i64) -> PlacedPad {
    serde_json::from_value(serde_json::json!({
        "uuid":Uuid::new_v4(),"package":Uuid::nil(),"name":"P","net":null,
        "position":{"x":0,"y":0},"layer":1,"shape":shape,
        "width":10,"height":6,"rotation":37,"drill":drill
    }))
    .unwrap()
}
fn track(from: Point, to: Point) -> Track {
    Track::straight(Uuid::new_v4(), Uuid::nil(), from, to, 1, 1)
}

#[test]
fn all_rotated_shapes_keep_exact_inner_contact_and_outer_gap_certificates() {
    let inner = track(Point::zero(), Point::new(1, 0));
    let distant = track(Point::new(100, 100), Point::new(110, 100));
    for shape in [PadShape::Rect, PadShape::Oval, PadShape::RoundRect] {
        let p = pad(shape, 0);
        validate_pad(&p).unwrap();
        for boundary in [DistanceBoundary::Inclusive, DistanceBoundary::Strict] {
            assert!(track_pad_within(&inner, &p, 0, boundary).unwrap());
            assert!(!track_pad_within(&distant, &p, 0, boundary).unwrap());
        }
        assert!(pad_contains_point(&p, Point::zero()).unwrap());
        assert!(!pad_contains_point(&p, Point::new(100, 100)).unwrap());
    }
}

#[test]
fn subtracting_the_same_hole_never_creates_copper_from_empty_center() {
    let p = pad(PadShape::Rect, 4);
    let inner = track(Point::zero(), Point::new(1, 0));
    assert!(!pad_contains_point(&p, Point::zero()).unwrap());
    assert!(!track_pad_within(&inner, &p, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!track_pad_within(&inner, &p, 0, DistanceBoundary::Strict).unwrap());
}

#[test]
fn pad_pairs_and_polygon_certificates_share_shape_and_clearance_authority() {
    let p = pad(PadShape::Oval, 0);
    let mut q = pad(PadShape::RoundRect, 0);
    assert!(pads_within(&p, &q, 0, DistanceBoundary::Inclusive).unwrap());
    q.position = Point::new(100, 100);
    assert!(!pads_within(&p, &q, 0, DistanceBoundary::Inclusive).unwrap());
    let via = Via {
        uuid: Uuid::new_v4(),
        net: Uuid::nil(),
        position: Point::zero(),
        diameter: 2,
        drill: 0,
        from_layer: 1,
        to_layer: 1,
    };
    assert!(pad_via_within(&p, &via, 0, DistanceBoundary::Strict).unwrap());
    let polygon = Polygon::new(vec![
        Point::new(-1, -1),
        Point::new(1, -1),
        Point::new(1, 1),
        Point::new(-1, 1),
    ]);
    assert!(pad_polygon_within(&p, &polygon, 0, DistanceBoundary::Strict).unwrap());
    assert_eq!(
        pads_within(&p, &q, -1, DistanceBoundary::Inclusive),
        Err(GeometryError::InvalidWidth)
    );
}

#[test]
fn odd_halves_are_exact_and_uncertified_boundary_is_not_false() {
    let mut p = pad(PadShape::Rect, 0);
    p.width = 7;
    p.height = 11;
    let t = track(Point::new(-1, 4), Point::new(1, 4));
    assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
    assert_eq!(
        track_pad_within(&t, &p, 0, DistanceBoundary::Strict),
        Err(GeometryError::UnresolvedPredicate)
    );
    // This is a bounding-certificate failure, never accepted separation.
    assert_eq!(
        pad_contains_point(&p, Point::new(0, 4)),
        Err(GeometryError::UnresolvedPredicate)
    );
}

#[test]
fn invalid_intrinsic_shape_and_arc_still_refuse() {
    let mut p = pad(PadShape::RoundRect, 0);
    p.roundrect_rratio_ppm = 500_001;
    assert_eq!(validate_pad(&p), Err(GeometryError::UnsupportedPadGeometry));
    p.roundrect_rratio_ppm = 250_000;
    p.drill = p.height;
    assert_eq!(validate_pad(&p), Err(GeometryError::UnsupportedPadGeometry));
    p.drill = 0;
    let mut t = track(Point::zero(), Point::new(1, 0));
    t.midpoint = Some(Point::zero());
    assert_eq!(
        track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive),
        Err(GeometryError::InvalidArc)
    );
}
