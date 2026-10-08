//! Finite exact boundary arrangement. Root-valued witnesses are query scratch.
use super::*;
#[derive(Clone, Copy)]
struct Witness {
    a: V,
    b: V,
    k: Q,
}
impl Witness {
    fn point(a: V) -> Self {
        Self {
            a,
            b: rational_point([zero(), zero()]),
            k: Q::rational(zero()),
        }
    }
    fn coordinate(&self, axis: usize) -> Scalar {
        Scalar {
            a: self.a[axis],
            b: self.b[axis],
            k: self.k,
        }
    }
}
#[derive(Clone, Copy)]
struct Scalar {
    a: Q,
    b: Q,
    k: Q,
}
impl Scalar {
    fn sign(self) -> Result<i8> {
        root_sign(self.a, self.b, self.k)
    }
    fn sub_q(self, q: Q) -> Result<Self> {
        Ok(Self {
            a: self.a.sub(q)?,
            ..self
        })
    }
    fn square(self) -> Result<Self> {
        Ok(Self {
            a: self.a.square()?.add(self.b.square()?.mul(self.k)?)?,
            b: self.a.mul(self.b)?.mul(Q::rational(R::integer(2)))?,
            ..self
        })
    }
    fn add(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            a: self.a.add(rhs.a)?,
            b: self.b.add(rhs.b)?,
            ..self
        })
    }
}
#[derive(Clone, Copy)]
struct Rounded {
    center: V,
    axes: [V; 2],
    half: [R; 2],
    radius: R,
    hole: R,
}
#[derive(Clone, Copy)]
enum Feature {
    Segment(V, V),
    Circle {
        center: V,
        radius: R,
        start: V,
        end: V,
        orientation: i8,
        full: bool,
    },
}
fn scalar_dot(p: Witness, d: V) -> Result<Scalar> {
    Ok(Scalar {
        a: dot(p.a, d)?,
        b: dot(p.b, d)?,
        k: p.k,
    })
}
fn scalar_cross(p: Witness, d: V) -> Result<Scalar> {
    Ok(Scalar {
        a: determinant(p.a, d)?,
        b: determinant(p.b, d)?,
        k: p.k,
    })
}
fn relative(p: Witness, c: V) -> Result<Witness> {
    Ok(Witness {
        a: difference(p.a, c)?,
        ..p
    })
}
fn norm_w(p: Witness) -> Result<Scalar> {
    p.coordinate(0).square()?.add(p.coordinate(1).square()?)
}
fn transformed(r: Rounded, p: V) -> Result<V> {
    sum(
        r.center,
        sum(scaled(r.axes[0], p[0])?, scaled(r.axes[1], p[1])?)?,
    )
}
fn vector(x: R, y: R) -> V {
    rational_point([x, y])
}
impl Rounded {
    fn contains(self, p: Witness) -> Result<bool> {
        let p = relative(p, self.center)?;
        let mut distance = Scalar {
            a: Q::rational(zero()),
            b: Q::rational(zero()),
            k: p.k,
        };
        for axis in 0..2 {
            let x = scalar_dot(p, self.axes[axis])?;
            let lo = Q::rational(self.half[axis].checked_neg()?);
            let hi = Q::rational(self.half[axis]);
            let delta = if x.sub_q(lo)?.sign()? < 0 {
                Some(x.sub_q(lo)?)
            } else if x.sub_q(hi)?.sign()? > 0 {
                Some(x.sub_q(hi)?)
            } else {
                None
            };
            if let Some(delta) = delta {
                distance = distance.add(delta.square()?)?;
            }
        }
        Ok(
            distance.sub_q(Q::rational(self.radius.square()?))?.sign()? <= 0
                && norm_w(p)?.sub_q(Q::rational(self.hole.square()?))?.sign()? >= 0,
        )
    }
    fn features(self) -> Result<Vec<Feature>> {
        let mut out = Vec::new();
        let [x, y] = self.half;
        if x.numerator == 0 && y.numerator == 0 {
            out.push(self.circle(self.center, self.radius)?);
        } else {
            let n = |v: R| v.checked_neg();
            let corners = [
                vector(n(x)?, n(y)?),
                vector(x, n(y)?),
                vector(x, y),
                vector(n(x)?, y),
            ];
            let offsets = [
                vector(zero(), n(self.radius)?),
                vector(self.radius, zero()),
                vector(zero(), self.radius),
                vector(n(self.radius)?, zero()),
            ];
            let orientation = determinant(self.axes[0], self.axes[1])?.sign()?;
            for i in 0..4 {
                let j = (i + 1) % 4;
                out.push(Feature::Segment(
                    transformed(self, sum(corners[i], offsets[i])?)?,
                    transformed(self, sum(corners[j], offsets[i])?)?,
                ));
                if self.radius.numerator > 0 {
                    let center = transformed(self, corners[j])?;
                    out.push(Feature::Circle {
                        center,
                        radius: self.radius,
                        start: difference(
                            transformed(self, sum(corners[j], offsets[i])?)?,
                            center,
                        )?,
                        end: difference(transformed(self, sum(corners[j], offsets[j])?)?, center)?,
                        orientation,
                        full: false,
                    });
                }
            }
        }
        if self.hole.numerator > 0 {
            out.push(self.circle(self.center, self.hole)?);
        }
        Ok(out)
    }
    fn circle(self, center: V, radius: R) -> Result<Feature> {
        let start = scaled(self.axes[0], Q::rational(radius))?;
        Ok(Feature::Circle {
            center,
            radius,
            start,
            end: start,
            orientation: 1,
            full: true,
        })
    }
}
fn rounded(pad: &SourceRelativePadGeometry) -> Result<Rounded> {
    let s = shape(pad)?;
    let axes = [
        pad.displacement([R::integer(1), zero()])?.map(Q::exact),
        pad.displacement([zero(), R::integer(1)])?.map(Q::exact),
    ];
    Ok(Rounded {
        center: pad.center.map(Q::exact),
        axes,
        half: [s.max[0], s.max[1]],
        radius: s.radius,
        hole: s.hole,
    })
}
fn imported(pad: &PlacedPad) -> Result<Rounded> {
    let mut local = pad.clone();
    local.position = Point::zero();
    local.rotation = 0;
    let s = RoundedBox::pad(&local)?;
    let q = Q::rational;
    let (c, t) = if pad.shape == PadShape::Circle {
        (q(R::integer(1)), q(zero()))
    } else {
        match pad.rotation.rem_euclid(360) {
            0 => (q(R::integer(1)), q(zero())),
            90 => (q(zero()), q(R::integer(1))),
            180 => (q(R::integer(-1)), q(zero())),
            270 => (q(zero()), q(R::integer(-1))),
            angle @ (45 | 135 | 225 | 315) => {
                let c = if angle == 45 || angle == 315 { 1 } else { -1 };
                let t = if angle == 45 || angle == 135 { 1 } else { -1 };
                (
                    Q::two(super::super::quadratic::Q::root(R::new(c, 2)?)),
                    Q::two(super::super::quadratic::Q::root(R::new(t, 2)?)),
                )
            }
            _ => return Err(GeometryError::UnsupportedPadGeometry),
        }
    };
    Ok(Rounded {
        center: rational_point(point(pad.position)),
        axes: [[c, t], [t.neg()?, c]],
        half: [s.max[0], s.max[1]],
        radius: s.radius,
        hole: s.hole,
    })
}
impl Feature {
    fn contains(self, p: Witness) -> Result<bool> {
        match self {
            Self::Segment(a, b) => {
                let d = difference(b, a)?;
                let p = relative(p, a)?;
                if squared(d)?.sign()? == 0 {
                    return Ok(norm_w(p)?.sign()? == 0);
                }
                if scalar_cross(p, d)?.sign()? != 0 {
                    return Ok(false);
                }
                let u = scalar_dot(p, d)?;
                Ok(u.sign()? >= 0 && u.sub_q(squared(d)?)?.sign()? <= 0)
            }
            Self::Circle {
                center,
                radius,
                start,
                end,
                orientation,
                full,
            } => {
                let p = relative(p, center)?;
                if norm_w(p)?.sub_q(Q::rational(radius.square()?))?.sign()? != 0 {
                    return Ok(false);
                }
                if full {
                    return Ok(true);
                }
                // Quarter arcs in each shape's exact frame.
                Ok(scalar_cross(p, start)?.sign()? * orientation <= 0
                    && scalar_cross(p, end)?.sign()? * orientation >= 0)
            }
        }
    }
    fn witness(self) -> Result<V> {
        match self {
            Self::Segment(a, _) => Ok(a),
            Self::Circle { center, start, .. } => sum(center, start),
        }
    }
}
fn intersections(a: Feature, b: Feature) -> Result<Vec<Witness>> {
    let mut out = Vec::new();
    match (a, b) {
        (Feature::Segment(a, b), Feature::Segment(c, d)) => {
            let u = difference(b, a)?;
            let v = difference(d, c)?;
            let den = determinant(u, v)?;
            if den.sign()? == 0 {
                out.extend([a, b, c, d].map(Witness::point));
            } else {
                out.push(Witness::point(sum(
                    a,
                    scaled(u, determinant(difference(c, a)?, v)?.div(den)?)?,
                )?));
            }
        }
        (Feature::Segment(a, b), Feature::Circle { center, radius, .. })
        | (Feature::Circle { center, radius, .. }, Feature::Segment(a, b)) => {
            let d = difference(b, a)?;
            let len = squared(d)?;
            if len.sign()? == 0 {
                out.push(Witness::point(a));
            } else {
                let v = difference(a, center)?;
                let u = dot(v, d)?.neg()?.div(len)?;
                let foot = sum(a, scaled(d, u)?)?;
                let k = Q::rational(radius.square()?)
                    .sub(squared(difference(foot, center)?)?)?
                    .div(len)?;
                if k.sign()? >= 0 {
                    for sign in [-1, 1] {
                        out.push(Witness {
                            a: foot,
                            b: scaled(d, Q::rational(R::integer(sign)))?,
                            k,
                        });
                    }
                }
            }
        }
        (
            Feature::Circle {
                center: a,
                radius: r,
                ..
            },
            Feature::Circle {
                center: b,
                radius: s,
                ..
            },
        ) => {
            let d = difference(b, a)?;
            let len = squared(d)?;
            if len.sign()? == 0 {
                // Coincident interval overlap is decided by exact endpoints from
                // both features, including the full-circle cardinal witness.
                if r == s {
                    out.push(Witness::point(a));
                } // endpoints appended below
            } else {
                let u = len
                    .add(Q::rational(r.square()?.checked_sub(s.square()?)?))?
                    .div(len.mul(Q::rational(R::integer(2)))?)?;
                let foot = sum(a, scaled(d, u)?)?;
                let k = Q::rational(r.square()?).div(len)?.sub(u.square()?)?;
                if k.sign()? >= 0 {
                    let perpendicular = [d[1].neg()?, d[0]];
                    for sign in [-1, 1] {
                        out.push(Witness {
                            a: foot,
                            b: scaled(perpendicular, Q::rational(R::integer(sign)))?,
                            k,
                        });
                    }
                }
            }
        }
    }
    // Also covers coincident intervals and degenerate boundary features.
    for f in [a, b] {
        out.push(Witness::point(f.witness()?));
        if let Feature::Circle { center, end, .. } = f {
            out.push(Witness::point(sum(center, end)?));
        }
        if let Feature::Segment(_, end) = f {
            out.push(Witness::point(end));
        }
    }
    let mut valid = Vec::new();
    for p in out {
        if a.contains(p)? && b.contains(p)? {
            valid.push(p);
        }
    }
    Ok(valid)
}
fn pair(a: Rounded, b: Rounded) -> Result<bool> {
    let af = a.features()?;
    let bf = b.features()?;
    for f in &af {
        if b.contains(Witness::point(f.witness()?))? {
            return Ok(true);
        }
    }
    for f in &bf {
        if a.contains(Witness::point(f.witness()?))? {
            return Ok(true);
        }
    }
    for f in af {
        for g in &bf {
            for p in intersections(f, *g)? {
                if a.contains(p)? && b.contains(p)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
pub(crate) fn pads(
    a: &SourceRelativePadGeometry,
    b: &crate::board::query_pad::QueryPadGeometry,
) -> Result<bool> {
    use crate::board::query_pad::QueryPadGeometry;
    pair(
        rounded(a)?,
        match b {
            QueryPadGeometry::Integer(p) => imported(p)?,
            QueryPadGeometry::SourceRelative(p) => rounded(p)?,
        },
    )
}
pub(crate) fn via(a: &SourceRelativePadGeometry, b: &Via) -> Result<bool> {
    let s = RoundedBox::via(b)?;
    pair(
        rounded(a)?,
        Rounded {
            center: rational_point(s.center),
            axes: [vector(R::integer(1), zero()), vector(zero(), R::integer(1))],
            half: [zero(), zero()],
            radius: s.radius,
            hole: s.hole,
        },
    )
}
fn polygon_contains(vertices: &[V], p: Witness) -> Result<bool> {
    let mut inside = false;
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        if Feature::Segment(a, b).contains(p)? {
            return Ok(true);
        }
        let ay = p.coordinate(1).sub_q(a[1])?.sign()?;
        let by = p.coordinate(1).sub_q(b[1])?.sign()?;
        if (ay >= 0) != (by >= 0) {
            let cross = scalar_cross(relative(p, a)?, difference(b, a)?)?.sign()?;
            if cross * b[1].sub(a[1])?.sign()? < 0 {
                inside = !inside;
            }
        }
    }
    Ok(inside)
}
pub(crate) fn polygon(pad: &SourceRelativePadGeometry, poly: &CertifiedPolygon) -> Result<bool> {
    let vertices = polygon_vertices(poly.source())?
        .into_iter()
        .map(rational_point)
        .collect::<Vec<_>>();
    let a = rounded(pad)?;
    let af = a.features()?;
    for p in &vertices {
        if a.contains(Witness::point(*p))? {
            return Ok(true);
        }
    }
    for f in &af {
        if polygon_contains(&vertices, Witness::point(f.witness()?))? {
            return Ok(true);
        }
    }
    for i in 0..vertices.len() {
        let g = Feature::Segment(vertices[i], vertices[(i + 1) % vertices.len()]);
        for f in &af {
            for p in intersections(*f, g)? {
                if a.contains(p)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
