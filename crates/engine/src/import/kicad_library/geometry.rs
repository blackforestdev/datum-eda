//! Exact scalar/point conversion and supported library drawing primitives.
use super::{Node, Result, refusal};
use crate::ir::geometry::{Arc, Point};
use crate::ir::units::{
    CanonicalAngleScale, LengthUnit, parse_decimal_degrees, parse_fixed_length,
};
use crate::pool::Primitive;

pub(super) fn length(token: &str) -> Result<i64> {
    parse_fixed_length(token, LengthUnit::Millimeter)
        .map(|n| n.get())
        .map_err(|e| refusal(format!("invalid millimeter coordinate {token}: {e:?}")))
}

pub(super) fn point(node: &Node) -> Result<Point> {
    let x = length(node.required_atom(1)?)?;
    let y = length(node.required_atom(2)?)?
        .checked_neg()
        .ok_or_else(|| refusal("Y coordinate overflow"))?;
    Ok(Point::new(x, y))
}

pub(super) fn angle(token: &str) -> Result<i32> {
    let parsed = parse_decimal_degrees(token, CanonicalAngleScale::new(1), true)
        .map_err(|e| refusal(format!("invalid angle {token}: {e:?}")))?;
    i32::try_from(parsed.canonical_value).map_err(|_| refusal("angle overflow"))
}

pub(super) fn drawing(node: &Node, warnings: &mut Vec<String>) -> Result<Vec<Primitive>> {
    let tag = node.tag().unwrap_or("");
    if tag == "text" {
        warnings.push(
            "Source text font, justification and effects use native display defaults.".into(),
        );
        let at = node.required_child("at")?;
        return Ok(vec![Primitive::Text {
            text: node.required_atom(1)?.into(),
            position: point(at)?,
            rotation: angle(at.required_atom(3)?)?
                .checked_neg()
                .ok_or_else(|| refusal("text angle overflow"))?,
        }]);
    }
    if let Some(fill) = node.child("fill") {
        match fill.required_child("type")?.required_atom(1)? {
            "none" => {},
            "background" | "outline" => warnings.push("Source filled shapes are rendered as outlines; native symbol fill is not represented.".into()),
            other => return Err(refusal(format!("unsupported fill {other}"))),
        }
    }
    let stroke = node.required_child("stroke")?;
    if let Some(kind) = stroke.child("type") {
        if !matches!(kind.atom(1), Some("default" | "solid")) {
            return Err(refusal("non-solid stroke is unsupported"));
        }
    }
    let mut width = length(stroke.required_child("width")?.required_atom(1)?)?;
    if width < 0 {
        return Err(refusal("negative stroke width"));
    }
    if width == 0 {
        width = 254_000;
        warnings.push("Source default stroke width is displayed as 0.254 mm.".into());
    }
    Ok(match tag {
        "rectangle" => {
            let a = point(node.required_child("start")?)?;
            let b = point(node.required_child("end")?)?;
            vec![Primitive::Rect {
                min: Point::new(a.x.min(b.x), a.y.min(b.y)),
                max: Point::new(a.x.max(b.x), a.y.max(b.y)),
                width,
            }]
        }
        "polyline" => {
            let points = node
                .required_child("pts")?
                .children("xy")
                .map(point)
                .collect::<Result<Vec<_>>>()?;
            if points.len() < 2 {
                return Err(refusal("polyline requires at least two points"));
            }
            points
                .windows(2)
                .map(|p| Primitive::Line {
                    from: p[0],
                    to: p[1],
                    width,
                })
                .collect()
        }
        "line" => vec![Primitive::Line {
            from: point(node.required_child("start")?)?,
            to: point(node.required_child("end")?)?,
            width,
        }],
        "circle" => {
            let radius = length(node.required_child("radius")?.required_atom(1)?)?;
            if radius <= 0 {
                return Err(refusal("circle requires positive radius"));
            }
            let center = point(node.required_child("center")?)?;
            checked_radial_extents(center, radius)?;
            vec![Primitive::Circle {
                center,
                radius,
                width,
            }]
        }
        "arc" => {
            let arc = arc(
                point(node.required_child("start")?)?,
                point(node.required_child("mid")?)?,
                point(node.required_child("end")?)?,
            )?;
            warnings.push(
                "Three-point arcs use native integer-nanometer centers and 0.1-degree angles."
                    .into(),
            );
            vec![Primitive::Arc { arc, width }]
        }
        _ => return Err(refusal(format!("unsupported drawing {tag}"))),
    })
}

fn checked_radial_extents(center: Point, radius: i64) -> Result<()> {
    for coordinate in [center.x, center.y] {
        if coordinate.checked_sub(radius).is_none() || coordinate.checked_add(radius).is_none() {
            return Err(refusal(
                "circle or arc extents exceed representable coordinates",
            ));
        }
    }
    Ok(())
}

fn arc(start: Point, mid: Point, end: Point) -> Result<Arc> {
    // Local differences reduce cancellation when the source has a large origin.
    let (bx, by) = (mid.x as f64 - start.x as f64, mid.y as f64 - start.y as f64);
    let (cx, cy) = (end.x as f64 - start.x as f64, end.y as f64 - start.y as f64);
    let d = 2.0 * (bx * cy - by * cx);
    if d == 0.0 {
        return Err(refusal("collinear or degenerate arc"));
    }
    let b2 = bx * bx + by * by;
    let c2 = cx * cx + cy * cy;
    let ux = (cy * b2 - by * c2) / d;
    let uy = (bx * c2 - cx * b2) / d;
    let integer = |value: f64| -> Result<i64> {
        if !value.is_finite() || value.abs() >= i64::MAX as f64 {
            return Err(refusal("arc geometry overflow"));
        }
        Ok(value.round() as i64)
    };
    let center = Point::new(integer(start.x as f64 + ux)?, integer(start.y as f64 + uy)?);
    let radius = integer(ux.hypot(uy))?;
    if radius <= 0 {
        return Err(refusal("arc radius is not representable"));
    }
    checked_radial_extents(center, radius)?;
    let theta = |x: f64, y: f64| ((y.atan2(x).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    let a = theta(-ux, -uy);
    let b = theta(bx - ux, by - uy);
    let c = theta(cx - ux, cy - uy);
    // Native arcs travel in increasing angles. Reverse endpoints if necessary
    // so the retained arc passes through the source middle point.
    let (start_angle, end_angle) = if (b - a).rem_euclid(3600) <= (c - a).rem_euclid(3600) {
        (a, c)
    } else {
        (c, a)
    };
    Ok(Arc {
        center,
        radius,
        start_angle,
        end_angle,
    })
}
