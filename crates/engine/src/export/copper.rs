use crate::board::{PadAperture, PlacedPad, Track, Via, Zone};
use crate::ir::geometry::LayerId;

use super::{ExportError, format_coord, format_mm_6};

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
