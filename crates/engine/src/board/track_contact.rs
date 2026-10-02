//! Source Track predicates shared by contact authority and nominal DRC.
use super::{
    Track,
    nominal_geometry::{
        CertifiedArc, DistanceBoundary, GeometryError, Rational, Result, cross, vector,
    },
    nominal_predicates::{
        arc_arc_within_boundary, arc_line_within_boundary, point_segment_within_boundary,
    },
};
use crate::ir::geometry::Point;
use std::cmp::Ordering;

/// `extra` is nominal clearance beyond the two exact half-widths.
pub fn tracks_within(a: &Track, b: &Track, extra: i64, boundary: DistanceBoundary) -> Result<bool> {
    if a.width <= 0 || b.width <= 0 || extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let half = Rational::new(i128::from(a.width) + i128::from(b.width), 2)?
        .checked_add(Rational::integer(i128::from(extra)))?;
    match (a.midpoint, b.midpoint) {
        (Some(am), Some(bm)) => arc_arc_within_boundary(
            CertifiedArc::new(a.from, am, a.to)?,
            CertifiedArc::new(b.from, bm, b.to)?,
            half,
            boundary,
        ),
        (Some(am), None) => arc_line_within_boundary(
            CertifiedArc::new(a.from, am, a.to)?,
            b.from,
            b.to,
            half,
            boundary,
        ),
        (None, Some(bm)) => arc_line_within_boundary(
            CertifiedArc::new(b.from, bm, b.to)?,
            a.from,
            a.to,
            half,
            boundary,
        ),
        (None, None) => segments_within(a.from, a.to, b.from, b.to, half, boundary),
    }
}
fn segments_within(
    a: Point,
    b: Point,
    c: Point,
    d: Point,
    half: Rational,
    boundary: DistanceBoundary,
) -> Result<bool> {
    for (p, from, to) in [(a, c, d), (b, c, d), (c, a, b), (d, a, b)] {
        if point_segment_within_boundary(p, from, to, half, boundary)? {
            return Ok(true);
        }
    }
    let ab = vector(b, a);
    let cd = vector(d, c);
    let ca = vector(c, a);
    let det = cross(ab, cd)?;
    if det.numerator == 0 {
        return Ok(false);
    }
    let u = cross(ca, cd)?.checked_div(det)?;
    let v = cross(ca, ab)?.checked_div(det)?;
    let within = |value: Rational| -> Result<bool> {
        Ok(value.numerator >= 0 && value.compare(Rational::integer(1))? != Ordering::Greater)
    };
    Ok(within(u)? && within(v)? && (boundary == DistanceBoundary::Inclusive || half.numerator > 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    fn track(from: Point, to: Point, width: i64) -> Track {
        Track::straight(Uuid::nil(), Uuid::nil(), from, to, width, 1)
    }
    #[test]
    fn round_cap_half_nanometer_contact_and_strict_clearance() {
        let a = track(Point::new(0, 0), Point::new(10, 0), 1);
        let b = track(Point::new(0, 1), Point::new(10, 1), 1);
        assert!(tracks_within(&a, &b, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(!tracks_within(&a, &b, 0, DistanceBoundary::Strict).unwrap());
        assert!(tracks_within(&a, &b, 1, DistanceBoundary::Strict).unwrap());
        let c = track(Point::new(5, -10), Point::new(5, 10), 1);
        assert!(tracks_within(&a, &c, 0, DistanceBoundary::Strict).unwrap());
    }
    #[test]
    fn arc_pair_classification_uses_locus_instead_of_chord() {
        let mut a = track(Point::new(-5, 0), Point::new(5, 0), 1);
        a.midpoint = Some(Point::new(0, 5));
        let b = track(Point::new(-1, 0), Point::new(1, 0), 1);
        assert!(!tracks_within(&a, &b, 0, DistanceBoundary::Inclusive).unwrap());
        let c = track(Point::new(-20, 6), Point::new(20, 6), 1);
        assert!(tracks_within(&a, &c, 0, DistanceBoundary::Inclusive).unwrap());
        assert!(!tracks_within(&a, &c, 0, DistanceBoundary::Strict).unwrap());
    }
}
