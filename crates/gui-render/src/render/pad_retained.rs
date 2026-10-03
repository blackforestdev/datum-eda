//! Actual retained pad copper and drill paint roles.
use super::*;
pub(crate) fn push_pad_primitive_world(
    out: &mut impl Output<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    bounds: datum_gui_protocol::RectNm,
    outer_color: [f32; 3],
    drill_nm: Option<i64>,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let outer_color = dim_authored_color(outer_color, dimmed);
    // Ellipses use 64 points; rounded rectangles use at most 36. Include
    // floating-point input and nanometre output overlap before construction.
    if !world_primitives::admit_outline_points(out, 64) {
        return;
    }
    let copper_outline =
        world_primitives::world_pad_outline(pad, bounds, 0.0, reference_projection);
    push_world_polygon_fill(out, &copper_outline, outer_color);
    if let Some(drill_nm) = drill_nm.filter(|value| *value > 0) {
        let half = drill_nm as f32 * 0.5;
        let center_x = (bounds.min_x + bounds.max_x) as f32 * 0.5;
        let center_y = (bounds.min_y + bounds.max_y) as f32 * 0.5;
        let hole = datum_gui_protocol::RectNm {
            min_x: (center_x - half).round() as i64,
            min_y: (center_y - half).round() as i64,
            max_x: (center_x + half).round() as i64,
            max_y: (center_y + half).round() as i64,
        };
        out.role(crate::presentation_ranges::Role::Void);
        push_world_ellipse_nm(
            out,
            hole,
            dim_structural_color([0.10, 0.11, 0.12], dimmed),
            128,
        );
        out.role(crate::presentation_ranges::Role::Material);
    }
}
