//! Final painting conversion of typed exact Pad pose.
use super::*;
/// Floating conversion belongs exclusively to painting, never qualification.
pub(super) fn paint_pad_center(center: datum_gui_protocol::PadCenter) -> (f32, f32) {
    match center {
        datum_gui_protocol::PadCenter::Integer(p) => (p.x as f32, p.y as f32),
        datum_gui_protocol::PadCenter::Exact { exact, .. } => {
            let paint = |q: datum_gui_protocol::Quadratic3| {
                (q.a.numerator as f64 / q.a.denominator as f64
                    + q.b.numerator as f64 / q.b.denominator as f64 * 3.0_f64.sqrt())
                    as f32
            };
            (paint(exact[0]), paint(exact[1]))
        }
    }
}
pub(super) fn project_pad_center(
    center: datum_gui_protocol::PadCenter,
    p: &Projection,
) -> (f32, f32) {
    let (x, y) = paint_pad_center(center);
    (
        p.offset_x + (x - p.bounds.min_x as f32) * p.scale,
        p.offset_y + (y - p.bounds.min_y as f32) * p.scale,
    )
}

pub(super) fn pad_dimensions_nm(pad: &datum_gui_protocol::PadPrimitive) -> (f32, f32) {
    if let datum_gui_protocol::PadCenter::Exact {
        local_width_nm,
        local_height_nm,
        ..
    } = pad.center
    {
        return (local_width_nm as f32, local_height_nm as f32);
    }
    (
        (pad.bounds.max_x - pad.bounds.min_x).max(1) as f32,
        (pad.bounds.max_y - pad.bounds.min_y).max(1) as f32,
    )
}
