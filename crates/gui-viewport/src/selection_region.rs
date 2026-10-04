//! Exact shared polygon/anchor qualification, independent of retained strokes.
use datum_gui_protocol::{
    PointNm,
    selection_geometry::{GeometryError as Error, Rational as R},
};
use std::cmp::Ordering;
type Result<T> = std::result::Result<T, Error>;
pub type Point = [R; 2];
pub fn point(p: PointNm) -> Point {
    [R::integer(p.x.into()), R::integer(p.y.into())]
}
fn sub(a: Point, b: Point) -> Result<Point> {
    Ok([a[0].checked_sub(b[0])?, a[1].checked_sub(b[1])?])
}
fn cross(a: Point, b: Point) -> Result<R> {
    a[0].checked_mul(b[1])?.checked_sub(a[1].checked_mul(b[0])?)
}
fn side(a: Point, b: Point, p: Point) -> Result<R> {
    cross(sub(b, a)?, sub(p, a)?)
}
fn area(poly: &[Point]) -> Result<R> {
    let mut sum = R::integer(0);
    for i in 0..poly.len() {
        sum = sum.checked_add(cross(poly[i], poly[(i + 1) % poly.len()])?)?;
    }
    if sum.numerator < 0 {
        sum = sum.checked_neg()?;
    }
    sum.checked_div(R::integer(2))
}
fn orient(mut poly: Vec<Point>) -> Result<Vec<Point>> {
    if poly.first() == poly.last() {
        poly.pop();
    }
    if poly.len() < 3 {
        return Err(Error::InvalidPolygon);
    }
    let mut sum = R::integer(0);
    for i in 0..poly.len() {
        sum = sum.checked_add(cross(poly[i], poly[(i + 1) % poly.len()])?)?;
    }
    if sum.numerator == 0 {
        return Err(Error::InvalidPolygon);
    }
    if sum.numerator < 0 {
        poly.reverse();
    }
    Ok(poly)
}
fn inside_triangle(triangle: &[Point; 3], p: Point) -> Result<bool> {
    Ok(side(triangle[0], triangle[1], p)?.numerator >= 0
        && side(triangle[1], triangle[2], p)?.numerator >= 0
        && side(triangle[2], triangle[0], p)?.numerator >= 0)
}
fn strict_cross(a: Point, b: Point, c: Point, d: Point) -> Result<bool> {
    let ab1 = side(a, b, c)?.numerator.signum();
    let ab2 = side(a, b, d)?.numerator.signum();
    let cd1 = side(c, d, a)?.numerator.signum();
    let cd2 = side(c, d, b)?.numerator.signum();
    Ok(ab1 * ab2 < 0 && cd1 * cd2 < 0)
}
fn triangulate(poly: Vec<Point>) -> Result<Vec<[Point; 3]>> {
    let mut poly = orient(poly)?;
    for i in 0..poly.len() {
        for j in i + 1..poly.len() {
            if j == i + 1 || (i == 0 && j == poly.len() - 1) {
                continue;
            }
            if strict_cross(
                poly[i],
                poly[(i + 1) % poly.len()],
                poly[j],
                poly[(j + 1) % poly.len()],
            )? {
                return Err(Error::InvalidPolygon);
            }
        }
    }
    let mut result = Vec::new();
    while poly.len() > 3 {
        let mut ear = None;
        for i in 0..poly.len() {
            let indices = [(i + poly.len() - 1) % poly.len(), i, (i + 1) % poly.len()];
            let tri = indices.map(|k| poly[k]);
            if side(tri[0], tri[1], tri[2])?.numerator <= 0 {
                continue;
            }
            let mut blocked = false;
            for (j, p) in poly.iter().enumerate() {
                if !indices.contains(&j) && inside_triangle(&tri, *p)? {
                    blocked = true;
                    break;
                }
            }
            if !blocked {
                ear = Some((i, tri));
                break;
            }
        }
        let Some((i, tri)) = ear else {
            return Err(Error::InvalidPolygon);
        };
        result.push(tri);
        poly.remove(i);
    }
    result.push([poly[0], poly[1], poly[2]]);
    Ok(result)
}
/// Horizontal arrangement strips implement even-odd fill, including crossings.
/// Vertex and intersection ordinates split every ordering change exactly.
fn even_odd_triangles(path: &[Point]) -> Result<Vec<[Point; 3]>> {
    if path.len() < 3 {
        return Err(Error::InvalidPolygon);
    }
    let edges: Vec<_> = (0..path.len())
        .map(|i| [path[i], path[(i + 1) % path.len()]])
        .filter(|[a, b]| a != b)
        .collect();
    let mut levels: Vec<_> = path.iter().map(|p| p[1]).collect();
    for (i, [a, b]) in edges.iter().enumerate() {
        for [c, d] in &edges[i + 1..] {
            let ab = sub(*b, *a)?;
            let cd = sub(*d, *c)?;
            let denominator = cross(ab, cd)?;
            if denominator.numerator == 0 {
                continue;
            }
            let ca = sub(*c, *a)?;
            let t = cross(ca, cd)?.checked_div(denominator)?;
            let u = cross(ca, ab)?.checked_div(denominator)?;
            if t.numerator >= 0
                && t.compare(R::integer(1))? != Ordering::Greater
                && u.numerator >= 0
                && u.compare(R::integer(1))? != Ordering::Greater
            {
                levels.push(a[1].checked_add(ab[1].checked_mul(t)?)?);
            }
        }
    }
    // Fallible insertion sorting keeps overflow a refusal rather than guessing order.
    let sort = |values: &mut Vec<R>| -> Result<()> {
        for i in 1..values.len() {
            let mut j = i;
            while j > 0 && values[j].compare(values[j - 1])? == Ordering::Less {
                values.swap(j, j - 1);
                j -= 1;
            }
        }
        values.dedup();
        Ok(())
    };
    sort(&mut levels)?;
    let x_at = |edge: &[Point; 2], y: R| -> Result<R> {
        let [a, b] = *edge;
        a[0].checked_add(
            b[0].checked_sub(a[0])?
                .checked_mul(y.checked_sub(a[1])?.checked_div(b[1].checked_sub(a[1])?)?)?,
        )
    };
    let mut triangles = Vec::new();
    for ys in levels.windows(2) {
        let mid = ys[0].checked_add(ys[1])?.checked_div(R::integer(2))?;
        let mut active = Vec::new();
        for edge in &edges {
            let below_a = edge[0][1].compare(mid)? == Ordering::Less;
            let below_b = edge[1][1].compare(mid)? == Ordering::Less;
            if below_a != below_b {
                active.push((x_at(edge, mid)?, *edge));
            }
        }
        for i in 1..active.len() {
            let mut j = i;
            while j > 0 && active[j].0.compare(active[j - 1].0)? == Ordering::Less {
                active.swap(j, j - 1);
                j -= 1;
            }
        }
        if active.len() % 2 != 0 {
            return Err(Error::UnresolvedPredicate);
        }
        for pair in active.as_chunks::<2>().0 {
            if pair[0].0 == pair[1].0 {
                continue;
            }
            let a = [x_at(&pair[0].1, ys[0])?, ys[0]];
            let b = [x_at(&pair[1].1, ys[0])?, ys[0]];
            let c = [x_at(&pair[1].1, ys[1])?, ys[1]];
            let d = [x_at(&pair[0].1, ys[1])?, ys[1]];
            for tri in [[a, b, c], [a, c, d]] {
                if side(tri[0], tri[1], tri[2])?.numerator > 0 {
                    triangles.push(tri);
                }
            }
        }
    }
    // A zero-area gesture qualifies nothing; it is not an unresolved predicate.
    Ok(triangles)
}
fn clip(mut polygon: Vec<Point>, boundary: &[Point]) -> Result<Vec<Point>> {
    for edge in 0..boundary.len() {
        let a = boundary[edge];
        let b = boundary[(edge + 1) % boundary.len()];
        let input = std::mem::take(&mut polygon);
        if input.is_empty() {
            break;
        }
        for i in 0..input.len() {
            let p = input[i];
            let q = input[(i + 1) % input.len()];
            let ps = side(a, b, p)?;
            let qs = side(a, b, q)?;
            if ps.numerator >= 0 {
                polygon.push(p);
            }
            if (ps.numerator < 0) != (qs.numerator < 0) {
                let t = ps.checked_div(ps.checked_sub(qs)?)?;
                let delta = sub(q, p)?;
                polygon.push([
                    p[0].checked_add(delta[0].checked_mul(t)?)?,
                    p[1].checked_add(delta[1].checked_mul(t)?)?,
                ]);
            }
        }
    }
    Ok(polygon)
}
/// Certify one simple convex boundary before bypassing even-odd triangles.
/// Duplicate winding and self-crossing paths cannot use this representation.
fn convex_boundary(path: Vec<Point>) -> Result<Option<Vec<Point>>> {
    let path = match orient(path) {
        Ok(path) => path,
        Err(Error::InvalidPolygon) => return Ok(None),
        Err(error) => return Err(error),
    };
    for i in 0..path.len() {
        if path[i + 1..].contains(&path[i]) {
            return Ok(None);
        }
        for p in &path {
            if side(path[i], path[(i + 1) % path.len()], *p)?.numerator < 0 {
                return Ok(None);
            }
        }
    }
    Ok(Some(path))
}
#[derive(Debug, Clone)]
pub struct SelectionRegion {
    convex: Option<Vec<Point>>,
    triangles: Vec<[Point; 3]>,
    boundary: Vec<[Point; 2]>,
}
impl SelectionRegion {
    pub fn new(path: &[PointNm]) -> Result<Self> {
        let boundary = (0..path.len())
            .map(|i| [point(path[i]), point(path[(i + 1) % path.len()])])
            .collect();
        let points: Vec<_> = path.iter().copied().map(point).collect();
        Ok(Self {
            convex: convex_boundary(points.clone())?,
            triangles: if path.len() < 3 {
                Vec::new()
            } else {
                even_odd_triangles(&points)?
            },
            boundary,
        })
    }
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }
    pub fn contains(&self, p: PointNm) -> Result<bool> {
        for tri in &self.triangles {
            if inside_triangle(tri, point(p))? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn anchors(&self, anchors: &[PointNm]) -> Result<bool> {
        if anchors.is_empty() {
            return Err(Error::UnresolvedPredicate);
        }
        let mut inside = 0;
        for p in anchors {
            inside += usize::from(self.contains(*p)?);
        }
        Ok(inside > anchors.len() / 2)
    }
    pub fn covered_area(&self, polygon: &[Point]) -> Result<R> {
        let subjects = triangulate(polygon.to_vec())?;
        let mut covered = R::integer(0);
        for subject in subjects {
            if let Some(boundary) = &self.convex {
                // Artificial triangulation diagonals can magnify exact fractions
                // beyond i128 even when the actual clipped area is small.
                covered = covered.checked_add(area(&clip(subject.to_vec(), boundary)?)?)?;
            } else {
                for triangle in &self.triangles {
                    covered = covered.checked_add(area(&clip(subject.to_vec(), triangle)?)?)?;
                }
            }
        }
        Ok(covered)
    }
    pub fn majority_layout(&self, polygon: &[Point]) -> Result<bool> {
        let total = area(polygon)?;
        if total.numerator == 0 {
            return Err(Error::UnresolvedPredicate);
        }
        Ok(self
            .covered_area(polygon)?
            .checked_mul(R::integer(2))?
            .compare(total)?
            == Ordering::Greater)
    }
    pub fn encloses(&self, polygon: &[Point]) -> Result<bool> {
        Ok(self.covered_area(polygon)?.compare(area(polygon)?)? == Ordering::Equal)
    }
}
impl SelectionRegion {
    pub fn encloses_disk(&self, center: PointNm, radius: i64) -> Result<bool> {
        if radius < 0 || !self.contains(center)? {
            return Ok(false);
        }
        let center = point(center);
        let limit = R::integer(radius.into()).square()?;
        // Boundary edges are retained rather than triangle diagonals.
        for [a, b] in &self.boundary {
            let delta = sub(*b, *a)?;
            let vector = sub(center, *a)?;
            let squared = delta[0].square()?.checked_add(delta[1].square()?)?;
            if squared.numerator == 0 {
                return Err(Error::InvalidPolygon);
            }
            let dot = vector[0]
                .checked_mul(delta[0])?
                .checked_add(vector[1].checked_mul(delta[1])?)?;
            let t = dot.checked_div(squared)?;
            let nearest = if t.numerator < 0 {
                *a
            } else if t.compare(R::integer(1))? == Ordering::Greater {
                *b
            } else {
                [
                    a[0].checked_add(delta[0].checked_mul(t)?)?,
                    a[1].checked_add(delta[1].checked_mul(t)?)?,
                ]
            };
            let gap = sub(center, nearest)?;
            let distance = gap[0].square()?.checked_add(gap[1].square()?)?;
            if distance.compare(limit)? == Ordering::Less {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub fn layout_contains(corners: &[Point], p: PointNm) -> Result<bool> {
    if corners.len() != 4 {
        return Err(Error::InvalidPolygon);
    }
    let p = point(p);
    let mut positive = false;
    let mut negative = false;
    for i in 0..4 {
        let sign = side(corners[i], corners[(i + 1) % 4], p)?
            .numerator
            .signum();
        positive |= sign > 0;
        negative |= sign < 0;
    }
    Ok(!(positive && negative) && (positive || negative))
}
