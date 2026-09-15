use serde::{Deserialize, Serialize};

/// A point in nanometer coordinates.
/// All coordinates in the engine are i64 nanometers — no floating point.
/// See docs/CANONICAL_IR.md §2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Distance squared (avoids sqrt, sufficient for comparison).
    pub fn distance_sq(&self, other: &Point) -> i64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub fn width(&self) -> i64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> i64 {
        self.max.y - self.min.y
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

/// A polygon defined by vertices with explicit closure semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub closed: bool,
}

impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Self {
        Self {
            vertices,
            closed: true,
        }
    }

    pub fn bounding_box(&self) -> Option<Rect> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;
        for v in &self.vertices {
            min_x = min_x.min(v.x);
            min_y = min_y.min(v.y);
            max_x = max_x.max(v.x);
            max_y = max_y.max(v.y);
        }
        Some(Rect::new(
            Point::new(min_x, min_y),
            Point::new(max_x, max_y),
        ))
    }
}

/// Arc defined by center, radius, and angle range.
/// Angles in tenths of degree (i32). 0 = right, 900 = up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point,
    pub radius: i64,
    pub start_angle: i32,
    pub end_angle: i32,
}

/// Layer identifier. Well-known values: 1=Top, N=Bottom, 2..N-1=Inner.
pub type LayerId = i32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let a = Point::new(0, 0);
        let b = Point::new(3_000_000, 4_000_000); // 3mm, 4mm in nm
        assert_eq!(a.distance_sq(&b), 25_000_000_000_000); // 5mm squared in nm²
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(Point::new(0, 0), Point::new(1_000_000, 1_000_000));
        assert!(r.contains(&Point::new(500_000, 500_000)));
        assert!(!r.contains(&Point::new(2_000_000, 500_000)));
    }

    #[test]
    fn polygon_bbox() {
        let p = Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 5_000_000),
            Point::new(0, 5_000_000),
        ]);
        let bb = p.bounding_box().unwrap();
        assert!(p.closed);
        assert_eq!(bb.width(), 10_000_000);
        assert_eq!(bb.height(), 5_000_000);
    }
}
