//! Bounded mixed contact scratch Q(sqrt(2))(sqrt(3)); source data stays unchanged.
//! No trigonometric rounding, interval tolerance or authored coordinate changes.
use super::*;
#[derive(Clone, Copy, Debug)]
pub(super) struct Q {
    pub a: super::quadratic::Q,
    pub b: super::quadratic::Q,
}
impl Q {
    pub fn exact(p: crate::board::quadratic3::Quadratic3) -> Self {
        Self {
            a: super::quadratic::Q::rational(p.a),
            b: super::quadratic::Q::rational(p.b),
        }
    }
    pub fn two(p: super::quadratic::Q) -> Self {
        Self {
            a: p,
            b: super::quadratic::Q::rational(zero()),
        }
    }
    pub fn rational(a: R) -> Self {
        Self {
            a: super::quadratic::Q::rational(a),
            b: super::quadratic::Q::rational(zero()),
        }
    }
    pub fn add(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            a: self.a.add(rhs.a)?,
            b: self.b.add(rhs.b)?,
        })
    }
    pub fn neg(self) -> Result<Self> {
        Ok(Self {
            a: self.a.neg()?,
            b: self.b.neg()?,
        })
    }
    pub fn sub(self, rhs: Self) -> Result<Self> {
        self.add(rhs.neg()?)
    }
    pub fn mul(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            a: self.a.mul(rhs.a)?.add(
                self.b
                    .mul(rhs.b)?
                    .mul(super::quadratic::Q::rational(R::integer(3)))?,
            )?,
            b: self.a.mul(rhs.b)?.add(self.b.mul(rhs.a)?)?,
        })
    }
    pub fn square(self) -> Result<Self> {
        self.mul(self)
    }
    pub fn div(self, rhs: Self) -> Result<Self> {
        if rhs.b.a.numerator == 0 && rhs.b.b.numerator == 0 {
            return Ok(Self {
                a: divide_two(self.a, rhs.a)?,
                b: divide_two(self.b, rhs.a)?,
            });
        }
        let denominator = rhs.a.square()?.sub(
            rhs.b
                .square()?
                .mul(super::quadratic::Q::rational(R::integer(3)))?,
        )?;
        let numerator = self.mul(Self {
            a: rhs.a,
            b: rhs.b.neg()?,
        })?;
        Ok(Self {
            a: divide_two(numerator.a, denominator)?,
            b: divide_two(numerator.b, denominator)?,
        })
    }
    pub fn sign(self) -> Result<i8> {
        if self.a.b.numerator == 0 && self.b.b.numerator == 0 {
            return crate::board::quadratic3::Quadratic3 {
                a: self.a.a,
                b: self.b.a,
            }
            .sign();
        }
        {
            let sa = self.a.sign()?;
            let sb = self.b.sign()?;
            if sa == 0 {
                return Ok(sb);
            }
            if sb == 0 || sa == sb {
                return Ok(sa);
            }
            Ok(self
                .a
                .square()?
                .sub(
                    self.b
                        .square()?
                        .mul(super::quadratic::Q::rational(R::integer(3)))?,
                )?
                .sign()?
                * sa)
        }
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

fn divide_two(a: super::quadratic::Q, b: super::quadratic::Q) -> Result<super::quadratic::Q> {
    if b.b.numerator == 0 {
        Ok(super::quadratic::Q {
            a: a.a.checked_div(b.a)?,
            b: a.b.checked_div(b.a)?,
        })
    } else {
        a.div(b)
    }
}
