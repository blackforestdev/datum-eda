//! Exact integer specializations. Fractional inputs and checked overflow use
//! the existing rational predicates; no coordinate rounding or tolerance.
use super::P;
fn integer(p: P) -> Option<[i128; 2]> {
    p.iter()
        .all(|v| v.denominator == 1)
        .then_some([p[0].numerator, p[1].numerator])
}
fn cross(a: [i128; 2], b: [i128; 2], p: [i128; 2]) -> Option<i128> {
    let x = b[0].checked_sub(a[0])?;
    let y = b[1].checked_sub(a[1])?;
    x.checked_mul(p[1].checked_sub(a[1])?)?
        .checked_sub(y.checked_mul(p[0].checked_sub(a[0])?)?)
}
fn bounded(p: [i128; 2], a: [i128; 2], b: [i128; 2]) -> bool {
    (0..2).all(|i| a[i].min(b[i]) <= p[i] && p[i] <= a[i].max(b[i]))
}
// Doubled integer bounds include half-nanometre vertices and widths exactly.
// A separating axis proves distance greater than half for either boundary.
pub(super) fn separated(a: P, b: P, c: P, d: P, half: super::R) -> Option<bool> {
    doubled_separated(a, b, c, d, half).or_else(|| {
        if half.numerator < 0 {
            return None;
        }
        for i in 0..2 {
            let (amin, amax) = if a[i].compare(b[i]).ok()?.is_le() {
                (a[i], b[i])
            } else {
                (b[i], a[i])
            };
            let (cmin, cmax) = if c[i].compare(d[i]).ok()?.is_le() {
                (c[i], d[i])
            } else {
                (d[i], c[i])
            };
            if cmin.compare(amax.checked_add(half).ok()?).ok()?.is_gt()
                || amin.compare(cmax.checked_add(half).ok()?).ok()?.is_gt()
            {
                return Some(true);
            }
        }
        Some(false)
    })
}
fn doubled_separated(a: P, b: P, c: P, d: P, half: super::R) -> Option<bool> {
    fn twice(v: super::R) -> Option<i128> {
        match v.denominator {
            1 => v.numerator.checked_mul(2),
            2 => Some(v.numerator),
            _ => None,
        }
    }
    let half = twice(half)?;
    if half < 0 {
        return None;
    }
    for axis in 0..2 {
        let (a, b, c, d) = (
            twice(a[axis])?,
            twice(b[axis])?,
            twice(c[axis])?,
            twice(d[axis])?,
        );
        if c.min(d).checked_sub(a.max(b))? > half || a.min(b).checked_sub(c.max(d))? > half {
            return Some(true);
        }
    }
    Some(false)
}
pub(super) fn segments_intersect(a: P, b: P, c: P, d: P) -> Option<bool> {
    let (a, b, c, d) = (integer(a)?, integer(b)?, integer(c)?, integer(d)?);
    if (0..2).any(|i| a[i].max(b[i]) < c[i].min(d[i]) || c[i].max(d[i]) < a[i].min(b[i])) {
        return Some(false);
    }
    let (ac, ad, ca, cb) = (
        cross(a, b, c)?,
        cross(a, b, d)?,
        cross(c, d, a)?,
        cross(c, d, b)?,
    );
    Some(
        (ac == 0 && bounded(c, a, b))
            || (ad == 0 && bounded(d, a, b))
            || (ca == 0 && bounded(a, c, d))
            || (cb == 0 && bounded(b, c, d))
            || (ac.signum() != ad.signum()
                && ca.signum() != cb.signum()
                && ac != 0
                && ad != 0
                && ca != 0
                && cb != 0),
    )
}
// Sweep exact x intervals rather than revisiting every disjoint edge pair.
// Candidate intersections retain the same endpoint/adjacency predicates.
pub(super) fn simple(vertices: &[P]) -> Option<bool> {
    let mut edges = Vec::with_capacity(vertices.len());
    for i in 0..vertices.len() {
        let a = integer(vertices[i])?;
        let b = integer(vertices[(i + 1) % vertices.len()])?;
        edges.push((a[0].min(b[0]), a[0].max(b[0]), i));
    }
    edges.sort_unstable();
    for (at, &(_, max, i)) in edges.iter().enumerate() {
        for &(min, _, j) in &edges[at + 1..] {
            if min > max {
                break;
            }
            if (i + 1) % vertices.len() == j || (j + 1) % vertices.len() == i {
                continue;
            }
            if segments_intersect(
                vertices[i],
                vertices[(i + 1) % vertices.len()],
                vertices[j],
                vertices[(j + 1) % vertices.len()],
            )? {
                return Some(false);
            }
        }
    }
    Some(true)
}
pub(super) fn contains(p: P, vertices: &[P]) -> Option<bool> {
    let p = integer(p)?;
    let mut winding = 0;
    for i in 0..vertices.len() {
        let (a, b) = (
            integer(vertices[i])?,
            integer(vertices[(i + 1) % vertices.len()])?,
        );
        if p[1] < a[1].min(b[1]) || p[1] > a[1].max(b[1]) {
            continue;
        }
        let sign = cross(a, b, p)?;
        if sign == 0 && bounded(p, a, b) {
            return Some(true);
        }
        if a[1] <= p[1] && b[1] > p[1] && sign > 0 {
            winding += 1;
        }
        if a[1] > p[1] && b[1] <= p[1] && sign < 0 {
            winding -= 1;
        }
    }
    Some(winding != 0)
}

#[cfg(test)]
#[path = "integer_polygon_tests.rs"]
mod tests;
