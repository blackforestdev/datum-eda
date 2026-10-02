use super::*;
use uuid::Uuid;
fn pad(shape: PadShape, w: i64, h: i64) -> PlacedPad {
    serde_json::from_value(serde_json::json!({"uuid":Uuid::new_v4(),"package":Uuid::nil(),"name":"P","net":null,"position":{"x":0,"y":0},"layer":1,"shape":shape,"width":w,"height":h,"rotation":45})).unwrap()
}
fn line(a: Point, b: Point, width: i64) -> Track {
    Track::straight(Uuid::new_v4(), Uuid::nil(), a, b, width, 1)
}
fn arc(r: i64, width: i64) -> Track {
    let mut t = line(Point::new(-r, 0), Point::new(r, 0), width);
    t.midpoint = Some(Point::new(0, r));
    t
}
#[test]
fn exact_corner_and_half_nm_gap_clearance_at_all_odd_eighth_turns() {
    for angle in [45, 135, 225, 315, -45, 405] {
        let mut p = pad(PadShape::Rect, 2_000_000, 2_000_000);
        p.rotation = angle;
        let t = line(Point::new(0, 1_400_000), Point::new(0, 1_400_001), 1);
        // Each local squared coordinate is y²/2 < 1mm², independently.
        assert!(1_400_000_i128.pow(2) < 2 * 1_000_000_i128.pow(2));
        assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Strict).unwrap());
        assert!(pad_contains_point(&p, t.from).unwrap());
        let gap = line(Point::new(0, 1_414_215), Point::new(0, 1_414_216), 1);
        // Closest point is the true radical corner (0,sqrt(2)*1mm).
        // y-half > sqrt(2)*1mm, proved by comparing positive squares.
        assert!(
            R::new(2 * 1_414_215 - 1, 2)
                .unwrap()
                .square()
                .unwrap()
                .compare(R::integer(2 * 1_000_000_i128.pow(2)))
                .unwrap()
                == Ordering::Greater
        );
        assert!(!track_pad_within(&gap, &p, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(track_pad_within(&gap, &p, 2, DistanceBoundary::Strict).unwrap());
    }
}
#[test]
fn directed_arc_corner_bulge_reverse_and_separation_are_exact() {
    let p = pad(PadShape::Rect, 2_000_000, 2_000_000);
    // Endpoints are separated; only the actual authored circular bulge enters
    // the corner. Circle centered (0,1.9mm), radius0.5mm, lower semicircle.
    let mut t = line(
        Point::new(-500_000, 1_900_000),
        Point::new(500_000, 1_900_000),
        1,
    );
    t.midpoint = Some(Point::new(0, 1_400_000));
    assert!(!pad_contains_point(&p, t.from).unwrap());
    assert!(!pad_contains_point(&p, t.to).unwrap());
    assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
    std::mem::swap(&mut t.from, &mut t.to);
    assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Strict).unwrap());
    t.midpoint = Some(Point::new(0, 2_400_000));
    assert!(!track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
}
#[test]
fn exact_tangent_gap_hole_and_rounding_states_share_boundary_law() {
    // A square oval is an exact circle despite its authored odd eighth-turn.
    for shape in [PadShape::Oval, PadShape::RoundRect] {
        let mut p = pad(shape, 10, 10);
        p.roundrect_rratio_ppm = 500_000;
        let t = line(Point::new(-10, 6), Point::new(10, 6), 2);
        assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(!track_pad_within(&t, &p, 0, DistanceBoundary::Strict).unwrap());
        let gap = line(Point::new(-10, 7), Point::new(10, 7), 2);
        assert!(!track_pad_within(&gap, &p, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(track_pad_within(&gap, &p, 2, DistanceBoundary::Strict).unwrap());
    }
    let mut p = pad(PadShape::Rect, 20, 20);
    p.drill = 12;
    let t = arc(5, 2);
    assert!(track_pad_within(&t, &p, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!track_pad_within(&t, &p, 0, DistanceBoundary::Strict).unwrap());
    assert!(!track_pad_within(&arc(5, 1), &p, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!pad_contains_point(&p, Point::zero()).unwrap());
}
#[test]
fn nonsquare_capsule_roundrect_checked_range_and_invalid_arc() {
    for shape in [PadShape::Rect, PadShape::Oval, PadShape::RoundRect] {
        let mut p = pad(shape, 20, 6);
        p.roundrect_rratio_ppm = 250_000;
        assert!(
            track_pad_within(
                &line(Point::new(5, 5), Point::new(6, 6), 1),
                &p,
                0,
                DistanceBoundary::Inclusive
            )
            .unwrap()
        );
        assert!(
            !track_pad_within(
                &line(Point::new(-20, 20), Point::new(-21, 21), 1),
                &p,
                0,
                DistanceBoundary::Inclusive
            )
            .unwrap()
        );
        let mut invalid = arc(5, 1);
        invalid.midpoint = Some(invalid.from);
        assert_eq!(
            track_pad_within(&invalid, &p, 0, DistanceBoundary::Inclusive),
            Err(GeometryError::InvalidArc)
        );
        assert_eq!(
            track_pad_within(&invalid, &p, -1, DistanceBoundary::Inclusive),
            Err(GeometryError::InvalidWidth)
        );
    }
    let p = pad(PadShape::Rect, 20, 6);
    let extreme = line(Point::new(i64::MAX, i64::MAX), Point::new(i64::MIN, 0), 1);
    assert_eq!(
        track_pad_within(&extreme, &p, 0, DistanceBoundary::Inclusive),
        Err(GeometryError::ArithmeticRange)
    );
}
