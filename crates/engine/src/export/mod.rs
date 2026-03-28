use crate::board::{BoardText, PadAperture, PlacedPad, Via};
use crate::ir::geometry::{LayerId, Polygon};
use thiserror::Error;
mod copper;
mod formatting;
mod gerber_mechanical;
mod outline;
mod silkscreen;

pub use copper::render_rs274x_copper_layer;
use formatting::{format_coord, format_mm_6, parse_mm_6_to_nm, render_polygon_points};
pub use gerber_mechanical::{MechanicalStroke, render_rs274x_mechanical_layer};
use outline::DEFAULT_OUTLINE_APERTURE_MM;
pub use silkscreen::{SilkscreenStroke, render_silkscreen_text_strokes};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("board outline export requires at least two vertices")]
    OutlineTooShort,
    #[error("outline aperture diameter must be positive")]
    InvalidAperture,
    #[error("copper-layer export requires positive track widths")]
    InvalidTrackWidth,
    #[error("copper-layer export requires positive via diameters")]
    InvalidViaDiameter,
    #[error("copper-layer export requires positive pad diameters")]
    InvalidPadDiameter,
    #[error("copper-layer export requires positive pad rectangle widths")]
    InvalidPadWidth,
    #[error("copper-layer export requires positive pad rectangle heights")]
    InvalidPadHeight,
    #[error("silkscreen export requires positive text heights")]
    InvalidTextHeight,
    #[error("silkscreen export requires positive text stroke widths")]
    InvalidTextStrokeWidth,
    #[error("silkscreen export encountered unsupported text character: {0}")]
    UnsupportedSilkscreenTextCharacter(char),
    #[error("drill export requires positive via drill diameters")]
    InvalidViaDrill,
}

pub fn render_rs274x_outline(
    polygon: &Polygon,
    aperture_diameter_nm: i64,
) -> Result<String, ExportError> {
    if polygon.vertices.len() < 2 {
        return Err(ExportError::OutlineTooShort);
    }
    if aperture_diameter_nm <= 0 {
        return Err(ExportError::InvalidAperture);
    }

    let mut lines = vec![
        String::from("G04 datum-eda native board outline*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
        format!("%ADD10C,{}*%", format_mm_6(aperture_diameter_nm)),
        String::from("D10*"),
    ];

    let first = polygon.vertices[0];
    lines.push(format!(
        "X{}Y{}D02*",
        format_coord(first.x),
        format_coord(first.y)
    ));
    for vertex in polygon.vertices.iter().skip(1) {
        lines.push(format!(
            "X{}Y{}D01*",
            format_coord(vertex.x),
            format_coord(vertex.y)
        ));
    }
    if polygon.closed {
        lines.push(format!(
            "X{}Y{}D01*",
            format_coord(first.x),
            format_coord(first.y)
        ));
    }
    lines.push(String::from("M02*"));
    Ok(lines.join("\n") + "\n")
}

pub fn render_rs274x_outline_default(polygon: &Polygon) -> Result<String, ExportError> {
    let aperture_nm = parse_mm_6_to_nm(DEFAULT_OUTLINE_APERTURE_MM)
        .expect("default RS-274X outline aperture must parse");
    render_rs274x_outline(polygon, aperture_nm)
}

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

pub fn render_rs274x_silkscreen_layer(
    layer_id: LayerId,
    texts: &[BoardText],
    strokes: &[SilkscreenStroke],
) -> Result<String, ExportError> {
    if texts.iter().any(|text| text.height_nm <= 0) {
        return Err(ExportError::InvalidTextHeight);
    }
    if texts.iter().any(|text| text.stroke_width_nm <= 0) {
        return Err(ExportError::InvalidTextStrokeWidth);
    }
    if strokes.iter().any(|stroke| stroke.width_nm <= 0) {
        return Err(ExportError::InvalidTrackWidth);
    }

    let mut stroke_widths = texts
        .iter()
        .map(|text| text.stroke_width_nm)
        .chain(strokes.iter().map(|stroke| stroke.width_nm))
        .collect::<Vec<_>>();
    stroke_widths.sort_unstable();
    stroke_widths.dedup();

    let mut lines = vec![
        format!("G04 datum-eda native silkscreen layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
    ];
    for (idx, width_nm) in stroke_widths.iter().enumerate() {
        let d_code = 10 + idx;
        lines.push(format!("%ADD{d_code}C,{}*%", format_mm_6(*width_nm)));
    }

    let mut ordered_texts = texts.to_vec();
    ordered_texts.sort_by(|a, b| {
        a.stroke_width_nm
            .cmp(&b.stroke_width_nm)
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.rotation.cmp(&b.rotation))
            .then_with(|| a.text.cmp(&b.text))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for text in ordered_texts {
        let d_code = 10
            + stroke_widths
                .binary_search(&text.stroke_width_nm)
                .expect("known text stroke width aperture");
        lines.push(format!("D{d_code}*"));
        for stroke in render_silkscreen_text_strokes(&text)? {
            lines.push(format!(
                "X{}Y{}D02*",
                format_coord(stroke.from.x),
                format_coord(stroke.from.y)
            ));
            lines.push(format!(
                "X{}Y{}D01*",
                format_coord(stroke.to.x),
                format_coord(stroke.to.y)
            ));
        }
    }

    let mut ordered_strokes = strokes.to_vec();
    ordered_strokes.sort_by(|a, b| {
        a.width_nm
            .cmp(&b.width_nm)
            .then_with(|| a.from.x.cmp(&b.from.x))
            .then_with(|| a.from.y.cmp(&b.from.y))
            .then_with(|| a.to.x.cmp(&b.to.x))
            .then_with(|| a.to.y.cmp(&b.to.y))
    });

    for stroke in ordered_strokes {
        let d_code = 10
            + stroke_widths
                .binary_search(&stroke.width_nm)
                .expect("known stroke width aperture");
        lines.push(format!("D{d_code}*"));
        lines.push(format!(
            "X{}Y{}D02*",
            format_coord(stroke.from.x),
            format_coord(stroke.from.y)
        ));
        lines.push(format!(
            "X{}Y{}D01*",
            format_coord(stroke.to.x),
            format_coord(stroke.to.y)
        ));
    }

    lines.push(String::from("M02*"));
    Ok(lines.join("\n") + "\n")
}
pub fn render_excellon_drill(vias: &[Via]) -> Result<String, ExportError> {
    if vias.iter().any(|via| via.drill <= 0) {
        return Err(ExportError::InvalidViaDrill);
    }

    let mut drills = vias.iter().map(|via| via.drill).collect::<Vec<_>>();
    drills.sort_unstable();
    drills.dedup();

    let mut lines = vec![String::from("M48"), String::from("METRIC,TZ")];
    for (idx, drill) in drills.iter().enumerate() {
        let tool_code = idx + 1;
        lines.push(format!("T{tool_code:02}C{}", format_mm_6(*drill)));
    }
    lines.push(String::from("%"));

    let mut ordered_vias = vias.to_vec();
    ordered_vias.sort_by(|a, b| {
        a.drill
            .cmp(&b.drill)
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    let mut current_tool = None;
    for via in ordered_vias {
        let tool_code = drills.binary_search(&via.drill).expect("known drill tool") + 1;
        if current_tool != Some(tool_code) {
            lines.push(format!("T{tool_code:02}"));
            current_tool = Some(tool_code);
        }
        lines.push(format!(
            "X{}Y{}",
            format_mm_6(via.position.x),
            format_mm_6(via.position.y)
        ));
    }

    lines.push(String::from("M30"));
    Ok(lines.join("\n") + "\n")
}

#[cfg(test)]
mod tests;
