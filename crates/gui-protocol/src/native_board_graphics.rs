//! PM057 projection and qualification consume recorded sources, never scene IDs.
use crate::selection_geometry::Rational as R;
use crate::selection_geometry_catalog::{LayoutPoint, QualificationGeometry as Q};
use crate::{BoardGraphicPrimitive, PointNm, layer_id};
use eda_engine::board::graphics::{BoardGraphic, GraphicGeometry as G};
use eda_engine::ir::geometry::Point;
use serde_json::Value;

fn point(p: Point) -> PointNm {
    PointNm { x: p.x, y: p.y }
}
fn ring(points: &[Point]) -> Vec<PointNm> {
    points.iter().copied().map(point).collect()
}
fn corners(a: Point, b: Point) -> Vec<Point> {
    vec![a, Point::new(b.x, a.y), b, Point::new(a.x, b.y)]
}
fn polygon(points: &[Point]) -> Vec<LayoutPoint> {
    points
        .iter()
        .map(|p| [R::integer(p.x.into()), R::integer(p.y.into())])
        .collect()
}

pub(super) fn project(board: &Value) -> anyhow::Result<Vec<BoardGraphicPrimitive>> {
    let Some(map) = board.get("graphics") else {
        return Ok(vec![]);
    };
    let mut result = Vec::new();
    for (key, value) in map
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid graphic source map"))?
    {
        let graphic: BoardGraphic = serde_json::from_value(value.clone())?;
        graphic
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid graphic source: {e:?}"))?;
        anyhow::ensure!(
            key == &graphic.uuid.to_string(),
            "Graphic key/source mismatch"
        );
        let filled = graphic.geometry.filled();
        let (kind, path, holes) = match &graphic.geometry {
            G::Line { from, to } => ("line", ring(&[*from, *to]), vec![]),
            G::Polyline { points } => ("polyline", ring(points), vec![]),
            G::Arc { from, midpoint, to } => (
                "arc",
                crate::kicad_scene_import::kicad_interpolate_arc_world(
                    point(*from),
                    point(*midpoint),
                    point(*to),
                ),
                vec![],
            ),
            G::Rectangle { from, to, .. } => (
                if filled { "polygon" } else { "polyline" },
                closed(ring(&corners(*from, *to))),
                vec![],
            ),
            G::Circle { center, radius, .. } => {
                // Display sampling is deliberately excluded from source qualification.
                let mut path = Vec::new();
                for i in 0..=64 {
                    let angle = std::f64::consts::TAU * f64::from(i) / 64.0;
                    path.push(PointNm {
                        x: center
                            .x
                            .checked_add((*radius as f64 * angle.cos()).round() as i64)
                            .ok_or_else(|| anyhow::anyhow!("Circle display range"))?,
                        y: center
                            .y
                            .checked_add((*radius as f64 * angle.sin()).round() as i64)
                            .ok_or_else(|| anyhow::anyhow!("Circle display range"))?,
                    });
                }
                (if filled { "polygon" } else { "polyline" }, path, vec![])
            }
            G::Polygon { outer, holes, .. } => (
                if filled { "polygon" } else { "polyline" },
                closed(ring(outer)),
                holes.iter().map(|h| closed(ring(h))).collect(),
            ),
        };
        result.push(BoardGraphicPrimitive {
            object_id: format!("board-graphic:{}", graphic.uuid),
            object_kind: "board_graphic".into(),
            primitive_kind: kind.into(),
            source_object_uuid: graphic.uuid.to_string(),
            layer_id: layer_id(graphic.layer),
            path,
            holes,
            width_nm: (graphic.width > 0).then_some(graphic.width),
        });
    }
    Ok(result)
}
fn closed(mut path: Vec<PointNm>) -> Vec<PointNm> {
    if let Some(&first) = path.first() {
        path.push(first);
    }
    path
}

pub(super) fn qualification(value: &Value) -> Option<Q> {
    let graphic: BoardGraphic = serde_json::from_value(value.clone()).ok()?;
    graphic.validate().ok()?;
    Some(match graphic.geometry {
        G::Line { from, to } => Q::Anchors(ring(&[from, to])),
        G::Polyline { points } => Q::Anchors(ring(&points)),
        G::Arc { from, midpoint, to } => Q::Anchors(ring(&[from, midpoint, to])),
        G::Rectangle { from, to, filled } => {
            let c = corners(from, to);
            if filled {
                Q::Filled(vec![polygon(&c)])
            } else {
                Q::Anchors(ring(&c))
            }
        }
        G::Circle { center, radius, .. } => Q::Disk {
            center: point(center),
            radius,
        },
        G::Polygon {
            outer,
            holes,
            filled,
        } => {
            if filled {
                Q::Filled(vec![polygon(&outer)])
            } else {
                Q::Anchors(ring(
                    &outer
                        .into_iter()
                        .chain(holes.into_iter().flatten())
                        .collect::<Vec<_>>(),
                ))
            }
        }
    })
}

#[cfg(test)]
mod tests;
