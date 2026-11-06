//! Exact integral wire-contact geometry. Every returned point lies exactly on
//! the source segment under the engine's incidence rule — no proximity
//! thresholds, no tolerances, no topology inference. Checked arithmetic fails
//! closed: a degenerate segment or any overflow refuses instead of silently
//! yielding a visually touching, electrically detached point.
use crate::PointNm;
use eda_engine::ir::geometry::Point;

/// The exact on-segment lattice point of `from..to` nearest to `desired`.
///
/// All integer points of a segment are `from + k * (v / gcd(v))` for
/// `k in 0..=gcd(v)`; the nearest `k` is chosen with checked i128 arithmetic
/// and clamped to the segment, so horizontal, vertical and diagonal source
/// wires all produce exact incidence.
pub fn exact_wire_contact(from: Point, to: Point, desired: PointNm) -> Result<PointNm, String> {
    let (vx, vy) = (
        i128::from(to.x) - i128::from(from.x),
        i128::from(to.y) - i128::from(from.y),
    );
    if vx == 0 && vy == 0 {
        return Err("Wire segment is degenerate; no exact contact point".into());
    }
    let g = i128::try_from(gcd(vx.unsigned_abs(), vy.unsigned_abs())).map_err(|_| overflow())?;
    let (sx, sy) = (vx / g, vy / g);
    let (dx, dy) = (
        i128::from(desired.x) - i128::from(from.x),
        i128::from(desired.y) - i128::from(from.y),
    );
    let num = dx
        .checked_mul(sx)
        .zip(dy.checked_mul(sy))
        .and_then(|(a, b)| a.checked_add(b))
        .ok_or_else(overflow)?;
    let den = sx
        .checked_mul(sx)
        .zip(sy.checked_mul(sy))
        .and_then(|(a, b)| a.checked_add(b))
        .ok_or_else(overflow)?;
    // Nearest integer k to num/den, then clamped onto the segment.
    let k = num
        .checked_mul(2)
        .zip(den.checked_mul(2))
        .and_then(|(n, d2)| n.checked_add(den).map(|n| n.div_euclid(d2)))
        .ok_or_else(overflow)?
        .clamp(0, g);
    let x = i128::from(from.x) + k * sx;
    let y = i128::from(from.y) + k * sy;
    match (i64::try_from(x), i64::try_from(y)) {
        (Ok(x), Ok(y)) => Ok(PointNm { x, y }),
        _ => Err(overflow()),
    }
}

fn overflow() -> String {
    "Wire contact exceeds exact integer coordinates".into()
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.max(1)
}

#[cfg(test)]
#[path = "schematic_wire_anchor_tests.rs"]
mod tests;
