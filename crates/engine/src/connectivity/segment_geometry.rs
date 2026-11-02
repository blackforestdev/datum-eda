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

/// Why a segment contact cannot yield one exact authored integer point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentIntersectionError {
    AmbiguousOverlap,
    NonIntegral,
    ArithmeticOverflow,
}
impl std::fmt::Display for SegmentIntersectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AmbiguousOverlap => "Overlapping wires have no unique intersection point",
            Self::NonIntegral => "Wire intersection is not an exact integer nanometre point",
            Self::ArithmeticOverflow => "Wire intersection exceeds exact arithmetic capacity",
        })
    }
}
impl std::error::Error for SegmentIntersectionError {}

/// Return the unique exact integer intersection of two closed segments.
/// Disjoint segments return `None`. Overlap, fractional coordinates and checked
/// arithmetic failure are distinct errors; no rounded contact is manufactured.
pub fn exact_segment_intersection(
    a: Point,
    b: Point,
    c: Point,
    d: Point,
) -> Result<Option<Point>, SegmentIntersectionError> {
    use SegmentIntersectionError::{AmbiguousOverlap, ArithmeticOverflow};
    if !segments_touch(a, b, c, d) {
        return Ok(None);
    }
    // Covers T contacts, shared endpoints, and zero-length segments without
    // determinant arithmetic. Two distinct common endpoints imply a continuum.
    let mut endpoint = None;
    for p in [a, b, c, d] {
        if point_on_wire_segment(p, a, b) && point_on_wire_segment(p, c, d) {
            if endpoint.is_some_and(|prior| prior != p) {
                return Err(AmbiguousOverlap);
            }
            endpoint = Some(p);
        }
    }
    if endpoint.is_some() {
        return Ok(endpoint);
    }
    fn difference(a: Point, b: Point) -> (i128, i128) {
        (
            i128::from(a.x) - i128::from(b.x),
            i128::from(a.y) - i128::from(b.y),
        )
    }
    fn cross(a: (i128, i128), b: (i128, i128)) -> Option<i128> {
        a.0.checked_mul(b.1)?.checked_sub(a.1.checked_mul(b.0)?)
    }
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    }
    // Exact incidence already established an interior crossing, so t is
    // strictly between zero and one; numerator and denominator share a sign.
    let r = difference(b, a);
    let s = difference(d, c);
    let numerator = cross(difference(c, a), s)
        .ok_or(ArithmeticOverflow)?
        .unsigned_abs();
    let denominator = cross(r, s).ok_or(ArithmeticOverflow)?.unsigned_abs();
    if denominator == 0 {
        return Err(AmbiguousOverlap);
    }
    let common = gcd(numerator, denominator);
    let (n, d) = (numerator / common, denominator / common);
    fn coordinate(
        base: i64,
        delta: i128,
        n: u128,
        d: u128,
    ) -> Result<i64, SegmentIntersectionError> {
        use SegmentIntersectionError::{ArithmeticOverflow, NonIntegral};
        if delta == 0 {
            return Ok(base);
        }
        // Cancel before multiplying. Because n/d is reduced, any remaining
        // denominator proves this coordinate is fractional.
        let common = gcd(delta.unsigned_abs(), d);
        if d / common != 1 {
            return Err(NonIntegral);
        }
        let common = i128::try_from(common).map_err(|_| ArithmeticOverflow)?;
        let n = i128::try_from(n).map_err(|_| ArithmeticOverflow)?;
        let offset = (delta / common).checked_mul(n).ok_or(ArithmeticOverflow)?;
        let value = i128::from(base)
            .checked_add(offset)
            .ok_or(ArithmeticOverflow)?;
        i64::try_from(value).map_err(|_| ArithmeticOverflow)
    }
    let point = Point::new(coordinate(a.x, r.0, n, d)?, coordinate(a.y, r.1, n, d)?);
    Ok(Some(point))
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
    #[test]
    fn exact_intersection_off_grid_t_diagonal_and_endpoint_symmetry() {
        let p = Point::new;
        for (a, b, c, d, expected) in [
            (
                p(0, 370_000),
                p(900_000, 370_000),
                p(250_000, 0),
                p(250_000, 900_000),
                p(250_000, 370_000),
            ),
            (p(-20, 7), p(20, 7), p(3, 7), p(3, 20), p(3, 7)),
            (p(-7, -7), p(9, 9), p(-7, 9), p(9, -7), p(1, 1)),
            (p(0, 0), p(10, 0), p(10, 0), p(20, 0), p(10, 0)),
            (p(3, 7), p(3, 7), p(-20, 7), p(20, 7), p(3, 7)),
            (p(3, 7), p(3, 7), p(3, 7), p(3, 7), p(3, 7)),
            (
                p(i64::MIN, 0),
                p(i64::MAX, 0),
                p(i64::MAX, 0),
                p(i64::MAX, 1),
                p(i64::MAX, 0),
            ),
        ] {
            for (a, b) in [(a, b), (b, a)] {
                for (c, d) in [(c, d), (d, c)] {
                    assert_eq!(exact_segment_intersection(a, b, c, d), Ok(Some(expected)));
                    assert_eq!(exact_segment_intersection(c, d, a, b), Ok(Some(expected)));
                }
            }
        }
    }

    #[test]
    fn exact_intersection_fractional_parallel_overlap_and_overflow_are_distinct() {
        use SegmentIntersectionError::{AmbiguousOverlap, ArithmeticOverflow, NonIntegral};
        let p = Point::new;
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(1, 1), p(0, 1), p(1, 0)),
            Err(NonIntegral)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(6, 3), p(3, -1), p(3, 5)),
            Err(NonIntegral)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(10, 0), p(0, 1), p(10, 1)),
            Ok(None)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(10, 0), p(11, 0), p(20, 0)),
            Ok(None)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(0, 0), p(1, 0), p(1, 0)),
            Ok(None)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(10, 0), p(5, 0), p(20, 0)),
            Err(AmbiguousOverlap)
        );
        assert_eq!(
            exact_segment_intersection(p(0, 0), p(10, 0), p(10, 0), p(0, 0)),
            Err(AmbiguousOverlap)
        );
        assert_eq!(
            exact_segment_intersection(
                p(i64::MIN, i64::MIN),
                p(i64::MAX, i64::MAX),
                p(i64::MIN, i64::MAX),
                p(i64::MAX, i64::MIN)
            ),
            Err(ArithmeticOverflow)
        );
    }
}
