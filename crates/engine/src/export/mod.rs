use crate::board::{BoardText, PadAperture, PlacedPad, Track, Via, Zone};
use crate::ir::geometry::{LayerId, Point, Polygon};
use thiserror::Error;

const DEFAULT_OUTLINE_APERTURE_MM: &str = "0.100000";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilkscreenStroke {
    pub from: Point,
    pub to: Point,
    pub width_nm: i64,
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

pub fn render_rs274x_copper_layer(
    layer_id: LayerId,
    pads: &[PlacedPad],
    tracks: &[Track],
    zones: &[Zone],
    vias: &[Via],
) -> Result<String, ExportError> {
    if tracks.iter().any(|track| track.width <= 0) {
        return Err(ExportError::InvalidTrackWidth);
    }
    if vias.iter().any(|via| via.diameter <= 0) {
        return Err(ExportError::InvalidViaDiameter);
    }
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

    let mut circle_apertures = tracks
        .iter()
        .map(|track| track.width)
        .chain(vias.iter().map(|via| via.diameter))
        .collect::<Vec<_>>();
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
        format!("G04 datum-eda native copper layer {layer_id}*"),
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

    let mut ordered_tracks = tracks.to_vec();
    ordered_tracks.sort_by(|a, b| {
        a.width
            .cmp(&b.width)
            .then_with(|| a.from.x.cmp(&b.from.x))
            .then_with(|| a.from.y.cmp(&b.from.y))
            .then_with(|| a.to.x.cmp(&b.to.x))
            .then_with(|| a.to.y.cmp(&b.to.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for track in ordered_tracks {
        let d_code = 10
            + circle_apertures
                .binary_search(&track.width)
                .expect("known width aperture");
        lines.push(format!("D{d_code}*"));
        lines.push(format!(
            "X{}Y{}D02*",
            format_coord(track.from.x),
            format_coord(track.from.y)
        ));
        lines.push(format!(
            "X{}Y{}D01*",
            format_coord(track.to.x),
            format_coord(track.to.y)
        ));
    }

    let mut ordered_vias = vias.to_vec();
    ordered_vias.sort_by(|a, b| {
        a.diameter
            .cmp(&b.diameter)
            .then_with(|| a.position.x.cmp(&b.position.x))
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.from_layer.cmp(&b.from_layer))
            .then_with(|| a.to_layer.cmp(&b.to_layer))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for via in ordered_vias {
        let d_code = 10
            + circle_apertures
                .binary_search(&via.diameter)
                .expect("known via aperture");
        lines.push(format!("D{d_code}*"));
        lines.push(format!(
            "X{}Y{}D03*",
            format_coord(via.position.x),
            format_coord(via.position.y)
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

    let mut ordered_zones = zones.to_vec();
    ordered_zones.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.polygon.vertices.len().cmp(&b.polygon.vertices.len()))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });

    for zone in ordered_zones {
        if zone.polygon.vertices.len() < 2 {
            continue;
        }
        let first = zone.polygon.vertices[0];
        lines.push(String::from("G36*"));
        lines.push(format!(
            "X{}Y{}D02*",
            format_coord(first.x),
            format_coord(first.y)
        ));
        for vertex in zone.polygon.vertices.iter().skip(1) {
            lines.push(format!(
                "X{}Y{}D01*",
                format_coord(vertex.x),
                format_coord(vertex.y)
            ));
        }
        if zone.polygon.closed {
            lines.push(format!(
                "X{}Y{}D01*",
                format_coord(first.x),
                format_coord(first.y)
            ));
        }
        lines.push(String::from("G37*"));
    }

    lines.push(String::from("M02*"));
    Ok(lines.join("\n") + "\n")
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

pub fn render_rs274x_mechanical_layer(
    layer_id: LayerId,
    polygons: &[Polygon],
) -> Result<String, ExportError> {
    let aperture_nm = parse_mm_6_to_nm(DEFAULT_OUTLINE_APERTURE_MM)
        .expect("default RS-274X outline aperture must parse");
    let mut lines = vec![
        format!("G04 datum-eda native mechanical layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
        format!("%ADD10C,{}*%", format_mm_6(aperture_nm)),
        String::from("D10*"),
    ];

    let mut ordered_polygons = polygons.to_vec();
    ordered_polygons.sort_by(|a, b| {
        render_polygon_points(&a.vertices)
            .cmp(&render_polygon_points(&b.vertices))
            .then_with(|| a.closed.cmp(&b.closed))
    });

    for polygon in ordered_polygons {
        if polygon.vertices.len() < 2 {
            continue;
        }
        let first = polygon.vertices[0];
        if polygon.closed {
            lines.push(String::from("G36*"));
        }
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
            lines.push(String::from("G37*"));
        }
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
        for (from, to) in render_silkscreen_text_strokes(&text)? {
            lines.push(format!(
                "X{}Y{}D02*",
                format_coord(from.x),
                format_coord(from.y)
            ));
            lines.push(format!(
                "X{}Y{}D01*",
                format_coord(to.x),
                format_coord(to.y)
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

fn render_silkscreen_text_strokes(
    text: &BoardText,
) -> Result<Vec<(crate::ir::geometry::Point, crate::ir::geometry::Point)>, ExportError> {
    let mut strokes = Vec::new();
    let scale_nm = text.height_nm / 5;
    let advance_nm = scale_nm * 4;
    let mut cursor_x = 0_i64;
    for ch in text.text.chars() {
        for ((x1, y1), (x2, y2)) in glyph_strokes(ch)? {
            let from = rotate_text_point(
                text.position,
                text.rotation,
                cursor_x + x1 * scale_nm,
                y1 * scale_nm,
            );
            let to = rotate_text_point(
                text.position,
                text.rotation,
                cursor_x + x2 * scale_nm,
                y2 * scale_nm,
            );
            strokes.push((from, to));
        }
        cursor_x += advance_nm;
    }
    Ok(strokes)
}

fn rotate_text_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    x_nm: i64,
    y_nm: i64,
) -> crate::ir::geometry::Point {
    let radians = f64::from(rotation_deg).to_radians();
    let x = x_nm as f64;
    let y = y_nm as f64;
    let rotated_x = x * radians.cos() - y * radians.sin();
    let rotated_y = x * radians.sin() + y * radians.cos();
    crate::ir::geometry::Point {
        x: origin.x + rotated_x.round() as i64,
        y: origin.y + rotated_y.round() as i64,
    }
}

type GlyphStroke = ((i64, i64), (i64, i64));

fn glyph_strokes(ch: char) -> Result<&'static [GlyphStroke], ExportError> {
    let glyph = match ch {
        ' ' => &[][..],
        '-' => &[((0, 2), (2, 2))][..],
        '_' => &[((0, 0), (2, 0))][..],
        '.' => &[((1, 0), (1, 0))][..],
        '/' => &[((0, 0), (2, 4))][..],
        '+' => &[((1, 0), (1, 4)), ((0, 2), (2, 2))][..],
        '0' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 0)),
            ((2, 0), (0, 0)),
        ][..],
        '1' => &[((1, 0), (1, 4))][..],
        '2' => &[
            ((0, 4), (2, 4)),
            ((2, 4), (2, 2)),
            ((2, 2), (0, 2)),
            ((0, 2), (0, 0)),
            ((0, 0), (2, 0)),
        ][..],
        '3' => &[
            ((0, 4), (2, 4)),
            ((2, 4), (2, 0)),
            ((0, 2), (2, 2)),
            ((0, 0), (2, 0)),
        ][..],
        '4' => &[((0, 4), (0, 2)), ((0, 2), (2, 2)), ((2, 4), (2, 0))][..],
        '5' => &[
            ((2, 4), (0, 4)),
            ((0, 4), (0, 2)),
            ((0, 2), (2, 2)),
            ((2, 2), (2, 0)),
            ((2, 0), (0, 0)),
        ][..],
        '6' => &[
            ((2, 4), (0, 4)),
            ((0, 4), (0, 0)),
            ((0, 0), (2, 0)),
            ((2, 0), (2, 2)),
            ((2, 2), (0, 2)),
        ][..],
        '7' => &[((0, 4), (2, 4)), ((2, 4), (1, 0))][..],
        '8' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 0)),
            ((2, 0), (0, 0)),
            ((0, 2), (2, 2)),
        ][..],
        '9' => &[
            ((2, 0), (2, 4)),
            ((2, 4), (0, 4)),
            ((0, 4), (0, 2)),
            ((0, 2), (2, 2)),
            ((2, 0), (0, 0)),
        ][..],
        'A' => &[
            ((0, 0), (0, 4)),
            ((2, 0), (2, 4)),
            ((0, 4), (2, 4)),
            ((0, 2), (2, 2)),
        ][..],
        'B' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 2)),
            ((2, 2), (0, 2)),
            ((2, 2), (2, 0)),
            ((2, 0), (0, 0)),
        ][..],
        'C' => &[((2, 4), (0, 4)), ((0, 4), (0, 0)), ((0, 0), (2, 0))][..],
        'D' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 3)),
            ((2, 3), (2, 1)),
            ((2, 1), (0, 0)),
        ][..],
        'E' => &[
            ((2, 4), (0, 4)),
            ((0, 4), (0, 0)),
            ((0, 2), (2, 2)),
            ((0, 0), (2, 0)),
        ][..],
        'F' => &[((0, 0), (0, 4)), ((0, 4), (2, 4)), ((0, 2), (2, 2))][..],
        'G' => &[
            ((2, 4), (0, 4)),
            ((0, 4), (0, 0)),
            ((0, 0), (2, 0)),
            ((2, 0), (2, 2)),
            ((2, 2), (1, 2)),
        ][..],
        'H' => &[((0, 0), (0, 4)), ((2, 0), (2, 4)), ((0, 2), (2, 2))][..],
        'I' => &[((0, 4), (2, 4)), ((1, 4), (1, 0)), ((0, 0), (2, 0))][..],
        'J' => &[((0, 4), (2, 4)), ((1, 4), (1, 0)), ((1, 0), (0, 0))][..],
        'K' => &[((0, 0), (0, 4)), ((2, 4), (0, 2)), ((0, 2), (2, 0))][..],
        'L' => &[((0, 4), (0, 0)), ((0, 0), (2, 0))][..],
        'M' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (1, 2)),
            ((1, 2), (2, 4)),
            ((2, 4), (2, 0)),
        ][..],
        'N' => &[((0, 0), (0, 4)), ((0, 4), (2, 0)), ((2, 0), (2, 4))][..],
        'O' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 0)),
            ((2, 0), (0, 0)),
        ][..],
        'P' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 2)),
            ((2, 2), (0, 2)),
        ][..],
        'Q' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 0)),
            ((2, 0), (0, 0)),
            ((1, 1), (2, 0)),
        ][..],
        'R' => &[
            ((0, 0), (0, 4)),
            ((0, 4), (2, 4)),
            ((2, 4), (2, 2)),
            ((2, 2), (0, 2)),
            ((0, 2), (2, 0)),
        ][..],
        'S' => &[
            ((2, 4), (0, 4)),
            ((0, 4), (0, 2)),
            ((0, 2), (2, 2)),
            ((2, 2), (2, 0)),
            ((2, 0), (0, 0)),
        ][..],
        'T' => &[((0, 4), (2, 4)), ((1, 4), (1, 0))][..],
        'U' => &[((0, 4), (0, 0)), ((0, 0), (2, 0)), ((2, 0), (2, 4))][..],
        'V' => &[((0, 4), (1, 0)), ((1, 0), (2, 4))][..],
        'W' => &[
            ((0, 4), (0, 0)),
            ((0, 0), (1, 2)),
            ((1, 2), (2, 0)),
            ((2, 0), (2, 4)),
        ][..],
        'X' => &[((0, 4), (2, 0)), ((0, 0), (2, 4))][..],
        'Y' => &[((0, 4), (1, 2)), ((2, 4), (1, 2)), ((1, 2), (1, 0))][..],
        'Z' => &[((0, 4), (2, 4)), ((2, 4), (0, 0)), ((0, 0), (2, 0))][..],
        'a'..='z' => return glyph_strokes(ch.to_ascii_uppercase()),
        _ => return Err(ExportError::UnsupportedSilkscreenTextCharacter(ch)),
    };
    Ok(glyph)
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

fn format_coord(nm: i64) -> String {
    nm.to_string()
}

fn format_mm_6(nm: i64) -> String {
    let sign = if nm < 0 { "-" } else { "" };
    let abs = nm.abs();
    let whole = abs / 1_000_000;
    let frac = abs % 1_000_000;
    format!("{sign}{whole}.{frac:06}")
}

fn render_polygon_points(points: &[crate::ir::geometry::Point]) -> String {
    points
        .iter()
        .map(|point| format!("({}, {})", point.x, point.y))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn parse_mm_6_to_nm(value: &str) -> Option<i64> {
    let mut parts = value.split('.');
    let whole = parts.next()?.parse::<i64>().ok()?;
    let frac_str = parts.next().unwrap_or("0");
    if parts.next().is_some() {
        return None;
    }
    let mut frac = frac_str.to_string();
    if frac.len() > 6 {
        return None;
    }
    while frac.len() < 6 {
        frac.push('0');
    }
    let frac = frac.parse::<i64>().ok()?;
    Some(whole * 1_000_000 + frac)
}

#[cfg(test)]
mod tests;
