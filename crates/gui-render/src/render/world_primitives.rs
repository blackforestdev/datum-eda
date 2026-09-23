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
