use crate::board::{PadAperture, PlacedPad};
use crate::ir::geometry::LayerId;

use super::{ExportError, format_coord, format_mm_6};

pub fn render_rs274x_soldermask_layer(
    layer_id: LayerId,
    pads: &[PlacedPad],
) -> Result<String, ExportError> {
    for pad in pads {
        match pad.aperture() {
            PadAperture::Circle { diameter_nm } if diameter_nm <= 0 => {
                return Err(ExportError::InvalidPadDiameter);
            }
            PadAperture::Rect { width_nm, .. } if width_nm <= 0 => {
                return Err(ExportError::InvalidPadWidth);
            }
            PadAperture::Rect { height_nm, .. } if height_nm <= 0 => {
                return Err(ExportError::InvalidPadHeight);
            }
            _ => {}
        }
    }

    let mut circle_apertures = Vec::new();
    let mut rect_apertures = Vec::new();
    for pad in pads {
        match pad.aperture() {
            PadAperture::Circle { diameter_nm } => circle_apertures.push(diameter_nm),
            PadAperture::Rect {
                width_nm,
                height_nm,
            } => rect_apertures.push((width_nm, height_nm)),
        }
    }
    circle_apertures.sort_unstable();
    circle_apertures.dedup();
    rect_apertures.sort_unstable();
    rect_apertures.dedup();

    let mut lines = vec![
        format!("G04 datum-eda native soldermask layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
    ];

    for (idx, diameter) in circle_apertures.iter().enumerate() {
        let d_code = 10 + idx;
        lines.push(format!("%ADD{d_code}C,{}*%", format_mm_6(*diameter)));
    }
    let rect_base_code = 10 + circle_apertures.len();
    for (idx, (width_nm, height_nm)) in rect_apertures.iter().enumerate() {
        let d_code = rect_base_code + idx;
        lines.push(format!(
            "%ADD{d_code}R,{}X{}*%",
            format_mm_6(*width_nm),
            format_mm_6(*height_nm)
        ));
    }

    let mut ordered_pads = pads.to_vec();
    ordered_pads.sort_by(|a, b| {
        a.shape
            .cmp(&b.shape)
            .then_with(|| a.diameter.cmp(&b.diameter))
            .then_with(|| a.width.cmp(&b.width))
            .then_with(|| a.height.cmp(&b.height))
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.layer.cmp(&b.layer))
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for pad in ordered_pads {
        let d_code = match pad.aperture() {
            PadAperture::Circle { diameter_nm } => {
                10 + circle_apertures
                    .binary_search(&diameter_nm)
                    .expect("known pad aperture")
            }
            PadAperture::Rect {
                width_nm,
                height_nm,
            } => {
                rect_base_code
                    + rect_apertures
                        .binary_search(&(width_nm, height_nm))
                        .expect("known rectangular pad aperture")
            }
        };
        lines.push(format!("D{d_code}*"));
        lines.push(format!(
            "X{}Y{}D03*",
            format_coord(pad.position.x),
            format_coord(pad.position.y)
        ));
    }

    lines.push(String::from("M02*"));
    Ok(lines.join("\n") + "\n")
}

pub fn render_rs274x_paste_layer(
    layer_id: LayerId,
    pads: &[PlacedPad],
) -> Result<String, ExportError> {
    for pad in pads {
        match pad.aperture() {
            PadAperture::Circle { diameter_nm } if diameter_nm <= 0 => {
                return Err(ExportError::InvalidPadDiameter);
            }
            PadAperture::Rect { width_nm, .. } if width_nm <= 0 => {
                return Err(ExportError::InvalidPadWidth);
            }
            PadAperture::Rect { height_nm, .. } if height_nm <= 0 => {
                return Err(ExportError::InvalidPadHeight);
            }
            _ => {}
        }
    }

    let mut circle_apertures = Vec::new();
    let mut rect_apertures = Vec::new();
    for pad in pads {
        match pad.aperture() {
            PadAperture::Circle { diameter_nm } => circle_apertures.push(diameter_nm),
            PadAperture::Rect {
                width_nm,
                height_nm,
            } => rect_apertures.push((width_nm, height_nm)),
        }
    }
    circle_apertures.sort_unstable();
    circle_apertures.dedup();
    rect_apertures.sort_unstable();
    rect_apertures.dedup();

    let mut lines = vec![
        format!("G04 datum-eda native paste layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
    ];

    for (idx, diameter) in circle_apertures.iter().enumerate() {
        let d_code = 10 + idx;
        lines.push(format!("%ADD{d_code}C,{}*%", format_mm_6(*diameter)));
    }
    let rect_base_code = 10 + circle_apertures.len();
    for (idx, (width_nm, height_nm)) in rect_apertures.iter().enumerate() {
        let d_code = rect_base_code + idx;
        lines.push(format!(
            "%ADD{d_code}R,{}X{}*%",
            format_mm_6(*width_nm),
            format_mm_6(*height_nm)
        ));
    }

    let mut ordered_pads = pads.to_vec();
    ordered_pads.sort_by(|a, b| {
        a.shape
            .cmp(&b.shape)
            .then_with(|| a.diameter.cmp(&b.diameter))
            .then_with(|| a.width.cmp(&b.width))
            .then_with(|| a.height.cmp(&b.height))
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.layer.cmp(&b.layer))
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for pad in ordered_pads {
        let d_code = match pad.aperture() {
            PadAperture::Circle { diameter_nm } => {
                10 + circle_apertures
                    .binary_search(&diameter_nm)
                    .expect("known pad aperture")
            }
            PadAperture::Rect {
                width_nm,
                height_nm,
            } => {
                rect_base_code
                    + rect_apertures
                        .binary_search(&(width_nm, height_nm))
                        .expect("known rectangular pad aperture")
            }
        };
        lines.push(format!("D{d_code}*"));
        lines.push(format!(
            "X{}Y{}D03*",
            format_coord(pad.position.x),
            format_coord(pad.position.y)
        ));
    }

    lines.push(String::from("M02*"));
    Ok(lines.join("\n") + "\n")
}
