//! Exact native aperture decoding and legacy-adoption correspondence.
use super::*;
use crate::{board::PadShape, pool::PadstackAperture};

pub(super) fn dimensions(
    aperture: &PadstackAperture,
) -> Result<(PadShape, i64, i64, i64, Option<u32>), EngineError> {
    Ok(match *aperture {
        PadstackAperture::Circle { diameter_nm } if diameter_nm > 0 => {
            (PadShape::Circle, diameter_nm, 0, 0, None)
        }
        PadstackAperture::Rect {
            width_nm,
            height_nm,
        } if width_nm > 0 && height_nm > 0 => (PadShape::Rect, 0, width_nm, height_nm, None),
        PadstackAperture::RoundRect {
            width_nm,
            height_nm,
            rratio_ppm,
        } if width_nm > 0 && height_nm > 0 && rratio_ppm <= 500_000 => (
            PadShape::RoundRect,
            0,
            width_nm,
            height_nm,
            Some(rratio_ppm),
        ),
        _ => return Err(invalid("invalid exact Padstack aperture")),
    })
}

pub(super) fn validate_legacy(
    previous: &[ComponentPadSource],
    pad: &Pad,
    stack: &Padstack,
    aperture: &crate::pool::PadstackAperture,
) -> Result<(), EngineError> {
    let (shape, diameter, width, height, ratio) = dimensions(aperture)?;
    let count = previous
        .iter()
        .filter(|old| {
            old.uuid == pad.uuid
                && old.name == pad.name
                && old.position == pad.position
                && old.padstack == pad.padstack
                && old.layer == pad.layer
                && old.drill_nm == stack.drill_nm
                && old.shape == Some(shape)
                && old.diameter_nm == diameter
                && old.width_nm == width
                && old.height_nm == height
                && old.roundrect_rratio_ppm == ratio
        })
        .count();
    if count != 1 {
        return Err(invalid(
            "legacy frozen geometry has no exact pinned source correspondence",
        ));
    }
    Ok(())
}
