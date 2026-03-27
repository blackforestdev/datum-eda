use crate::board::{Track, Via, Zone};
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
    #[error("copper-layer export requires positive via diameters")]
    InvalidViaDiameter,
    #[error("drill export requires positive via drill diameters")]
    InvalidViaDrill,
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

pub fn render_rs274x_copper_layer(
    layer_id: LayerId,
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

    let mut circle_apertures = tracks
        .iter()
        .map(|track| track.width)
        .chain(vias.iter().map(|via| via.diameter))
        .collect::<Vec<_>>();
    circle_apertures.sort_unstable();
    circle_apertures.dedup();

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

        let gerber =
            render_rs274x_copper_layer(1, &tracks, &[], &[]).expect("copper should render");
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

        let err =
            render_rs274x_copper_layer(1, &tracks, &[], &[]).expect_err("copper should fail");
        assert!(matches!(err, ExportError::InvalidTrackWidth));
    }

    #[test]
    fn render_rs274x_copper_layer_emits_zone_region() {
        let zones = vec![Zone {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            polygon: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1_000_000, y: 0 },
                    Point {
                        x: 1_000_000,
                        y: 500_000,
                    },
                ],
                closed: true,
            },
            layer: 1,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 0,
            thermal_spoke_width: 0,
        }];

        let gerber = render_rs274x_copper_layer(1, &[], &zones, &[]).expect("zone should render");
        assert!(gerber.contains("G36*"));
        assert!(gerber.contains("G37*"));
        assert!(gerber.contains("X0Y0D02*"));
        assert!(gerber.contains("X1000000Y0D01*"));
        assert!(gerber.contains("X1000000Y500000D01*"));
    }

    #[test]
    fn render_rs274x_copper_layer_emits_via_flashes() {
        let vias = vec![Via {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            position: Point {
                x: 250_000,
                y: 750_000,
            },
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 2,
        }];

        let gerber = render_rs274x_copper_layer(1, &[], &[], &vias).expect("via should render");
        assert!(gerber.contains("%ADD10C,0.600000*%"));
        assert!(gerber.contains("D10*"));
        assert!(gerber.contains("X250000Y750000D03*"));
    }

    #[test]
    fn render_rs274x_copper_layer_rejects_non_positive_via_diameter() {
        let vias = vec![Via {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            position: Point { x: 0, y: 0 },
            drill: 300_000,
            diameter: 0,
            from_layer: 1,
            to_layer: 2,
        }];

        let err =
            render_rs274x_copper_layer(1, &[], &[], &vias).expect_err("via diameter should fail");
        assert!(matches!(err, ExportError::InvalidViaDiameter));
    }

    #[test]
    fn render_excellon_drill_assigns_tools_by_drill() {
        let vias = vec![
            Via {
                uuid: uuid::Uuid::nil(),
                net: uuid::Uuid::nil(),
                position: Point {
                    x: 1_000_000,
                    y: 1_500_000,
                },
                drill: 300_000,
                diameter: 600_000,
                from_layer: 1,
                to_layer: 2,
            },
            Via {
                uuid: uuid::Uuid::from_u128(1),
                net: uuid::Uuid::nil(),
                position: Point {
                    x: 2_000_000,
                    y: 3_000_000,
                },
                drill: 350_000,
                diameter: 700_000,
                from_layer: 1,
                to_layer: 2,
            },
        ];

        let excellon = render_excellon_drill(&vias).expect("drill should render");
        assert!(excellon.contains("M48"));
        assert!(excellon.contains("METRIC,TZ"));
        assert!(excellon.contains("T01C0.300000"));
        assert!(excellon.contains("T02C0.350000"));
        assert!(excellon.contains("T01\nX1.000000Y1.500000"));
        assert!(excellon.contains("T02\nX2.000000Y3.000000"));
        assert!(excellon.ends_with("M30\n"));
    }

    #[test]
    fn render_excellon_drill_rejects_non_positive_drill() {
        let vias = vec![Via {
            uuid: uuid::Uuid::nil(),
            net: uuid::Uuid::nil(),
            position: Point { x: 0, y: 0 },
            drill: 0,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 2,
        }];

        let err = render_excellon_drill(&vias).expect_err("drill should fail");
        assert!(matches!(err, ExportError::InvalidViaDrill));
    }
}
