use crate::board::Track;
use crate::ir::geometry::{LayerId, Polygon};
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
}

pub fn render_rs274x_outline(polygon: &Polygon, aperture_diameter_nm: i64) -> Result<String, ExportError> {
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
    lines.push(format!("X{}Y{}D02*", format_coord(first.x), format_coord(first.y)));
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

pub fn render_rs274x_copper_layer(layer_id: LayerId, tracks: &[Track]) -> Result<String, ExportError> {
    if tracks.iter().any(|track| track.width <= 0) {
        return Err(ExportError::InvalidTrackWidth);
    }

    let mut widths = tracks.iter().map(|track| track.width).collect::<Vec<_>>();
    widths.sort_unstable();
    widths.dedup();

    let mut lines = vec![
        format!("G04 datum-eda native copper layer {layer_id}*"),
        String::from("%FSLAX46Y46*%"),
        String::from("%MOMM*%"),
        String::from("%LPD*%"),
    ];

    for (idx, width) in widths.iter().enumerate() {
        let d_code = 10 + idx;
        lines.push(format!("%ADD{d_code}C,{}*%", format_mm_6(*width)));
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
        let d_code = 10 + widths.binary_search(&track.width).expect("known width aperture");
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

    lines.push(String::from("M02*"));
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
mod tests {
    use super::*;
    use crate::ir::geometry::Point;

    #[test]
    fn render_rs274x_outline_closed_polygon() {
        let polygon = Polygon {
            vertices: vec![
                Point { x: 0, y: 0 },
                Point { x: 1_000_000, y: 0 },
                Point { x: 1_000_000, y: 500_000 },
            ],
            closed: true,
        };

        let gerber = render_rs274x_outline_default(&polygon).expect("outline should render");
        assert!(gerber.contains("%FSLAX46Y46*%"));
        assert!(gerber.contains("%MOMM*%"));
        assert!(gerber.contains("%ADD10C,0.100000*%"));
        assert!(gerber.contains("X0Y0D02*"));
        assert!(gerber.contains("X1000000Y0D01*"));
        assert!(gerber.contains("X1000000Y500000D01*"));
        assert!(gerber.contains("X0Y0D01*"));
        assert!(gerber.ends_with("M02*\n"));
    }

    #[test]
    fn render_rs274x_outline_open_polygon_does_not_close() {
        let polygon = Polygon {
            vertices: vec![
                Point { x: 0, y: 0 },
                Point { x: 500_000, y: 500_000 },
            ],
            closed: false,
        };

        let gerber = render_rs274x_outline_default(&polygon).expect("outline should render");
        let occurrences = gerber.matches("X0Y0").count();
        assert_eq!(occurrences, 1);
    }

    #[test]
    fn render_rs274x_outline_requires_two_vertices() {
        let polygon = Polygon {
            vertices: vec![Point { x: 0, y: 0 }],
            closed: true,
        };

        let err = render_rs274x_outline_default(&polygon).expect_err("outline should fail");
        assert!(matches!(err, ExportError::OutlineTooShort));
    }

    #[test]
    fn render_rs274x_copper_layer_assigns_apertures_by_width() {
        let tracks = vec![
            Track {
                uuid: uuid::Uuid::nil(),
                net: uuid::Uuid::nil(),
                from: Point { x: 0, y: 0 },
                to: Point {
                    x: 1_000_000,
                    y: 0,
                },
                width: 200_000,
                layer: 1,
            },
            Track {
                uuid: uuid::Uuid::from_u128(1),
                net: uuid::Uuid::nil(),
                from: Point {
                    x: 0,
                    y: 500_000,
                },
                to: Point {
                    x: 1_000_000,
                    y: 500_000,
                },
                width: 300_000,
                layer: 1,
            },
        ];

        let gerber = render_rs274x_copper_layer(1, &tracks).expect("copper should render");
        assert!(gerber.contains("%ADD10C,0.200000*%"));
        assert!(gerber.contains("%ADD11C,0.300000*%"));
        assert!(gerber.contains("D10*"));
        assert!(gerber.contains("D11*"));
        assert!(gerber.contains("X0Y0D02*"));
        assert!(gerber.contains("X1000000Y0D01*"));
        assert!(gerber.ends_with("M02*\n"));
    }

    #[test]
    fn render_rs274x_copper_layer_rejects_non_positive_width() {
        let tracks = vec![Track {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            from: Point { x: 0, y: 0 },
            to: Point { x: 1, y: 1 },
            width: 0,
            layer: 1,
        }];

        let err = render_rs274x_copper_layer(1, &tracks).expect_err("copper should fail");
        assert!(matches!(err, ExportError::InvalidTrackWidth));
    }
}
