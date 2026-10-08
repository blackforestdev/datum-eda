//! Typed read-only placed geometry. Local source Points never masquerade as world Points.
use super::{
    PlacedPad,
    nominal_geometry::{GeometryError, Rational},
    quadratic3::{ExactPointNm, Quadratic3},
};
use crate::ir::geometry::Point;
use crate::substrate::component_pad_source::ComponentPadSourceBinding;
type Result<T> = std::result::Result<T, GeometryError>;
#[derive(Clone, Debug)]
pub enum QueryPadGeometry {
    Integer(PlacedPad),
    SourceRelative(SourceRelativePadGeometry),
}
#[derive(Clone, Debug)]
pub struct SourceRelativePadGeometry {
    /// Intrinsic local geometry and terminal metadata; NOT world geometry.
    pub local: PlacedPad,
    pub origin: Point,
    pub back: bool,
    pub center: ExactPointNm,
    pub source_binding: ComponentPadSourceBinding,
}
impl SourceRelativePadGeometry {
    pub fn thirty(
        local: PlacedPad,
        origin: Point,
        back: bool,
        source_binding: ComponentPadSourceBinding,
    ) -> Result<Self> {
        let mut x = i128::from(local.position.x);
        if back {
            x = -x;
        }
        let y = i128::from(local.position.y);
        let center = [
            Quadratic3 {
                a: Rational::new(2 * i128::from(origin.x) - y, 2)?,
                b: Rational::new(x, 2)?,
            },
            Quadratic3 {
                a: Rational::new(2 * i128::from(origin.y) + x, 2)?,
                b: Rational::new(y, 2)?,
            },
        ];
        Ok(Self {
            local,
            origin,
            back,
            center,
            source_binding,
        })
    }
    /// Map local aperture displacement about its centre into board space.
    pub fn displacement(&self, mut p: [Rational; 2]) -> Result<ExactPointNm> {
        if self.back {
            p[0] = p[0].checked_neg()?;
        }
        let half = Rational::new(1, 2)?;
        Ok([
            Quadratic3 {
                a: p[1].checked_neg()?.checked_mul(half)?,
                b: p[0].checked_mul(half)?,
            },
            Quadratic3 {
                a: p[0].checked_mul(half)?,
                b: p[1].checked_mul(half)?,
            },
        ])
    }
    pub fn local_point(&self, p: ExactPointNm) -> Result<ExactPointNm> {
        let x = p[0].sub(self.center[0])?;
        let y = p[1].sub(self.center[1])?;
        let c = Quadratic3::root(Rational::new(1, 2)?);
        let s = Quadratic3::rational(Rational::new(1, 2)?);
        let mut u = x.mul(c)?.add(y.mul(s)?)?;
        if self.back {
            u = u.neg()?;
        }
        Ok([u, y.mul(c)?.sub(x.mul(s)?)?])
    }
}
impl QueryPadGeometry {
    pub fn terminal(&self) -> &PlacedPad {
        match self {
            Self::Integer(p) => p,
            Self::SourceRelative(p) => &p.local,
        }
    }
    pub fn terminal_mut(&mut self) -> &mut PlacedPad {
        match self {
            Self::Integer(p) => p,
            Self::SourceRelative(p) => &mut p.local,
        }
    }
    pub fn center(&self) -> ExactPointNm {
        match self {
            Self::Integer(p) => [
                Quadratic3::rational(Rational::integer(i128::from(p.position.x))),
                Quadratic3::rational(Rational::integer(i128::from(p.position.y))),
            ],
            Self::SourceRelative(p) => p.center,
        }
    }
}

impl QueryPadGeometry {
    pub fn contains(&self, p: Point) -> Result<bool> {
        match self {
            Self::Integer(pad) => super::occupied_copper::pad_contains_point(pad, p),
            Self::SourceRelative(pad) => super::occupied_copper::source_relative::contains(pad, p),
        }
    }
    pub fn contact_track(&self, track: &super::Track) -> Result<bool> {
        match self {
            Self::Integer(pad) => super::occupied_copper::track_pad_within(
                track,
                pad,
                0,
                super::nominal_geometry::DistanceBoundary::Inclusive,
            ),
            Self::SourceRelative(pad) => super::occupied_copper::source_relative::track(
                pad,
                track,
                0,
                super::nominal_geometry::DistanceBoundary::Inclusive,
            ),
        }
    }
    pub fn contact_pad(&self, rhs: &Self) -> Result<bool> {
        match (self, rhs) {
            (Self::Integer(a), Self::Integer(b)) => super::occupied_copper::pads_within(
                a,
                b,
                0,
                super::nominal_geometry::DistanceBoundary::Inclusive,
            ),
            (Self::SourceRelative(a), b) | (b, Self::SourceRelative(a)) => {
                super::occupied_copper::source_relative::pads(a, b)
            }
        }
    }
    pub fn contact_via(&self, via: &super::Via) -> Result<bool> {
        match self {
            Self::Integer(pad) => super::occupied_copper::pad_via_within(
                pad,
                via,
                0,
                super::nominal_geometry::DistanceBoundary::Inclusive,
            ),
            Self::SourceRelative(pad) => super::occupied_copper::source_relative::via(pad, via),
        }
    }
    pub(crate) fn contact_polygon(
        &self,
        polygon: &super::occupied_copper::CertifiedPolygon,
    ) -> Result<bool> {
        match self {
            Self::Integer(pad) => {
                polygon.pad_within(pad, 0, super::nominal_geometry::DistanceBoundary::Inclusive)
            }
            Self::SourceRelative(pad) => {
                super::occupied_copper::source_relative::polygon(pad, polygon)
            }
        }
    }
}
