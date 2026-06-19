//! Net-reference parsing and resolution for imported KiCad boards.
//!
//! Handles both the integer-code net references used by formats through
//! `20241229` (resolved against the top-level net table) and the quoted
//! name references introduced by `20260206`, which carry no top-level
//! integer net table at all.

use std::collections::HashMap;

use uuid::Uuid;

use crate::board::Net;

use super::parser_helpers::deterministic_kicad_board_uuid;

/// A net reference inside a board object block.
///
/// KiCad formats through `20241229` reference nets by integer code against a
/// top-level `(net <code> "<name>")` table. KiCad `20260206`+ references nets
/// directly by quoted name (e.g. `(net "/IN_P")`) and emits no top-level
/// integer net table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NetRef {
    Code(i32),
    Name(String),
}

pub(super) fn block_net_ref(block: &str) -> Option<NetRef> {
    block.lines().find_map(|line| {
        let trimmed = line.trim_start();
        let after = trimmed.strip_prefix("(net ")?;
        let first = after.split_whitespace().next()?;
        if let Ok(code) = first.trim_end_matches(')').parse::<i32>() {
            return Some(NetRef::Code(code));
        }
        // Name form: extract the quoted string before any paren trimming, so
        // names with trailing parens (e.g. "Net-(Q1-C)") survive intact.
        let start = after.find('"')?;
        let rest = &after[start + 1..];
        let end = rest.find('"')?;
        let name = &rest[..end];
        if name.is_empty() {
            return None;
        }
        Some(NetRef::Name(name.to_string()))
    })
}

/// Resolve a parsed net reference to a board net UUID.
///
/// Integer codes resolve through the top-level net table (formats through
/// `20241229`). Name references (`20260206`+, which has no top-level integer
/// net table) derive a deterministic UUID from the net name and register the
/// net on first sight so `board.nets` stays populated.
pub(super) fn resolve_board_net_ref(
    net_ref: NetRef,
    net_lookup: &HashMap<i32, Uuid>,
    nets: &mut HashMap<Uuid, Net>,
) -> Uuid {
    match net_ref {
        NetRef::Code(code) => net_lookup
            .get(&code)
            .copied()
            .unwrap_or_else(|| deterministic_kicad_board_uuid("net", &code.to_string())),
        NetRef::Name(name) => {
            let uuid = deterministic_kicad_board_uuid("net", &name);
            nets.entry(uuid).or_insert_with(|| Net {
                uuid,
                name,
                class: Uuid::nil(),
            });
            uuid
        }
    }
}
