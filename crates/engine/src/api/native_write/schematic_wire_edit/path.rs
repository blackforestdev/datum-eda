//! Checked shared path geometry: caller owns electrical snapping and intent.
use super::types::{WireCornerOrder, WireRouteConfig, WireRouting};
use crate::{error::EngineError, ir::geometry::Point};

/// Route between exact already-snapped endpoints. No grid, source mutation,
/// object picking, semantic identity or input ownership is implied.
/// Start means horizontal-first for 90 degrees, diagonal-first for 45 degrees;
/// End selects the complementary leg order. Coincident endpoints form one point,
/// so callers can preview the initial position without introducing a zero edge.
pub fn routed_wire_points(
    from: Point,
    to: Point,
    config: WireRouteConfig,
) -> Result<Vec<Point>, EngineError> {
    let mut points = vec![from];
    let corner = match config.routing {
        WireRouting::Free => None,
        WireRouting::Orthogonal => Some(match config.corner {
            WireCornerOrder::Start => Point::new(to.x, from.y),
            WireCornerOrder::End => Point::new(from.x, to.y),
        }),
        WireRouting::Diagonal45 => {
            let dx = i128::from(to.x) - i128::from(from.x);
            let dy = i128::from(to.y) - i128::from(from.y);
            let diagonal = dx.abs().min(dy.abs());
            let sx = dx.signum() * diagonal;
            let sy = dy.signum() * diagonal;
            let (x, y) = match config.corner {
                WireCornerOrder::Start => (
                    i128::from(from.x).checked_add(sx),
                    i128::from(from.y).checked_add(sy),
                ),
                WireCornerOrder::End => (
                    i128::from(to.x).checked_sub(sx),
                    i128::from(to.y).checked_sub(sy),
                ),
            };
            let coordinate = |value: Option<i128>| {
                value.and_then(|v| i64::try_from(v).ok()).ok_or_else(|| {
                    EngineError::Validation("Wire route coordinate exceeds integer range".into())
                })
            };
            Some(Point::new(coordinate(x)?, coordinate(y)?))
        }
    };
    if let Some(corner) = corner {
        if corner != from && corner != to {
            points.push(corner);
        }
    }
    if to != from {
        points.push(to);
    }
    Ok(points)
}
