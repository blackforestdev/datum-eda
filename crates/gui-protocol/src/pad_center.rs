//! Version-1 integer compatibility and explicit version-2 exact placement data.
use crate::{PointNm, RectNm};
pub use eda_engine::board::quadratic3::{ExactPointNm, Quadratic3};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PadCenter {
    Integer(PointNm),
    Exact {
        exact: ExactPointNm,
        local_width_nm: i64,
        local_height_nm: i64,
        back: bool,
    },
}
impl From<PointNm> for PadCenter {
    fn from(p: PointNm) -> Self {
        Self::Integer(p)
    }
}
impl PadCenter {
    pub fn integer_point(self) -> Option<PointNm> {
        match self {
            Self::Integer(p) => Some(p),
            Self::Exact { exact, .. } => Some(PointNm {
                x: exact[0].integer_nm().ok()?,
                y: exact[1].integer_nm().ok()?,
            }),
        }
    }
    pub fn exact_point(self) -> ExactPointNm {
        match self {
            Self::Exact { exact, .. } => exact,
            Self::Integer(p) => {
                use eda_engine::board::nominal_geometry::Rational as R;
                [
                    Quadratic3::rational(R::integer(p.x.into())),
                    Quadratic3::rational(R::integer(p.y.into())),
                ]
            }
        }
    }
    pub fn bounds(
        self,
    ) -> std::result::Result<RectNm, eda_engine::board::nominal_geometry::GeometryError> {
        let p = self.exact_point();
        let x = p[0].floor_ceil_nm()?;
        let y = p[1].floor_ceil_nm()?;
        Ok(RectNm {
            min_x: x.0,
            max_x: x.1,
            min_y: y.0,
            max_y: y.1,
        })
    }
}
