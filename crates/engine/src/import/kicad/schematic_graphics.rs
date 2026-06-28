use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use uuid::Uuid;

use crate::ir::geometry::{Arc, Point};
use crate::schematic::{SchematicPrimitive, SchematicText};
use crate::substrate::{ImportKey, ImportMapEntry};

use super::board_objects::KiCadSchematicImportIdentity;
use super::parser_helpers::*;
use super::schematic_identity::schematic_import_id;
use super::symbol_helpers::mm_point_to_nm;

pub(super) fn parse_schematic_texts(
    path: &Path,
    contents: &str,
    import_map: Option<&BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadSchematicImportIdentity>>,
) -> HashMap<Uuid, SchematicText> {
    let mut texts = HashMap::new();
    let mut import_identities = import_identities;
    for block in top_level_blocks(contents, "text") {
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "text",
            "schematic_text",
            import_map,
            import_identities.as_deref_mut(),
        );
        let Some(text) = block_head_string(&block, "text") else {
            continue;
        };
        let Some(position) = block_at_point(&block) else {
            continue;
        };
        texts.insert(
            uuid,
            SchematicText {
                uuid,
                text,
                position,
                rotation: block_rotation(&block).unwrap_or(0),
            },
        );
    }
    texts
}

pub(super) fn parse_schematic_drawings(
    path: &Path,
    contents: &str,
    import_map: Option<&BTreeMap<ImportKey, ImportMapEntry>>,
    import_identities: Option<&mut Vec<KiCadSchematicImportIdentity>>,
) -> HashMap<Uuid, SchematicPrimitive> {
    let mut drawings = HashMap::new();
    let mut import_identities = import_identities;
    for block in top_level_blocks(contents, "polyline") {
        let points = block_xy_points(&block);
        for (index, pair) in points.windows(2).enumerate() {
            let source_uuid = indexed_or_block_uuid(&block, "polyline-segment", index);
            let uuid = schematic_import_id(
                path,
                source_uuid,
                "drawing",
                "schematic_drawing",
                import_map,
                import_identities.as_deref_mut(),
            );
            drawings.insert(
                uuid,
                SchematicPrimitive::Line {
                    uuid,
                    from: pair[0],
                    to: pair[1],
                },
            );
        }
    }
    for block in top_level_blocks(contents, "rectangle") {
        let Some((min, max)) = start_end_points(&block) else {
            continue;
        };
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "drawing",
            "schematic_drawing",
            import_map,
            import_identities.as_deref_mut(),
        );
        drawings.insert(uuid, SchematicPrimitive::Rect { uuid, min, max });
    }
    for block in top_level_blocks(contents, "circle") {
        let Some(center) = xy_like_anywhere(&block, "center") else {
            continue;
        };
        let Some(end) = xy_like_anywhere(&block, "end") else {
            continue;
        };
        let radius = distance_nm(center, end);
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "drawing",
            "schematic_drawing",
            import_map,
            import_identities.as_deref_mut(),
        );
        drawings.insert(
            uuid,
            SchematicPrimitive::Circle {
                uuid,
                center,
                radius,
            },
        );
    }
    for block in top_level_blocks(contents, "arc") {
        let Some(start) = xy_like_anywhere(&block, "start") else {
            continue;
        };
        let Some(mid) = xy_like_anywhere(&block, "mid") else {
            continue;
        };
        let Some(end) = xy_like_anywhere(&block, "end") else {
            continue;
        };
        let Some(arc) = arc_from_three_points(start, mid, end) else {
            continue;
        };
        let source_uuid = block_uuid(&block).unwrap_or_else(Uuid::new_v4);
        let uuid = schematic_import_id(
            path,
            source_uuid,
            "drawing",
            "schematic_drawing",
            import_map,
            import_identities.as_deref_mut(),
        );
        drawings.insert(uuid, SchematicPrimitive::Arc { uuid, arc });
    }
    drawings
}

fn indexed_or_block_uuid(block: &str, family: &str, index: usize) -> Uuid {
    block_uuid(block).map_or_else(
        || Uuid::new_v4(),
        |uuid| {
            crate::ir::ids::import_uuid(
                &crate::ir::ids::namespace_kicad(),
                &format!("schematic-{family}/{uuid}/{index}"),
            )
        },
    )
}

fn start_end_points(block: &str) -> Option<(Point, Point)> {
    let start = xy_like_anywhere(block, "start")?;
    let end = xy_like_anywhere(block, "end")?;
    Some((
        Point::new(start.x.min(end.x), start.y.min(end.y)),
        Point::new(start.x.max(end.x), start.y.max(end.y)),
    ))
}

fn xy_like_anywhere(block: &str, form: &str) -> Option<Point> {
    block
        .lines()
        .find_map(|line| parse_xy_like(line.trim_start(), form))
}

fn parse_xy_like(trimmed: &str, form: &str) -> Option<Point> {
    let marker = format!("({form} ");
    let start = trimmed.find(&marker)? + marker.len();
    let rest = &trimmed[start..];
    let end = rest.find(')').unwrap_or(rest.len());
    let mut parts = rest[..end].split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(mm_point_to_nm(x, y))
}

fn distance_nm(a: Point, b: Point) -> i64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    ((dx * dx + dy * dy) as f64).sqrt().round() as i64
}

fn arc_from_three_points(start: Point, mid: Point, end: Point) -> Option<Arc> {
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
    Some(Arc {
        center,
        radius,
        start_angle,
        end_angle,
    })
}
