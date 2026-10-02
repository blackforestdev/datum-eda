use super::*;
use crate::board::Board;
use uuid::Uuid;
fn arc(radius: i64, width: i64) -> Track {
    let mut track = Track::straight(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Point::new(-radius, 0),
        Point::new(radius, 0),
        width,
        8,
    );
    track.midpoint = Some(Point::new(0, radius));
    track
}
fn pad(shape: PadShape, position: Point, w: i64, h: i64) -> PlacedPad {
    serde_json::from_value(serde_json::json!({"uuid":Uuid::new_v4(),"package":Uuid::new_v4(),"name":"","net":Uuid::new_v4(),"position":position,"layer":8,"shape":shape,"diameter":w,"width":w,"height":h})).unwrap()
}
fn rectangle(x1: i64, y1: i64, x2: i64, y2: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x1, y1),
        Point::new(x2, y1),
        Point::new(x2, y2),
        Point::new(x1, y2),
    ])
}
#[test]
fn authored_arc_pad_boundaries_preserve_half_nm_and_actual_bulge() {
    let track = arc(5_000_000, 1);
    let circle = pad(
        PadShape::Circle,
        Point::new(0, 6_000_000),
        1_999_999,
        1_999_999,
    );
    assert!(track_pad_within(&track, &circle, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!track_pad_within(&track, &circle, 0, DistanceBoundary::Strict).unwrap());
    let mut gap = circle.clone();
    gap.diameter -= 1;
    assert!(!track_pad_within(&track, &gap, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(track_pad_within(&track, &gap, 1, DistanceBoundary::Strict).unwrap());
    let chord = pad(PadShape::Circle, Point::new(0, 0), 1_000_000, 1_000_000);
    assert!(!track_pad_within(&track, &chord, 0, DistanceBoundary::Inclusive).unwrap());
    for shape in [PadShape::Rect, PadShape::Oval, PadShape::RoundRect] {
        let p = pad(shape, Point::new(0, 6_000_000), 2_000_001, 1_999_999);
        assert!(track_pad_within(&track, &p, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(!track_pad_within(&track, &p, 0, DistanceBoundary::Strict).unwrap());
        let mut rotate = p.clone();
        rotate.rotation = 90;
        assert!(track_pad_within(&track, &rotate, 0, DistanceBoundary::Inclusive).unwrap());
        rotate.rotation = 37;
        assert_eq!(
            track_pad_within(&track, &rotate, 0, DistanceBoundary::Inclusive),
            Err(GeometryError::UnsupportedPadGeometry)
        );
    }
}
#[test]
fn drill_holes_are_empty_and_inner_tangency_is_not_a_clearance_violation() {
    let via = Via {
        uuid: Uuid::new_v4(),
        net: Uuid::new_v4(),
        position: Point::new(0, 0),
        diameter: 20,
        drill: 8,
        from_layer: 8,
        to_layer: 7,
    };
    assert!(!track_via_within(&arc(2, 1), &via, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(track_via_within(&arc(3, 2), &via, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!track_via_within(&arc(3, 2), &via, 0, DistanceBoundary::Strict).unwrap());
    assert!(track_via_within(&arc(3, 2), &via, 1, DistanceBoundary::Strict).unwrap());
    let p = pad(PadShape::Circle, Point::new(0, 0), 2, 2);
    assert!(!pad_via_within(&p, &via, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(
        !via_polygon_within(
            &via,
            &rectangle(-1, -1, 1, 1),
            0,
            DistanceBoundary::Inclusive
        )
        .unwrap()
    );
    let mut ring = pad(PadShape::Circle, Point::new(0, 0), 20, 20);
    ring.drill = 8;
    assert!(!pads_within(&p, &ring, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(pads_within(&ring, &ring, 0, DistanceBoundary::Inclusive).unwrap());
}
#[test]
fn current_fill_holes_are_actual_empty_cells_not_authored_outlines() {
    let frame = [
        rectangle(-10, 2, 10, 10),
        rectangle(-10, -10, 10, -2),
        rectangle(-10, -2, -2, 2),
        rectangle(2, -2, 10, 2),
    ];
    let in_hole = arc(1, 1);
    let touching = arc(2, 1);
    assert!(
        frame
            .iter()
            .all(|p| !track_polygon_within(&in_hole, p, 0, DistanceBoundary::Inclusive).unwrap())
    );
    assert!(
        frame
            .iter()
            .any(|p| track_polygon_within(&touching, p, 0, DistanceBoundary::Inclusive).unwrap())
    );
    let mut reverse = frame[0].clone();
    reverse.vertices.reverse();
    assert_eq!(
        track_polygon_within(&touching, &reverse, 0, DistanceBoundary::Inclusive),
        track_polygon_within(&touching, &frame[0], 0, DistanceBoundary::Inclusive)
    );
    let invalid = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(2, 2),
        Point::new(0, 2),
        Point::new(2, 0),
    ]);
    assert_eq!(
        track_polygon_within(&touching, &invalid, 0, DistanceBoundary::Inclusive),
        Err(GeometryError::InvalidPolygon)
    );
}
#[test]
fn actual_stackup_span_uses_copper_order_not_numeric_layer_ids() {
    let board:Board=serde_json::from_value(serde_json::json!({"uuid":Uuid::new_v4(),"name":"","stackup":{"layers":[
        {"id":8,"name":"A","layer_type":"Copper","thickness_nm":1},
        {"id":100,"name":"D","layer_type":"Dielectric","thickness_nm":1},
        {"id":20,"name":"I","layer_type":"Copper","thickness_nm":1},
        {"id":7,"name":"B","layer_type":"Copper","thickness_nm":1}]},"outline":rectangle(0,0,10,10),"packages":{},"pads":{},"tracks":{},"vias":{},"zones":{},"nets":{},"net_classes":{},"rules":[],"keepouts":[],"dimensions":[],"texts":[]})).unwrap();
    let mut via = Via {
        uuid: Uuid::new_v4(),
        net: Uuid::new_v4(),
        position: Point::new(0, 0),
        diameter: 20,
        drill: 8,
        from_layer: 8,
        to_layer: 7,
    };
    assert_eq!(via_layers(&board.stackup, &via).unwrap(), vec![8, 20, 7]);
    via.from_layer = 20;
    assert_eq!(via_layers(&board.stackup, &via).unwrap(), vec![20, 7]);
    via.from_layer = 100;
    assert_eq!(
        via_layers(&board.stackup, &via),
        Err(GeometryError::UnknownConductiveLayer)
    );
    let mut p = pad(PadShape::Circle, Point::new(0, 0), 10, 10);
    p.copper_layers = vec![20, 8];
    assert_eq!(pad_layers(&board.stackup, &p).unwrap(), vec![8, 20]);
    p.copper_layers.push(100);
    assert_eq!(
        pad_layers(&board.stackup, &p),
        Err(GeometryError::UnknownConductiveLayer)
    );
}
#[test]
fn zero_clearance_rectangles_distinguish_tangency_overlap_and_gap() {
    let a = pad(PadShape::Rect, Point::new(0, 0), 4, 4);
    let touching = pad(PadShape::Rect, Point::new(4, 0), 4, 4);
    assert!(pads_within(&a, &touching, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!pads_within(&a, &touching, 0, DistanceBoundary::Strict).unwrap());
    let overlapping = pad(PadShape::Rect, Point::new(3, 0), 4, 4);
    assert!(pads_within(&a, &overlapping, 0, DistanceBoundary::Strict).unwrap());
    let gap = pad(PadShape::Rect, Point::new(5, 0), 4, 4);
    assert!(!pads_within(&a, &gap, 0, DistanceBoundary::Inclusive).unwrap());
    assert!(!pads_within(&a, &gap, 1, DistanceBoundary::Strict).unwrap());
}

#[test]
fn invalid_arc_cannot_pass_by_endpoint_containment_in_filled_copper() {
    let mut invalid = arc(2, 1);
    invalid.midpoint = Some(Point::new(0, 0));
    assert_eq!(
        track_polygon_within(
            &invalid,
            &rectangle(-10, -10, 10, 10),
            0,
            DistanceBoundary::Inclusive
        ),
        Err(GeometryError::InvalidArc)
    );
    let p = pad(PadShape::Rect, Point::new(0, 0), 4, 4);
    assert_eq!(
        pad_polygon_within(&p, &rectangle(2, -2, 4, 2), 0, DistanceBoundary::Strict),
        Err(GeometryError::UnresolvedPredicate)
    );
}
