//! Conservative exact bounds for already-validated physical sources. Bounds
//! only exclude disjoint pairs; surviving pairs use the existing contact kernel.
use super::*;
use crate::board::nominal_geometry::CertifiedArc;
type P = [Rational; 2];
#[derive(Clone)]
pub(super) struct Bounds {
    min: P,
    max: P,
}
impl Bounds {
    pub(super) fn new(shape: &Shape) -> Option<Self> {
        Self::checked(shape).ok()
    }
    fn checked(shape: &Shape) -> Result<Self, GeometryError> {
        let p = |p: Point| {
            [
                Rational::integer(i128::from(p.x)),
                Rational::integer(i128::from(p.y)),
            ]
        };
        match shape {
            Shape::Fill(polygon) => {
                let (min, max) = polygon.bounds();
                Ok(Self { min, max })
            }
            Shape::Via(via) => Self::expanded(
                p(via.position),
                p(via.position),
                Rational::new(i128::from(via.diameter), 2)?,
            ),
            Shape::Pad(query) => {
                let pad = query.terminal();
                if let QueryPadGeometry::SourceRelative(exact) = query {
                    let x = exact.center[0].floor_ceil_nm()?;
                    let y = exact.center[1].floor_ceil_nm()?;
                    let diameter = if pad.shape == crate::board::PadShape::Circle {
                        i128::from(pad.diameter)
                    } else {
                        i128::from(pad.width) + i128::from(pad.height)
                    };
                    return Self::expanded(
                        [
                            Rational::integer(i128::from(x.0)),
                            Rational::integer(i128::from(y.0)),
                        ],
                        [
                            Rational::integer(i128::from(x.1)),
                            Rational::integer(i128::from(y.1)),
                        ],
                        Rational::new(diameter, 2)?,
                    );
                }
                let diameter = if pad.shape == crate::board::PadShape::Circle {
                    i128::from(pad.diameter)
                } else {
                    i128::from(pad.width) + i128::from(pad.height)
                };
                // The enclosing disk is rotation invariant. No pad centre/pose
                // is rounded or certified by this rejection-only bound.
                Self::expanded(
                    p(pad.position),
                    p(pad.position),
                    Rational::new(diameter, 2)?,
                )
            }
            Shape::Track(track) => {
                let half = Rational::new(i128::from(track.width), 2)?;
                if let Some(mid) = track.midpoint {
                    let arc = CertifiedArc::new(track.from, mid, track.to)?;
                    let abs = |v: Rational| {
                        if v.numerator < 0 {
                            v.checked_neg()
                        } else {
                            Ok(v)
                        }
                    };
                    let radius = abs(arc.center[0])?
                        .checked_add(abs(arc.center[1])?)?
                        .checked_add(half)?;
                    let from = p(track.from);
                    let center = [
                        from[0].checked_add(arc.center[0])?,
                        from[1].checked_add(arc.center[1])?,
                    ];
                    Self::expanded(center, center, radius)
                } else {
                    let a = p(track.from);
                    let b = p(track.to);
                    Self::expanded(
                        std::array::from_fn(|i| {
                            Rational::integer(a[i].numerator.min(b[i].numerator))
                        }),
                        std::array::from_fn(|i| {
                            Rational::integer(a[i].numerator.max(b[i].numerator))
                        }),
                        half,
                    )
                }
            }
        }
    }
    fn expanded(min: P, max: P, radius: Rational) -> Result<Self, GeometryError> {
        Ok(Self {
            min: [min[0].checked_sub(radius)?, min[1].checked_sub(radius)?],
            max: [max[0].checked_add(radius)?, max[1].checked_add(radius)?],
        })
    }
    pub(super) fn disjoint(&self, other: &Self) -> bool {
        (0..2).any(|i| {
            self.min[i].compare(other.max[i]).is_ok_and(|o| o.is_gt())
                || other.min[i].compare(self.max[i]).is_ok_and(|o| o.is_gt())
        })
    }
}
