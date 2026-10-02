//! Checked quadratic field Q(sqrt(2)) for exact odd eighth-turn geometry.
//! No trigonometric rounding, interval tolerance or authored coordinate changes.
use super::*;
#[derive(Clone, Copy, Debug)]
pub(super) struct Q {
    pub a: R,
    pub b: R,
}
impl Q {
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
                .checked_add(self.b.checked_mul(rhs.b)?.checked_mul(R::integer(2))?)?,
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
        let denominator = rhs
            .a
            .square()?
            .checked_sub(rhs.b.square()?.checked_mul(R::integer(2))?)?;
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
        super::quadratic_sign::sign(self.a, self.b)
    }
    pub fn compare(self, rhs: Self) -> Result<Ordering> {
        Ok(self.sub(rhs)?.sign()?.cmp(&0))
    }
}
pub(super) type V = [Q; 2];
pub(super) fn difference(a: V, b: V) -> Result<V> {
    Ok([a[0].sub(b[0])?, a[1].sub(b[1])?])
}
pub(super) fn sum(a: V, b: V) -> Result<V> {
    Ok([a[0].add(b[0])?, a[1].add(b[1])?])
}
pub(super) fn scaled(a: V, s: Q) -> Result<V> {
    Ok([a[0].mul(s)?, a[1].mul(s)?])
}
pub(super) fn dot(a: V, b: V) -> Result<Q> {
    a[0].mul(b[0])?.add(a[1].mul(b[1])?)
}
pub(super) fn determinant(a: V, b: V) -> Result<Q> {
    a[0].mul(b[1])?.sub(a[1].mul(b[0])?)
}
pub(super) fn squared(a: V) -> Result<Q> {
    dot(a, a)
}
pub(super) fn rational_point(p: P) -> V {
    [Q::rational(p[0]), Q::rational(p[1])]
}
/// Sign(a+b*sqrt(k)) in the ordered real embedding, with k in Q(sqrt(2)).
/// Compare squares only when a and b have opposite signs. Equality is exact.
pub(super) fn root_sign(a: Q, b: Q, k: Q) -> Result<i8> {
    let sk = k.sign()?;
    if sk < 0 {
        return Err(GeometryError::ArithmeticRange);
    }
    let sa = a.sign()?;
    if sk == 0 {
        return Ok(sa);
    }
    let sb = b.sign()?;
    if sa == 0 {
        return Ok(sb);
    }
    if sb == 0 || sa == sb {
        return Ok(sa);
    }
    Ok(a.square()?.sub(b.square()?.mul(k)?)?.sign()? * sa)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quadratic_and_nested_radical_signs_preserve_exact_equality_and_range() {
        let root = Q::root(R::integer(1));
        let one = Q::rational(R::integer(1));
        assert_eq!(
            root.square()
                .unwrap()
                .compare(Q::rational(R::integer(2)))
                .unwrap(),
            Ordering::Equal
        );
        let sum = root.add(one).unwrap();
        let k = Q {
            a: R::integer(3),
            b: R::integer(2),
        };
        assert_eq!(root_sign(sum.neg().unwrap(), one, k).unwrap(), 0);
        let step = Q::rational(R::new(1, 1_000_000_000).unwrap());
        assert_eq!(
            root_sign(sum.neg().unwrap().add(step).unwrap(), one, k).unwrap(),
            1
        );
        assert_eq!(
            root_sign(sum.neg().unwrap().sub(step).unwrap(), one, k).unwrap(),
            -1
        );
        assert_eq!(sum.div(sum).unwrap().compare(one).unwrap(), Ordering::Equal);
        assert!(matches!(
            one.div(Q::rational(zero())),
            Err(GeometryError::ArithmeticRange)
        ));
        assert_eq!(
            root_sign(one, one, Q::rational(R::integer(-1))),
            Err(GeometryError::ArithmeticRange)
        );
    }
}
