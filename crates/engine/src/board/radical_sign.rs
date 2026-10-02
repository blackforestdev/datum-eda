//! Sign of at most three real square roots by checked exact squaring.
//! Coefficients use the basis indexed by a bit-mask of radicands. This is
//! bounded arithmetic, not a floating-point filter or tolerance.
use super::nominal_geometry::{GeometryError, Rational, Result};
#[derive(Clone)]
pub(super) struct Expression {
    pub coefficients: [Rational; 8],
    pub radicands: [Rational; 3],
}
impl Expression {
    pub fn new(radicands: [Rational; 3]) -> Result<Self> {
        if radicands.iter().any(|r| r.numerator < 0) {
            return Err(GeometryError::ArithmeticRange);
        }
        Ok(Self {
            coefficients: [Rational::integer(0); 8],
            radicands,
        })
    }
    pub fn sign(&self) -> Result<i8> {
        let mut coefficients = self.coefficients;
        for (k, d) in self.radicands.iter().enumerate() {
            let n = d.numerator as u128;
            let den = d.denominator as u128;
            let rn = integer_sqrt(n);
            let rd = integer_sqrt(den);
            if rn * rn == n && rd * rd == den {
                let root = Rational::new(rn as i128, rd as i128)?;
                for mask in 0..8 {
                    if mask & (1 << k) != 0 {
                        let target = mask ^ (1 << k);
                        coefficients[target] = coefficients[target]
                            .checked_add(coefficients[mask].checked_mul(root)?)?;
                        coefficients[mask] = Rational::integer(0);
                    }
                }
            }
        }
        sign(&coefficients, &self.radicands, 3)
    }
}
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << ((128 - n.leading_zeros()).div_ceil(2));
    loop {
        let next = (x + n / x) / 2;
        if next >= x {
            return x;
        }
        x = next;
    }
}
fn product(
    a: &[Rational; 8],
    b: &[Rational; 8],
    ds: &[Rational; 3],
    level: usize,
) -> Result<[Rational; 8]> {
    let mut out = [Rational::integer(0); 8];
    for i in 0..(1 << level) {
        for j in 0..(1 << level) {
            if a[i].numerator == 0 || b[j].numerator == 0 {
                continue;
            }
            let mut v = a[i].checked_mul(b[j])?;
            for (k, d) in ds.iter().enumerate().take(level) {
                if (i & j) & (1 << k) != 0 {
                    v = v.checked_mul(*d)?;
                }
            }
            out[i ^ j] = out[i ^ j].checked_add(v)?;
        }
    }
    Ok(out)
}
fn sign(cs: &[Rational; 8], ds: &[Rational; 3], level: usize) -> Result<i8> {
    if level == 0 {
        return Ok(cs[0].numerator.signum() as i8);
    }
    let bit = 1 << (level - 1);
    let mut a = [Rational::integer(0); 8];
    let mut b = a;
    a[..bit].copy_from_slice(&cs[..bit]);
    b[..bit].copy_from_slice(&cs[bit..2 * bit]);
    let sa = sign(&a, ds, level - 1)?;
    if ds[level - 1].numerator == 0 {
        return Ok(sa);
    }
    let sb = sign(&b, ds, level - 1)?;
    if sa == 0 {
        return Ok(sb);
    }
    if sb == 0 || sa == sb {
        return Ok(sa);
    }
    let aa = product(&a, &a, ds, level - 1)?;
    let bb = product(&b, &b, ds, level - 1)?;
    let mut delta = a;
    for i in 0..bit {
        delta[i] = aa[i].checked_sub(bb[i].checked_mul(ds[level - 1])?)?;
    }
    Ok(sign(&delta, ds, level - 1)? * sa)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_cancel_and_radical_inequalities_do_not_round() {
        let mut e = Expression::new([
            Rational::integer(2),
            Rational::integer(8),
            Rational::integer(0),
        ])
        .unwrap();
        e.coefficients[1] = Rational::integer(2);
        e.coefficients[2] = Rational::integer(-1);
        assert_eq!(e.sign().unwrap(), 0);
        e.coefficients[0] = Rational::new(1, 1_000_000_000).unwrap();
        assert_eq!(e.sign().unwrap(), 1);
        e.coefficients[0] = Rational::new(-1, 1_000_000_000).unwrap();
        assert_eq!(e.sign().unwrap(), -1);
        let mut e = Expression::new([
            Rational::integer(2),
            Rational::integer(3),
            Rational::integer(10),
        ])
        .unwrap();
        e.coefficients[1] = Rational::integer(1);
        e.coefficients[2] = Rational::integer(1);
        e.coefficients[4] = Rational::integer(-1);
        // sqrt(2)+sqrt(3) < sqrt(10) because 5+2sqrt(6) < 10.
        assert_eq!(e.sign().unwrap(), -1);
    }
}
