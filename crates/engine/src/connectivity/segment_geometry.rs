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
    // An i64 difference fits signed i128. Its absolute product fits unsigned
    // i128, including the full i64 coordinate domain. Compare signed products
    // rather than subtracting them or overflowing an i64 determinant.
    fn product(a: i128, b: i128) -> (bool, u128) {
        let magnitude = a.unsigned_abs() * b.unsigned_abs();
        (magnitude != 0 && (a < 0) != (b < 0), magnitude)
    }
    product(
        i128::from(point.y) - i128::from(a.y),
        i128::from(b.x) - i128::from(a.x),
    ) == product(
        i128::from(point.x) - i128::from(a.x),
        i128::from(b.y) - i128::from(a.y),
    )
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
}
