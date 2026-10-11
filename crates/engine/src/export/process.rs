//! Exact CAM admission over canonical frozen process geometry, before any output write.
use super::ExportError;
use crate::board::{PadShape, PlacedPad};
use crate::ir::geometry::{LayerId, Point};
use crate::pool::process::EffectiveProcessAperture;
use crate::substrate::component_pad_source::process::PlacedProcessGeometry;

pub fn process_pad_for_layer(
    process: &PlacedProcessGeometry,
    copper: &PlacedPad,
    layer: LayerId,
    paste: bool,
) -> Result<Option<PlacedPad>, ExportError> {
    let refusal = |reason: &str| ExportError::RefusedProcessGeometry {
        source_id: process.pad_id,
        reason: reason.into(),
    };
    if process.pad_id != copper.uuid {
        return Err(refusal("process/copper source identity mismatch"));
    }
    let (geometry, layers) = if paste {
        (&process.effective.paste, &process.membership.paste_layers)
    } else {
        (&process.effective.mask, &process.membership.mask_layers)
    };
    if !layers.contains(&layer) {
        return Ok(None);
    }
    let mut pad = copper.clone();
    pad.position = Point {
        x: process.center_nm[0].integer_nm().map_err(|_| {
            refusal("exact process centre is unsupported by integer-nanometre Gerber")
        })?,
        y: process.center_nm[1].integer_nm().map_err(|_| {
            refusal("exact process centre is unsupported by integer-nanometre Gerber")
        })?,
    };
    match geometry {
        EffectiveProcessAperture::Suppressed => {
            return Err(refusal(
                "suppressed aperture has nonempty process membership",
            ));
        }
        EffectiveProcessAperture::Circle { diameter_nm } => {
            pad.shape = PadShape::Circle;
            pad.diameter = diameter_nm
                .integer_nm()
                .map_err(|_| refusal("exact process diameter is nonintegral or out of range"))?;
            if pad.diameter <= 0 {
                return Err(refusal("nonpositive process diameter"));
            }
        }
        EffectiveProcessAperture::Rect {
            width_nm,
            height_nm,
        } => {
            if process.rotation.rem_euclid(90) != 0 {
                return Err(refusal(
                    "rectangular process rotation has no exact supported aperture",
                ));
            }
            let width = width_nm
                .integer_nm()
                .map_err(|_| refusal("exact process width is nonintegral or out of range"))?;
            let height = height_nm
                .integer_nm()
                .map_err(|_| refusal("exact process height is nonintegral or out of range"))?;
            if width <= 0 || height <= 0 {
                return Err(refusal("nonpositive process rectangle"));
            }
            pad.shape = PadShape::Rect;
            (pad.width, pad.height) = if process.rotation.rem_euclid(180) == 90 {
                (height, width)
            } else {
                (width, height)
            };
        }
    }
    // Offsets are already resolved; the legacy writer receives the exact aperture.
    pad.rotation = 0;
    pad.solder_mask_margin_nm = 0;
    pad.solder_paste_margin_nm = 0;
    pad.solder_paste_margin_ratio_ppm = 0;
    Ok(Some(pad))
}
