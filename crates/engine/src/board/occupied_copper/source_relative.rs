//! Exact source-relative 30-degree point and Track predicates.
//! Rigidly transform query features into the pad frame; source
//! pose is immutable. Both straight and circular Track candidates are complete.
use super::mixed_field::*;
use super::*;
use crate::board::query_pad::SourceRelativePadGeometry;
mod arrangement;
pub(crate) use arrangement::{pads, polygon, via};

fn local(pad: &SourceRelativePadGeometry, p: P) -> Result<V> {
    let point = pad.local_point(p.map(crate::board::quadratic3::Quadratic3::rational))?;
    Ok(point.map(Q::exact))
}
fn shape(pad: &SourceRelativePadGeometry) -> Result<RoundedBox> {
    let mut intrinsic = pad.local.clone();
    intrinsic.rotation = 0;
    intrinsic.position = Point::zero();
    RoundedBox::pad(&intrinsic)
}
fn clamped(x: Q, lo: R, hi: R) -> Result<Q> {
    if x.compare(Q::rational(lo))? == Ordering::Less {
        Ok(Q::rational(lo))
    } else if x.compare(Q::rational(hi))? == Ordering::Greater {
        Ok(Q::rational(hi))
    } else {
        Ok(x)
    }
}
fn distance_box(p: V, shape: RoundedBox) -> Result<Q> {
    squared(difference(
        p,
        [
            clamped(p[0], shape.min[0], shape.max[0])?,
            clamped(p[1], shape.min[1], shape.max[1])?,
        ],
    )?)
}
fn in_core(p: V, shape: RoundedBox) -> Result<bool> {
    Ok(distance_box(p, shape)?.sign()? == 0)
}
fn unit(u: Q) -> Result<bool> {
    Ok(u.sign()? >= 0 && u.compare(Q::rational(R::integer(1)))? != Ordering::Greater)
}
fn point_segment(p: V, a: V, b: V, half: R, boundary: DistanceBoundary) -> Result<bool> {
    let delta = difference(b, a)?;
    let v = difference(p, a)?;
    let len = squared(delta)?;
    if len.sign()? == 0 {
        return Ok(boundary.accepts(squared(v)?.compare(Q::rational(half.square()?))?));
    }
    let u = dot(v, delta)?.div(len)?;
    if !unit(u)? {
        let endpoint = if u.sign()? < 0 { a } else { b };
        return Ok(boundary
            .accepts(squared(difference(p, endpoint)?)?.compare(Q::rational(half.square()?))?));
    }
    Ok(boundary.accepts(
        determinant(v, delta)?
            .square()?
            .compare(Q::rational(half.square()?).mul(len)?)?,
    ))
}
struct Arc {
    source: CertifiedArc,
    from: V,
    to: V,
    center: V,
    start: V,
    end: V,
}
impl Arc {
    fn new(pad: &SourceRelativePadGeometry, track: &Track) -> Result<Self> {
        let source = CertifiedArc::new(
            track.from,
            track.midpoint.ok_or(GeometryError::InvalidArc)?,
            track.to,
        )?;
        let from = local(pad, point(track.from))?;
        let to = local(pad, point(track.to))?;
        let center = local(
            pad,
            [
                R::integer(i128::from(track.from.x)).checked_add(source.center[0])?,
                R::integer(i128::from(track.from.y)).checked_add(source.center[1])?,
            ],
        )?;
        Ok(Self {
            source,
            from,
            to,
            center,
            start: difference(from, center)?,
            end: difference(to, center)?,
        })
    }
    fn direction(&self, a: V, b: V, k: Q) -> Result<bool> {
        let orientation = if self.source.counterclockwise { 1 } else { -1 };
        let first = root_sign(determinant(self.start, a)?, determinant(self.start, b)?, k)?
            * orientation
            >= 0;
        let last =
            root_sign(determinant(a, self.end)?, determinant(b, self.end)?, k)? * orientation >= 0;
        if determinant(self.start, self.end)?.sign()? * orientation >= 0 {
            Ok(first && last)
        } else {
            Ok(first || last)
        }
    }
    fn point(&self, p: V, half: R, boundary: DistanceBoundary) -> Result<bool> {
        let v = difference(p, self.center)?;
        let zeros = rational_point([zero(), zero()]);
        if squared(v)?.sign()? == 0 || self.direction(v, zeros, Q::rational(zero()))? {
            return radial_gap(squared(v)?, self.source.radius_squared, half, boundary);
        }
        Ok(boundary
            .accepts(squared(difference(p, self.from)?)?.compare(Q::rational(half.square()?))?)
            || boundary
                .accepts(squared(difference(p, self.to)?)?.compare(Q::rational(half.square()?))?))
    }
    fn edge(&self, a: V, b: V, half: R, boundary: DistanceBoundary) -> Result<bool> {
        if self.point(a, half, boundary)?
            || self.point(b, half, boundary)?
            || point_segment(self.from, a, b, half, boundary)?
            || point_segment(self.to, a, b, half, boundary)?
        {
            return Ok(true);
        }
        let relative = difference(a, self.center)?;
        let d = difference(b, a)?;
        let len = squared(d)?;
        if len.sign()? == 0 {
            return Ok(false);
        }
        let u = dot(relative, d)?.neg()?.div(len)?;
        let foot = sum(relative, scaled(d, u)?)?;
        let q = squared(foot)?;
        let zeros = rational_point([zero(), zero()]);
        if unit(u)?
            && self.direction(foot, zeros, Q::rational(zero()))?
            && radial_gap(q, self.source.radius_squared, half, boundary)?
        {
            return Ok(true);
        }
        let k = Q::rational(self.source.radius_squared).sub(q)?.div(len)?;
        if k.sign()? < 0 {
            return Ok(false);
        }
        for sign in [-1, 1] {
            let s = Q::rational(R::integer(sign));
            if root_sign(u, s, k)? >= 0
                && root_sign(Q::rational(R::integer(1)).sub(u)?, s.neg()?, k)? >= 0
                && self.direction(foot, scaled(d, s)?, k)?
            {
                return Ok(boundary == DistanceBoundary::Inclusive || half.numerator > 0);
            }
        }
        Ok(false)
    }
}
fn radial_gap(distance: Q, radius: R, half: R, boundary: DistanceBoundary) -> Result<bool> {
    let lhs = distance.add(Q::rational(radius.checked_sub(half.square()?)?))?;
    if lhs.sign()? < 0 {
        return Ok(true);
    }
    let product = distance.mul(Q::rational(radius.checked_mul(R::integer(4))?))?;
    if lhs.sign()? == 0 {
        return Ok(boundary == DistanceBoundary::Inclusive || product.sign()? > 0);
    }
    Ok(boundary.accepts(lhs.square()?.compare(product)?))
}
pub(crate) fn contains(pad: &SourceRelativePadGeometry, p: Point) -> Result<bool> {
    let shape = shape(pad)?;
    Ok(
        distance_box(local(pad, point(p))?, shape)?.compare(Q::rational(shape.radius.square()?))?
            != Ordering::Greater
            && squared(local(pad, point(p))?)?.compare(Q::rational(shape.hole.square()?))?
                != Ordering::Less,
    )
}
pub(crate) fn track(
    pad: &SourceRelativePadGeometry,
    track: &Track,
    extra: i64,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if extra < 0 {
        return Err(GeometryError::InvalidWidth);
    }
    let shape = shape(pad)?;
    let half = track_half(track)?
        .checked_add(shape.radius)?
        .checked_add(R::integer(i128::from(extra)))?;
    let from = local(pad, point(track.from))?;
    let to = local(pad, point(track.to))?;
    let vertices = shape.vertices().map(rational_point);
    let mut contact = in_core(from, shape)? || in_core(to, shape)?;
    if track.midpoint.is_some() {
        let arc = Arc::new(pad, track)?;
        for i in 0..4 {
            contact |= arc.edge(vertices[i], vertices[(i + 1) % 4], half, boundary)?;
        }
    } else {
        for p in [from, to] {
            contact |=
                boundary.accepts(distance_box(p, shape)?.compare(Q::rational(half.square()?))?);
        }
        for p in vertices {
            contact |= point_segment(p, from, to, half, boundary)?;
        }
        // Segment crosses the central rectangle with endpoints outside.
        for i in 0..4 {
            let a = vertices[i];
            let b = vertices[(i + 1) % 4];
            let d = difference(to, from)?;
            let edge = difference(b, a)?;
            let offset = difference(a, from)?;
            let det = determinant(d, edge)?;
            if det.sign()? != 0
                && unit(determinant(offset, edge)?.div(det)?)?
                && unit(determinant(offset, d)?.div(det)?)?
            {
                contact = true;
            }
        }
    }
    Ok(contact
        && !inside_hole(
            pad,
            track,
            shape.hole,
            R::integer(i128::from(extra)),
            boundary,
        )?)
}
/// The entire stroked Track lies strictly within the removed drill disk.
fn inside_hole(
    pad: &SourceRelativePadGeometry,
    track: &Track,
    hole: R,
    extra: R,
    boundary: DistanceBoundary,
) -> Result<bool> {
    if hole.numerator == 0 {
        return Ok(false);
    }
    let limit = hole.checked_sub(extra)?.checked_sub(track_half(track)?)?;
    if limit.numerator <= 0 {
        return Ok(false);
    }
    let accepts = |d: Q| -> Result<bool> {
        let order = d.compare(Q::rational(limit.square()?))?;
        Ok(order == Ordering::Less
            || (boundary == DistanceBoundary::Strict && order == Ordering::Equal))
    };
    if !accepts(squared(local(pad, point(track.from))?)?)?
        || !accepts(squared(local(pad, point(track.to))?)?)?
    {
        return Ok(false);
    }
    if track.midpoint.is_some() {
        let arc = Arc::new(pad, track)?;
        let u = squared(arc.center)?;
        let v = Q::rational(arc.source.radius_squared);
        // Farthest radial extremum from the drill centre, if on the sweep.
        if u.sign()? == 0
            || arc.direction(
                arc.center,
                rational_point([zero(), zero()]),
                Q::rational(zero()),
            )?
        {
            let w = Q::rational(limit.square()?).sub(u)?.sub(v)?;
            if w.sign()? < 0 {
                return Ok(false);
            }
            let order = w
                .square()?
                .compare(u.mul(v)?.mul(Q::rational(R::integer(4)))?)?;
            if order == Ordering::Less
                || (order == Ordering::Equal && boundary == DistanceBoundary::Inclusive)
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
