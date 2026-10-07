//! Exact PM057 documentation graphics in the existing FS46 manufacturing stream.
//! Ucamco 2026.05 §§4.4, 4.5, 4.7, 4.10: aperture holes are transparent;
//! clear objects in the image plane would erase unrelated underlying graphics.
use super::{ArcOutputDisposition, ExportError, arc_output_witness, format_coord, format_mm_6};
use crate::board::{
    Track,
    graphics::{BoardGraphic, GraphicGeometry as G},
};
use crate::ir::geometry::Point;
use std::collections::BTreeSet;

const LIMIT: u64 = 9_999_999_999;
type Result<T> = std::result::Result<T, ExportError>;
fn refused(g: &BoardGraphic, reason: impl Into<String>) -> ExportError {
    ExportError::RefusedBoardGraphic {
        source_id: g.uuid,
        reason: reason.into(),
    }
}
fn checked_point(g: &BoardGraphic, p: Point) -> Result<()> {
    if p.x.unsigned_abs() > LIMIT || p.y.unsigned_abs() > LIMIT {
        return Err(refused(g, "coordinate outside exact FS46 output range"));
    }
    Ok(())
}
fn coordinate(p: Point, operation: u8) -> String {
    format!("X{}Y{}D0{operation}*", format_coord(p.x), format_coord(p.y))
}
fn path(g: &BoardGraphic, points: &[Point], close: bool, lines: &mut Vec<String>) -> Result<()> {
    for &p in points {
        checked_point(g, p)?;
    }
    lines.push("G01*".into());
    lines.push(coordinate(points[0], 2));
    for &p in &points[1..] {
        lines.push(coordinate(p, 1));
    }
    if close {
        lines.push(coordinate(points[0], 1));
    }
    Ok(())
}
fn aperture(
    g: &BoardGraphic,
    code: &mut usize,
    diameter: i64,
    hole: Option<i64>,
    lines: &mut Vec<String>,
) -> Result<usize> {
    if diameter <= 0 {
        return Err(refused(g, "nonpositive output aperture"));
    }
    let current = *code;
    *code = code
        .checked_add(1)
        .ok_or_else(|| refused(g, "aperture namespace exhausted"))?;
    let params = match hole {
        Some(h) if h > 0 && h < diameter => format!("{}X{}", format_mm_6(diameter), format_mm_6(h)),
        Some(_) => return Err(refused(g, "invalid transparent aperture hole")),
        None => format_mm_6(diameter),
    };
    lines.push(format!("%ADD{current}C,{params}*%"));
    Ok(current)
}
fn polygon(
    g: &BoardGraphic,
    outer: &[Point],
    holes: &[Vec<Point>],
    filled: bool,
    code: &mut usize,
    lines: &mut Vec<String>,
) -> Result<()> {
    for ring in std::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
        for &p in ring {
            checked_point(g, p)?;
        }
    }
    if filled {
        for ring in std::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
            // Gerber 4.10.3/4.14 requires robust contours. This format
            // constraint refuses small features; it never moves a coordinate.
            use crate::board::{nominal_geometry::DistanceBoundary, track_contact::tracks_within};
            for (i, &point) in ring.iter().enumerate() {
                let next = ring[(i + 1) % ring.len()];
                let dx = i128::from(point.x) - i128::from(next.x);
                let dy = i128::from(point.y) - i128::from(next.y);
                if dx * dx + dy * dy < 1_000_000 {
                    return Err(refused(
                        g,
                        "adjacent contour vertices below Gerber robustness requirement; required output remains pending",
                    ));
                }
                for j in 0..ring.len() {
                    let end = (j + 1) % ring.len();
                    if i == j || i == end {
                        continue;
                    }
                    let probe =
                        Track::straight(g.uuid, uuid::Uuid::nil(), point, point, 1, g.layer);
                    let edge =
                        Track::straight(g.uuid, uuid::Uuid::nil(), ring[j], ring[end], 1, g.layer);
                    if tracks_within(&probe, &edge, 999, DistanceBoundary::Strict)
                        .map_err(|e| refused(g, format!("contour certificate failed: {e:?}")))?
                    {
                        return Err(refused(
                            g,
                            "contour separation below Gerber robustness requirement; required output remains pending",
                        ));
                    }
                }
            }
        }
        let rings: Vec<_> = std::iter::once(outer)
            .chain(holes.iter().map(Vec::as_slice))
            .collect();
        for (i, ring) in rings.iter().enumerate() {
            for other in &rings[i + 1..] {
                for j in 0..ring.len() {
                    for k in 0..other.len() {
                        use crate::board::{
                            nominal_geometry::DistanceBoundary, track_contact::tracks_within,
                        };
                        let a = Track::straight(
                            g.uuid,
                            uuid::Uuid::nil(),
                            ring[j],
                            ring[(j + 1) % ring.len()],
                            1,
                            g.layer,
                        );
                        let b = Track::straight(
                            g.uuid,
                            uuid::Uuid::nil(),
                            other[k],
                            other[(k + 1) % other.len()],
                            1,
                            g.layer,
                        );
                        if tracks_within(&a, &b, 999, DistanceBoundary::Strict)
                            .map_err(|e| refused(g, format!("contour certificate failed: {e:?}")))?
                        {
                            return Err(refused(
                                g,
                                "distinct contour separation below Gerber robustness requirement; required output remains pending",
                            ));
                        }
                    }
                }
            }
        }
        if holes.is_empty() {
            lines.push("G36*".into());
            path(g, outer, true, lines)?;
            lines.push("G37*".into());
        } else {
            // Exposure 0 clears only the aperture image, preserving overlapping
            // independent objects. Multiple region contours instead form a union.
            let name = format!("datumGraphic{}", g.uuid.simple());
            lines.push(format!("%AM{name}*"));
            for (i, ring) in std::iter::once(outer)
                .chain(holes.iter().map(Vec::as_slice))
                .enumerate()
            {
                if ring.len() > 5000 {
                    return Err(refused(
                        g,
                        "Gerber outline macro vertex limit; required output remains pending",
                    ));
                }
                let mut fields = vec![
                    "4".into(),
                    if i == 0 { "1" } else { "0" }.into(),
                    ring.len().to_string(),
                ];
                for p in ring.iter().chain(std::iter::once(&ring[0])) {
                    fields.push(format_mm_6(p.x));
                    fields.push(format_mm_6(p.y));
                }
                fields.push("0".into());
                lines.push(format!("{}*", fields.join(",")));
            }
            lines.push("%".into());
            let current = *code;
            *code = code
                .checked_add(1)
                .ok_or_else(|| refused(g, "aperture namespace exhausted"))?;
            lines.push(format!("%ADD{current}{name}*%"));
            lines.push(format!("D{current}*"));
            lines.push("X0Y0D03*".into());
        }
    }
    if g.width > 0 {
        let d = aperture(g, code, g.width, None, lines)?;
        lines.push(format!("D{d}*"));
        for ring in std::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
            path(g, ring, true, lines)?;
        }
    }
    Ok(())
}
fn graphic(g: &BoardGraphic, code: &mut usize, lines: &mut Vec<String>) -> Result<()> {
    g.validate()
        .map_err(|e| refused(g, format!("invalid source geometry: {e:?}")))?;
    lines.push(format!("G04 BoardGraphic {}*", g.uuid));
    lines.push("%LPD*%".into());
    match &g.geometry {
        G::Line { from, to } => {
            let d = aperture(g, code, g.width, None, lines)?;
            lines.push(format!("D{d}*"));
            path(g, &[*from, *to], false, lines)?;
        }
        G::Polyline { points } => {
            let d = aperture(g, code, g.width, None, lines)?;
            lines.push(format!("D{d}*"));
            path(g, points, false, lines)?;
        }
        G::Arc { from, midpoint, to } => {
            let source = Track {
                uuid: g.uuid,
                net: uuid::Uuid::nil(),
                layer: g.layer,
                from: *from,
                to: *to,
                midpoint: Some(*midpoint),
                width: g.width,
            };
            let witness = arc_output_witness(&source)
                .map_err(|e| refused(g, format!("arc certificate failed: {e:?}")))?;
            if witness.disposition() != ArcOutputDisposition::Exact || !witness.endpoints_stable() {
                return Err(refused(
                    g,
                    format!(
                        "arc {:?}; exact rational-centre CAM/T04 remains pending",
                        witness.disposition()
                    ),
                ));
            }
            let d = aperture(g, code, g.width, None, lines)?;
            lines.push(format!("D{d}*"));
            lines.push("G75*".into());
            for p in witness.output_primitives() {
                checked_point(g, p.from)?;
                checked_point(g, p.to)?;
                lines.push(if p.counterclockwise { "G03*" } else { "G02*" }.into());
                lines.push(coordinate(p.from, 2));
                lines.push(format!(
                    "X{}Y{}I{}J{}D01*",
                    format_coord(p.to.x),
                    format_coord(p.to.y),
                    format_coord(p.center_offset.x),
                    format_coord(p.center_offset.y)
                ));
            }
            lines.push("G01*".into());
        }
        G::Rectangle { from, to, filled } => polygon(
            g,
            &[
                *from,
                Point::new(to.x, from.y),
                *to,
                Point::new(from.x, to.y),
            ],
            &[],
            *filled,
            code,
            lines,
        )?,
        G::Polygon {
            outer,
            holes,
            filled,
        } => polygon(g, outer, holes, *filled, code, lines)?,
        G::Circle {
            center,
            radius,
            filled,
        } => {
            checked_point(g, *center)?;
            for p in [
                Point::new(center.x + radius, center.y),
                Point::new(center.x - radius, center.y),
                Point::new(center.x, center.y + radius),
                Point::new(center.x, center.y - radius),
            ] {
                checked_point(g, p)?;
            }
            let diameter = radius
                .checked_mul(2)
                .and_then(|d| d.checked_add(g.width))
                .ok_or_else(|| refused(g, "circle aperture range"))?;
            let hole = if *filled {
                None
            } else {
                radius
                    .checked_mul(2)
                    .and_then(|d| d.checked_sub(g.width))
                    .filter(|h| *h > 0)
            };
            let d = aperture(g, code, diameter, hole, lines)?;
            lines.push(format!("D{d}*"));
            lines.push(coordinate(*center, 3));
        }
    }
    Ok(())
}

/// Add recorded graphics to a complete engine-produced layer stream. Refusal
/// occurs before any caller can replace an output artifact. Empty input preserves
/// the historical stream byte-for-byte; no geometry is sampled or rounded.
pub fn append_rs274x_board_graphics(
    base: &str,
    layer: i32,
    graphics: &[BoardGraphic],
) -> Result<String> {
    let mut selected: Vec<_> = graphics.iter().filter(|g| g.layer == layer).collect();
    if selected.is_empty() {
        return Ok(base.into());
    }
    selected.sort_by_key(|g| g.uuid);
    let mut seen = BTreeSet::new();
    for g in &selected {
        if !seen.insert(g.uuid) {
            return Err(refused(g, "duplicate source UUID"));
        }
    }
    let first = selected[0];
    let prefix = base
        .strip_suffix("M02*\n")
        .ok_or_else(|| refused(first, "incomplete engine layer stream"))?;
    if !prefix.contains("%FSLAX46Y46*%") || !prefix.contains("%MOMM*%") {
        return Err(refused(first, "unsupported engine layer profile"));
    }
    let mut code = 10;
    for line in prefix.lines() {
        if let Some(rest) = line.strip_prefix("%ADD") {
            let number: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let n = number
                .parse::<usize>()
                .map_err(|_| refused(first, "invalid existing aperture namespace"))?;
            code = code.max(
                n.checked_add(1)
                    .ok_or_else(|| refused(first, "aperture namespace exhausted"))?,
            );
        }
    }
    let mut lines = Vec::new();
    for g in selected {
        graphic(g, &mut code, &mut lines)?;
    }
    // Keep definitions in the header so the established translation-only
    // panel owner repeats geometry, never redeclares apertures per instance.
    let mut definitions = String::new();
    let mut body = Vec::new();
    let mut in_macro = false;
    for line in lines {
        if line.starts_with("%AM") {
            in_macro = true;
        }
        if in_macro || line.starts_with("%ADD") {
            definitions.push_str(&line);
            definitions.push('\n');
        } else {
            body.push(line.clone());
        }
        if in_macro && line == "%" {
            in_macro = false;
        }
    }
    let mut at = 0;
    for line in prefix.split_inclusive('\n') {
        if line.starts_with('D') || line.starts_with('X') || line == "G36*\n" {
            break;
        }
        at += line.len();
    }
    Ok(format!(
        "{}{definitions}{}{}\nM02*\n",
        &prefix[..at],
        &prefix[at..],
        body.join("\n")
    ))
}

#[cfg(test)]
mod tests;
