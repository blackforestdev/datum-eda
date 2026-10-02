//! Exact equality of unions of closed simple filled cells. No overlap lineage.
//! Split every boundary at exact intersections, then compare occupied sides of
//! each atomic segment. This is independent of decomposition, winding and order.
use super::nominal_geometry::{GeometryError, Rational as R, Result, cross};
use super::occupied_copper::validate_polygon;
use crate::ir::geometry::Polygon;
use std::cmp::Ordering;
type P = [R; 2];
#[derive(Clone, Copy)]
struct Edge {
    a: P,
    b: P,
}
fn sub(a: P, b: P) -> Result<P> {
    Ok([a[0].checked_sub(b[0])?, a[1].checked_sub(b[1])?])
}
fn at(e: Edge, t: R) -> Result<P> {
    let d = sub(e.b, e.a)?;
    Ok([
        e.a[0].checked_add(d[0].checked_mul(t)?)?,
        e.a[1].checked_add(d[1].checked_mul(t)?)?,
    ])
}
fn unit(t: R) -> Result<bool> {
    Ok(t.compare(R::integer(0))? != Ordering::Less
        && t.compare(R::integer(1))? != Ordering::Greater)
}
fn parameter(e: Edge, p: P) -> Result<R> {
    let d = sub(e.b, e.a)?;
    let axis = usize::from(d[0].numerator == 0);
    p[axis].checked_sub(e.a[axis])?.checked_div(d[axis])
}
fn on(e: Edge, p: P) -> Result<bool> {
    Ok(cross(sub(e.b, e.a)?, sub(p, e.a)?)?.numerator == 0 && unit(parameter(e, p)?)?)
}
fn sorted_unique(values: &mut Vec<R>) -> Result<()> {
    // Ordering itself can fail by checked range; never hide it in a comparator.
    for i in 1..values.len() {
        let mut j = i;
        while j > 0 && values[j].compare(values[j - 1])? == Ordering::Less {
            values.swap(j, j - 1);
            j -= 1;
        }
    }
    values.dedup();
    Ok(())
}
fn splits(a: Edge, b: Edge, out: &mut Vec<R>) -> Result<()> {
    let av = sub(a.b, a.a)?;
    let bv = sub(b.b, b.a)?;
    let offset = sub(b.a, a.a)?;
    let determinant = cross(av, bv)?;
    if determinant.numerator == 0 {
        if cross(av, offset)?.numerator == 0 {
            for p in [b.a, b.b] {
                let t = parameter(a, p)?;
                if unit(t)? {
                    out.push(t);
                }
            }
        }
    } else {
        let t = cross(offset, bv)?.checked_div(determinant)?;
        let u = cross(offset, av)?.checked_div(determinant)?;
        if unit(t)? && unit(u)? {
            out.push(t);
        }
    }
    Ok(())
}
fn cells(polygons: &[Polygon]) -> Result<Vec<Vec<Edge>>> {
    let mut result = vec![];
    for polygon in polygons {
        validate_polygon(polygon)?;
        let mut points = polygon.vertices.clone();
        points.dedup();
        if points.first() == points.last() {
            points.pop();
        }
        let mut points: Vec<P> = points
            .into_iter()
            .map(|p| [R::integer(i128::from(p.x)), R::integer(i128::from(p.y))])
            .collect();
        let mut area = R::integer(0);
        for i in 0..points.len() {
            area = area.checked_add(cross(points[i], points[(i + 1) % points.len()])?)?;
        }
        if area.numerator < 0 {
            points.reverse();
        }
        result.push(
            (0..points.len())
                .map(|i| Edge {
                    a: points[i],
                    b: points[(i + 1) % points.len()],
                })
                .collect(),
        );
    }
    Ok(result)
}
fn occupied_sides(cells: &[Vec<Edge>], p: P, direction: P) -> Result<(bool, bool)> {
    let mut left = false;
    let mut right = false;
    for cell in cells {
        let mut boundary = false;
        let mut winding = 0_i64;
        for e in cell {
            if on(*e, p)? {
                let d = sub(e.b, e.a)?;
                let alignment = d[0]
                    .checked_mul(direction[0])?
                    .checked_add(d[1].checked_mul(direction[1])?)?;
                if alignment.numerator == 0 {
                    return Err(GeometryError::UnresolvedPredicate);
                }
                left |= alignment.numerator > 0;
                right |= alignment.numerator < 0;
                boundary = true;
            }
            let sign = cross(sub(e.b, e.a)?, sub(p, e.a)?)?.numerator.signum();
            if e.a[1].compare(p[1])? != Ordering::Greater
                && e.b[1].compare(p[1])? == Ordering::Greater
                && sign > 0
            {
                winding += 1;
            }
            if e.a[1].compare(p[1])? == Ordering::Greater
                && e.b[1].compare(p[1])? != Ordering::Greater
                && sign < 0
            {
                winding -= 1;
            }
        }
        if !boundary && winding != 0 {
            left = true;
            right = true;
        }
    }
    Ok((left, right))
}
pub(crate) fn equivalent(a: &[Polygon], b: &[Polygon]) -> Result<bool> {
    let a = cells(a)?;
    let b = cells(b)?;
    let edges: Vec<Edge> = a.iter().chain(&b).flatten().copied().collect();
    for edge in &edges {
        let mut parameters = vec![R::integer(0), R::integer(1)];
        for peer in &edges {
            splits(*edge, *peer, &mut parameters)?;
        }
        sorted_unique(&mut parameters)?;
        let direction = sub(edge.b, edge.a)?;
        for interval in parameters.windows(2) {
            let midpoint = interval[0]
                .checked_add(interval[1])?
                .checked_div(R::integer(2))?;
            let p = at(*edge, midpoint)?;
            if occupied_sides(&a, p, direction)? != occupied_sides(&b, p, direction)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
#[path = "occupied_region_tests.rs"]
mod tests;
