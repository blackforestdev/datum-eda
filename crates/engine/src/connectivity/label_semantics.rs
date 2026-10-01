//! Existing scalar/range label semantics shared by summaries and identity topology.
use crate::schematic::NetLabel;
pub(super) fn canonical_label_name(name: &str) -> String {
    parse_scalar_bus_member_name(name).unwrap_or_else(|| name.to_string())
}

pub(super) fn is_bus_container_label(label: &NetLabel) -> bool {
    parse_bus_range_members(&label.name).is_some()
}

pub(super) fn has_bus_syntax(name: &str) -> bool {
    name.contains('[') || name.contains(']')
}

pub(super) fn parse_scalar_bus_member_name(name: &str) -> Option<String> {
    let open = name.rfind('[')?;
    let close = name.rfind(']')?;
    if close <= open + 1 || close != name.len() - 1 {
        return None;
    }
    let base = name[..open].trim();
    if base.is_empty() {
        return None;
    }
    let body = &name[open + 1..close];
    if body.contains("..") || body.contains(',') {
        return None;
    }
    let index = body.trim().parse::<i32>().ok()?;
    Some(format!("{base}{index}"))
}

fn parse_bus_range_members(name: &str) -> Option<(i32, i32)> {
    let open = name.rfind('[')?;
    let close = name.rfind(']')?;
    if close <= open + 1 || close != name.len() - 1 || name[..open].trim().is_empty() {
        return None;
    }
    let (start, end) = name[open + 1..close].split_once("..")?;
    Some((start.trim().parse().ok()?, end.trim().parse().ok()?))
}
