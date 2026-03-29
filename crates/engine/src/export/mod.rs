use crate::board::BoardText;
use crate::ir::geometry::{LayerId, Polygon};
use thiserror::Error;
mod copper;
mod excellon;
mod formatting;
mod gerber_mechanical;
mod mask;
mod outline;
mod silkscreen;

pub use copper::render_rs274x_copper_layer;
pub use excellon::render_excellon_drill;
use formatting::{format_coord, format_mm_6, parse_mm_6_to_nm, render_polygon_points};
pub use gerber_mechanical::{MechanicalStroke, render_rs274x_mechanical_layer};
pub use mask::{render_rs274x_paste_layer, render_rs274x_soldermask_layer};
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
#[cfg(test)]
mod tests;
