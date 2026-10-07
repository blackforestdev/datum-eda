//! PM057 nonconductive authored graphic source. Render paths are projections.
use super::nominal_geometry::{CertifiedArc, DistanceBoundary, GeometryError, Rational as R};
use super::occupied_copper::{polygon_contains_point, polygons_within, validate_polygon};
use crate::ir::geometry::{Point, Polygon};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardGraphic {
    pub uuid: Uuid,
    pub layer: i32,
    pub width: i64,
    pub geometry: GraphicGeometry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GraphicGeometry {
    Line {
        from: Point,
        to: Point,
    },
    Polyline {
        points: Vec<Point>,
    },
    Arc {
        from: Point,
        midpoint: Point,
        to: Point,
    },
    Rectangle {
        from: Point,
        to: Point,
        filled: bool,
    },
    Circle {
        center: Point,
        radius: i64,
        filled: bool,
    },
    Polygon {
        outer: Vec<Point>,
        holes: Vec<Vec<Point>>,
        filled: bool,
    },
}

impl GraphicGeometry {
    pub fn filled(&self) -> bool {
        matches!(
            self,
            Self::Rectangle { filled: true, .. }
                | Self::Circle { filled: true, .. }
                | Self::Polygon { filled: true, .. }
        )
    }
    pub fn validate(&self) -> Result<(), GeometryError> {
        let invalid = || GeometryError::InvalidPolygon;
        match self {
            Self::Line { from, to } if from == to => return Err(invalid()),
            Self::Polyline { points }
                if points.len() < 2
                    || points.first() == points.last()
                    || points.windows(2).any(|p| p[0] == p[1]) =>
            {
                return Err(invalid());
            }
            Self::Arc { from, midpoint, to } => {
                CertifiedArc::new(*from, *midpoint, *to)?;
            }
            Self::Rectangle { from, to, .. } if from.x == to.x || from.y == to.y => {
                return Err(invalid());
            }
            Self::Circle { center, radius, .. } => {
                if *radius <= 0 {
                    return Err(invalid());
                }
                for v in [center.x, center.y] {
                    v.checked_add(*radius)
                        .ok_or(GeometryError::ArithmeticRange)?;
                    v.checked_sub(*radius)
                        .ok_or(GeometryError::ArithmeticRange)?;
                }
            }
            Self::Polygon { outer, holes, .. } => {
                let outer = Polygon::new(outer.clone());
                validate_polygon(&outer)?;
                let mut previous: Vec<Polygon> = vec![];
                for ring in holes {
                    let hole = Polygon::new(ring.clone());
                    validate_polygon(&hole)?;
                    // No touching/crossing the outer boundary or another hole.
                    for p in &hole.vertices {
                        if !polygon_contains_point(&outer, *p)? {
                            return Err(invalid());
                        }
                    }
                    for i in 0..outer.vertices.len() {
                        let a = outer.vertices[i];
                        let b = outer.vertices[(i + 1) % outer.vertices.len()];
                        for j in 0..hole.vertices.len() {
                            let c = hole.vertices[j];
                            let d = hole.vertices[(j + 1) % hole.vertices.len()];
                            let p = |v: Point| [R::integer(v.x.into()), R::integer(v.y.into())];
                            if super::occupied_copper::segment_pair(
                                p(a),
                                p(b),
                                p(c),
                                p(d),
                                R::integer(0),
                                DistanceBoundary::Inclusive,
                            )? {
                                return Err(invalid());
                            }
                        }
                    }
                    for peer in &previous {
                        if polygons_within(peer, &hole, 0, DistanceBoundary::Inclusive)? {
                            return Err(invalid());
                        }
                    }
                    previous.push(hole);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl BoardGraphic {
    pub fn validate(&self) -> Result<(), GeometryError> {
        if self.uuid.is_nil() || self.width < 0 || (!self.geometry.filled() && self.width == 0) {
            return Err(GeometryError::InvalidWidth);
        }
        self.geometry.validate()
    }
}
