//! Query-only exact coordinates in Q(sqrt(3)); never persisted as source Points.
//! No trigonometric rounding, interval tolerance or authored coordinate changes.
use super::nominal_geometry::{GeometryError, Rational as R};
use std::cmp::Ordering;
type Result<T> = std::result::Result<T, GeometryError>;
fn zero() -> R {
    R::integer(0)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quadratic3 {
    pub a: R,
    pub b: R,
}
impl Quadratic3 {
    pub fn rational(a: R) -> Self {
        Self { a, b: zero() }
    }
    pub fn root(b: R) -> Self {
        Self { a: zero(), b }
    }
    pub fn add(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            a: self.a.checked_add(rhs.a)?,
            b: self.b.checked_add(rhs.b)?,
        })
    }
    pub fn neg(self) -> Result<Self> {
        Ok(Self {
            a: self.a.checked_neg()?,
            b: self.b.checked_neg()?,
        })
    }
    pub fn sub(self, rhs: Self) -> Result<Self> {
        self.add(rhs.neg()?)
    }
    pub fn mul(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            a: self
                .a
                .checked_mul(rhs.a)?
                .checked_add(self.b.checked_mul(rhs.b)?.checked_mul(R::integer(3))?)?,
            b: self
                .a
                .checked_mul(rhs.b)?
                .checked_add(self.b.checked_mul(rhs.a)?)?,
        })
    }
    pub fn square(self) -> Result<Self> {
        self.mul(self)
    }
    pub fn div(self, rhs: Self) -> Result<Self> {
        if rhs.b.numerator == 0 {
            return Ok(Self {
                a: self.a.checked_div(rhs.a)?,
                b: self.b.checked_div(rhs.a)?,
            });
        }
        let denominator = rhs
            .a
            .square()?
            .checked_sub(rhs.b.square()?.checked_mul(R::integer(3))?)?;
        let numerator = self.mul(Self {
            a: rhs.a,
            b: rhs.b.checked_neg()?,
        })?;
        Ok(Self {
            a: numerator.a.checked_div(denominator)?,
            b: numerator.b.checked_div(denominator)?,
        })
    }
    pub fn sign(self) -> Result<i8> {
        sign(self.a, self.b)
    }
    pub fn compare(self, rhs: Self) -> Result<Ordering> {
        Ok(self.sub(rhs)?.sign()?.cmp(&0))
    }
}

pub type ExactPointNm = [Quadratic3; 2];
fn sign(a: R, b: R) -> Result<i8> {
    let sa = a.numerator.signum() as i8;
    let sb = b.numerator.signum() as i8;
    if sa == 0 {
        return Ok(sb);
    }
    if sb == 0 || sa == sb {
        return Ok(sa);
    }
    let an = a.numerator.unsigned_abs();
    let ad = a.denominator as u128;
    let bn = b.numerator.unsigned_abs();
    let bd = b.denominator as u128;
    let product = super::occupied_copper::quadratic_sign::product;
    let order = product(&[an, an, bd, bd])?
        .iter()
        .rev()
        .cmp(product(&[bn, bn, ad, ad, 3])?.iter().rev());
    Ok(match order {
        Ordering::Less => -sa,
        Ordering::Equal => 0,
        Ordering::Greater => sa,
    })
}
impl Quadratic3 {
    /// Explicit extraction for legacy integer consumers; no rounding permitted.
    pub fn integer_nm(self) -> Result<i64> {
        if self.b.numerator != 0 || self.a.denominator != 1 {
            return Err(GeometryError::UnsupportedPadGeometry);
        }
        i64::try_from(self.a.numerator).map_err(|_| GeometryError::ArithmeticRange)
    }
    /// Conservative bounds are exact; they nominate candidates only.
    pub fn floor_ceil_nm(self) -> Result<(i64, i64)> {
        // The final world/index domain is i64. Binary search compares exact
        // field values, avoiding a floating sqrt even in candidate bounds.
        let min = Self::rational(R::integer(i128::from(i64::MIN)));
        let max = Self::rational(R::integer(i128::from(i64::MAX)));
        if self.compare(min)? == Ordering::Less || self.compare(max)? == Ordering::Greater {
            return Err(GeometryError::ArithmeticRange);
        }
        let mut lo = i128::from(i64::MIN);
        let mut hi = i128::from(i64::MAX);
        while lo < hi {
            let mid = lo + (hi - lo + 1) / 2;
            if self.compare(Self::rational(R::integer(mid)))? == Ordering::Less {
                hi = mid - 1;
            } else {
                lo = mid;
            }
        }
        let floor = i64::try_from(lo).map_err(|_| GeometryError::ArithmeticRange)?;
        let ceil = if self.compare(Self::rational(R::integer(lo)))? == Ordering::Equal {
            floor
        } else {
            floor.checked_add(1).ok_or(GeometryError::ArithmeticRange)?
        };
        Ok((floor, ceil))
    }
}
// Transient protocol coefficients are decimal strings, not lossy JSON numbers.
#[derive(serde::Serialize, serde::Deserialize)]
struct Coefficient {
    numerator: String,
    denominator: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Transport {
    a: Coefficient,
    b: Coefficient,
    radicand: u8,
    unit: String,
}
impl serde::Serialize for Quadratic3 {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let coefficient = |r: R| Coefficient {
            numerator: r.numerator.to_string(),
            denominator: r.denominator.to_string(),
        };
        Transport {
            a: coefficient(self.a),
            b: coefficient(self.b),
            radicand: 3,
            unit: "nm".into(),
        }
        .serialize(serializer)
    }
}
impl<'de> serde::Deserialize<'de> for Quadratic3 {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        let value = Transport::deserialize(deserializer)?;
        if value.radicand != 3 || value.unit != "nm" {
            return Err(D::Error::custom("unsupported exact coordinate"));
        }
        let coefficient = |c: Coefficient| -> std::result::Result<R, D::Error> {
            let n = c.numerator.parse().map_err(D::Error::custom)?;
            let d = c.denominator.parse().map_err(D::Error::custom)?;
            R::new(n, d).map_err(|e| D::Error::custom(format!("{e:?}")))
        };
        Ok(Self {
            a: coefficient(value.a)?,
            b: coefficient(value.b)?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_three_sign_inverse_bounds_and_transport() {
        let root = Quadratic3::root(R::integer(1));
        assert_eq!(root.square().unwrap(), Quadratic3::rational(R::integer(3)));
        assert_eq!(root.floor_ceil_nm().unwrap(), (1, 2));
        assert_eq!(root.neg().unwrap().floor_ceil_nm().unwrap(), (-2, -1));
        let value = root
            .add(Quadratic3::rational(R::new(1, 2).unwrap()))
            .unwrap();
        assert_eq!(
            value.div(value).unwrap(),
            Quadratic3::rational(R::integer(1))
        );
        assert_eq!(sign(R::integer(i128::MAX), R::integer(-i128::MAX)), Ok(-1));
        assert_eq!(
            super::super::occupied_copper::quadratic_sign::product(&[
                u128::MAX,
                u128::MAX,
                u128::MAX,
                u128::MAX,
                3
            ])
            .unwrap()[8],
            2
        );
        let wire = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<Quadratic3>(&wire).unwrap(), value);
        assert!(root.integer_nm().is_err());
    }
}
