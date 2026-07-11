use std::collections::HashMap;

use crate::board::PadExpansionSetup;
use crate::error::EngineError;

use super::parser_helpers::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NonCopperLayerKind {
    Mask,
    Paste,
}

impl NonCopperLayerKind {
    fn wildcard(self) -> &'static str {
        match self {
            Self::Mask => "*.Mask",
            Self::Paste => "*.Paste",
        }
    }

    fn front_name(self) -> &'static str {
        match self {
            Self::Mask => "F.Mask",
            Self::Paste => "F.Paste",
        }
    }

    fn back_name(self) -> &'static str {
        match self {
            Self::Mask => "B.Mask",
            Self::Paste => "B.Paste",
        }
    }

    fn matches_name(self, name: &str) -> bool {
        name == self.front_name() || name == self.back_name()
    }
}

pub(super) fn parse_pad_non_copper_layers_anywhere(
    block: &str,
    layer_table: &HashMap<String, i32>,
    kind: NonCopperLayerKind,
) -> Result<Vec<i32>, EngineError> {
    let Some(layers) = block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.contains("(layers ") {
            return None;
        }
        Some(quoted_tokens(trimmed))
    }) else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    if layers.iter().any(|entry| entry == kind.wildcard()) {
        out.push(resolve_layer_id(kind.front_name(), layer_table)?);
        out.push(resolve_layer_id(kind.back_name(), layer_table)?);
    }
    for entry in &layers {
        if !kind.matches_name(entry) {
            continue;
        }
        let id = resolve_layer_id(entry, layer_table)?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out.sort_unstable();
    Ok(out)
}

pub(super) fn parse_pad_expansion_setup(contents: &str) -> PadExpansionSetup {
    PadExpansionSetup {
        pad_to_mask_clearance_nm: parse_setup_mm_value(contents, "pad_to_mask_clearance")
            .unwrap_or(0),
        pad_to_paste_clearance_nm: parse_setup_mm_value(contents, "pad_to_paste_clearance")
            .unwrap_or(0),
        pad_to_paste_ratio_ppm: parse_setup_ratio_ppm(contents, "pad_to_paste_clearance_ratio")
            .or_else(|| parse_setup_ratio_ppm(contents, "pad_to_paste_ratio"))
            .unwrap_or(0),
        solder_mask_min_width_nm: parse_setup_mm_value(contents, "solder_mask_min_width")
            .unwrap_or(0),
    }
}

pub(super) fn parse_block_mm_value_anywhere(block: &str, key: &str) -> Option<i64> {
    let needle = format!("({key} ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let rest = trimmed[needle.len()..].split(')').next().unwrap_or("");
        rest.split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(mm_to_nm)
    })
}

pub(super) fn parse_block_ratio_ppm_anywhere(block: &str, key: &str) -> Option<i32> {
    let needle = format!("({key} ");
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let rest = trimmed[needle.len()..].split(')').next().unwrap_or("");
        rest.split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|value| (value * 1_000_000.0).round() as i32)
    })
}

pub(super) fn parse_footprint_mm_value_before_pads(block: &str, key: &str) -> Option<i64> {
    let needle = format!("({key} ");
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(pad ") {
            break;
        }
        if !trimmed.starts_with(&needle) {
            continue;
        }
        let rest = trimmed[needle.len()..].split(')').next().unwrap_or("");
        return rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(mm_to_nm);
    }
    None
}

pub(super) fn parse_footprint_ratio_ppm_before_pads(block: &str, key: &str) -> Option<i32> {
    let needle = format!("({key} ");
    for line in block.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("(pad ") {
            break;
        }
        if !trimmed.starts_with(&needle) {
            continue;
        }
        let rest = trimmed[needle.len()..].split(')').next().unwrap_or("");
        return rest
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|value| (value * 1_000_000.0).round() as i32);
    }
    None
}

fn parse_setup_mm_value(contents: &str, key: &str) -> Option<i64> {
    let needle = format!("({key} ");
    contents.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let rest = trimmed
            .trim_start_matches(&needle)
            .split(')')
            .next()
            .unwrap_or("");
        rest.split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(mm_to_nm)
    })
}

fn parse_setup_ratio_ppm(contents: &str, key: &str) -> Option<i32> {
    let needle = format!("({key} ");
    contents.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(&needle) {
            return None;
        }
        let rest = trimmed
            .trim_start_matches(&needle)
            .split(')')
            .next()
            .unwrap_or("");
        rest.split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| (v * 1_000_000.0).round() as i32)
    })
}
