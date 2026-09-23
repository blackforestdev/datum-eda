//! Shared component body inference from pad bounds, without scratch allocation.
pub(super) fn compact_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    inferred_component_body_bounds(pads).filter(|body| {
        let width = body.max_x - body.min_x;
        let height = body.max_y - body.min_y;
        width > 0 && height > 0 && width <= 4_500_000 && height <= 4_500_000
    })
}

pub(super) fn inferred_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    inferred_component_body_bounds_iter(pads.iter().copied())
}

pub(super) fn inferred_component_body_bounds_iter<'a>(
    pads: impl IntoIterator<Item = &'a datum_gui_protocol::PadPrimitive>,
) -> Option<datum_gui_protocol::RectNm> {
    let mut pads = pads.into_iter();
    let first = pads.next()?;
    let pad_union = pads.fold(first.bounds, |mut acc, pad| {
        acc.min_x = acc.min_x.min(pad.bounds.min_x);
        acc.min_y = acc.min_y.min(pad.bounds.min_y);
        acc.max_x = acc.max_x.max(pad.bounds.max_x);
        acc.max_y = acc.max_y.max(pad.bounds.max_y);
        acc
    });
    let spread_x = (pad_union.max_x - pad_union.min_x) as f32;
    let spread_y = (pad_union.max_y - pad_union.min_y) as f32;
    let body = if spread_x >= spread_y {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.28).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.06).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.28).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.06).round() as i64,
        }
    } else {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.08).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.28).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.08).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.28).round() as i64,
        }
    };
    (body.max_x > body.min_x && body.max_y > body.min_y).then_some(body)
}
