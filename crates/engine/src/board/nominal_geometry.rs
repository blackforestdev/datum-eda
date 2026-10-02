//! Certified nominal geometry. Every integer/rational operation is checked.
//! Unsupported arithmetic is explicit; no epsilon or rounded-center fallback.
use crate::ir::geometry::Point;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    InvalidArc,
    InvalidWidth,
    ArithmeticRange,
}
pub(super) type Result<T> = std::result::Result<T, GeometryError>;

/// Contacts include equality; minimum-clearance violations exclude equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceBoundary {
    Inclusive,
    Strict,
}
impl DistanceBoundary {
    pub(super) fn accepts(self, ordering: Ordering) -> bool {
        ordering == Ordering::Less || (self == Self::Inclusive && ordering == Ordering::Equal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub numerator: i128,
    pub denominator: i128,
}
impl Rational {
    pub fn new(n: i128, d: i128) -> Result<Self> {
        if d == 0 {
            return Err(GeometryError::ArithmeticRange);
        }
        let n = if d < 0 {
            n.checked_neg().ok_or(GeometryError::ArithmeticRange)?
        } else {
            n
        };
        let d = d.checked_abs().ok_or(GeometryError::ArithmeticRange)?;
        let mut a = n.unsigned_abs();
        let mut b = d as u128;
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        let g = i128::try_from(a).map_err(|_| GeometryError::ArithmeticRange)?;
        Ok(Self {
            numerator: n / g,
            denominator: d / g,
        })
    }
    pub fn integer(n: i128) -> Self {
        Self {
            numerator: n,
            denominator: 1,
        }
    }
    pub fn checked_add(self, rhs: Self) -> Result<Self> {
        let g = gcd(self.denominator as u128, rhs.denominator as u128) as i128;
        let left = rhs.denominator / g;
        let right = self.denominator / g;
        Self::new(
            add(mul(self.numerator, left)?, mul(rhs.numerator, right)?)?,
            mul(self.denominator, left)?,
        )
    }
    pub fn checked_neg(self) -> Result<Self> {
        Self::new(
            self.numerator
                .checked_neg()
                .ok_or(GeometryError::ArithmeticRange)?,
            self.denominator,
        )
    }
    pub fn checked_sub(self, rhs: Self) -> Result<Self> {
        self.checked_add(rhs.checked_neg()?)
    }
    pub fn checked_mul(self, rhs: Self) -> Result<Self> {
        let g1 = gcd(self.numerator.unsigned_abs(), rhs.denominator as u128) as i128;
        let g2 = gcd(rhs.numerator.unsigned_abs(), self.denominator as u128) as i128;
        Self::new(
            mul(self.numerator / g1, rhs.numerator / g2)?,
            mul(self.denominator / g2, rhs.denominator / g1)?,
        )
    }
    pub fn checked_div(self, rhs: Self) -> Result<Self> {
        self.checked_mul(Self::new(rhs.denominator, rhs.numerator)?)
    }
    pub fn square(self) -> Result<Self> {
        self.checked_mul(self)
    }
    pub fn compare(self, rhs: Self) -> Result<Ordering> {
        let g = gcd(self.denominator as u128, rhs.denominator as u128) as i128;
        Ok(mul(self.numerator, rhs.denominator / g)?
            .cmp(&mul(rhs.numerator, self.denominator / g)?))
    }
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
fn add(a: i128, b: i128) -> Result<i128> {
    a.checked_add(b).ok_or(GeometryError::ArithmeticRange)
}
fn mul(a: i128, b: i128) -> Result<i128> {
    a.checked_mul(b).ok_or(GeometryError::ArithmeticRange)
}
pub(super) fn cross(a: [Rational; 2], b: [Rational; 2]) -> Result<Rational> {
    a[0].checked_mul(b[1])?.checked_sub(a[1].checked_mul(b[0])?)
}
pub(super) fn norm(a: [Rational; 2]) -> Result<Rational> {
    a[0].square()?.checked_add(a[1].square()?)
}
pub(super) fn vector(a: Point, b: Point) -> [Rational; 2] {
    [
        Rational::integer(i128::from(a.x) - i128::from(b.x)),
        Rational::integer(i128::from(a.y) - i128::from(b.y)),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CertifiedArc {
    pub from: Point,
    pub midpoint: Point,
    pub to: Point,
    /// Exact center relative to from; never a quantized graphic angle/center.
    pub center: [Rational; 2],
    pub radius_squared: Rational,
    pub counterclockwise: bool,
}
impl CertifiedArc {
    pub fn new(from: Point, midpoint: Point, to: Point) -> Result<Self> {
        if from == midpoint || from == to || midpoint == to {
            return Err(GeometryError::InvalidArc);
        }
        let u = vector(midpoint, from);
        let v = vector(to, from);
        let determinant = cross(u, v)?;
        if determinant.numerator == 0 {
            return Err(GeometryError::InvalidArc);
        }
        let denominator = determinant.checked_mul(Rational::integer(2))?;
        let un = norm(u)?;
        let vn = norm(v)?;
        let center = [
            un.checked_mul(v[1])?
                .checked_sub(vn.checked_mul(u[1])?)?
                .checked_div(denominator)?,
            u[0].checked_mul(vn)?
                .checked_sub(v[0].checked_mul(un)?)?
                .checked_div(denominator)?,
        ];
        Ok(Self {
            from,
            midpoint,
            to,
            center,
            radius_squared: norm(center)?,
            counterclockwise: determinant.numerator > 0,
        })
    }
    pub fn radial_vector(self, point: Point) -> Result<[Rational; 2]> {
        let v = vector(point, self.from);
        Ok([
            v[0].checked_sub(self.center[0])?,
            v[1].checked_sub(self.center[1])?,
        ])
    }
    /// Direction only: multiplication by any positive scale preserves membership.
    pub fn contains_direction(self, direction: [Rational; 2]) -> Result<bool> {
        if direction.iter().all(|x| x.numerator == 0) {
            return Ok(false);
        }
        let start = self.radial_vector(self.from)?;
        let end = self.radial_vector(self.to)?;
        let signed = |a, b| -> Result<Rational> {
            let c = cross(a, b)?;
            if self.counterclockwise {
                Ok(c)
            } else {
                c.checked_neg()
            }
        };
        let first = signed(start, direction)?.numerator >= 0;
        let last = signed(direction, end)?.numerator >= 0;
        if signed(start, end)?.numerator >= 0 {
            Ok(first && last)
        } else {
            Ok(first || last)
        }
    }
    pub fn contains_copper_point(self, point: Point, width: i64) -> Result<bool> {
        if width <= 0 {
            return Err(GeometryError::InvalidWidth);
        }
        self.point_within(point, Rational::new(i128::from(width), 2)?)
    }
    pub(super) fn point_within(self, point: Point, half: Rational) -> Result<bool> {
        self.point_within_boundary(point, half, DistanceBoundary::Inclusive)
    }
    pub(super) fn point_within_boundary(
        self,
        point: Point,
        half: Rational,
        boundary: DistanceBoundary,
    ) -> Result<bool> {
        let radial = self.radial_vector(point)?;
        if self.contains_direction(radial)? || radial.iter().all(|x| x.numerator == 0) {
            return radial_gap_boundary(norm(radial)?, self.radius_squared, half, boundary);
        }
        Ok(
            boundary.accepts(norm(vector(point, self.from))?.compare(half.square()?)?)
                || boundary.accepts(norm(vector(point, self.to))?.compare(half.square()?)?),
        )
    }
}
/// |sqrt(distance_squared)-sqrt(radius_squared)| <= half_width, squared
/// only after checking the sign of the radical-free left side.
pub fn radial_gap_within(
    distance_squared: Rational,
    radius_squared: Rational,
    half_width: Rational,
) -> Result<bool> {
    radial_gap_boundary(
        distance_squared,
        radius_squared,
        half_width,
        DistanceBoundary::Inclusive,
    )
}
pub(super) fn radial_gap_boundary(
    distance_squared: Rational,
    radius_squared: Rational,
    half_width: Rational,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if distance_squared.numerator < 0 || radius_squared.numerator < 0 || half_width.numerator < 0 {
        return Err(GeometryError::ArithmeticRange);
    }
    let lhs = distance_squared
        .checked_add(radius_squared)?
        .checked_sub(half_width.square()?)?;
    let product = distance_squared
        .checked_mul(radius_squared)?
        .checked_mul(Rational::integer(4))?;
    if lhs.numerator < 0 {
        return Ok(true);
    }
    if lhs.numerator == 0 {
        return Ok(boundary == DistanceBoundary::Inclusive || product.numerator > 0);
    }
    Ok(boundary.accepts(lhs.square()?.compare(product)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_arc_locus_tangency_half_nm_and_mirror() {
        let arc = CertifiedArc::new(Point::new(0, 0), Point::new(5, 5), Point::new(10, 0)).unwrap();
        assert_eq!(arc.center, [Rational::integer(5), Rational::integer(0)]);
        assert!(arc.contains_copper_point(Point::new(5, 6), 2).unwrap());
        assert!(!arc.contains_copper_point(Point::new(5, 6), 1).unwrap());
        assert!(!arc.contains_copper_point(Point::new(5, 0), 2).unwrap());
        assert!(!arc.contains_copper_point(Point::new(5, -5), 2).unwrap());
        let reverse = CertifiedArc::new(arc.to, arc.midpoint, arc.from).unwrap();
        let mirror =
            CertifiedArc::new(Point::new(0, 0), Point::new(5, -5), Point::new(10, 0)).unwrap();
        for p in [Point::new(5, 6), Point::new(0, 1), Point::new(5, -5)] {
            assert_eq!(
                arc.contains_copper_point(p, 2),
                reverse.contains_copper_point(p, 2)
            );
            assert_eq!(
                arc.contains_copper_point(p, 2),
                mirror.contains_copper_point(Point::new(p.x, -p.y), 2)
            );
        }
    }
    #[test]
    fn rational_center_major_sweep_and_invalid_arithmetic_are_explicit() {
        let arc = CertifiedArc::new(Point::new(0, 0), Point::new(1, 3), Point::new(3, 0)).unwrap();
        assert_eq!(
            arc.center,
            [Rational::new(3, 2).unwrap(), Rational::new(7, 6).unwrap()]
        );
        assert!(arc.contains_copper_point(arc.midpoint, 1).unwrap());
        assert!(!arc.contains_copper_point(Point::new(1, 0), 1).unwrap());
        assert_eq!(
            CertifiedArc::new(Point::new(0, 0), Point::new(1, 0), Point::new(2, 0)),
            Err(GeometryError::InvalidArc)
        );
        assert_eq!(
            CertifiedArc::new(
                Point::new(i64::MIN, i64::MIN),
                Point::new(i64::MAX, 0),
                Point::new(0, i64::MAX)
            ),
            Err(GeometryError::ArithmeticRange)
        );
    }
}
