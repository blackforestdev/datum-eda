//! Exact source-vertex decomposition of KiCad's bridged hole contours.
//! No new coordinates, floating point, tolerance or manufacturing allowance.
use crate::{
    error::EngineError,
    ir::geometry::{Point, Polygon},
};
use std::collections::BTreeMap;

// Capture saved coordinates on the inherited integer-nanometre source grid.
// Floating-point rounding cannot supply a certificate of exact occupied copper.
pub(super) fn points(section: &str) -> Result<Vec<Point>, EngineError> {
    let mut result = Vec::new();
    let mut tail = section;
    while let Some(start) = tail.find("(xy") {
        tail = &tail[start + 3..];
        let end = tail.find(')').ok_or_else(invalid)?;
        let values: Vec<_> = tail[..end].split_whitespace().collect();
        if values.len() != 2 {
            return Err(invalid());
        }
        result.push(Point::new(nanometres(values[0])?, nanometres(values[1])?));
        tail = &tail[end + 1..];
    }
    Ok(result)
}
fn nanometres(value: &str) -> Result<i64, EngineError> {
    let (base, exponent) = match value.split_once(['e', 'E']) {
        Some((base, exponent)) => (base, exponent.parse::<i32>().map_err(|_| invalid())?),
        None => (value, 0),
    };
    let fractional = base
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    if base.matches('.').count() > 1 {
        return Err(invalid());
    }
    let digits = base.replace('.', "");
    let integer = digits.parse::<i128>().map_err(|_| invalid())?;
    let power = exponent
        .checked_add(6)
        .and_then(|p| p.checked_sub(i32::try_from(fractional).ok()?))
        .ok_or_else(invalid)?;
    let scale = 10_i128
        .checked_pow(power.unsigned_abs())
        .ok_or_else(invalid)?;
    let nm = if power >= 0 {
        integer.checked_mul(scale).ok_or_else(invalid)?
    } else if integer % scale == 0 {
        integer / scale
    } else {
        return Err(invalid());
    };
    i64::try_from(nm).map_err(|_| invalid())
}

fn invalid() -> EngineError {
    EngineError::Import("saved zone fill has unresolved or invalid occupied geometry".into())
}
fn cross(a: Point, b: Point, c: Point) -> Result<i128, EngineError> {
    let x = i128::from(b.x) - i128::from(a.x);
    let y = i128::from(b.y) - i128::from(a.y);
    x.checked_mul(i128::from(c.y) - i128::from(a.y))
        .and_then(|p| {
            y.checked_mul(i128::from(c.x) - i128::from(a.x))
                .and_then(|q| p.checked_sub(q))
        })
        .ok_or_else(invalid)
}
fn area(points: &[Point]) -> Result<i128, EngineError> {
    let mut sum = 0_i128;
    for i in 1..points.len().saturating_sub(1) {
        sum = sum
            .checked_add(cross(points[0], points[i], points[i + 1])?)
            .ok_or_else(invalid)?;
    }
    Ok(sum)
}
fn proper_cross(a: Point, b: Point, c: Point, d: Point) -> Result<bool, EngineError> {
    Ok(cross(a, b, c)?.signum() * cross(a, b, d)?.signum() < 0
        && cross(c, d, a)?.signum() * cross(c, d, b)?.signum() < 0)
}
type Edge = ((i64, i64), (i64, i64));
fn boundary(
    points: &[Point],
    vertices: &[Point],
    out: &mut BTreeMap<Edge, i64>,
) -> Result<(), EngineError> {
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        if a == b {
            continue;
        }
        let mut cuts = Vec::new();
        for &p in vertices {
            if cross(a, b, p)? == 0
                && p.x >= a.x.min(b.x)
                && p.x <= a.x.max(b.x)
                && p.y >= a.y.min(b.y)
                && p.y <= a.y.max(b.y)
            {
                cuts.push((p.x, p.y));
            }
        }
        cuts.sort();
        cuts.dedup();
        let sign = if (a.x, a.y) < (b.x, b.y) { 1 } else { -1 };
        for pair in cuts.windows(2) {
            *out.entry((pair[0], pair[1])).or_default() += sign;
        }
    }
    out.retain(|_, n| *n != 0);
    Ok(())
}
pub(super) fn certify(polygon: &Polygon) -> Result<Vec<Polygon>, EngineError> {
    if !polygon.closed {
        return Err(invalid());
    }
    let mut points = polygon.vertices.clone();
    points.dedup();
    if points.first() == points.last() {
        points.pop();
    }
    if points.len() < 3 {
        return Err(invalid());
    }
    // Simple contours retain their original source vertices and decomposition.
    if crate::board::occupied_copper::validate_polygon(&Polygon::new(points.clone())).is_ok() {
        return Ok(vec![Polygon::new(points)]);
    }
    // Doubled bridges may touch, but a true crossing is not a hole encoding.
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            if proper_cross(
                points[i],
                points[(i + 1) % points.len()],
                points[j],
                points[(j + 1) % points.len()],
            )? {
                return Err(invalid());
            }
        }
    }
    let signed = area(&points)?;
    if signed == 0 {
        return Err(invalid());
    }
    if signed < 0 {
        points.reverse();
    }
    let mut ring = points.clone();
    let mut triangles = Vec::new();
    while ring.len() > 3 {
        let mut ear = None;
        for i in 0..ring.len() {
            let a = ring[(i + ring.len() - 1) % ring.len()];
            let b = ring[i];
            let c = ring[(i + 1) % ring.len()];
            if cross(a, b, c)? <= 0 {
                continue;
            }
            let mut blocked = false;
            for &p in &ring {
                if p != a
                    && p != b
                    && p != c
                    && cross(a, b, p)? >= 0
                    && cross(b, c, p)? >= 0
                    && cross(c, a, p)? >= 0
                {
                    blocked = true;
                    break;
                }
            }
            if !blocked {
                for j in 0..ring.len() {
                    if proper_cross(a, c, ring[j], ring[(j + 1) % ring.len()])? {
                        blocked = true;
                        break;
                    }
                }
            }
            if !blocked {
                ear = Some((i, Polygon::new(vec![a, b, c])));
                break;
            }
        }
        let Some((i, triangle)) = ear else {
            return Err(invalid());
        };
        triangles.push(triangle);
        ring.remove(i);
    }
    if area(&ring)? > 0 {
        triangles.push(Polygon::new(ring));
    }
    let mut total = 0_i128;
    let mut actual = BTreeMap::new();
    let mut expected = BTreeMap::new();
    boundary(&points, &points, &mut expected)?;
    for triangle in &triangles {
        crate::board::occupied_copper::validate_polygon(triangle).map_err(|_| invalid())?;
        total = total
            .checked_add(area(&triangle.vertices)?)
            .ok_or_else(invalid)?;
        boundary(&triangle.vertices, &points, &mut actual)?;
    }
    if total != area(&points)? || actual != expected {
        return Err(invalid());
    }
    Ok(triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_keyhole_preserves_void_area_and_winding() {
        let polygon = Polygon::new(
            [
                (0, 0),
                (10, 0),
                (10, 10),
                (0, 10),
                (0, 0),
                (2, 2),
                (2, 4),
                (4, 4),
                (4, 2),
                (2, 2),
            ]
            .map(|(x, y)| Point::new(x, y))
            .to_vec(),
        );
        for reversed in [false, true] {
            let mut p = polygon.clone();
            if reversed {
                p.vertices.reverse();
            }
            let pieces = certify(&p).unwrap();
            assert_eq!(
                pieces
                    .iter()
                    .map(|p| area(&p.vertices).unwrap())
                    .sum::<i128>(),
                192
            );
            assert!(pieces.iter().all(|p| {
                !crate::board::occupied_copper::polygon_contains_point(p, Point::new(3, 3)).unwrap()
            }));
            assert!(pieces.iter().any(|p| {
                crate::board::occupied_copper::polygon_contains_point(p, Point::new(8, 8)).unwrap()
            }));
        }
    }
    #[test]
    fn source_grid_is_exact_and_never_rounded() {
        assert_eq!(nanometres("-1.234567").unwrap(), -1_234_567);
        assert_eq!(nanometres("1.2345670").unwrap(), 1_234_567);
        assert_eq!(nanometres("1e-6").unwrap(), 1);
        assert_eq!(nanometres("9223372036854.775807").unwrap(), i64::MAX);
        assert!(nanometres("0.0000001").is_err());
        assert!(nanometres("9223372036854.775808").is_err());
        assert!(points("(pts (xy 1 bad) (xy 2 3))").is_err());
    }
    #[test]
    fn true_crossing_and_coordinate_overflow_refuse() {
        let p = Polygon::new(
            [(0, 0), (10, 10), (0, 10), (10, 0)]
                .map(|(x, y)| Point::new(x, y))
                .to_vec(),
        );
        assert!(certify(&p).is_err());
        assert!(
            cross(
                Point::new(i64::MIN, i64::MIN),
                Point::new(i64::MAX, i64::MIN),
                Point::new(i64::MIN, i64::MAX)
            )
            .is_err()
        );
    }
}
