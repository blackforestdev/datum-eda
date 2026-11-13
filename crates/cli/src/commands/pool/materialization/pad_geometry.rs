//! Native library pad shape decoding for legacy CLI materialization.
use super::*;
use eda_engine::{board::PadShape, pool::PadstackAperture};

pub(super) fn native_component_pad(
    pad: eda_engine::pool::Pad,
    padstack: Option<&Padstack>,
) -> NativeComponentPad {
    let (shape, diameter_nm, width_nm, height_nm) =
        match padstack.and_then(|padstack| padstack.aperture.as_ref()) {
            Some(PadstackAperture::Circle { diameter_nm }) => {
                (Some(PadShape::Circle), *diameter_nm, 0, 0)
            }
            Some(PadstackAperture::Rect {
                width_nm,
                height_nm,
            }) => (Some(PadShape::Rect), 0, *width_nm, *height_nm),
            Some(PadstackAperture::RoundRect {
                width_nm,
                height_nm,
                ..
            }) => (Some(PadShape::RoundRect), 0, *width_nm, *height_nm),
            None => (None, 0, 0, 0),
        };
    NativeComponentPad {
        source_binding: None,
        uuid: pad.uuid,
        name: pad.name,
        position: native_point(pad.position),
        padstack: pad.padstack,
        layer: pad.layer,
        drill_nm: padstack.and_then(|padstack| padstack.drill_nm),
        shape,
        diameter_nm,
        width_nm,
        height_nm,
        roundrect_rratio_ppm: padstack.and_then(|s| match s.aperture {
            Some(PadstackAperture::RoundRect { rratio_ppm, .. }) => Some(rratio_ppm),
            _ => None,
        }),
    }
}
