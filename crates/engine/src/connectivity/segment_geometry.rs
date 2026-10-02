//! Exact integer segment incidence shared by logical and physical occurrence owners.
use crate::ir::geometry::Point;

pub(super) fn point_on_wire_segment(point: Point, a: Point, b: Point) -> bool {
    if point.x < a.x.min(b.x)
        || point.x > a.x.max(b.x)
        || point.y < a.y.min(b.y)
        || point.y > a.y.max(b.y)
    {
        return false;
    }
    orientation(a, b, point) == std::cmp::Ordering::Equal
}

/// Exact incidence of two closed authored path segments, including contact and
/// overlap. This is geometric evidence, not a scalar wire-junction rule.
pub(super) fn segments_touch(a: Point, b: Point, c: Point, d: Point) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);
    use std::cmp::Ordering::Equal;
    (ab_c == Equal && point_on_wire_segment(c, a, b))
        || (ab_d == Equal && point_on_wire_segment(d, a, b))
        || (cd_a == Equal && point_on_wire_segment(a, c, d))
        || (cd_b == Equal && point_on_wire_segment(b, c, d))
        || (ab_c != Equal
            && ab_d != Equal
            && ab_c != ab_d
            && cd_a != Equal
            && cd_b != Equal
            && cd_a != cd_b)
}

fn orientation(a: Point, b: Point, p: Point) -> std::cmp::Ordering {
    // Full-domain i64 differences fit i128; product magnitudes fit u128.
    // Comparing signed products avoids overflowing a determinant subtraction.
    fn product(a: i128, b: i128) -> (bool, u128) {
        let magnitude = a.unsigned_abs() * b.unsigned_abs();
        (magnitude != 0 && (a < 0) != (b < 0), magnitude)
    }
    let left = product(
        i128::from(b.x) - i128::from(a.x),
        i128::from(p.y) - i128::from(a.y),
    );
    let right = product(
        i128::from(b.y) - i128::from(a.y),
        i128::from(p.x) - i128::from(a.x),
    );
    match (left.0, right.0) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => left.1.cmp(&right.1),
        (true, true) => right.1.cmp(&left.1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_integer_domain_and_degenerate_incidence() {
        let a = Point::new(i64::MIN, i64::MIN);
        let b = Point::new(i64::MAX, i64::MAX);
        assert!(point_on_wire_segment(Point::new(0, 0), a, b));
        assert!(!point_on_wire_segment(Point::new(0, 1), a, b));
        assert!(point_on_wire_segment(a, a, a));
        assert!(!point_on_wire_segment(b, a, a));
        assert!(point_on_wire_segment(
            Point::new(0, 0),
            Point::new(i64::MIN, 0),
            Point::new(i64::MAX, 0)
        ));
    }
    #[test]
    fn bus_path_contact_full_domain_crossing_overlap_and_gap() {
        let a = Point::new(i64::MIN, i64::MIN);
        let b = Point::new(i64::MAX, i64::MAX);
        assert!(segments_touch(
            a,
            b,
            Point::new(i64::MIN, i64::MAX),
            Point::new(i64::MAX, i64::MIN)
        ));
        assert!(segments_touch(a, b, Point::new(0, 0), Point::new(1, 1)));
        assert!(segments_touch(a, b, Point::zero(), Point::zero()));
        assert!(!segments_touch(
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(11, 0),
            Point::new(20, 0)
        ));
        assert!(!segments_touch(
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(0, 1),
            Point::new(10, 1)
        ));
    }
}
