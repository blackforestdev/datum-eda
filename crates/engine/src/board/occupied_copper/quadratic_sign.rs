//! Exact comparison of a² and 2b² for rational a,b without i128 squaring.
//! Four u128 factors and the factor two fit 513 bits; nine u64 limbs suffice.
//! This is private bounded sign arithmetic, not arbitrary-precision source data.
use super::*;
fn product(factors: &[u128]) -> Result<[u64; 9]> {
    let mut out = [0u64; 9];
    out[0] = 1;
    for factor in factors {
        let words = [*factor as u64, (*factor >> 64) as u64];
        let input = out;
        out = [0; 9];
        for (j, word) in words.into_iter().enumerate() {
            let mut carry = 0u128;
            for i in 0..9 - j {
                // (2^64-1)^2 + two (2^64-1) terms <= 2^128-1.
                let v = u128::from(input[i]) * u128::from(word) + u128::from(out[i + j]) + carry;
                out[i + j] = v as u64;
                carry = v >> 64;
            }
            if carry != 0 || (word != 0 && input[9 - j..].iter().any(|v| *v != 0)) {
                return Err(GeometryError::ArithmeticRange);
            }
        }
    }
    Ok(out)
}
pub(super) fn sign(a: R, b: R) -> Result<i8> {
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
    let left = product(&[an, an, bd, bd])?;
    let right = product(&[bn, bn, ad, ad, 2])?;
    let order = left.iter().rev().cmp(right.iter().rev());
    let direction = match order {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    };
    Ok(direction * sa)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rational_radical_comparison_spans_full_coefficients_without_rounding() {
        let big = R::integer(i128::MAX);
        assert_eq!(sign(big, big.checked_neg().unwrap()), Ok(-1));
        assert_eq!(sign(big.checked_neg().unwrap(), big), Ok(1));
        assert_eq!(
            sign(
                R::new(1, i128::MAX).unwrap(),
                R::new(-1, i128::MAX - 1).unwrap()
            ),
            Ok(-1)
        );
        // Continued-fraction bounds 1393/985 < sqrt(2) < 3363/2378.
        assert_eq!(sign(R::new(1393, 985).unwrap(), R::integer(-1)), Ok(-1));
        assert_eq!(sign(R::new(3363, 2378).unwrap(), R::integer(-1)), Ok(1));
        assert_eq!(
            product(&[u128::MAX, u128::MAX, u128::MAX, u128::MAX, 2]).unwrap()[8],
            1
        );
        assert_eq!(sign(R::integer(0), R::integer(0)), Ok(0));
    }
}
