use std::path::Path;

use crate::error::EngineError;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::ir::geometry::{Point, Polygon};
use crate::pool::{Pad, Padstack, PadstackAperture, Primitive};
use crate::schematic::PinElectricalType;
use crate::substrate::{ImportKey, ImportMapEntry};

mod board_objects;
mod graphics;
pub use graphics::{
    GraphicImportDisposition, NativeGraphicImport, board_graphic_import_key, native_graphic_sources,
};
mod saved_fill_geometry;
mod zones;
pub use zones::native_zone_sources;
mod footprint;
mod net_refs;
mod numeric;
mod pad_expansion;
mod parser_helpers;
mod schematic_bus;
mod schematic_graphics;
mod schematic_identity;
mod schematic_import;
mod skeleton;
mod symbol_helpers;

use parser_helpers::*;
mod board_document;

// KiCad importer — see specs/IMPORT_SPEC.md §3
pub use board_objects::{
    KiCadBoardImportIdentity, KiCadSchematicImportIdentity, board_footprint_import_key,
    board_pad_import_key, board_segment_import_key, board_via_import_key, board_zone_import_key,
};
pub use footprint::{
    ImportedKiCadFootprint, footprint_package_import_key, import_footprint_document,
    import_footprint_document_with_import_map,
};
pub use schematic_identity::{
    schematic_bus_entry_import_key, schematic_bus_import_key, schematic_drawing_import_key,
    schematic_junction_import_key, schematic_label_import_key, schematic_noconnect_import_key,
    schematic_sheet_definition_import_key, schematic_sheet_instance_import_key,
    schematic_sheet_port_import_key, schematic_symbol_import_key, schematic_text_import_key,
    schematic_wire_import_key,
};
pub use schematic_import::{
    import_schematic_document, import_schematic_document_with_import_map_identities,
    import_schematic_file,
};

pub use board_document::{
    import_board_document, import_board_document_with_import_map,
    import_board_document_with_import_map_identities, import_board_file,
};

pub fn import_project_file(path: &Path) -> Result<ImportReport, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&contents).map_err(|err| {
        EngineError::Import(format!(
            "failed to parse KiCad project JSON {}: {err}",
            path.display()
        ))
    })?;

    let mut report = ImportReport::new(
        ImportKind::KiCadProject,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad project metadata only; board and schematic import are not implemented yet",
    );

    if let Some(meta) = value.get("meta").and_then(|v| v.as_object()) {
        if let Some(filename) = meta.get("filename").and_then(|v| v.as_str()) {
            report = report.with_metadata("project_name", filename);
        }
        if let Some(version) = meta.get("version") {
            report = report.with_metadata("project_version", version.to_string());
        }
    }

    if !report.metadata.contains_key("project_name")
        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
    {
        report = report.with_metadata("project_name", stem);
    }

    Ok(report)
}

fn footprint_name(contents: &str) -> Option<String> {
    let first = contents.lines().next()?.trim_start();
    if !first.starts_with("(footprint ") {
        return None;
    }
    parse_quoted_token(first)
}

fn import_footprint_graphics(
    path: &Path,
    contents: &str,
) -> Result<(Vec<Primitive>, Vec<Primitive>), EngineError> {
    let mut silkscreen = Vec::new();
    let mut mechanical = Vec::new();

    for (form, parser) in [
        (
            "fp_line",
            parse_primitive_line as fn(&str) -> Option<Primitive>,
        ),
        ("fp_rect", parse_primitive_rect),
        ("fp_circle", parse_primitive_circle),
        ("fp_poly", parse_primitive_polygon),
        ("fp_arc", parse_primitive_arc),
    ] {
        for block in nested_blocks(contents, form) {
            let Some(layer) = block_layer_name_anywhere(&block) else {
                continue;
            };
            let Some(primitive) = parser(&block) else {
                continue;
            };
            match layer.as_str() {
                "F.SilkS" | "B.SilkS" => silkscreen.push(primitive),
                "F.CrtYd" | "B.CrtYd" | "F.Fab" | "B.Fab" => mechanical.push(primitive),
                _ => {}
            }
        }
    }

    for block in nested_blocks(contents, "property") {
        let first = block.lines().next().unwrap_or("").trim_start();
        if !first.starts_with("(property \"Reference\"")
            && !first.starts_with("(property \"Value\"")
        {
            continue;
        }
        let Some(layer) = block_layer_name_anywhere(&block) else {
            continue;
        };
        let Some(position) = block
            .lines()
            .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "at"))
        else {
            continue;
        };
        let text = block
            .lines()
            .next()
            .map(|line| quoted_tokens(line.trim_start()))
            .and_then(|tokens| tokens.get(1).cloned())
            .unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        let primitive = Primitive::Text {
            text,
            position,
            rotation: block_rotation(&block).unwrap_or(0),
        };
        match layer.as_str() {
            "F.SilkS" | "B.SilkS" => silkscreen.push(primitive),
            "F.CrtYd" | "B.CrtYd" | "F.Fab" | "B.Fab" => mechanical.push(primitive),
            _ => {}
        }
    }

    if silkscreen.is_empty() && mechanical.is_empty() {
        return Err(EngineError::Import(format!(
            "KiCad footprint {} contained no supported graphics",
            path.display()
        )));
    }

    Ok((silkscreen, mechanical))
}

fn import_footprint_pads(
    path: &Path,
    contents: &str,
) -> Result<(std::collections::HashMap<uuid::Uuid, Pad>, Vec<Padstack>), EngineError> {
    let mut pads = std::collections::HashMap::new();
    let mut padstacks = Vec::new();
    for (index, block) in nested_blocks(contents, "pad").into_iter().enumerate() {
        let pad_name = parse_pad_name(&block).unwrap_or_else(|| format!("P{}", index + 1));
        let position = block
            .lines()
            .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "at"))
            .unwrap_or(Point::zero());
        let (width_nm, height_nm) = parse_pad_size(&block).ok_or_else(|| {
            EngineError::Import(format!(
                "KiCad footprint {} pad {pad_name} missing size",
                path.display()
            ))
        })?;
        let drill_nm = parse_pad_drill(&block);
        let padstack_uuid = crate::ir::ids::import_uuid(
            &crate::ir::ids::namespace_kicad(),
            &format!("footprint-padstack/{}/{}", path.display(), pad_name),
        );
        let pad_uuid = crate::ir::ids::import_uuid(
            &crate::ir::ids::namespace_kicad(),
            &format!("footprint-pad/{}/{}", path.display(), pad_name),
        );
        let aperture = Some(match parse_pad_shape(&block).as_deref() {
            Some("circle") => PadstackAperture::Circle {
                diameter_nm: width_nm.min(height_nm),
            },
            _ => PadstackAperture::Rect {
                width_nm,
                height_nm,
            },
        });
        padstacks.push(Padstack {
            uuid: padstack_uuid,
            name: format!(
                "{}_{}",
                path.file_stem().and_then(|s| s.to_str()).unwrap_or("pad"),
                pad_name
            ),
            aperture,
            drill_nm,
            plated: drill_nm.map(|_| true),
            layer_span: Default::default(),
            mask_policy: Default::default(),
            paste_policy: Default::default(),
            annular_ring_nm: None,
            thermal: None,
            antipad: None,
        });
        pads.insert(
            pad_uuid,
            Pad {
                uuid: pad_uuid,
                name: pad_name,
                position,
                padstack: padstack_uuid,
                layer: parse_pad_copper_layer(&block).unwrap_or(1),
            },
        );
    }
    Ok((pads, padstacks))
}

fn import_footprint_courtyard(mechanical: &[Primitive], silkscreen: &[Primitive]) -> Polygon {
    primitive_bounds(mechanical)
        .or_else(|| primitive_bounds(silkscreen))
        .map(|(min, max)| {
            Polygon::new(vec![
                min,
                Point::new(max.x, min.y),
                max,
                Point::new(min.x, max.y),
            ])
        })
        .unwrap_or_else(|| Polygon::new(Vec::new()))
}

fn parse_primitive_line(block: &str) -> Option<Primitive> {
    let (from, to) = block_start_end_points_anywhere(block)?;
    Some(Primitive::Line {
        from,
        to,
        width: block_width_nm_import(block).unwrap_or(120_000),
    })
}

fn parse_primitive_rect(block: &str) -> Option<Primitive> {
    let (start, end) = block_start_end_points_anywhere(block)?;
    Some(Primitive::Rect {
        min: Point::new(start.x.min(end.x), start.y.min(end.y)),
        max: Point::new(start.x.max(end.x), start.y.max(end.y)),
        width: block_width_nm_import(block).unwrap_or(120_000),
    })
}

fn parse_primitive_circle(block: &str) -> Option<Primitive> {
    let center = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "center"))?;
    let end = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "end"))?;
    let dx = end.x - center.x;
    let dy = end.y - center.y;
    Some(Primitive::Circle {
        center,
        radius: ((dx * dx + dy * dy) as f64).sqrt().round() as i64,
        width: block_width_nm_import(block).unwrap_or(120_000),
    })
}

fn parse_primitive_polygon(block: &str) -> Option<Primitive> {
    let vertices = block_xy_points(block);
    if vertices.is_empty() {
        return None;
    }
    Some(Primitive::Polygon {
        polygon: Polygon::new(vertices),
        width: block_width_nm_import(block).unwrap_or(120_000),
    })
}

fn parse_primitive_arc(block: &str) -> Option<Primitive> {
    let start = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "start"))?;
    let mid = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "mid"))?;
    let end = block
        .lines()
        .find_map(|line| parse_xy_like_anywhere(line.trim_start(), "end"))?;
    let (center, radius, start_angle, end_angle) = arc_from_three_points(start, mid, end)?;
    Some(Primitive::Arc {
        arc: crate::ir::geometry::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        },
        width: block_width_nm_import(block).unwrap_or(120_000),
    })
}

fn parse_pad_name(block: &str) -> Option<String> {
    let first = block.lines().next()?.trim_start();
    let prefix = "(pad ";
    if !first.starts_with(prefix) {
        return None;
    }
    let rest = &first[prefix.len()..];
    parse_quoted_token(rest)
}

fn parse_pad_shape(block: &str) -> Option<String> {
    let first = block.lines().next()?.trim_start();
    let tokens: Vec<&str> = first.trim_end_matches(')').split_whitespace().collect();
    if tokens.len() < 4 {
        return None;
    }
    Some(tokens[3].trim_matches('"').to_string())
}

fn parse_pad_size(block: &str) -> Option<(i64, i64)> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let start = trimmed.find("(size ")? + "(size ".len();
        let rest = &trimmed[start..];
        let end = rest.find(')').unwrap_or(rest.len());
        let rest = &rest[..end];
        let mut parts = rest.split_whitespace();
        let x = parts.next()?.parse::<f64>().ok()?;
        let y = parts.next()?.parse::<f64>().ok()?;
        Some((checked_mm_to_nm_import(x)?, checked_mm_to_nm_import(y)?))
    })
}

fn parse_pad_drill(block: &str) -> Option<i64> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let start = trimmed.find("(drill ")? + "(drill ".len();
        let rest = &trimmed[start..];
        let end = rest.find(')').unwrap_or(rest.len());
        let rest = &rest[..end];
        let first = rest.split_whitespace().next()?.parse::<f64>().ok()?;
        checked_mm_to_nm_import(first)
    })
}

fn parse_pad_copper_layer(block: &str) -> Option<i32> {
    let first = block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(layers ") {
            return None;
        }
        Some(quoted_tokens(trimmed))
    })?;
    if first.iter().any(|layer| layer == "F.Cu") {
        Some(1)
    } else if first.iter().any(|layer| layer == "B.Cu") {
        Some(31)
    } else {
        None
    }
}

fn primitive_bounds(primitives: &[Primitive]) -> Option<(Point, Point)> {
    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;
    let mut saw = false;
    for primitive in primitives {
        let (local_min, local_max) = match primitive {
            Primitive::Line { from, to, .. } => (
                Point::new(from.x.min(to.x), from.y.min(to.y)),
                Point::new(from.x.max(to.x), from.y.max(to.y)),
            ),
            Primitive::Rect { min, max, .. } => (*min, *max),
            Primitive::Circle { center, radius, .. } => (
                Point::new(center.x - radius, center.y - radius),
                Point::new(center.x + radius, center.y + radius),
            ),
            Primitive::Polygon { polygon, .. } => {
                polygon.bounding_box().map(|bbox| (bbox.min, bbox.max))?
            }
            Primitive::Arc { arc, .. } => (
                Point::new(arc.center.x - arc.radius, arc.center.y - arc.radius),
                Point::new(arc.center.x + arc.radius, arc.center.y + arc.radius),
            ),
            Primitive::Text { position, .. } => (
                Point::new(position.x - 500_000, position.y - 500_000),
                Point::new(position.x + 500_000, position.y + 500_000),
            ),
        };
        min_x = min_x.min(local_min.x);
        min_y = min_y.min(local_min.y);
        max_x = max_x.max(local_max.x);
        max_y = max_y.max(local_max.y);
        saw = true;
    }
    saw.then_some((Point::new(min_x, min_y), Point::new(max_x, max_y)))
}

fn arc_from_three_points(start: Point, mid: Point, end: Point) -> Option<(Point, i64, i32, i32)> {
    let x1 = start.x as f64;
    let y1 = start.y as f64;
    let x2 = mid.x as f64;
    let y2 = mid.y as f64;
    let x3 = end.x as f64;
    let y3 = end.y as f64;
    let d = 2.0 * (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2));
    if d.abs() < f64::EPSILON {
        return None;
    }
    let ux = ((x1 * x1 + y1 * y1) * (y2 - y3)
        + (x2 * x2 + y2 * y2) * (y3 - y1)
        + (x3 * x3 + y3 * y3) * (y1 - y2))
        / d;
    let uy = ((x1 * x1 + y1 * y1) * (x3 - x2)
        + (x2 * x2 + y2 * y2) * (x1 - x3)
        + (x3 * x3 + y3 * y3) * (x2 - x1))
        / d;
    let center = Point::new(ux.round() as i64, uy.round() as i64);
    let radius = (((x1 - ux).powi(2) + (y1 - uy).powi(2)).sqrt()).round() as i64;
    let start_angle =
        (((y1 - uy).atan2(x1 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    let end_angle =
        (((y3 - uy).atan2(x3 - ux).to_degrees() * 10.0).round() as i32).rem_euclid(3600);
    Some((center, radius, start_angle, end_angle))
}

fn checked_mm_to_nm_import(mm: f64) -> Option<i64> {
    crate::ir::units::checked_f64_length(mm, crate::ir::units::LengthUnit::Millimeter)
        .ok()
        .map(|value| value.get())
}

fn block_width_nm_import(block: &str) -> Option<i64> {
    block_width_mm(block).and_then(checked_mm_to_nm_import)
}

fn block_layer_name_anywhere(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let start = trimmed.find("(layer ")? + "(layer ".len();
        parse_quoted_token(&trimmed[start..]).map(|name| canonicalize_kicad_layer_name(&name))
    })
}

fn parse_xy_like_anywhere(trimmed: &str, form: &str) -> Option<Point> {
    let marker = format!("({form} ");
    let start = trimmed.find(&marker)? + marker.len();
    let rest = &trimmed[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(Point::new(
        checked_mm_to_nm_import(x)?,
        checked_mm_to_nm_import(y)?,
    ))
}

fn block_start_end_points_anywhere(block: &str) -> Option<(Point, Point)> {
    let mut start = None;
    let mut end = None;
    for line in block.lines() {
        let trimmed = line.trim_start();
        if start.is_none() {
            start = parse_xy_like_anywhere(trimmed, "start");
        }
        if end.is_none() {
            end = parse_xy_like_anywhere(trimmed, "end");
        }
    }
    Some((start?, end?))
}

fn extract_kicad_board_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

#[derive(Debug, Clone)]
pub(super) struct LibraryPinTemplate {
    pub(super) number: String,
    pub(super) name: String,
    pub(super) electrical_type: PinElectricalType,
    pub(super) position: Point,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/import/kicad")
            .join(name)
    }

    fn optional_doa2526_board_path() -> Option<std::path::PathBuf> {
        std::env::var_os("DATUM_RUN_EXTERNAL_DOA2526_TESTS")?;
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb");
        path.exists().then_some(path)
    }

    fn optional_doa2526_schematic_path() -> Option<std::path::PathBuf> {
        std::env::var_os("DATUM_RUN_EXTERNAL_DOA2526_TESTS")?;
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch");
        path.exists().then_some(path)
    }

    fn optional_datum_test_board_path() -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb");
        path.exists().then_some(path)
    }

    #[path = "mod_tests_import_kicad_basics.rs"]
    mod import_kicad_basics;
    #[path = "mod_tests_import_kicad_doa2526.rs"]
    mod import_kicad_doa2526;
    #[path = "mod_tests_import_kicad_footprint.rs"]
    mod import_kicad_footprint;
    #[path = "mod_tests_import_kicad_import_map.rs"]
    mod import_kicad_import_map;
    #[path = "mod_tests_import_kicad_layers.rs"]
    mod import_kicad_layers;
    #[path = "mod_tests_import_kicad_pad_fallbacks.rs"]
    mod import_kicad_pad_fallbacks;
    #[path = "mod_tests_import_kicad_pad_layers.rs"]
    mod import_kicad_pad_layers;
    #[path = "mod_tests_import_kicad_pad_rotation.rs"]
    mod import_kicad_pad_rotation;
    #[path = "mod_tests_import_kicad_schematic.rs"]
    mod import_kicad_schematic;
}
