use crate::ir::geometry::{LayerId, Polygon};

use super::{
    DEFAULT_OUTLINE_APERTURE_MM, ExportError, format_coord, format_mm_6, parse_mm_6_to_nm,
    render_polygon_points,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MechanicalStroke {
    pub from: crate::ir::geometry::Point,
    pub to: crate::ir::geometry::Point,
    pub width_nm: i64,
}

pub fn render_rs274x_mechanical_layer(
    layer_id: LayerId,
    polygons: &[Polygon],
    strokes: &[MechanicalStroke],
) -> Result<String, ExportError> {
    if strokes.iter().any(|stroke| stroke.width_nm <= 0) {
        return Err(ExportError::InvalidTrackWidth);
    }
    let aperture_nm = parse_mm_6_to_nm(DEFAULT_OUTLINE_APERTURE_MM)
        .expect("default RS-274X outline aperture must parse");
    let mut stroke_widths = strokes
        .iter()
        .map(|stroke| stroke.width_nm)
        .collect::<Vec<_>>();
    stroke_widths.push(aperture_nm);
    stroke_widths.sort_unstable();
    stroke_widths.dedup();
    let mut lines = vec![
        format!("G04 datum-eda native mechanical layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
    ];
    for (idx, width_nm) in stroke_widths.iter().enumerate() {
        let d_code = 10 + idx;
        lines.push(format!("%ADD{d_code}C,{}*%", format_mm_6(*width_nm)));
    }
    let default_d_code = 10
        + stroke_widths
            .binary_search(&aperture_nm)
            .expect("default mechanical aperture must be defined");
    lines.push(format!("D{default_d_code}*"));

    let ordered_polygons = ordered_mechanical_polygons(polygons);

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

    let ordered_strokes = ordered_mechanical_strokes(strokes);
    for stroke in ordered_strokes {
        let d_code = 10
            + stroke_widths
                .binary_search(&stroke.width_nm)
                .expect("known mechanical stroke aperture");
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

fn ordered_mechanical_polygons(polygons: &[Polygon]) -> Vec<Polygon> {
    let mut ordered = polygons.to_vec();
    ordered.sort_by(|a, b| {
        render_polygon_points(&a.vertices)
            .cmp(&render_polygon_points(&b.vertices))
            .then_with(|| a.closed.cmp(&b.closed))
    });
    ordered
}

fn ordered_mechanical_strokes(strokes: &[MechanicalStroke]) -> Vec<MechanicalStroke> {
    let mut ordered = strokes.to_vec();
    ordered.sort_by(|a, b| {
        a.width_nm
            .cmp(&b.width_nm)
            .then_with(|| a.from.x.cmp(&b.from.x))
            .then_with(|| a.from.y.cmp(&b.from.y))
            .then_with(|| a.to.x.cmp(&b.to.x))
            .then_with(|| a.to.y.cmp(&b.to.y))
    });
    ordered
}
