pub(crate) use super::command_project_gerber_semantics_utils::render_mm_6;
use super::*;
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
    Circle { diameter_nm: i64 },
    Rect { width_nm: i64, height_nm: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedGerberGeometry {
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

pub(crate) fn parse_rs274x_subset(gerber: &str) -> Result<ParsedGerber> {
    let mut aperture_map = BTreeMap::<usize, ParsedGerberAperture>::new();
    let mut current_aperture = None;
    let mut current_position = None;
    let mut pending_stroke = None::<PendingStroke>;
    let mut in_region = false;
    let mut region_points = Vec::<Point>::new();
    let mut geometries = Vec::new();

    for raw_line in gerber.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("G04") {
            continue;
        }
        if line == "%FSLAX46Y46*%" || line == "%MOMM*%" || line == "%LPD*%" {
            continue;
        }
        if let Some(rest) = line.strip_prefix("%ADD") {
            let split_idx = rest
                .find(|ch: char| ch.is_ascii_alphabetic())
                .context("unsupported Gerber aperture definition in comparison input")?;
            let (code_str, definition) = rest.split_at(split_idx);
            let (kind, params) = definition.split_at(1);
            let params = params
                .strip_prefix(',')
                .context("unsupported Gerber aperture definition in comparison input")?
                .strip_suffix("*%")
                .context("unterminated Gerber aperture definition in comparison input")?;
            let code = code_str
                .parse::<usize>()
                .context("invalid Gerber aperture code in comparison input")?;
            let aperture = match kind {
                "C" => ParsedGerberAperture::Circle {
                    diameter_nm: parse_mm_6_to_nm(params)
                        .context("invalid Gerber circular aperture diameter in comparison input")?,
                },
                "R" => {
                    let (width_str, height_str) = params.split_once('X').context(
                        "invalid Gerber rectangular aperture definition in comparison input",
                    )?;
                    ParsedGerberAperture::Rect {
                        width_nm: parse_mm_6_to_nm(width_str).context(
                            "invalid Gerber rectangular aperture width in comparison input",
                        )?,
                        height_nm: parse_mm_6_to_nm(height_str).context(
                            "invalid Gerber rectangular aperture height in comparison input",
                        )?,
                    }
                }
                _ => bail!("unsupported Gerber aperture definition in comparison input: {line}"),
            };
            aperture_map.insert(code, aperture);
            continue;
        }
        if line == "G36*" {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            if in_region {
                bail!("nested Gerber regions are not supported by semantic comparison");
            }
            in_region = true;
            region_points.clear();
            continue;
        }
        if line == "G37*" {
            if !in_region {
                bail!("Gerber region end encountered without region start");
            }
            in_region = false;
            if region_points.len() >= 2 {
                geometries.push(ParsedGerberGeometry::Region {
                    points: canonicalize_path_points(&region_points),
                });
            }
            region_points.clear();
            continue;
        }
        if let Some(code) = parse_aperture_select(line)? {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            current_aperture = Some(code);
            continue;
        }
        if line == "M02*" {
            break;
        }

        let Some((point, operation)) = parse_gerber_coordinate_operation(line)? else {
            bail!("unsupported Gerber command in semantic comparison input: {line}");
        };
        let previous_position = current_position;
        current_position = Some(point);

        if in_region {
            match operation {
                2 => {
                    region_points.clear();
                    region_points.push(point);
                }
                1 => {
                    if region_points.is_empty() {
                        region_points.push(point);
                    } else {
                        region_points.push(point);
                    }
                }
                3 => bail!("Gerber flashes inside regions are not supported"),
                _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
            }
            continue;
        }

        let aperture_code = current_aperture.context(
            "Gerber semantic comparison requires an active circular aperture before geometry",
        )?;
        let aperture = aperture_map.get(&aperture_code).with_context(|| {
            format!("unknown Gerber aperture D{aperture_code} in comparison input")
        })?;

        match operation {
            2 => {
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    ParsedGerberAperture::Rect { .. } => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                pending_stroke = Some(PendingStroke {
                    aperture_diameter_nm,
                    points: vec![point],
                });
            }
            1 => {
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    ParsedGerberAperture::Rect { .. } => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                let stroke = pending_stroke.get_or_insert_with(|| PendingStroke {
                    aperture_diameter_nm,
                    points: previous_position.into_iter().collect::<Vec<_>>(),
                });
                if stroke.aperture_diameter_nm != aperture_diameter_nm {
                    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                    pending_stroke = Some(PendingStroke {
                        aperture_diameter_nm,
                        points: previous_position
                            .into_iter()
                            .chain(std::iter::once(point))
                            .collect::<Vec<_>>(),
                    });
                } else {
                    stroke.points.push(point);
                }
            }
            3 => {
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                geometries.push(ParsedGerberGeometry::Flash {
                    aperture: aperture.clone(),
                    position: point,
                });
            }
            _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
        }
    }

    if in_region {
        bail!("unterminated Gerber region in comparison input");
    }
    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
    Ok(ParsedGerber { geometries })
}
fn finalize_pending_stroke(
    pending_stroke: &mut Option<PendingStroke>,
    geometries: &mut Vec<ParsedGerberGeometry>,
) {
    if let Some(stroke) = pending_stroke.take() {
        if stroke.points.len() >= 2 {
            geometries.push(ParsedGerberGeometry::Stroke {
                aperture_diameter_nm: stroke.aperture_diameter_nm,
                points: canonicalize_path_points(&stroke.points),
            });
        }
    }
}
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
fn parse_aperture_select(line: &str) -> Result<Option<usize>> {
    if !line.starts_with('D') || !line.ends_with('*') {
        return Ok(None);
    }
    let code = line[1..line.len() - 1]
        .parse::<usize>()
        .context("invalid Gerber aperture selection in comparison input")?;
    if code < 10 {
        return Ok(None);
    }
    Ok(Some(code))
}
fn parse_gerber_coordinate_operation(line: &str) -> Result<Option<(Point, u8)>> {
    let Some(rest) = line.strip_prefix('X') else {
        return Ok(None);
    };
    let (x_str, rest) = rest
        .split_once('Y')
        .context("invalid Gerber coordinate command in comparison input")?;
    let (y_str, d_str) = rest
        .split_once('D')
        .context("invalid Gerber coordinate command in comparison input")?;
    let operation = d_str
        .strip_suffix('*')
        .context("unterminated Gerber coordinate command in comparison input")?
        .parse::<u8>()
        .context("invalid Gerber coordinate operation in comparison input")?;
    Ok(Some((
        Point {
            x: x_str
                .parse::<i64>()
                .context("invalid Gerber X coordinate in comparison input")?,
            y: y_str
                .parse::<i64>()
                .context("invalid Gerber Y coordinate in comparison input")?,
        },
        operation,
    )))
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
    root: &Path,
    layer: i32,
) -> Result<(StackupLayer, i32, Vec<PlacedPad>)> {
    let stackup = query_native_project_board_stackup(root)?;
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

    let pads = query_native_project_emitted_copper_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == associated_copper_layer)
        .collect::<Vec<_>>();

    Ok((mask_layer, associated_copper_layer, pads))
}
