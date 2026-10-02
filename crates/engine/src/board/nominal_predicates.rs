//! Complete feature candidates for nominal directed circular arc contacts.
use super::nominal_geometry::{
    CertifiedArc, DistanceBoundary, GeometryError, Rational as R, Result, cross, norm,
    radial_gap_boundary, vector,
};
use super::radical_sign::Expression;
use crate::ir::geometry::Point;
use std::cmp::Ordering;
fn dot(a: [R; 2], b: [R; 2]) -> Result<R> {
    a[0].checked_mul(b[0])?.checked_add(a[1].checked_mul(b[1])?)
}
fn scale(a: [R; 2], s: R) -> Result<[R; 2]> {
    Ok([a[0].checked_mul(s)?, a[1].checked_mul(s)?])
}
fn plus(a: [R; 2], b: [R; 2]) -> Result<[R; 2]> {
    Ok([a[0].checked_add(b[0])?, a[1].checked_add(b[1])?])
}
fn minus(a: [R; 2], b: [R; 2]) -> Result<[R; 2]> {
    Ok([a[0].checked_sub(b[0])?, a[1].checked_sub(b[1])?])
}
fn unit_interval(u: R) -> Result<bool> {
    Ok(u.numerator >= 0 && u.compare(R::integer(1))? != Ordering::Greater)
}
fn root_sign(a: R, b: R, k: R) -> Result<i8> {
    let mut e = Expression::new([k, R::integer(0), R::integer(0)])?;
    e.coefficients[0] = a;
    e.coefficients[1] = b;
    e.sign()
}
fn root_direction(arc: CertifiedArc, a: [R; 2], b: [R; 2], k: R) -> Result<bool> {
    let start = arc.radial_vector(arc.from)?;
    let end = arc.radial_vector(arc.to)?;
    let orientation = if arc.counterclockwise { 1 } else { -1 };
    let first = root_sign(cross(start, a)?, cross(start, b)?, k)? * orientation >= 0;
    let last = root_sign(cross(a, end)?, cross(b, end)?, k)? * orientation >= 0;
    if cross(start, end)?.numerator.signum() * i128::from(orientation) >= 0 {
        Ok(first && last)
    } else {
        Ok(first || last)
    }
}
pub fn point_segment_within(p: Point, from: Point, to: Point, half: R) -> Result<bool> {
    point_segment_within_boundary(p, from, to, half, DistanceBoundary::Inclusive)
}
pub fn point_segment_within_boundary(
    p: Point,
    from: Point,
    to: Point,
    half: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    rational_point_segment_within(point(p), point(from), point(to), half, boundary)
}
pub(super) fn point(p: Point) -> [R; 2] {
    [R::integer(i128::from(p.x)), R::integer(i128::from(p.y))]
}
pub(super) fn rational_point_segment_within(
    p: [R; 2],
    from: [R; 2],
    to: [R; 2],
    half: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if half.numerator < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let a = minus(p, from)?;
    let d = minus(to, from)?;
    let len = norm(d)?;
    if len.numerator == 0 {
        return Ok(boundary.accepts(norm(a)?.compare(half.square()?)?));
    }
    let u = dot(a, d)?.checked_div(len)?;
    if !unit_interval(u)? {
        let endpoint = if u.numerator < 0 { from } else { to };
        return Ok(boundary.accepts(norm(minus(p, endpoint)?)?.compare(half.square()?)?));
    }
    Ok(boundary.accepts(
        cross(a, d)?
            .square()?
            .compare(half.square()?.checked_mul(len)?)?,
    ))
}
pub fn arc_line_within(arc: CertifiedArc, from: Point, to: Point, half: R) -> Result<bool> {
    arc_line_within_boundary(arc, from, to, half, DistanceBoundary::Inclusive)
}
pub fn arc_line_within_boundary(
    arc: CertifiedArc,
    from: Point,
    to: Point,
    half: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    rational_arc_segment_within(arc, point(from), point(to), half, boundary)
}
pub(super) fn rational_arc_segment_within(
    arc: CertifiedArc,
    from: [R; 2],
    to: [R; 2],
    half: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if half.numerator < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    if arc.rational_point_within_boundary(from, half, boundary)?
        || arc.rational_point_within_boundary(to, half, boundary)?
        || rational_point_segment_within(point(arc.from), from, to, half, boundary)?
        || rational_point_segment_within(point(arc.to), from, to, half, boundary)?
    {
        return Ok(true);
    }
    let a = minus(minus(from, point(arc.from))?, arc.center)?;
    let d = minus(to, from)?;
    let len = norm(d)?;
    if len.numerator == 0 {
        return Ok(false);
    }
    let u = dot(a, d)?.checked_neg()?.checked_div(len)?;
    let foot = plus(a, scale(d, u)?)?;
    let q = norm(foot)?;
    if unit_interval(u)?
        && arc.contains_direction(foot)?
        && radial_gap_boundary(q, arc.radius_squared, half, boundary)?
    {
        return Ok(true);
    }
    let k = arc.radius_squared.checked_sub(q)?.checked_div(len)?;
    if k.numerator < 0 {
        return Ok(false);
    }
    for direction in [-1, 1] {
        let s = R::integer(direction);
        if root_sign(u, s, k)? >= 0
            && root_sign(R::integer(1).checked_sub(u)?, s.checked_neg()?, k)? >= 0
            && root_direction(arc, foot, scale(d, s)?, k)?
        {
            return Ok(boundary == DistanceBoundary::Inclusive || half.numerator > 0);
        }
    }
    Ok(false)
}
pub fn arc_arc_within(a: CertifiedArc, b: CertifiedArc, half: R) -> Result<bool> {
    arc_arc_within_boundary(a, b, half, DistanceBoundary::Inclusive)
}
pub fn arc_arc_within_boundary(
    a: CertifiedArc,
    b: CertifiedArc,
    half: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if half.numerator < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    if a.point_within_boundary(b.from, half, boundary)?
        || a.point_within_boundary(b.to, half, boundary)?
        || b.point_within_boundary(a.from, half, boundary)?
        || b.point_within_boundary(a.to, half, boundary)?
    {
        return Ok(true);
    }
    let delta = plus(vector(b.from, a.from), minus(b.center, a.center)?)?;
    let len = norm(delta)?;
    if len.numerator == 0 {
        let overlap = a.contains_direction(b.radial_vector(b.from)?)?
            || a.contains_direction(b.radial_vector(b.to)?)?
            || b.contains_direction(a.radial_vector(a.from)?)?
            || b.contains_direction(a.radial_vector(a.to)?)?;
        return Ok(
            overlap && radial_gap_boundary(a.radius_squared, b.radius_squared, half, boundary)?
        );
    }
    // Interior extrema are radial to the center-center line. Check all four
    // orientation pairs, including nested circles; no squared-sign shortcut.
    for sa in [-1, 1] {
        for sb in [-1, 1] {
            if !a.contains_direction(scale(delta, R::integer(sa))?)?
                || !b.contains_direction(scale(delta, R::integer(sb))?)?
            {
                continue;
            }
            let mut e = Expression::new([len, a.radius_squared, b.radius_squared])?;
            e.coefficients[1] = R::integer(1);
            e.coefficients[2] = R::integer(-sa);
            e.coefficients[4] = R::integer(sb);
            e.coefficients[0] = half.checked_neg()?;
            if !boundary.accepts(e.sign()?.cmp(&0)) {
                continue;
            }
            e.coefficients[0] = half;
            if boundary.accepts(0.cmp(&e.sign()?)) {
                return Ok(true);
            }
        }
    }
    // Circle-circle intersection points = foot +/- perpendicular*sqrt(k).
    let u = a
        .radius_squared
        .checked_sub(b.radius_squared)?
        .checked_add(len)?
        .checked_div(len.checked_mul(R::integer(2))?)?;
    let foot = scale(delta, u)?;
    let k = a
        .radius_squared
        .checked_div(len)?
        .checked_sub(u.square()?)?;
    if k.numerator < 0 {
        return Ok(false);
    }
    let perpendicular = [delta[1].checked_neg()?, delta[0]];
    for s in [-1, 1] {
        let d = scale(perpendicular, R::integer(s))?;
        if root_direction(a, foot, d, k)? && root_direction(b, minus(foot, delta)?, d, k)? {
            return Ok(boundary == DistanceBoundary::Inclusive || half.numerator > 0);
        }
    }
    Ok(false)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn upper(x: i64, y: i64, r: i64) -> CertifiedArc {
        CertifiedArc::new(
            Point::new(x - r, y),
            Point::new(x, y + r),
            Point::new(x + r, y),
        )
        .unwrap()
    }
    #[test]
    fn arc_line_intersection_tangent_gap_and_chord_false_positive() {
        let a = upper(0, 0, 5);
        assert!(arc_line_within(a, Point::new(-20, 3), Point::new(20, 3), R::integer(0)).unwrap());
        assert!(arc_line_within(a, Point::new(-20, 6), Point::new(20, 6), R::integer(1)).unwrap());
        assert!(
            !arc_line_within(
                a,
                Point::new(-20, 6),
                Point::new(20, 6),
                R::new(1, 2).unwrap()
            )
            .unwrap()
        );
        assert!(!arc_line_within(a, Point::new(-1, 0), Point::new(1, 0), R::integer(1)).unwrap());
    }
    #[test]
    fn arc_arc_external_tangent_crossing_nested_and_coincident() {
        let a = upper(0, 0, 5);
        assert!(arc_arc_within(a, upper(10, 0, 5), R::integer(0)).unwrap());
        assert!(!arc_arc_within(a, upper(11, 0, 5), R::new(1, 2).unwrap()).unwrap());
        assert!(arc_arc_within(a, upper(6, 0, 5), R::integer(0)).unwrap());
        assert!(arc_arc_within(a, upper(0, 0, 4), R::integer(1)).unwrap());
        assert!(!arc_arc_within(a, upper(0, 0, 4), R::new(1, 2).unwrap()).unwrap());
        assert!(arc_arc_within(a, a, R::integer(0)).unwrap());
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    #[test]
    fn exact_clearance_equality_is_not_a_violation() {
        let arc = CertifiedArc::new(Point::new(-5, 0), Point::new(0, 5), Point::new(5, 0)).unwrap();
        assert!(
            arc_line_within(arc, Point::new(-20, 6), Point::new(20, 6), R::integer(1)).unwrap()
        );
        assert!(
            !arc_line_within_boundary(
                arc,
                Point::new(-20, 6),
                Point::new(20, 6),
                R::integer(1),
                DistanceBoundary::Strict
            )
            .unwrap()
        );
        assert!(
            arc_line_within_boundary(
                arc,
                Point::new(-20, 6),
                Point::new(20, 6),
                R::new(3, 2).unwrap(),
                DistanceBoundary::Strict
            )
            .unwrap()
        );
        assert!(
            !arc_arc_within_boundary(arc, arc, R::integer(0), DistanceBoundary::Strict).unwrap()
        );
        assert!(
            arc_arc_within_boundary(arc, arc, R::new(1, 2).unwrap(), DistanceBoundary::Strict)
                .unwrap()
        );
    }
}

#[cfg(test)]
mod physical_scale_tests {
    use super::*;
    #[test]
    fn millimeter_rational_center_crossing_is_certified() {
        let a = CertifiedArc::new(
            Point::new(0, 0),
            Point::new(1_000_000, 3_000_000),
            Point::new(3_000_000, 0),
        )
        .unwrap();
        let b = CertifiedArc::new(
            Point::new(2_000_000, 0),
            Point::new(3_000_000, 3_000_000),
            Point::new(5_000_000, 0),
        )
        .unwrap();
        // Equal circles, centers 2mm apart, radius sqrt(130)/6 mm.
        // Their upper intersection lies on both major sweeps. No shared anchors.
        assert_eq!(arc_arc_within(a, b, R::integer(0)), Ok(true));
    }
}
