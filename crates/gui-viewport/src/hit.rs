//! Shared retained hit regions and bounded spatial lookup for editor surfaces.

use datum_gui_protocol::{PointNm, RectNm};
use std::collections::BinaryHeap;

/// Maximum exact-shape candidates examined by one pointer query.
pub const DEFAULT_HIT_QUERY_BUDGET: usize = 4_096;

/// Surface-authored hit geometry. Identity and visibility policy stay generic.
#[derive(Debug, Clone, PartialEq)]
pub enum HitShape {
    Rect(RectNm),
    Polyline {
        path: Vec<PointNm>,
        half_width_nm: f32,
    },
    Polygon(Vec<PointNm>),
    Circle {
        center: PointNm,
        radius_nm: f32,
    },
}

impl HitShape {
    pub fn bounds(&self) -> Option<RectNm> {
        match self {
            Self::Rect(rect) => Some(*rect),
            Self::Polyline {
                path,
                half_width_nm,
            } => expanded_bounds(path, (*half_width_nm).ceil() as i64),
            Self::Polygon(path) => expanded_bounds(path, 0),
            Self::Circle { center, radius_nm } => {
                let radius = radius_nm.ceil() as i64;
                Some(RectNm {
                    min_x: center.x.saturating_sub(radius),
                    min_y: center.y.saturating_sub(radius),
                    max_x: center.x.saturating_add(radius),
                    max_y: center.y.saturating_add(radius),
                })
            }
        }
    }

    pub fn contains(&self, point: PointNm) -> bool {
        match self {
            Self::Rect(rect) => point_in_rect(point, *rect),
            Self::Polyline {
                path,
                half_width_nm,
            } => polyline_contains_point(path, point, *half_width_nm),
            Self::Polygon(path) => point_in_polygon(path, point),
            Self::Circle { center, radius_nm } => {
                let dx = point.x as f64 - center.x as f64;
                let dy = point.y as f64 - center.y as f64;
                dx * dx + dy * dy <= f64::from(*radius_nm).powi(2)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitRegion<T> {
    pub target: T,
    pub layer_id: Option<String>,
    pub shape: HitShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    bounds: RectNm,
    start: usize,
    end: usize,
    left: Option<usize>,
    right: Option<usize>,
    max_order: usize,
}

/// Static AABB hierarchy over retained world hit regions.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialHitIndex<T> {
    regions: Vec<HitRegion<T>>,
    order: Vec<usize>,
    nodes: Vec<Node>,
    query_budget: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitQuery<'a, T> {
    pub target: Option<&'a T>,
    pub examined: usize,
    pub budget_exhausted: bool,
}

impl<T> SpatialHitIndex<T> {
    pub fn new(regions: Vec<HitRegion<T>>) -> Self {
        Self::with_budget(regions, DEFAULT_HIT_QUERY_BUDGET)
    }

    pub fn with_budget(regions: Vec<HitRegion<T>>, query_budget: usize) -> Self {
        let mut index = Self {
            order: (0..regions.len()).collect(),
            regions,
            nodes: Vec::new(),
            query_budget: query_budget.max(1),
        };
        if !index.order.is_empty() {
            index.build_node(0, index.order.len());
        }
        index
    }

    pub fn regions(&self) -> &[HitRegion<T>] {
        &self.regions
    }

    /// Return the topmost visible exact hit. Later retained regions are topmost.
    pub fn hit_test(
        &self,
        point: PointNm,
        visible: impl Fn(&HitRegion<T>) -> bool,
    ) -> HitQuery<'_, T> {
        let mut examined = 0;
        // Max-order priority guarantees the first exact hit is the topmost one
        // without scanning unrelated lower-order regions.
        let mut queue = BinaryHeap::new();
        if !self.nodes.is_empty() {
            queue.push((self.nodes[0].max_order, false, 0));
        }
        while let Some((_, is_region, item_index)) = queue.pop() {
            if is_region {
                if examined >= self.query_budget {
                    return HitQuery {
                        target: None,
                        examined,
                        budget_exhausted: true,
                    };
                }
                examined += 1;
                let region = &self.regions[item_index];
                if visible(region) && region.shape.contains(point) {
                    return HitQuery {
                        target: Some(&region.target),
                        examined,
                        budget_exhausted: false,
                    };
                }
                continue;
            }
            let node_index = item_index;
            let node = self.nodes[node_index];
            if !point_in_rect(point, node.bounds) {
                continue;
            }
            match (node.left, node.right) {
                (Some(left), Some(right)) => {
                    queue.push((self.nodes[left].max_order, false, left));
                    queue.push((self.nodes[right].max_order, false, right));
                }
                _ => {
                    for &region_index in &self.order[node.start..node.end] {
                        if self.regions[region_index]
                            .shape
                            .bounds()
                            .is_some_and(|bounds| point_in_rect(point, bounds))
                        {
                            queue.push((region_index, true, region_index));
                        }
                    }
                }
            }
        }
        HitQuery {
            target: None,
            examined,
            budget_exhausted: false,
        }
    }

    fn build_node(&mut self, start: usize, end: usize) -> usize {
        let bounds = union_bounds(
            self.order[start..end]
                .iter()
                .filter_map(|&index| self.regions[index].shape.bounds()),
        )
        .unwrap_or(RectNm {
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
        });
        let max_order = self.order[start..end].iter().copied().max().unwrap_or(0);
        let node_index = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            start,
            end,
            left: None,
            right: None,
            max_order,
        });
        if end - start > 8 {
            let split_x = bounds.max_x.saturating_sub(bounds.min_x)
                >= bounds.max_y.saturating_sub(bounds.min_y);
            self.order[start..end].sort_unstable_by_key(|&index| {
                let b = self.regions[index].shape.bounds().unwrap_or(bounds);
                if split_x {
                    ((b.min_x as i128 + b.max_x as i128) / 2) as i64
                } else {
                    ((b.min_y as i128 + b.max_y as i128) / 2) as i64
                }
            });
            let mid = start + (end - start) / 2;
            let left = self.build_node(start, mid);
            let right = self.build_node(mid, end);
            self.nodes[node_index].left = Some(left);
            self.nodes[node_index].right = Some(right);
        }
        node_index
    }
}

fn expanded_bounds(points: &[PointNm], padding: i64) -> Option<RectNm> {
    let first = points.first()?;
    let mut bounds = RectNm {
        min_x: first.x,
        min_y: first.y,
        max_x: first.x,
        max_y: first.y,
    };
    for point in &points[1..] {
        bounds.min_x = bounds.min_x.min(point.x);
        bounds.min_y = bounds.min_y.min(point.y);
        bounds.max_x = bounds.max_x.max(point.x);
        bounds.max_y = bounds.max_y.max(point.y);
    }
    bounds.min_x = bounds.min_x.saturating_sub(padding);
    bounds.min_y = bounds.min_y.saturating_sub(padding);
    bounds.max_x = bounds.max_x.saturating_add(padding);
    bounds.max_y = bounds.max_y.saturating_add(padding);
    Some(bounds)
}

fn union_bounds(bounds: impl Iterator<Item = RectNm>) -> Option<RectNm> {
    bounds.reduce(|a, b| RectNm {
        min_x: a.min_x.min(b.min_x),
        min_y: a.min_y.min(b.min_y),
        max_x: a.max_x.max(b.max_x),
        max_y: a.max_y.max(b.max_y),
    })
}

fn point_in_rect(point: PointNm, rect: RectNm) -> bool {
    point.x >= rect.min_x && point.x <= rect.max_x && point.y >= rect.min_y && point.y <= rect.max_y
}

fn polyline_contains_point(path: &[PointNm], point: PointNm, half_width_nm: f32) -> bool {
    path.windows(2).any(|segment| {
        let (ax, ay) = (segment[0].x as f64, segment[0].y as f64);
        let (bx, by) = (segment[1].x as f64, segment[1].y as f64);
        let (px, py) = (point.x as f64, point.y as f64);
        let (dx, dy) = (bx - ax, by - ay);
        let length_sq = dx * dx + dy * dy;
        let t = if length_sq <= f64::EPSILON {
            0.0
        } else {
            (((px - ax) * dx + (py - ay) * dy) / length_sq).clamp(0.0, 1.0)
        };
        let (cx, cy) = (ax + t * dx, ay + t * dy);
        let (ex, ey) = (px - cx, py - cy);
        ex * ex + ey * ey <= f64::from(half_width_nm).powi(2)
    })
}

fn point_in_polygon(path: &[PointNm], point: PointNm) -> bool {
    if path.len() < 3 {
        return false;
    }
    let (px, py) = (point.x as f64, point.y as f64);
    let mut inside = false;
    let mut previous = path[path.len() - 1];
    for current in path {
        let (x1, y1) = (previous.x as f64, previous.y as f64);
        let (x2, y2) = (current.x as f64, current.y as f64);
        if (y1 > py) != (y2 > py) && px < (x2 - x1) * (py - y1) / (y2 - y1) + x1 {
            inside = !inside;
        }
        previous = *current;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> HitShape {
        HitShape::Rect(RectNm {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    #[test]
    fn topmost_visible_region_wins() {
        let index = SpatialHitIndex::new(vec![
            HitRegion {
                target: "bottom",
                layer_id: None,
                shape: rect(0, 0, 10, 10),
            },
            HitRegion {
                target: "top",
                layer_id: None,
                shape: rect(0, 0, 10, 10),
            },
        ]);
        assert_eq!(
            index.hit_test(PointNm { x: 5, y: 5 }, |_| true).target,
            Some(&"top")
        );
    }

    #[test]
    fn query_work_is_hard_bounded() {
        let regions = (0..100)
            .map(|i| HitRegion {
                target: i,
                layer_id: None,
                shape: rect(0, 0, 100, 100),
            })
            .collect();
        let index = SpatialHitIndex::with_budget(regions, 7);
        let query = index.hit_test(PointNm { x: 50, y: 50 }, |_| false);
        assert_eq!(query.examined, 7);
        assert!(query.budget_exhausted);
    }

    #[test]
    fn large_origins_keep_circle_precision() {
        let center = PointNm {
            x: 9_000_000_000_000,
            y: -9_000_000_000_000,
        };
        let shape = HitShape::Circle {
            center,
            radius_nm: 10.0,
        };
        assert!(shape.contains(PointNm {
            x: center.x + 9,
            y: center.y
        }));
        assert!(!shape.contains(PointNm {
            x: center.x + 11,
            y: center.y
        }));
    }
}
