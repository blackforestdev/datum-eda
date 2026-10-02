//! Existing scalar/range label semantics shared by summaries and identity topology.
use crate::schematic::{LabelKind, NetLabel};
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

pub(super) fn preferred_name(
    labels: &[super::LabelRef],
    ports: &[super::PortRef],
) -> Option<String> {
    labels
        .iter()
        .find(|label| matches!(label.kind, LabelKind::Global))
        .or_else(|| {
            labels
                .iter()
                .find(|label| matches!(label.kind, LabelKind::Hierarchical))
        })
        .or_else(|| {
            labels
                .iter()
                .find(|label| matches!(label.kind, LabelKind::Local))
        })
        .map(|label| label.name.clone())
        .or_else(|| ports.first().map(|port| port.name.clone()))
}

pub(super) fn child_hierarchical_labels_by_name(
    child_sheet: &crate::schematic::Sheet,
) -> std::collections::BTreeMap<String, Vec<&NetLabel>> {
    let mut grouped: std::collections::BTreeMap<String, Vec<&NetLabel>> =
        std::collections::BTreeMap::new();
    let mut labels: Vec<_> = child_sheet
        .labels
        .values()
        .filter(|label| matches!(label.kind, LabelKind::Hierarchical))
        .filter(|label| !is_bus_container_label(label))
        .collect();
    labels.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    for label in labels {
        grouped.entry(label.name.clone()).or_default().push(label);
    }
    grouped
}
