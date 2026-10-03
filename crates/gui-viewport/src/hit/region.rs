//! Resumable exact region broad phase over the same retained AABB owner.
use super::*;
/// Node/leaf work permitted per region evaluation tick; never a result cap.
pub const DEFAULT_REGION_QUERY_BUDGET: usize = 4_096;
#[derive(Debug)]
pub struct RegionQueryCursor {
    bounds: RectNm,
    stack: Vec<Visit>,
    pub examined: usize,
}
#[derive(Debug)]
enum Visit {
    Node(usize),
    Leaf { next: usize, end: usize },
}
#[derive(Debug, PartialEq, Eq)]
pub struct RegionQueryStep {
    pub work: usize,
    pub complete: bool,
}
fn intersects(a: RectNm, b: RectNm) -> bool {
    a.min_x <= b.max_x && b.min_x <= a.max_x && a.min_y <= b.max_y && b.min_y <= a.max_y
}
impl<T> SpatialHitIndex<T> {
    /// Bind a cursor to this immutable index. Complete before replacing its owner.
    pub fn begin_region(&self, bounds: RectNm) -> RegionQueryCursor {
        RegionQueryCursor {
            bounds,
            stack: if self.nodes.is_empty() {
                Vec::new()
            } else {
                vec![Visit::Node(0)]
            },
            examined: 0,
        }
    }
    /// Visit every intersecting source bound exactly once. Caller owns exact
    /// qualification and stages results until `complete`; cancellation drops them.
    pub fn advance_region(
        &self,
        cursor: &mut RegionQueryCursor,
        budget: usize,
        mut candidate: impl FnMut(&T),
    ) -> RegionQueryStep {
        self.advance_region_records(cursor, budget, |_, region| candidate(&region.target))
    }
    /// Record-aware traversal for complete local overlap menus.
    pub fn advance_region_records(
        &self,
        cursor: &mut RegionQueryCursor,
        budget: usize,
        mut candidate: impl FnMut(usize, &HitRegion<T>),
    ) -> RegionQueryStep {
        let mut work = 0;
        while work < budget {
            let Some(visit) = cursor.stack.pop() else {
                break;
            };
            work += 1;
            match visit {
                Visit::Node(index) => {
                    let node = self.nodes[index];
                    if !intersects(node.bounds, cursor.bounds) {
                        continue;
                    }
                    match (node.left, node.right) {
                        (Some(left), Some(right)) => {
                            cursor.stack.push(Visit::Node(right));
                            cursor.stack.push(Visit::Node(left));
                        }
                        _ => cursor.stack.push(Visit::Leaf {
                            next: node.start,
                            end: node.end,
                        }),
                    }
                }
                Visit::Leaf { next, end } => {
                    if next + 1 < end {
                        cursor.stack.push(Visit::Leaf {
                            next: next + 1,
                            end,
                        });
                    }
                    let region = &self.regions[self.order[next]];
                    if region
                        .shape
                        .bounds()
                        .is_some_and(|bounds| intersects(bounds, cursor.bounds))
                    {
                        cursor.examined += 1;
                        candidate(self.order[next], region);
                    }
                }
            }
        }
        RegionQueryStep {
            work,
            complete: cursor.stack.is_empty(),
        }
    }
}
