use uuid::Uuid;

use crate::board::{Stackup, StackupLayer, StackupLayerType};
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

pub(super) fn top_level_blocks(contents: &str, form: &str) -> Vec<String> {
    nested_blocks_with_max_indent(contents, form, 2)
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
    let first = block
        .lines()
        .find_map(|line| parse_at_rotation(line.trim_start()))?;
    Some(first)
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

pub(super) fn block_net_code(block: &str) -> Option<i32> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(net ") {
            return None;
        }
        trimmed
            .trim_start_matches("(net ")
            .trim_end_matches(')')
            .split_whitespace()
            .next()?
            .parse::<i32>()
            .ok()
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
    let points = block_xy_points(block);
    if points.is_empty() {
        None
    } else {
        Some(Polygon::new(points))
    }
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

pub(super) fn outline_from_edge_cuts(contents: &str) -> Option<Polygon> {
    let mut points = Vec::new();
    for form in ["gr_line", "gr_arc"] {
        for block in top_level_blocks(contents, form) {
            let Some(layer_name) = block_layer_name(&block) else {
                continue;
            };
            if layer_name != "Edge.Cuts" {
                continue;
            }
            for line in block.lines() {
                let trimmed = line.trim_start();
                if (trimmed.starts_with("(start ")
                    || trimmed.starts_with("(end ")
                    || trimmed.starts_with("(mid "))
                    && let Some(point) = parse_xy_like(
                        trimmed,
                        if trimmed.starts_with("(start ") {
                            "start"
                        } else if trimmed.starts_with("(end ") {
                            "end"
                        } else {
                            "mid"
                        },
                    )
                {
                    points.push(point);
                }
            }
        }
    }

    if points.is_empty() {
        return None;
    }

    let min_x = points.iter().map(|p| p.x).min()?;
    let min_y = points.iter().map(|p| p.y).min()?;
    let max_x = points.iter().map(|p| p.x).max()?;
    let max_y = points.iter().map(|p| p.y).max()?;
    Some(Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ]))
}

pub(super) fn default_outline() -> Polygon {
    Polygon::new(vec![
        Point::new(0, 0),
        Point::new(10_000_000, 0),
        Point::new(10_000_000, 10_000_000),
        Point::new(0, 10_000_000),
    ])
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

pub(super) fn kicad_layer_name_to_id(name: &str) -> i32 {
    match name {
        "F.Cu" => 0,
        "B.Cu" => 31,
        "B.SilkS" => 36,
        "F.SilkS" => 37,
        "Edge.Cuts" => 44,
        _ => 0,
    }
}

pub(super) fn mm_to_nm(mm: f64) -> i64 {
    (mm * 1_000_000.0).round() as i64
}
