//! Certified occupied copper shared by physical membership and nominal DRC.
//! Orthogonal pad shapes have exact rational boundaries; arbitrary noncircular
//! rotations remain explicit capability failure, never rounded/chord geometry.
use super::nominal_geometry::{
    CertifiedArc, DistanceBoundary, GeometryError, Rational as R, Result, cross, norm,
};
use super::nominal_predicates::{
    point, rational_arc_segment_within, rational_point_segment_within,
};
use super::radical_sign::Expression;
use super::{PadShape, PlacedPad, Stackup, StackupLayerType, Track, Via};
use crate::ir::geometry::{LayerId, Point, Polygon};
use std::cmp::Ordering;
type P = [R; 2];
fn sub(a: P, b: P) -> Result<P> {
    Ok([a[0].checked_sub(b[0])?, a[1].checked_sub(b[1])?])
}
fn lt(a: R, b: R) -> Result<bool> {
    Ok(a.compare(b)? == Ordering::Less)
}
fn zero() -> R {
    R::integer(0)
}
fn max(a: R, b: R) -> Result<R> {
    Ok(if lt(a, b)? { b } else { a })
}
fn between(p: R, a: R, b: R) -> Result<bool> {
    Ok(p.compare(a)? != Ordering::Less && p.compare(b)? != Ordering::Greater)
}

pub fn track_point_within(
    track: &Track,
    p: Point,
    extra: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra.numerator < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let half = track_half(track)?.checked_add(extra)?;
    if half.numerator < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    match track.midpoint {
        Some(mid) => {
            CertifiedArc::new(track.from, mid, track.to)?.point_within_boundary(p, half, boundary)
        }
        None => rational_point_segment_within(
            point(p),
            point(track.from),
            point(track.to),
            half,
            boundary,
        ),
    }
}
pub fn validate_track(track: &Track) -> Result<()> {
    track_half(track)?;
    if let Some(mid) = track.midpoint {
        CertifiedArc::new(track.from, mid, track.to)?;
    }
    Ok(())
}

pub fn validate_pad(pad: &PlacedPad) -> Result<()> {
    RoundedBox::pad(pad)?;
    Ok(())
}
pub fn validate_via(via: &Via) -> Result<()> {
    RoundedBox::via(via)?;
    Ok(())
}
pub fn validate_polygon(polygon: &Polygon) -> Result<()> {
    polygon_vertices(polygon)?;
    Ok(())
}
fn box_contains_point(shape: RoundedBox, p: Point) -> Result<bool> {
    let p = point(p);
    let closest = [
        max(
            shape.min[0],
            if lt(shape.max[0], p[0])? {
                shape.max[0]
            } else {
                p[0]
            },
        )?,
        max(
            shape.min[1],
            if lt(shape.max[1], p[1])? {
                shape.max[1]
            } else {
                p[1]
            },
        )?,
    ];
    Ok(
        norm(sub(p, closest)?)?.compare(shape.radius.square()?)? != Ordering::Greater
            && norm(sub(p, shape.center)?)?.compare(shape.hole.square()?)? != Ordering::Less,
    )
}
pub fn pad_contains_point(pad: &PlacedPad, p: Point) -> Result<bool> {
    box_contains_point(RoundedBox::pad(pad)?, p)
}
pub fn via_contains_point(via: &Via, p: Point) -> Result<bool> {
    box_contains_point(RoundedBox::via(via)?, p)
}
pub fn conductive_layer(stackup: &Stackup, id: LayerId) -> Result<()> {
    layer_index(stackup, id)?;
    Ok(())
}

fn track_half(track: &Track) -> Result<R> {
    if track.width <= 0 {
        return Err(GeometryError::InvalidWidth);
    }
    R::new(i128::from(track.width), 2)
}
fn segment_pair(a: P, b: P, c: P, d: P, half: R, boundary: DistanceBoundary) -> Result<bool> {
    for (p, x, y) in [(a, c, d), (b, c, d), (c, a, b), (d, a, b)] {
        if rational_point_segment_within(p, x, y, half, boundary)? {
            return Ok(true);
        }
    }
    let ab = sub(b, a)?;
    let cd = sub(d, c)?;
    let ca = sub(c, a)?;
    let det = cross(ab, cd)?;
    if det.numerator == 0 {
        return Ok(false);
    }
    let u = cross(ca, cd)?.checked_div(det)?;
    let v = cross(ca, ab)?.checked_div(det)?;
    Ok(between(u, zero(), R::integer(1))?
        && between(v, zero(), R::integer(1))?
        && (boundary == DistanceBoundary::Inclusive || half.numerator > 0))
}
fn track_edge(track: &Track, a: P, b: P, half: R, boundary: DistanceBoundary) -> Result<bool> {
    match track.midpoint {
        Some(mid) => rational_arc_segment_within(
            CertifiedArc::new(track.from, mid, track.to)?,
            a,
            b,
            half,
            boundary,
        ),
        None => segment_pair(point(track.from), point(track.to), a, b, half, boundary),
    }
}

#[derive(Clone, Copy)]
struct RoundedBox {
    min: P,
    max: P,
    radius: R,
    center: P,
    hole: R,
}
impl RoundedBox {
    fn pad(pad: &PlacedPad) -> Result<Self> {
        let (mut w, mut h) = match pad.shape {
            PadShape::Circle => (pad.diameter, pad.diameter),
            _ => (pad.width, pad.height),
        };
        if w <= 0 || h <= 0 || pad.drill < 0 {
            return Err(GeometryError::InvalidWidth);
        }
        if pad.shape != PadShape::Circle {
            if pad.rotation.rem_euclid(90) != 0 {
                return Err(GeometryError::UnsupportedPadGeometry);
            }
            if pad.rotation.rem_euclid(180) != 0 {
                std::mem::swap(&mut w, &mut h)
            }
        }
        // A drill intersecting the outer edge may divide one source into more
        // than one copper component. This kernel does not invent that qualifier.
        if pad.drill >= w.min(h) {
            return Err(GeometryError::UnsupportedPadGeometry);
        }
        let radius = match pad.shape {
            PadShape::Circle | PadShape::Oval => R::new(i128::from(w.min(h)), 2)?,
            PadShape::Rect => zero(),
            PadShape::RoundRect => {
                if pad.roundrect_rratio_ppm > 500_000 {
                    return Err(GeometryError::UnsupportedPadGeometry);
                }
                R::new(
                    i128::from(w.min(h)) * i128::from(pad.roundrect_rratio_ppm),
                    1_000_000,
                )?
            }
        };
        let center = point(pad.position);
        let half = [
            R::new(i128::from(w), 2)?.checked_sub(radius)?,
            R::new(i128::from(h), 2)?.checked_sub(radius)?,
        ];
        Ok(Self {
            min: [
                center[0].checked_sub(half[0])?,
                center[1].checked_sub(half[1])?,
            ],
            max: [
                center[0].checked_add(half[0])?,
                center[1].checked_add(half[1])?,
            ],
            radius,
            center,
            hole: R::new(i128::from(pad.drill), 2)?,
        })
    }
    fn via(via: &Via) -> Result<Self> {
        if via.diameter <= 0 || via.drill < 0 || via.drill >= via.diameter {
            return Err(GeometryError::InvalidWidth);
        }
        let center = point(via.position);
        Ok(Self {
            min: center,
            max: center,
            center,
            radius: R::new(i128::from(via.diameter), 2)?,
            hole: R::new(i128::from(via.drill), 2)?,
        })
    }
    fn vertices(self) -> [P; 4] {
        [
            self.min,
            [self.max[0], self.min[1]],
            self.max,
            [self.min[0], self.max[1]],
        ]
    }
    fn contains_inner(self, p: P) -> Result<bool> {
        Ok(between(p[0], self.min[0], self.max[0])? && between(p[1], self.min[1], self.max[1])?)
    }
}
fn within_hole_vertices(
    vertices: &[P],
    radius: R,
    center: P,
    hole: R,
    extra: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    let limit = hole.checked_sub(extra)?.checked_sub(radius)?;
    if limit.numerator <= 0 {
        return Ok(false);
    }
    // Contact counts tangency; strict clearance excludes equality. A shape
    // entirely inside a hole has no occupied contact with the annular copper.
    let required = if boundary == DistanceBoundary::Inclusive {
        DistanceBoundary::Strict
    } else {
        DistanceBoundary::Inclusive
    };
    for v in vertices {
        if !required.accepts(norm(sub(*v, center)?)?.compare(limit.square()?)?) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn track_inside_hole(
    track: &Track,
    shape: RoundedBox,
    extra: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    let limit = shape
        .hole
        .checked_sub(extra)?
        .checked_sub(track_half(track)?)?;
    if limit.numerator <= 0 {
        return Ok(false);
    }
    let required = if boundary == DistanceBoundary::Inclusive {
        DistanceBoundary::Strict
    } else {
        DistanceBoundary::Inclusive
    };
    for v in [track.from, track.to] {
        if !required.accepts(norm(sub(point(v), shape.center)?)?.compare(limit.square()?)?) {
            return Ok(false);
        }
    }
    if let Some(mid) = track.midpoint {
        let arc = CertifiedArc::new(track.from, mid, track.to)?;
        let target = sub(sub(shape.center, point(arc.from))?, arc.center)?;
        let farthest = [target[0].checked_neg()?, target[1].checked_neg()?];
        if target.iter().all(|v| v.numerator == 0) || arc.contains_direction(farthest)? {
            let mut expression = Expression::new([norm(target)?, arc.radius_squared, zero()])?;
            expression.coefficients[0] = limit.checked_neg()?;
            expression.coefficients[1] = R::integer(1);
            expression.coefficients[2] = R::integer(1);
            if !required.accepts(expression.sign()?.cmp(&0)) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
fn track_box(
    track: &Track,
    shape: RoundedBox,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let extra = R::integer(i128::from(extra));
    let half = track_half(track)?
        .checked_add(shape.radius)?
        .checked_add(extra)?;
    let mut contact =
        shape.contains_inner(point(track.from))? || shape.contains_inner(point(track.to))?;
    let vertices = shape.vertices();
    for i in 0..4 {
        contact |= track_edge(track, vertices[i], vertices[(i + 1) % 4], half, boundary)?;
    }
    Ok(contact && !track_inside_hole(track, shape, extra, boundary)?)
}
pub fn track_pad_within(
    track: &Track,
    pad: &PlacedPad,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    track_box(track, RoundedBox::pad(pad)?, extra, boundary)
}
pub fn track_via_within(
    track: &Track,
    via: &Via,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    track_box(track, RoundedBox::via(via)?, extra, boundary)
}
fn boxes_within(
    a: RoundedBox,
    b: RoundedBox,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let extra = R::integer(i128::from(extra));
    let delta = [
        max(
            zero(),
            max(
                a.min[0].checked_sub(b.max[0])?,
                b.min[0].checked_sub(a.max[0])?,
            )?,
        )?,
        max(
            zero(),
            max(
                a.min[1].checked_sub(b.max[1])?,
                b.min[1].checked_sub(a.max[1])?,
            )?,
        )?,
    ];
    let radius = a.radius.checked_add(b.radius)?.checked_add(extra)?;
    let outer_contact = if radius.numerator == 0 && boundary == DistanceBoundary::Strict {
        lt(a.min[0], b.max[0])?
            && lt(b.min[0], a.max[0])?
            && lt(a.min[1], b.max[1])?
            && lt(b.min[1], a.max[1])?
    } else {
        boundary.accepts(norm(delta)?.compare(radius.square()?)?)
    };
    if !outer_contact {
        return Ok(false);
    }
    Ok(
        !within_hole_vertices(&a.vertices(), a.radius, b.center, b.hole, extra, boundary)?
            && !within_hole_vertices(&b.vertices(), b.radius, a.center, a.hole, extra, boundary)?,
    )
}
pub fn pads_within(
    a: &PlacedPad,
    b: &PlacedPad,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    boxes_within(RoundedBox::pad(a)?, RoundedBox::pad(b)?, extra, boundary)
}
pub fn pad_via_within(
    a: &PlacedPad,
    b: &Via,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    boxes_within(RoundedBox::pad(a)?, RoundedBox::via(b)?, extra, boundary)
}
pub fn vias_within(a: &Via, b: &Via, extra: i64, boundary: DistanceBoundary) -> Result<bool> {
    boxes_within(RoundedBox::via(a)?, RoundedBox::via(b)?, extra, boundary)
}

fn polygon_vertices(polygon: &Polygon) -> Result<Vec<P>> {
    if !polygon.closed {
        return Err(GeometryError::InvalidPolygon);
    }
    let mut vertices = polygon.vertices.clone();
    vertices.dedup();
    if vertices.first() == vertices.last() {
        vertices.pop();
    }
    if vertices.len() < 3 {
        return Err(GeometryError::InvalidPolygon);
    }
    let vertices: Vec<_> = vertices.into_iter().map(point).collect();
    let mut area = zero();
    for i in 0..vertices.len() {
        area = area.checked_add(cross(vertices[i], vertices[(i + 1) % vertices.len()])?)?;
    }
    if area.numerator == 0 {
        return Err(GeometryError::InvalidPolygon);
    }
    for i in 0..vertices.len() {
        for j in i + 1..vertices.len() {
            if (i + 1) % vertices.len() == j || (j + 1) % vertices.len() == i {
                continue;
            }
            if segment_pair(
                vertices[i],
                vertices[(i + 1) % vertices.len()],
                vertices[j],
                vertices[(j + 1) % vertices.len()],
                zero(),
                DistanceBoundary::Inclusive,
            )? {
                return Err(GeometryError::InvalidPolygon);
            }
        }
    }
    Ok(vertices)
}
fn inside(p: P, vertices: &[P]) -> Result<bool> {
    let mut winding = 0;
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        if rational_point_segment_within(p, a, b, zero(), DistanceBoundary::Inclusive)? {
            return Ok(true);
        }
        let sign = cross(sub(b, a)?, sub(p, a)?)?.numerator.signum();
        if a[1].compare(p[1])? != Ordering::Greater
            && b[1].compare(p[1])? == Ordering::Greater
            && sign > 0
        {
            winding += 1
        }
        if a[1].compare(p[1])? == Ordering::Greater
            && b[1].compare(p[1])? != Ordering::Greater
            && sign < 0
        {
            winding -= 1
        }
    }
    Ok(winding != 0)
}
pub fn polygon_contains_point(polygon: &Polygon, p: Point) -> Result<bool> {
    inside(point(p), &polygon_vertices(polygon)?)
}
pub fn track_polygon_within(
    track: &Track,
    polygon: &Polygon,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    validate_track(track)?;
    let vertices = polygon_vertices(polygon)?;
    let half = track_half(track)?.checked_add(R::integer(i128::from(extra)))?;
    if inside(point(track.from), &vertices)? || inside(point(track.to), &vertices)? {
        return Ok(true);
    }
    for i in 0..vertices.len() {
        if track_edge(
            track,
            vertices[i],
            vertices[(i + 1) % vertices.len()],
            half,
            boundary,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}
fn polygon_box(
    polygon: &Polygon,
    shape: RoundedBox,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let vertices = polygon_vertices(polygon)?;
    let box_vertices = shape.vertices();
    let extra = R::integer(i128::from(extra));
    let half = shape.radius.checked_add(extra)?;
    if half.numerator == 0 && boundary == DistanceBoundary::Strict {
        return Err(GeometryError::UnresolvedPredicate);
    }
    let mut contact = false;
    for v in &vertices {
        contact |= shape.contains_inner(*v)?;
    }
    for v in box_vertices {
        contact |= inside(v, &vertices)?;
    }
    for i in 0..vertices.len() {
        for j in 0..4 {
            contact |= segment_pair(
                vertices[i],
                vertices[(i + 1) % vertices.len()],
                box_vertices[j],
                box_vertices[(j + 1) % 4],
                half,
                boundary,
            )?;
        }
    }
    Ok(contact
        && !within_hole_vertices(&vertices, zero(), shape.center, shape.hole, extra, boundary)?)
}
pub fn pad_polygon_within(
    pad: &PlacedPad,
    polygon: &Polygon,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    polygon_box(polygon, RoundedBox::pad(pad)?, extra, boundary)
}
pub fn via_polygon_within(
    via: &Via,
    polygon: &Polygon,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    polygon_box(polygon, RoundedBox::via(via)?, extra, boundary)
}
pub fn polygons_within(
    a: &Polygon,
    b: &Polygon,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    if extra == 0 && boundary == DistanceBoundary::Strict {
        return Err(GeometryError::UnresolvedPredicate);
    }
    let a = polygon_vertices(a)?;
    let b = polygon_vertices(b)?;
    if inside(a[0], &b)? || inside(b[0], &a)? {
        return Ok(true);
    }
    let half = R::integer(i128::from(extra));
    for i in 0..a.len() {
        for j in 0..b.len() {
            if segment_pair(
                a[i],
                a[(i + 1) % a.len()],
                b[j],
                b[(j + 1) % b.len()],
                half,
                boundary,
            )? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn layer_index(stackup: &Stackup, id: LayerId) -> Result<usize> {
    let indices: Vec<_> = stackup
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.id == id && l.layer_type == StackupLayerType::Copper)
        .map(|(i, _)| i)
        .collect();
    if indices.len() != 1 {
        return Err(GeometryError::UnknownConductiveLayer);
    }
    Ok(indices[0])
}
pub fn track_layers(stackup: &Stackup, track: &Track) -> Result<Vec<LayerId>> {
    layer_index(stackup, track.layer)?;
    Ok(vec![track.layer])
}
pub fn pad_layers(stackup: &Stackup, pad: &PlacedPad) -> Result<Vec<LayerId>> {
    let mut layers = if pad.copper_layers.is_empty() {
        vec![pad.layer]
    } else {
        pad.copper_layers.clone()
    };
    for id in &layers {
        layer_index(stackup, *id)?;
    }
    layers.sort();
    layers.dedup();
    Ok(layers)
}
pub fn via_layers(stackup: &Stackup, via: &Via) -> Result<Vec<LayerId>> {
    let a = layer_index(stackup, via.from_layer)?;
    let b = layer_index(stackup, via.to_layer)?;
    Ok(stackup.layers[a.min(b)..=a.max(b)]
        .iter()
        .filter(|l| l.layer_type == StackupLayerType::Copper)
        .map(|l| l.id)
        .collect())
}

#[cfg(test)]
#[path = "occupied_copper_tests.rs"]
mod tests;
