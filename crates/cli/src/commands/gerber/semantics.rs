#[path = "graphic_semantics.rs"]
pub(super) mod graphic_semantics;
#[path = "semantics_parser.rs"]
mod parser;
pub(crate) use super::semantics_utils::render_mm_6;
use crate::*;
pub(crate) use parser::parse_rs274x_subset;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) fn classify_via_hole_class(
    start: i32,
    end: i32,
    top_copper: Option<i32>,
    bottom_copper: Option<i32>,
) -> String {
    match (top_copper, bottom_copper) {
        (Some(top), Some(bottom)) if start == top && end == bottom => "through".to_string(),
        (Some(top), Some(bottom)) if start == top || end == bottom => "blind".to_string(),
        (Some(_), Some(_)) => "buried".to_string(),
        _ => "unclassified".to_string(),
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedGerberAperture {
    Circle {
        diameter_nm: i64,
    },
    Rect {
        width_nm: i64,
        height_nm: i64,
    },
    Ring {
        diameter_nm: i64,
        hole_nm: i64,
    },
    Polygon {
        outer: Vec<Point>,
        holes: Vec<Vec<Point>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedGerberGeometry {
    Arc {
        width_nm: i64,
        from: Point,
        to: Point,
        center_offset: Point,
        counterclockwise: bool,
    },
    Stroke {
        aperture_diameter_nm: i64,
        points: Vec<Point>,
    },
    Flash {
        aperture: ParsedGerberAperture,
        position: Point,
    },
    Region {
        points: Vec<Point>,
    },
}
#[derive(Debug)]
pub(crate) struct ParsedGerber {
    pub(crate) geometries: Vec<ParsedGerberGeometry>,
}
#[derive(Debug, Default)]
struct PendingStroke {
    aperture_diameter_nm: i64,
    points: Vec<Point>,
}
pub(crate) fn compare_entry_views(
    expected: BTreeMap<(String, String), usize>,
    actual: BTreeMap<(String, String), usize>,
) -> (
    usize,
    usize,
    usize,
    Vec<NativeProjectGerberGeometryEntryView>,
    Vec<NativeProjectGerberGeometryEntryView>,
    Vec<NativeProjectGerberGeometryEntryView>,
) {
    let keys = expected
        .keys()
        .chain(actual.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut matched_count = 0;
    let mut missing_count = 0;
    let mut extra_count = 0;
    let mut matched = Vec::new();
    let mut missing = Vec::new();
    let mut extra = Vec::new();

    for (kind, geometry) in keys {
        let key = (kind.clone(), geometry.clone());
        let expected_count = *expected.get(&key).unwrap_or(&0);
        let actual_count = *actual.get(&key).unwrap_or(&0);
        let matched_instances = expected_count.min(actual_count);
        let missing_instances = expected_count.saturating_sub(actual_count);
        let extra_instances = actual_count.saturating_sub(expected_count);

        if matched_instances > 0 {
            matched_count += matched_instances;
            matched.push(NativeProjectGerberGeometryEntryView {
                kind: kind.clone(),
                geometry: geometry.clone(),
                count: matched_instances,
            });
        }
        if missing_instances > 0 {
            missing_count += missing_instances;
            missing.push(NativeProjectGerberGeometryEntryView {
                kind: kind.clone(),
                geometry: geometry.clone(),
                count: missing_instances,
            });
        }
        if extra_instances > 0 {
            extra_count += extra_instances;
            extra.push(NativeProjectGerberGeometryEntryView {
                kind,
                geometry,
                count: extra_instances,
            });
        }
    }

    (
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    )
}
pub(crate) fn gerber_outline_expected_entries(
    outline: &Polygon,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    let outline_points = if outline.closed {
        canonicalize_closed_loop(&outline.vertices)
    } else {
        canonicalize_open_path(&outline.vertices)
    };
    entries.insert(
        (
            "outline".to_string(),
            render_stroke_geometry(DEFAULT_GERBER_OUTLINE_APERTURE_NM, &outline_points),
        ),
        1,
    );
    entries
}
pub(crate) fn gerber_outline_actual_entries(
    gerber: &ParsedGerber,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "outline".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Arc { .. } => (
                "arc".to_string(),
                graphic_semantics::arc_signature(geometry),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}
pub(crate) fn gerber_copper_expected_entries(
    pads: &[PlacedPad],
    tracks: &[Track],
    zones: &[Zone],
    vias: &[Via],
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for pad in pads {
        *entries
            .entry(("pad".to_string(), render_pad_flash_geometry(pad)))
            .or_insert(0) += 1;
    }
    for track in tracks {
        *entries
            .entry((
                "track".to_string(),
                render_stroke_geometry(track.width, &[track.from, track.to]),
            ))
            .or_insert(0) += 1;
    }
    for zone in zones {
        *entries
            .entry((
                "zone".to_string(),
                render_region_geometry(&zone.polygon.vertices),
            ))
            .or_insert(0) += 1;
    }
    for via in vias {
        *entries
            .entry((
                "via".to_string(),
                render_circular_flash_geometry(via.diameter, &via.position),
            ))
            .or_insert(0) += 1;
    }
    entries
}
pub(crate) fn gerber_copper_actual_entries(
    gerber: &ParsedGerber,
    expected_pads: &BTreeSet<String>,
    expected_vias: &BTreeSet<String>,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "track".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => {
                let signature = render_parsed_flash_geometry(aperture, position);
                let kind = if expected_pads.contains(&signature) {
                    "pad"
                } else if expected_vias.contains(&signature) {
                    "via"
                } else {
                    "flash"
                };
                (kind.to_string(), signature)
            }
            ParsedGerberGeometry::Arc { .. } => (
                "arc".to_string(),
                graphic_semantics::arc_signature(geometry),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("zone".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}
pub(crate) fn gerber_soldermask_expected_entries(
    pads: &[PlacedPad],
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for pad in pads {
        *entries
            .entry(("pad".to_string(), render_pad_flash_geometry(pad)))
            .or_insert(0) += 1;
    }
    entries
}
pub(crate) fn gerber_soldermask_actual_entries(
    gerber: &ParsedGerber,
    expected_pads: &BTreeSet<String>,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "track".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => {
                let signature = render_parsed_flash_geometry(aperture, position);
                let kind = if expected_pads.contains(&signature) {
                    "pad"
                } else {
                    "flash"
                };
                (kind.to_string(), signature)
            }
            ParsedGerberGeometry::Arc { .. } => (
                "arc".to_string(),
                graphic_semantics::arc_signature(geometry),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}
pub(crate) fn gerber_silkscreen_expected_entries(
    gerber: &ParsedGerber,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "stroke".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Arc { .. } => (
                "arc".to_string(),
                graphic_semantics::arc_signature(geometry),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}
pub(crate) fn render_stroke_geometry(aperture_diameter_nm: i64, points: &[Point]) -> String {
    let points = canonicalize_path_points(points);
    format!(
        "aperture_mm={} points={}",
        render_mm_6(aperture_diameter_nm),
        render_point_path(&points)
    )
}
pub(crate) fn render_circular_flash_geometry(
    aperture_diameter_nm: i64,
    position: &Point,
) -> String {
    format!(
        "shape=circle diameter_mm={} at=({}, {})",
        render_mm_6(aperture_diameter_nm),
        position.x,
        position.y
    )
}
fn render_rect_flash_geometry(width_nm: i64, height_nm: i64, position: &Point) -> String {
    format!(
        "shape=rect width_mm={} height_mm={} at=({}, {})",
        render_mm_6(width_nm),
        render_mm_6(height_nm),
        position.x,
        position.y
    )
}
pub(crate) fn render_pad_flash_geometry(pad: &PlacedPad) -> String {
    match pad.aperture() {
        PadAperture::Circle { diameter_nm } => {
            render_circular_flash_geometry(diameter_nm, &pad.position)
        }
        PadAperture::Rect {
            width_nm,
            height_nm,
        } => render_rect_flash_geometry(width_nm, height_nm, &pad.position),
    }
}
pub(crate) fn render_parsed_flash_geometry(
    aperture: &ParsedGerberAperture,
    position: &Point,
) -> String {
    match aperture {
        ParsedGerberAperture::Ring {
            diameter_nm,
            hole_nm,
        } => format!(
            "shape=ring diameter_mm={} hole_mm={} at=({}, {})",
            render_mm_6(*diameter_nm),
            render_mm_6(*hole_nm),
            position.x,
            position.y
        ),
        ParsedGerberAperture::Polygon { outer, holes } => {
            graphic_semantics::polygon_signature(outer, holes, *position)
        }
        ParsedGerberAperture::Circle { diameter_nm } => {
            render_circular_flash_geometry(*diameter_nm, position)
        }
        ParsedGerberAperture::Rect {
            width_nm,
            height_nm,
        } => render_rect_flash_geometry(*width_nm, *height_nm, position),
    }
}
pub(crate) fn render_region_geometry(points: &[Point]) -> String {
    let points = canonicalize_path_points(points);
    format!("points={}", render_point_path(&points))
}
fn render_point_path(points: &[Point]) -> String {
    points
        .iter()
        .map(|point| format!("({}, {})", point.x, point.y))
        .collect::<Vec<_>>()
        .join(" -> ")
}
pub(crate) const DEFAULT_GERBER_OUTLINE_APERTURE_NM: i64 = 100_000;

fn canonicalize_path_points(points: &[Point]) -> Vec<Point> {
    let mut normalized = points.to_vec();
    if normalized.len() >= 2 && normalized.first() == normalized.last() {
        normalized.pop();
        return canonicalize_closed_loop(&normalized);
    }
    canonicalize_open_path(&normalized)
}
fn canonicalize_open_path(points: &[Point]) -> Vec<Point> {
    let reversed = points.iter().rev().copied().collect::<Vec<_>>();
    if point_path_cmp(points, &reversed).is_gt() {
        reversed
    } else {
        points.to_vec()
    }
}
fn canonicalize_closed_loop(points: &[Point]) -> Vec<Point> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut best = rotate_points(points, 0);
    for start in 1..points.len() {
        let candidate = rotate_points(points, start);
        if point_path_cmp(&candidate, &best).is_lt() {
            best = candidate;
        }
    }

    let reversed_points = points.iter().rev().copied().collect::<Vec<_>>();
    for start in 0..reversed_points.len() {
        let candidate = rotate_points(&reversed_points, start);
        if point_path_cmp(&candidate, &best).is_lt() {
            best = candidate;
        }
    }

    best
}
fn rotate_points(points: &[Point], start: usize) -> Vec<Point> {
    points[start..]
        .iter()
        .chain(points[..start].iter())
        .copied()
        .collect()
}
fn point_path_cmp(a: &[Point], b: &[Point]) -> std::cmp::Ordering {
    for (lhs, rhs) in a.iter().zip(b.iter()) {
        let ordering = point_cmp(lhs, rhs);
        if !ordering.is_eq() {
            return ordering;
        }
    }
    a.len().cmp(&b.len())
}
fn point_cmp(a: &Point, b: &Point) -> std::cmp::Ordering {
    a.x.cmp(&b.x).then_with(|| a.y.cmp(&b.y))
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
    let whole_nm = whole.checked_mul(1_000_000)?;
    if whole >= 0 {
        whole_nm.checked_add(frac)
    } else {
        whole_nm.checked_sub(frac)
    }
}
pub(crate) fn resolve_native_project_soldermask_context(
    project: &LoadedNativeProject,
    layer: i32,
) -> Result<(StackupLayer, i32)> {
    let stackup = project
        .board
        .stackup
        .layers
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()?;
    let mask_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(mask_layer.layer_type, StackupLayerType::SolderMask) {
        bail!("board stackup layer is not a soldermask layer: {layer}");
    }
    let associated_copper_layer = stackup
        .iter()
        .filter(|entry| matches!(entry.layer_type, StackupLayerType::Copper))
        .min_by(|a, b| {
            (i64::from((a.id - layer).abs()), a.id).cmp(&(i64::from((b.id - layer).abs()), b.id))
        })
        .map(|entry| entry.id)
        .ok_or_else(|| {
            anyhow::anyhow!("no copper layer available to derive soldermask openings")
        })?;
    Ok((mask_layer, associated_copper_layer))
}
