//! Exact incidence proof for the shared wire-contact service.
use super::exact_wire_contact;
use crate::PointNm;
use eda_engine::ir::geometry::Point;

fn p(x: i64, y: i64) -> Point {
    Point { x, y }
}
fn d(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}

#[test]
fn horizontal_contact_recovers_the_run91_detached_label_case() {
    // run91: DATA0 clicked on a child wire at y=2_110_000 was grid-stored at
    // y=2_100_000, exactly off the wire. The contact point must sit ON the
    // wire's y, at the nearest exact x.
    let at = exact_wire_contact(
        p(0, 2_110_000),
        p(10_000_000, 2_110_000),
        d(5_000_000, 2_100_000),
    )
    .unwrap();
    assert_eq!(at, d(5_000_000, 2_110_000));
}

#[test]
fn vertical_and_diagonal_segments_produce_exact_lattice_incidence() {
    assert_eq!(
        exact_wire_contact(p(7_000, 0), p(7_000, 10_000), d(6_500, 4_321)).unwrap(),
        d(7_000, 4_321)
    );
    // v=(9,6): gcd 3, step (3,2); lattice points are (0,0),(3,2),(6,4),(9,6).
    assert_eq!(
        exact_wire_contact(p(0, 0), p(9, 6), d(4, 3)).unwrap(),
        d(3, 2)
    );
    assert_eq!(
        exact_wire_contact(p(0, 0), p(9, 6), d(5, 4)).unwrap(),
        d(6, 4)
    );
}

#[test]
fn contacts_clamp_to_endpoints_and_never_leave_the_segment() {
    assert_eq!(
        exact_wire_contact(p(0, 0), p(9, 6), d(1_000, 1_000)).unwrap(),
        d(9, 6)
    );
    assert_eq!(
        exact_wire_contact(p(0, 0), p(9, 6), d(-1_000, -1_000)).unwrap(),
        d(0, 0)
    );
}

#[test]
fn degenerate_and_overflowing_segments_fail_closed() {
    assert!(exact_wire_contact(p(5, 5), p(5, 5), d(5, 5)).is_err());
    // Near-degenerate full-range diagonal: the normalized step squares
    // overflow i128 and must refuse rather than approximate.
    assert!(exact_wire_contact(p(0, 0), p(i64::MAX, i64::MAX - 1), d(1, 1)).is_err());
}
