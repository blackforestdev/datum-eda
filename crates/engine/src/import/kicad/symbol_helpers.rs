use std::collections::HashMap;

use uuid::Uuid;

use crate::ir::geometry::Point;
use crate::ir::ids::{import_uuid, namespace_kicad};
use crate::schematic::{
    HierarchicalPort, PinElectricalType, PlacedSymbol, PortDirection, SymbolField,
};

use super::{
    LibraryPinTemplate, block_at_point, block_head_string, nested_blocks, parse_at_point,
    parse_quoted_token, quoted_tokens, top_level_blocks,
};

pub(super) fn transform_symbol_pin(
    origin: Point,
    rotation_deg: i32,
    mirrored_y: bool,
    local: Point,
) -> Point {
    let local = if mirrored_y {
        // KiCad `mirror y` reflects the symbol about the Y axis, so the
        // local X coordinate changes sign before rotation is applied.
        Point::new(-local.x, local.y)
    } else {
        local
    };
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => Point::new(-local.y, local.x),
        180 => Point::new(-local.x, -local.y),
        270 => Point::new(local.y, -local.x),
        _ => local,
    };
    Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

pub(super) fn transform_board_local_point(origin: Point, rotation_deg: i32, local: Point) -> Point {
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => Point::new(-local.y, local.x),
        180 => Point::new(-local.x, -local.y),
        270 => Point::new(local.y, -local.x),
        _ => local,
    };
    Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

pub(super) fn extract_sheet_property(block: &str, key: &str) -> Option<String> {
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

pub(super) fn extract_symbol_property(block: &str, key: &str) -> Option<String> {
    extract_sheet_property(block, key)
}

pub(super) fn symbol_is_mirrored_y(block: &str) -> bool {
    block.lines().any(|line| line.trim() == "(mirror y)")
}

pub(super) fn extract_symbol_lib_id(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(lib_id ") {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

pub(super) fn extract_symbol_unit(block: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(unit ") {
            return None;
        }
        Some(
            trimmed
                .trim_start_matches("(unit ")
                .trim_end_matches(')')
                .trim()
                .to_string(),
        )
    })
}

pub(super) fn symbol_fields(symbol_uuid: Uuid, block: &str) -> Vec<SymbolField> {
    let mut fields = Vec::new();
    for (index, line) in block.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("(property ") {
            continue;
        }
        let tokens = quoted_tokens(trimmed);
        if tokens.len() < 2 {
            continue;
        }
        fields.push(SymbolField {
            uuid: import_uuid(
                &namespace_kicad(),
                &format!("schematic-symbol-field/{symbol_uuid}/{index}/{}", tokens[0]),
            ),
            key: tokens[0].clone(),
            value: tokens[1].clone(),
            position: parse_at_point(trimmed),
            visible: true,
        });
    }
    fields
}

pub(super) fn parse_library_symbol_pins(
    contents: &str,
) -> HashMap<String, Vec<LibraryPinTemplate>> {
    let mut libraries = HashMap::new();
    let Some(lib_symbols_block) = top_level_blocks(contents, "lib_symbols").into_iter().next()
    else {
        return libraries;
    };

    for symbol_block in nested_blocks(&lib_symbols_block, "symbol") {
        let Some(lib_id) = block_head_string(&symbol_block, "symbol") else {
            continue;
        };
        if !lib_id.contains(':') {
            continue;
        }
        let mut pins = Vec::new();
        for pin_block in nested_blocks(&symbol_block, "pin") {
            let number = extract_named_subfield(&pin_block, "number").unwrap_or_else(|| "?".into());
            let name = extract_named_subfield(&pin_block, "name").unwrap_or_else(|| number.clone());
            let electrical_type = parse_kicad_pin_electrical_type(&pin_block);
            let position = block_at_point(&pin_block).unwrap_or_else(Point::zero);
            pins.push(LibraryPinTemplate {
                number,
                name,
                electrical_type,
                position,
            });
        }
        libraries.insert(lib_id, pins);
    }

    libraries
}

pub(super) fn extract_named_subfield(block: &str, field: &str) -> Option<String> {
    let needle = format!("({field} ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        parse_quoted_token(trimmed)
    })
}

pub(super) fn parse_kicad_pin_electrical_type(pin_block: &str) -> PinElectricalType {
    let first = pin_block
        .lines()
        .next()
        .map(str::trim_start)
        .unwrap_or_default();
    if first.contains("(pin output ") || first.contains("(pin tri_state ") {
        PinElectricalType::Output
    } else if first.contains("(pin input ") {
        PinElectricalType::Input
    } else if first.contains("(pin bidirectional ") {
        PinElectricalType::Bidirectional
    } else if first.contains("(pin power_in ") {
        PinElectricalType::PowerIn
    } else if first.contains("(pin power_out ") {
        PinElectricalType::PowerOut
    } else {
        PinElectricalType::Passive
    }
}

pub(super) fn extract_sheet_pins(sheet_instance_uuid: Uuid, block: &str) -> Vec<HierarchicalPort> {
    let mut ports = Vec::new();
    for (index, pin_block) in nested_blocks(block, "pin").into_iter().enumerate() {
        let Some(first_line) = pin_block.lines().next() else {
            continue;
        };
        let trimmed = first_line.trim_start();
        let tokens = quoted_tokens(trimmed);
        let Some(name) = tokens.first().cloned() else {
            continue;
        };
        let direction = if trimmed.contains(" output") {
            PortDirection::Output
        } else if trimmed.contains(" bidirectional") {
            PortDirection::Bidirectional
        } else if trimmed.contains(" passive") {
            PortDirection::Passive
        } else {
            PortDirection::Input
        };
        let position = block_at_point(&pin_block).unwrap_or_else(Point::zero);
        ports.push(HierarchicalPort {
            uuid: import_uuid(
                &namespace_kicad(),
                &format!("schematic-sheet-port/{sheet_instance_uuid}/{index}/{name}"),
            ),
            name,
            direction,
            position,
        });
    }
    ports
}

pub(super) fn mm_point_to_nm(x_mm: f64, y_mm: f64) -> Point {
    Point::new(
        (x_mm * 1_000_000.0).round() as i64,
        (y_mm * 1_000_000.0).round() as i64,
    )
}

pub(super) fn pin_at_position(
    symbols: &HashMap<Uuid, PlacedSymbol>,
    position: Point,
) -> Option<(Uuid, Uuid)> {
    let mut matches: Vec<(Uuid, Uuid)> = symbols
        .iter()
        .flat_map(|(symbol_uuid, symbol)| {
            symbol
                .pins
                .iter()
                .filter(|pin| pin.position == position)
                .map(|pin| (*symbol_uuid, pin.uuid))
                .collect::<Vec<_>>()
        })
        .collect();
    matches.sort();
    matches.into_iter().next()
}
