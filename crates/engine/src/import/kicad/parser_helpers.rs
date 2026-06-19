use uuid::Uuid;

use crate::board::{Stackup, StackupLayer, StackupLayerType};
use crate::error::EngineError;
use crate::ir::geometry::{Point, Polygon};

use super::symbol_helpers::mm_point_to_nm;

pub(super) fn count_top_level_form_lines(contents: &str, form: &str) -> usize {
    let prefix = format!("({form}");
    contents
        .lines()
        .filter(|line| {
            let indent = line.len() - line.trim_start().len();
            let trimmed = line.trim_start();
            indent <= 2
                && trimmed.starts_with(&prefix)
                && matches!(
                    trimmed.as_bytes().get(prefix.len()),
                    Some(b' ') | Some(b'\t') | Some(b')') | None
                )
        })
        .count()
}

pub(super) fn count_top_level_form_lines_by_form(
    contents: &str,
    forms: &[&str],
) -> std::collections::HashMap<String, usize> {
    let prefixes = forms
        .iter()
        .map(|form| ((*form).to_string(), format!("({form}")))
        .collect::<Vec<_>>();
    let mut counts = forms
        .iter()
        .map(|form| ((*form).to_string(), 0_usize))
        .collect::<std::collections::HashMap<_, _>>();

    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        if indent > 2 {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some((form, _)) = prefixes.iter().find(|(_, prefix)| {
            trimmed.starts_with(prefix)
                && matches!(
                    trimmed.as_bytes().get(prefix.len()),
                    Some(b' ') | Some(b'\t') | Some(b')') | None
                )
        }) {
            *counts.entry(form.clone()).or_default() += 1;
        }
    }

    counts
}

pub(super) fn top_level_blocks(contents: &str, form: &str) -> Vec<String> {
    nested_blocks_with_max_indent(contents, form, 2)
}

pub(super) fn top_level_blocks_by_form(
    contents: &str,
    forms: &[&str],
) -> std::collections::HashMap<String, Vec<String>> {
    nested_blocks_by_form_with_max_indent(contents, forms, 2)
}

pub(super) fn nested_blocks(contents: &str, form: &str) -> Vec<String> {
    nested_blocks_with_max_indent(contents, form, usize::MAX)
}

pub(super) fn nested_blocks_with_max_indent(
    contents: &str,
    form: &str,
    max_indent: usize,
) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut capturing = false;
    let mut depth: i32 = 0;
    let prefix = format!("({form}");

    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();

        if !capturing
            && indent <= max_indent
            && trimmed.starts_with(&prefix)
            && matches!(
                trimmed.as_bytes().get(prefix.len()),
                Some(b' ') | Some(b'\t') | Some(b')') | None
            )
        {
            capturing = true;
            current.clear();
            depth = 0;
        }

        if capturing {
            current.push(line.to_string());
            depth += paren_delta(line);
            if depth <= 0 {
                blocks.push(current.join("\n"));
                current.clear();
                capturing = false;
            }
        }
    }
    blocks
}

fn nested_blocks_by_form_with_max_indent(
    contents: &str,
    forms: &[&str],
    max_indent: usize,
) -> std::collections::HashMap<String, Vec<String>> {
    let mut blocks = forms
        .iter()
        .map(|form| ((*form).to_string(), Vec::new()))
        .collect::<std::collections::HashMap<_, _>>();
    let prefixes = forms
        .iter()
        .map(|form| ((*form).to_string(), format!("({form}")))
        .collect::<Vec<_>>();
    let mut current = Vec::new();
    let mut capturing_form: Option<String> = None;
    let mut depth: i32 = 0;

    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();

        if capturing_form.is_none() {
            if let Some((form, _)) = prefixes.iter().find(|(_, prefix)| {
                indent <= max_indent
                    && trimmed.starts_with(prefix)
                    && matches!(
                        trimmed.as_bytes().get(prefix.len()),
                        Some(b' ') | Some(b'\t') | Some(b')') | None
                    )
            }) {
                capturing_form = Some(form.clone());
                current.clear();
                depth = 0;
            }
        }

        if let Some(form) = capturing_form.as_ref() {
            current.push(line.to_string());
            depth += paren_delta(line);
            if depth <= 0 {
                blocks
                    .entry(form.clone())
                    .or_default()
                    .push(current.join("\n"));
                current.clear();
                capturing_form = None;
            }
        }
    }

    blocks
}

pub(super) fn paren_delta(line: &str) -> i32 {
    let opens = line.chars().filter(|c| *c == '(').count() as i32;
    let closes = line.chars().filter(|c| *c == ')').count() as i32;
    opens - closes
}

pub(super) fn find_top_level_uuid(contents: &str) -> Option<Uuid> {
    for line in contents.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if indent <= 2
            && trimmed.starts_with("(uuid ")
            && let Some(uuid) = parse_uuid_line(trimmed)
        {
            return Some(uuid);
        }
    }
    None
}

pub(super) fn block_uuid(block: &str) -> Option<Uuid> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(uuid ") {
            parse_uuid_line(trimmed)
        } else {
            None
        }
    })
}

pub(super) fn parse_uuid_line(trimmed: &str) -> Option<Uuid> {
    if let Some(token) = parse_quoted_token(trimmed) {
        return Uuid::parse_str(&token).ok();
    }

    let token = trimmed
        .trim_start_matches("(uuid ")
        .trim_end_matches(')')
        .split_whitespace()
        .next()?;
    Uuid::parse_str(token).ok()
}

pub(super) fn block_at_point(block: &str) -> Option<Point> {
    block
        .lines()
        .find_map(|line| parse_at_point(line.trim_start()))
}

pub(super) fn parse_at_point(trimmed: &str) -> Option<Point> {
    if !trimmed.starts_with("(at ") {
        return None;
    }
    let rest = trimmed.trim_start_matches("(at ").trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(mm_point_to_nm(x, y))
}

pub(super) fn block_xy_points(block: &str) -> Vec<Point> {
    let mut points = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim_start();
        points.extend(parse_xy_points_from_line(trimmed));
    }
    points
}

pub(super) fn parse_xy_points_from_line(line: &str) -> Vec<Point> {
    let mut points = Vec::new();
    let mut rest = line;
    let marker = "(xy ";

    while let Some(start) = rest.find(marker) {
        let after = &rest[start + marker.len()..];
        let Some(end) = after.find(')') else {
            break;
        };
        let mut parts = after[..end].split_whitespace();
        let Some(x) = parts.next().and_then(|v| v.parse::<f64>().ok()) else {
            rest = &after[end + 1..];
            continue;
        };
        let Some(y) = parts.next().and_then(|v| v.parse::<f64>().ok()) else {
            rest = &after[end + 1..];
            continue;
        };
        points.push(mm_point_to_nm(x, y));
        rest = &after[end + 1..];
    }

    points
}

pub(super) fn block_head_string(block: &str, form: &str) -> Option<String> {
    let first = block.lines().next()?.trim_start();
    let prefix = format!("({form} ");
    if !first.starts_with(&prefix) {
        return None;
    }
    let after = &first[prefix.len()..];
    let start = after.find('"')?;
    let rest = &after[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(super) fn parse_net_block(block: &str) -> Option<(i32, String)> {
    let first = block.lines().next()?.trim_start();
    if !first.starts_with("(net ") {
        return None;
    }
    let after = first.trim_start_matches("(net ").trim_end_matches(')');
    let mut chars = after.chars().peekable();
    let mut code = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() {
            break;
        }
        code.push(*ch);
        chars.next();
    }
    let code = code.parse::<i32>().ok()?;
    let rest: String = chars.collect();
    let start = rest.find('"')?;
    let rest = &rest[start + 1..];
    let end = rest.find('"')?;
    Some((code, rest[..end].to_string()))
}

pub(super) fn block_rotation(block: &str) -> Option<i32> {
    // Only check the block's own (at ...) line, not child elements.
    // The footprint's (at x y) or (at x y rotation) is always the first (at line.
    let first_at_line = block
        .lines()
        .map(|line| line.trim_start())
        .find(|line| line.starts_with("(at "))?;
    parse_at_rotation(first_at_line)
}

pub(super) fn parse_at_rotation(trimmed: &str) -> Option<i32> {
    if !trimmed.starts_with("(at ") {
        return None;
    }
    let rest = trimmed.trim_start_matches("(at ").trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    parts.next()?;
    parts.next()?;
    let rotation = parts.next()?.parse::<f64>().ok()?;
    Some(rotation.round() as i32)
}

pub(super) fn extract_footprint_property(block: &str, key: &str) -> Option<String> {
    let needle = format!("(property \"{key}\" ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let after = &trimmed[needle.len()..];
        let start = after.find('"')?;
        let rest = &after[start + 1..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    })
}

pub(super) fn block_layer_name(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(layer ") {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

pub(super) fn block_layer_names(block: &str) -> Vec<String> {
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(layers ") {
            let inner = trimmed.trim_start_matches("(layers ").trim_end_matches(')');
            return inner
                .split('"')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
                .collect();
        }
    }
    Vec::new()
}

pub(super) fn block_layers_pair(block: &str) -> Option<(String, String)> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(layers ") {
            return None;
        }
        let tokens = quoted_tokens(trimmed);
        if tokens.len() >= 2 {
            Some((tokens[0].clone(), tokens[1].clone()))
        } else {
            None
        }
    })
}

pub(super) fn block_width_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "width"))
}

pub(super) fn block_size_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "size"))
}

pub(super) fn block_size_point(block: &str) -> Option<Point> {
    block
        .lines()
        .find_map(|line| parse_xy_like(line.trim_start(), "size"))
}

pub(super) fn block_drill_mm(block: &str) -> Option<f64> {
    block
        .lines()
        .find_map(|line| parse_scalar_mm(line.trim_start(), "drill"))
}

pub(super) fn parse_scalar_mm(trimmed: &str, form: &str) -> Option<f64> {
    let prefix = format!("({form} ");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    trimmed[prefix.len()..]
        .trim_end_matches(')')
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
}

pub(super) fn block_start_end_points(block: &str) -> Option<(Point, Point)> {
    let mut start = None;
    let mut end = None;
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(start ") {
            start = parse_xy_like(trimmed, "start");
        } else if trimmed.starts_with("(end ") {
            end = parse_xy_like(trimmed, "end");
        }
    }
    Some((start?, end?))
}

pub(super) fn parse_xy_like(trimmed: &str, form: &str) -> Option<Point> {
    let prefix = format!("({form} ");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    let rest = trimmed[prefix.len()..].trim_end_matches(')');
    let mut parts = rest.split_whitespace();
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some(mm_point_to_nm(x, y))
}

pub(super) fn block_polygon(block: &str) -> Option<Polygon> {
    // Prefer filled_polygon (actual computed fill) over outline polygon
    if let Some(fp) = block_filled_polygon(block) {
        return Some(fp);
    }
    // Fallback: extract only from the (polygon (pts ...)) section, not filled_polygon
    if let Some(start) = block.find("(polygon\n") {
        let rest = &block[start..];
        // Find the closing of this polygon section
        if let Some(end) = find_matching_paren(rest) {
            let section = &rest[..end];
            let points = block_xy_points(section);
            if !points.is_empty() {
                return Some(Polygon::new(points));
            }
        }
    }
    let points = block_xy_points(block);
    if points.is_empty() {
        None
    } else {
        Some(Polygon::new(points))
    }
}

/// Extract the filled_polygon points from a zone block.
pub(super) fn block_filled_polygon(block: &str) -> Option<Polygon> {
    let marker = "(filled_polygon\n";
    let start = block
        .find(marker)
        .or_else(|| block.find("(filled_polygon\r"))?;
    let rest = &block[start..];
    let end = find_matching_paren(rest)?;
    let section = &rest[..end];
    let points = block_xy_points(section);
    if points.len() >= 3 {
        Some(Polygon::new(points))
    } else {
        None
    }
}

pub(super) fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn parse_board_layers(contents: &str) -> Stackup {
    let mut layers = Vec::new();
    for block in top_level_blocks(contents, "layers") {
        for line in block.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('(') || trimmed.starts_with("(layers") {
                continue;
            }
            if let Some(layer) = parse_layer_line(trimmed) {
                layers.push(layer);
            }
        }
    }

    if layers.is_empty() {
        layers.push(StackupLayer {
            id: 0,
            name: "F.Cu".into(),
            layer_type: StackupLayerType::Copper,
            thickness_nm: 35_000,
        });
    }

    layers.sort_by_key(|layer| layer.id);
    Stackup { layers }
}

pub(super) fn parse_layer_line(trimmed: &str) -> Option<StackupLayer> {
    let inner = trimmed.strip_prefix('(')?.trim_end_matches(')');
    let mut parts = inner.split_whitespace();
    let id = parts.next()?.parse::<i32>().ok()?;
    let name = parse_next_quoted_from(inner)?;
    let layer_type = if inner.contains(" signal") {
        StackupLayerType::Copper
    } else {
        StackupLayerType::Mechanical
    };
    Some(StackupLayer {
        id,
        name,
        layer_type,
        thickness_nm: 0,
    })
}

/// Apply a footprint's `(at x y [rot])` placement to a point authored in the
/// footprint's local coordinate system, producing a world-space point. Used
/// only for footprint-embedded outline contributors under the bounded M7-IMP-003
/// Option A ownership rule.
fn apply_footprint_transform(local: Point, origin: Point, rot_deg: i32) -> Point {
    if rot_deg == 0 {
        return Point::new(local.x + origin.x, local.y + origin.y);
    }
    let rad = (rot_deg as f64).to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let lx = local.x as f64;
    let ly = local.y as f64;
    let wx = lx * cos - ly * sin + origin.x as f64;
    let wy = lx * sin + ly * cos + origin.y as f64;
    Point::new(wx.round() as i64, wy.round() as i64)
}

/// Extract Edge.Cuts outline contributor segments from one scope. Used for both
/// the top-level PCB scope (with the top-level `gr_line`/`gr_arc` form names
/// and no transform) and each footprint body (with `fp_line`/`fp_arc` and the
/// footprint's own transform applied to every point).
fn collect_edge_cuts_segments(
    scope: &str,
    line_form: &str,
    arc_form: &str,
    transform: Option<(Point, i32)>,
) -> Vec<(Point, Vec<Point>)> {
    let mut segments: Vec<(Point, Vec<Point>)> = Vec::new();
    let blocks = match transform {
        None => top_level_blocks(scope, line_form),
        Some(_) => nested_blocks(scope, line_form),
    };
    for block in blocks {
        let Some(layer_name) = block_layer_name(&block) else {
            continue;
        };
        if layer_name != "Edge.Cuts" {
            continue;
        }
        let mut start = None;
        let mut end = None;
        for line in block.lines() {
            let trimmed = line.trim_start();
            if let Some(p) = parse_xy_like(trimmed, "start") {
                start = Some(p);
            }
            if let Some(p) = parse_xy_like(trimmed, "end") {
                end = Some(p);
            }
        }
        if let (Some(s), Some(e)) = (start, end) {
            let (s, e) = match transform {
                Some((origin, rot)) => (
                    apply_footprint_transform(s, origin, rot),
                    apply_footprint_transform(e, origin, rot),
                ),
                None => (s, e),
            };
            segments.push((s, vec![e]));
        }
    }

    let blocks = match transform {
        None => top_level_blocks(scope, arc_form),
        Some(_) => nested_blocks(scope, arc_form),
    };
    for block in blocks {
        let Some(layer_name) = block_layer_name(&block) else {
            continue;
        };
        if layer_name != "Edge.Cuts" {
            continue;
        }
        let mut start = None;
        let mut mid = None;
        let mut end = None;
        for line in block.lines() {
            let trimmed = line.trim_start();
            if let Some(p) = parse_xy_like(trimmed, "start") {
                start = Some(p);
            }
            if let Some(p) = parse_xy_like(trimmed, "mid") {
                mid = Some(p);
            }
            if let Some(p) = parse_xy_like(trimmed, "end") {
                end = Some(p);
            }
        }
        if let (Some(s), Some(m), Some(e)) = (start, mid, end) {
            let (s, m, e) = match transform {
                Some((origin, rot)) => (
                    apply_footprint_transform(s, origin, rot),
                    apply_footprint_transform(m, origin, rot),
                    apply_footprint_transform(e, origin, rot),
                ),
                None => (s, m, e),
            };
            let arc_points = interpolate_arc(s, m, e, 64);
            segments.push((s, arc_points));
        }
    }

    segments
}

/// Extract the imported KiCad board outline from Edge.Cuts contributors.
///
/// Accepted contributors per M7-IMP-003 Option A:
/// - top-level `gr_line` / `gr_arc` on Edge.Cuts
/// - footprint-embedded `fp_line` / `fp_arc` on Edge.Cuts under the
///   footprint's `(at x y rot)` transform
///
/// Returns the assembled outline polygon paired with an optional warning
/// message. Missing or unassemblable outline is NOT a hard import failure:
/// Datum must be able to open incomplete/in-progress boards and let users
/// finish them. When the outline cannot be recovered, the returned polygon
/// is empty and the warning string names the specific degraded case.
pub(super) fn outline_from_edge_cuts(contents: &str) -> (Polygon, Option<String>) {
    let mut segments: Vec<(Point, Vec<Point>)> =
        collect_edge_cuts_segments(contents, "gr_line", "gr_arc", None);

    for footprint in top_level_blocks(contents, "footprint") {
        let Some(origin) = block_at_point(&footprint) else {
            continue;
        };
        let rot = block_rotation(&footprint).unwrap_or(0);
        segments.extend(collect_edge_cuts_segments(
            &footprint,
            "fp_line",
            "fp_arc",
            Some((origin, rot)),
        ));
    }

    if segments.is_empty() {
        return (
            Polygon::new(Vec::new()),
            Some("no supported Edge.Cuts contributors found; outline is empty".to_string()),
        );
    }

    // Chain segments into an ordered polygon.
    // Each segment is (start_point, [interior_and_end_points...]).
    // Match end of current chain to start of next segment.
    let tolerance = 50_000; // 50 micron
    let mut remaining = segments;
    let first = remaining.remove(0);
    let mut ordered = vec![first.0];
    ordered.extend_from_slice(&first.1);

    for _ in 0..remaining.len() + 1 {
        if remaining.is_empty() {
            break;
        }
        let tail = *ordered.last().unwrap();
        let mut found = None;
        for (i, seg) in remaining.iter().enumerate() {
            let seg_end = seg.1.last().copied().unwrap_or(seg.0);
            if point_near(tail, seg.0, tolerance) {
                // Forward: chain start matches our tail
                found = Some((i, false));
                break;
            } else if point_near(tail, seg_end, tolerance) {
                // Reversed: chain end matches our tail
                found = Some((i, true));
                break;
            }
        }
        if let Some((idx, reversed)) = found {
            let seg = remaining.remove(idx);
            if reversed {
                // Walk backwards: end...interior...start
                let mut pts = vec![seg.0];
                pts.extend_from_slice(&seg.1);
                pts.reverse();
                // Skip the first point (it's the same as our tail)
                for p in pts.iter().skip(1) {
                    ordered.push(*p);
                }
            } else {
                // Forward: skip start (same as tail), add interior+end
                for p in &seg.1 {
                    ordered.push(*p);
                }
            }
        } else {
            break;
        }
    }

    if ordered.len() < 3 {
        return (
            Polygon::new(Vec::new()),
            Some(format!(
                "Edge.Cuts contributors could not be assembled into a closed outline \
                 (chained {} point(s), {} contributor segment(s) unchained); outline is empty",
                ordered.len(),
                remaining.len()
            )),
        );
    }

    (Polygon::new(ordered), None)
}

fn point_near(a: Point, b: Point, tolerance: i64) -> bool {
    (a.x - b.x).abs() <= tolerance && (a.y - b.y).abs() <= tolerance
}

fn interpolate_arc(start: Point, mid: Point, end: Point, segments: usize) -> Vec<Point> {
    let ax = start.x as f64;
    let ay = start.y as f64;
    let bx = mid.x as f64;
    let by = mid.y as f64;
    let cx = end.x as f64;
    let cy = end.y as f64;

    // Find circumcircle of three points
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1.0 {
        return vec![mid, end];
    }
    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    let r = ((ax - ux).powi(2) + (ay - uy).powi(2)).sqrt();

    let a0 = (ay - uy).atan2(ax - ux);
    let a1 = (by - uy).atan2(bx - ux);
    let a2 = (cy - uy).atan2(cx - ux);

    // Compute sweep from start to end going through mid.
    // Use two half-sweeps: start→mid and mid→end.
    let tau = 2.0 * std::f64::consts::PI;
    let half1 = {
        let mut d = a1 - a0;
        // Try both directions, pick the shorter one
        while d > std::f64::consts::PI {
            d -= tau;
        }
        while d < -std::f64::consts::PI {
            d += tau;
        }
        d
    };
    let half2 = {
        let mut d = a2 - a1;
        while d > std::f64::consts::PI {
            d -= tau;
        }
        while d < -std::f64::consts::PI {
            d += tau;
        }
        d
    };
    // If both halves go the same direction, total sweep is their sum.
    // If they disagree, use the long way around.
    let sweep = if (half1 > 0.0) == (half2 > 0.0) {
        half1 + half2
    } else {
        // Fallback: go the short way from start to end
        let mut d = a2 - a0;
        while d > std::f64::consts::PI {
            d -= tau;
        }
        while d < -std::f64::consts::PI {
            d += tau;
        }
        d
    };

    let mut points = Vec::with_capacity(segments);
    for i in 1..segments {
        let t = i as f64 / segments as f64;
        let angle = a0 + sweep * t;
        let px = (ux + r * angle.cos()).round() as i64;
        let py = (uy + r * angle.sin()).round() as i64;
        points.push(Point::new(px, py));
    }
    // Force last point to be exactly the end point for precise chaining
    points.push(end);
    points
}

pub(super) fn parse_quoted_token(trimmed: &str) -> Option<String> {
    let start = trimmed.find('"')?;
    let rest = &trimmed[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(super) fn quoted_tokens(trimmed: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = trimmed;
    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        tokens.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }
    tokens
}

pub(super) fn parse_next_quoted_from(inner: &str) -> Option<String> {
    let start = inner.find('"')?;
    let rest = &inner[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(super) fn deterministic_kicad_board_uuid(kind: &str, key: &str) -> Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/{kind}/{key}"),
    )
}

pub(super) fn canonicalize_kicad_layer_name(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "f.cu" => "F.Cu".to_string(),
        "b.cu" => "B.Cu".to_string(),
        "b.adhes" => "B.Adhes".to_string(),
        "f.adhes" => "F.Adhes".to_string(),
        "b.paste" => "B.Paste".to_string(),
        "f.paste" => "F.Paste".to_string(),
        "b.silks" => "B.SilkS".to_string(),
        "f.silks" => "F.SilkS".to_string(),
        "b.mask" => "B.Mask".to_string(),
        "f.mask" => "F.Mask".to_string(),
        "dwgs.user" => "Dwgs.User".to_string(),
        "cmts.user" => "Cmts.User".to_string(),
        "eco1.user" => "Eco1.User".to_string(),
        "eco2.user" => "Eco2.User".to_string(),
        "edge.cuts" => "Edge.Cuts".to_string(),
        "margin" => "Margin".to_string(),
        "b.crtyd" => "B.CrtYd".to_string(),
        "f.crtyd" => "F.CrtYd".to_string(),
        "b.fab" => "B.Fab".to_string(),
        "f.fab" => "F.Fab".to_string(),
        _ => name.to_string(),
    }
}

/// Hardcoded fallback for layer names on boards where the PCB's own
/// `(layers ...)` table is absent or unparsed. Only a narrow set of layer
/// names is recognized here; unknown names must be handled by the caller,
/// not silently collapsed onto F.Cu.
pub(super) fn kicad_layer_name_to_id(name: &str) -> Option<i32> {
    match canonicalize_kicad_layer_name(name).as_str() {
        "F.Cu" => Some(0),
        "B.Cu" => Some(31),
        "B.Adhes" => Some(32),
        "F.Adhes" => Some(33),
        "B.Paste" => Some(34),
        "F.Paste" => Some(35),
        "B.SilkS" => Some(36),
        "F.SilkS" => Some(37),
        "B.Mask" => Some(38),
        "F.Mask" => Some(39),
        "Dwgs.User" => Some(40),
        "Cmts.User" => Some(41),
        "Eco1.User" => Some(42),
        "Eco2.User" => Some(43),
        "Edge.Cuts" => Some(44),
        "Margin" => Some(45),
        "B.CrtYd" => Some(46),
        "F.CrtYd" => Some(47),
        "B.Fab" => Some(48),
        "F.Fab" => Some(49),
        _ => None,
    }
}

/// Parse the (layers ...) section from a KiCad PCB file and build a name→id map.
pub(super) fn parse_kicad_layer_table(contents: &str) -> std::collections::HashMap<String, i32> {
    let mut map = std::collections::HashMap::new();
    // Find the (layers ...) top-level block
    if let Some(start) = contents.find("\n\t(layers\n") {
        let rest = &contents[start..];
        if let Some(end) = rest.find("\n\t)\n") {
            let block = &rest[..end];
            for line in block.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('(') && !trimmed.starts_with("(layers") {
                    // Format: (id "name" type) or (id "name" type "display_name")
                    let inner = trimmed.trim_start_matches('(').trim_end_matches(')');
                    let mut parts = inner.split_whitespace();
                    if let Some(id_str) = parts.next() {
                        if let Ok(id) = id_str.parse::<i32>() {
                            if let Some(name) = parts.next() {
                                let name = canonicalize_kicad_layer_name(name.trim_matches('"'));
                                map.insert(name.to_string(), id);
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/// Resolve a KiCad layer name to its layer id, using the PCB's own parsed
/// `(layers ...)` table as the authority and the narrow hardcoded fallback
/// map for boards that lack or don't parse a table. Unknown layer names are
/// an explicit import error — never silently mapped to F.Cu.
pub(super) fn resolve_layer_id(
    name: &str,
    table: &std::collections::HashMap<String, i32>,
) -> Result<i32, EngineError> {
    let canonical_name = canonicalize_kicad_layer_name(name);
    if let Some(&id) = table.get(&canonical_name) {
        return Ok(id);
    }
    kicad_layer_name_to_id(&canonical_name).ok_or_else(|| {
        EngineError::Import(format!(
            "unknown KiCad layer name: {name:?} (not present in PCB layer table and not in fallback set)"
        ))
    })
}

pub(super) fn mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}
