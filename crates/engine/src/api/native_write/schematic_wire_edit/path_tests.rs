//! Independent coordinate expectations for the proposed checked shared router.
use super::{
    path::routed_wire_points,
    types::{WireCornerOrder as Corner, WireRouteConfig, WireRouting as Mode},
};
use crate::ir::geometry::Point;
fn p(x: i64, y: i64) -> Point {
    Point::new(x, y)
}
fn path(a: Point, b: Point, routing: Mode, corner: Corner) -> Vec<Point> {
    routed_wire_points(a, b, WireRouteConfig { routing, corner }).unwrap()
}

#[test]
fn orthogonal_postures_keep_exact_endpoints_and_choose_different_elbows() {
    let a = p(10, 20);
    let b = p(70, 90);
    assert_eq!(
        path(a, b, Mode::Orthogonal, Corner::Start),
        vec![a, p(70, 20), b]
    );
    assert_eq!(
        path(a, b, Mode::Orthogonal, Corner::End),
        vec![a, p(10, 90), b]
    );
}

#[test]
fn diagonal_postures_choose_diagonal_first_or_axis_first_without_rounding() {
    let a = p(10, 20);
    let b = p(70, 90);
    assert_eq!(
        path(a, b, Mode::Diagonal45, Corner::Start),
        vec![a, p(70, 80), b]
    );
    assert_eq!(
        path(a, b, Mode::Diagonal45, Corner::End),
        vec![a, p(10, 30), b]
    );
    let a = p(100, -200);
    let b = p(-130, 80);
    assert_eq!(
        path(a, b, Mode::Diagonal45, Corner::Start),
        vec![a, p(-130, 30), b]
    );
    assert_eq!(
        path(a, b, Mode::Diagonal45, Corner::End),
        vec![a, p(100, -150), b]
    );
}

#[test]
fn all_quadrants_have_only_exact_axis_or_45_degree_segments_and_reverse_posture_symmetry() {
    let a = p(-73, 91);
    for b in [p(217, 501), p(-363, 501), p(217, -319), p(-363, -319)] {
        for mode in [Mode::Orthogonal, Mode::Diagonal45] {
            for corner in [Corner::Start, Corner::End] {
                let points = path(a, b, mode, corner);
                assert_eq!(points.first(), Some(&a));
                assert_eq!(points.last(), Some(&b));
                for pair in points.windows(2) {
                    let dx = (i128::from(pair[1].x) - i128::from(pair[0].x)).abs();
                    let dy = (i128::from(pair[1].y) - i128::from(pair[0].y)).abs();
                    assert!(dx != 0 || dy != 0);
                    assert!(dx == 0 || dy == 0 || (mode == Mode::Diagonal45 && dx == dy));
                }
                let opposite = if corner == Corner::Start {
                    Corner::End
                } else {
                    Corner::Start
                };
                let mut reversed = path(b, a, mode, opposite);
                reversed.reverse();
                assert_eq!(points, reversed);
            }
        }
    }
}

#[test]
fn off_grid_pin_contact_survives_every_posture_exactly() {
    let a = p(0, 0);
    let pin = p(123_457, -987_653);
    for mode in [Mode::Free, Mode::Orthogonal, Mode::Diagonal45] {
        for corner in [Corner::Start, Corner::End] {
            let points = path(a, pin, mode, corner);
            assert_eq!(points[0], a);
            assert_eq!(*points.last().unwrap(), pin);
        }
    }
}

#[test]
fn full_integer_domain_has_no_overflow_and_no_zero_length_legs() {
    for (a, b) in [
        (p(i64::MIN, i64::MIN), p(i64::MAX, i64::MAX)),
        (p(i64::MIN, i64::MAX), p(i64::MAX, i64::MIN)),
        (p(i64::MIN, 5), p(i64::MAX, 6)),
        (p(i64::MAX, 6), p(i64::MIN, 5)),
    ] {
        for mode in [Mode::Free, Mode::Orthogonal, Mode::Diagonal45] {
            for corner in [Corner::Start, Corner::End] {
                let points = path(a, b, mode, corner);
                assert_eq!(points.first(), Some(&a));
                assert_eq!(points.last(), Some(&b));
                assert!(points.windows(2).all(|pair| pair[0] != pair[1]));
            }
        }
    }
}

#[test]
fn initial_coincident_or_already_constrained_paths_do_not_create_extra_vertices() {
    for mode in [Mode::Free, Mode::Orthogonal, Mode::Diagonal45] {
        for corner in [Corner::Start, Corner::End] {
            assert_eq!(path(p(5, 7), p(5, 7), mode, corner), vec![p(5, 7)]);
            assert_eq!(
                path(p(5, 7), p(5, 99), mode, corner),
                vec![p(5, 7), p(5, 99)]
            );
        }
    }
    for corner in [Corner::Start, Corner::End] {
        assert_eq!(
            path(p(0, 0), p(999, 999), Mode::Diagonal45, corner),
            vec![p(0, 0), p(999, 999)]
        );
    }
    assert_eq!(
        path(p(1, 2), p(98, 51), Mode::Free, Corner::Start),
        vec![p(1, 2), p(98, 51)]
    );
}

#[test]
fn graph_endpoint_uses_shared_45_geometry_with_fixed_opposite_contact() {
    use super::tests::{id, owner};
    use super::{WireEditBasis, WireEndpoint, WireHandle};
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(11), id(90)).unwrap();
    let plan = basis
        .plan_move(
            &WireHandle::Endpoint {
                end: WireEndpoint::From,
            },
            p(2_000_000, 700_000),
            Mode::Diagonal45,
        )
        .unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)], sheet.wires[&id(10)]);
    assert_eq!(after.junctions, sheet.junctions);
    for wire in plan.wires.values() {
        let dx = (i128::from(wire.to.x) - i128::from(wire.from.x)).abs();
        let dy = (i128::from(wire.to.y) - i128::from(wire.from.y)).abs();
        assert!(dx == 0 || dy == 0 || dx == dy);
    }
    assert!(
        plan.wires
            .values()
            .any(|wire| wire.to == p(-2_400_000, 1_200_000))
    );
}

#[test]
fn junction_shortcut_honors_45_posture_without_relaxing_retained_marker_gate() {
    use super::tests::{id, owner};
    use super::{WireEditBasis, WireHandle};
    let sheet = owner();
    let basis = WireEditBasis::capture_junction(&sheet, id(20), id(90)).unwrap();
    let handle = WireHandle::Vertex {
        point: p(-2_400_000, 1_200_000),
    };
    let allowed = basis
        .plan_move(&handle, p(-2_400_000, 700_000), Mode::Diagonal45)
        .unwrap();
    assert_eq!(
        allowed.apply_to_sheet(&sheet).wires[&id(10)],
        sheet.wires[&id(10)]
    );
    assert!(
        basis
            .plan_move(&handle, p(-2_400_000, 900_000), Mode::Diagonal45)
            .is_err(),
        "an arbitrary-angle shortcut must not bypass requested posture or the retained Junction gate"
    );
}
