//! Shared world-space quad and triangle emission.
use super::*;

pub(super) fn push_world_quad(
    out: &mut impl Output<Quad>,
    quad: &[(f32, f32); 4],
    color: [f32; 3],
) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

pub(super) fn push_world_triangle(
    out: &mut impl Output<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [f32; 3],
) {
    out.push(Quad {
        points: [a, b, c, c],
        color,
    });
}

pub(super) fn push_convex_polygon_fill(
    out: &mut impl Output<Quad>,
    polygon: &[(f32, f32)],
    color: [f32; 3],
) {
    if polygon.len() < 3 {
        return;
    }
    let origin = polygon[0];
    for edge in polygon[1..].windows(2) {
        push_world_triangle(out, origin, edge[0], edge[1], color);
    }
}

pub(super) fn push_world_rect_nm(
    out: &mut impl Output<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
) {
    out.push(Quad {
        points: [
            (rect.min_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.max_y as f32),
            (rect.min_x as f32, rect.max_y as f32),
        ],
        color,
    });
}

/// Close only when needed, with the retained output's construction admission.
pub(super) fn closed_world_path<'a>(
    out: &mut impl Output<Quad>,
    points: &'a [PointNm],
) -> Option<std::borrow::Cow<'a, [PointNm]>> {
    if let (Some(first), Some(last)) = (points.first(), points.last())
        && first != last
    {
        let mut closed = out.scratch(points.len().saturating_add(1))?;
        closed.extend_from_slice(points);
        closed.push(*first);
        Some(std::borrow::Cow::Owned(closed))
    } else {
        Some(std::borrow::Cow::Borrowed(points))
    }
}

pub(super) fn admit_outline_points(out: &mut impl Output<Quad>, count: usize) -> bool {
    use crate::cpu_alloc::heap::capacity_bytes;
    out.admit_temporary(
        capacity_bytes::<(f32, f32)>(count).saturating_add(capacity_bytes::<PointNm>(count)),
    )
}

pub(super) fn world_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    inset_nm: f32,
    reference_projection: &Projection,
) -> Vec<PointNm> {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let center = (pad.center.x as f32, pad.center.y as f32);
    let width_nm = (width_nm - inset_nm * 2.0).max(1.0);
    let height_nm = (height_nm - inset_nm * 2.0).max(1.0);
    let points = match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_nm, height_nm, pad.rotation_degrees, 64),
        _ => {
            let radius_nm =
                pad_corner_radius_nm(pad, width_nm, height_nm, reference_projection, inset_nm);
            rounded_rect_points(center, width_nm, height_nm, pad.rotation_degrees, radius_nm)
        }
    };
    points
        .into_iter()
        .map(|(x, y)| PointNm {
            x: x.round() as i64,
            y: y.round() as i64,
        })
        .collect()
}
