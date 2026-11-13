use super::{Node, Result, length, point, refusal};
use crate::{
    ir::geometry::{Point, Polygon},
    pool::Primitive,
};

pub(super) struct Graphics {
    pub courtyard: Polygon,
    pub silkscreen: Vec<Primitive>,
    pub fab: Vec<Primitive>,
    pub mechanical: Vec<Primitive>,
}

pub(super) fn convert(root: &Node, warnings: &mut Vec<String>) -> Result<Graphics> {
    let mut out = Graphics {
        courtyard: Polygon::new(vec![]),
        silkscreen: vec![],
        fab: vec![],
        mechanical: vec![],
    };
    let mut edges = Vec::new();
    let mut text_notice = false;
    for node in root.items().iter().skip(2) {
        let Some(tag) = node.tag() else {
            return Err(refusal("unsupported root flag"));
        };
        if !matches!(
            tag,
            "fp_line" | "fp_rect" | "fp_circle" | "fp_text" | "property"
        ) {
            if !matches!(
                tag,
                "version"
                    | "generator"
                    | "generator_version"
                    | "layer"
                    | "descr"
                    | "tags"
                    | "attr"
                    | "pad"
                    | "model"
                    | "embedded_fonts"
                    | "solder_mask_margin"
                    | "solder_paste_margin"
                    | "solder_paste_ratio"
                    | "uuid"
                    | "tstamp"
            ) {
                return Err(refusal(format!("unsupported source feature {tag}")));
            }
            continue;
        }
        let layer = node.required_child("layer")?.required_atom(1)?;
        let target = match layer {
            "F.SilkS" => &mut out.silkscreen,
            "F.Fab" => &mut out.fab,
            "F.CrtYd" => &mut out.mechanical,
            _ => return Err(refusal(format!("unsupported drawing layer {layer}"))),
        };
        if matches!(tag, "fp_text" | "property") {
            if layer == "F.CrtYd" {
                return Err(refusal("text cannot define a courtyard"));
            }
            let at = node.required_child("at")?;
            let rotation = if let Some(s) = at.atom(3) {
                let angle = crate::ir::units::parse_decimal_degrees(
                    s,
                    crate::ir::units::CanonicalAngleScale::new(1),
                    true,
                )
                .map_err(|_| refusal("invalid text rotation"))?
                .canonical_value;
                i32::try_from(angle)
                    .map_err(|_| refusal("text rotation exceeds the native angle range"))?
            } else {
                0
            };
            let hidden = node.flag("hide");
            if !hidden {
                target.push(Primitive::Text {
                    text: node.required_atom(2)?.into(),
                    position: point(at)?,
                    rotation,
                });
            }
            text_notice = true;
            continue;
        }
        let width = if let Some(stroke) = node.child("stroke") {
            if stroke
                .child("type")
                .is_some_and(|n| !matches!(n.atom(1), Some("solid" | "default")))
            {
                return Err(refusal("non-solid graphics stroke"));
            }
            length(stroke.required_child("width")?, 1)?
        } else {
            length(node.required_child("width")?, 1)?
        };
        if width <= 0 {
            return Err(refusal("nonpositive graphics width"));
        }
        if node.child("fill").is_some_and(|n| n.atom(1) != Some("no")) {
            return Err(refusal("filled footprint graphics are unsupported"));
        }
        let a = point(node.required_child(if tag == "fp_circle" {
            "center"
        } else {
            "start"
        })?)?;
        let b = point(node.required_child("end")?)?;
        let primitive = match tag {
            "fp_line" => {
                if layer == "F.CrtYd" {
                    edges.push((a, b));
                }
                Primitive::Line {
                    from: a,
                    to: b,
                    width,
                }
            }
            "fp_rect" => {
                let min = Point::new(a.x.min(b.x), a.y.min(b.y));
                let max = Point::new(a.x.max(b.x), a.y.max(b.y));
                if layer == "F.CrtYd" {
                    let points = [
                        min,
                        Point::new(max.x, min.y),
                        max,
                        Point::new(min.x, max.y),
                        min,
                    ];
                    edges.extend(points.windows(2).map(|p| (p[0], p[1])));
                }
                Primitive::Rect { min, max, width }
            }
            "fp_circle" => {
                if layer == "F.CrtYd" {
                    return Err(refusal(
                        "curved courtyard is not represented by native Polygon",
                    ));
                }
                let dx = i128::from(b.x) - i128::from(a.x);
                let dy = i128::from(b.y) - i128::from(a.y);
                let square = dx
                    .checked_mul(dx)
                    .and_then(|v| dy.checked_mul(dy).and_then(|w| v.checked_add(w)))
                    .ok_or_else(|| refusal("circle extent overflow"))?;
                let radius = (square as u128).isqrt();
                if radius == 0 || radius * radius != square as u128 || radius > i64::MAX as u128 {
                    return Err(refusal("circle radius is not exact integer nanometres"));
                }
                Primitive::Circle {
                    center: a,
                    radius: radius as i64,
                    width,
                }
            }
            _ => unreachable!(),
        };
        target.push(primitive);
    }
    if text_notice {
        warnings.push("Text contents/position/rotation retained; font, size and effects use native display defaults. Original source remains in provenance.".into());
        warnings.push("Footprint preview shows copper and supported outlines; retained reference/value text is not shown in this preview.".into());
    }
    if edges.is_empty() {
        warnings.push("No source courtyard; none has been synthesized.".into());
    } else {
        out.courtyard = stitch(edges)?;
    }
    Ok(out)
}

fn stitch(mut edges: Vec<(Point, Point)>) -> Result<Polygon> {
    let (start, mut end) = edges.remove(0);
    let mut points = vec![start];
    while end != start {
        points.push(end);
        let candidates: Vec<_> = edges
            .iter()
            .enumerate()
            .filter(|(_, e)| e.0 == end || e.1 == end)
            .map(|(i, _)| i)
            .collect();
        if candidates.len() != 1 {
            return Err(refusal("courtyard is open, branching or ambiguous"));
        }
        let edge = edges.remove(candidates[0]);
        end = if edge.0 == end { edge.1 } else { edge.0 };
    }
    if !edges.is_empty() || points.len() < 3 {
        return Err(refusal("multiple or degenerate courtyard contours"));
    }
    Ok(Polygon::new(points))
}
