use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use uuid::Uuid;

use crate::ir::geometry::Point;
use crate::schematic::{
    ConnectivityDiagnosticInfo, HierarchicalLinkInfo, LabelKind, NetLabel, NetPinRef, Schematic,
    SchematicNetInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeKey {
    sheet: Uuid,
    point: Point,
}

#[derive(Default)]
struct UnionFind {
    parent: HashMap<NodeKey, NodeKey>,
}

impl UnionFind {
    fn add(&mut self, node: NodeKey) {
        self.parent.entry(node).or_insert(node);
    }

    fn find(&mut self, node: NodeKey) -> NodeKey {
        let parent = *self.parent.entry(node).or_insert(node);
        if parent == node {
            node
        } else {
            let root = self.find(parent);
            self.parent.insert(node, root);
            root
        }
    }

    fn union(&mut self, a: NodeKey, b: NodeKey) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent.insert(root_b, root_a);
        }
    }
}

#[derive(Debug, Clone)]
struct LabelRef {
    name: String,
    kind: LabelKind,
}

#[derive(Debug, Clone)]
struct PortRef {
    uuid: Uuid,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedHierarchicalLink {
    parent_sheet: Uuid,
    child_sheet: Uuid,
    parent_port: Uuid,
    child_port: Uuid,
}

#[derive(Debug, Default)]
struct HierarchyResolution {
    links: Vec<ResolvedHierarchicalLink>,
    diagnostics: Vec<ConnectivityDiagnosticInfo>,
}

#[derive(Debug, Default, Clone)]
struct NetAggregate {
    labels: Vec<LabelRef>,
    ports: Vec<PortRef>,
    pins: Vec<NetPinRef>,
    sheets: BTreeSet<String>,
}

pub fn schematic_net_info(schematic: &Schematic) -> Vec<SchematicNetInfo> {
    let mut uf = UnionFind::default();

    for sheet in schematic.sheets.values() {
        let mut attachment_points = HashSet::new();
        for junction in sheet.junctions.values() {
            attachment_points.insert(junction.position);
        }
        for label in sheet.labels.values() {
            if is_bus_container_label(label) {
                continue;
            }
            attachment_points.insert(label.position);
        }
        for port in sheet.ports.values() {
            attachment_points.insert(port.position);
        }
        for symbol in sheet.symbols.values() {
            for pin in &symbol.pins {
                attachment_points.insert(pin.position);
            }
        }

        for wire in sheet.wires.values() {
            let a = NodeKey {
                sheet: sheet.uuid,
                point: wire.from,
            };
            let b = NodeKey {
                sheet: sheet.uuid,
                point: wire.to,
            };
            uf.add(a);
            uf.add(b);
            uf.union(a, b);

            for point in &attachment_points {
                if point_on_wire_segment(*point, wire.from, wire.to) {
                    let node = NodeKey {
                        sheet: sheet.uuid,
                        point: *point,
                    };
                    uf.add(node);
                    uf.union(a, node);
                }
            }
        }
        for junction in sheet.junctions.values() {
            uf.add(NodeKey {
                sheet: sheet.uuid,
                point: junction.position,
            });
        }
        for label in sheet.labels.values() {
            if is_bus_container_label(label) {
                continue;
            }
            uf.add(NodeKey {
                sheet: sheet.uuid,
                point: label.position,
            });
        }
        for port in sheet.ports.values() {
            uf.add(NodeKey {
                sheet: sheet.uuid,
                point: port.position,
            });
        }
        for symbol in sheet.symbols.values() {
            for pin in &symbol.pins {
                uf.add(NodeKey {
                    sheet: sheet.uuid,
                    point: pin.position,
                });
            }
        }
    }

    let _ = apply_hierarchy_resolution(schematic, &mut uf);

    let mut point_groups: HashMap<NodeKey, NetAggregate> = HashMap::new();
    let mut global_label_groups_by_name: BTreeMap<String, Vec<NodeKey>> = BTreeMap::new();

    for sheet in schematic.sheets.values() {
        for label in sheet.labels.values() {
            if is_bus_container_label(label) {
                continue;
            }
            let node = NodeKey {
                sheet: sheet.uuid,
                point: label.position,
            };
            let root = uf.find(node);
            point_groups.entry(root).or_default().labels.push(LabelRef {
                name: canonical_label_name(&label.name),
                kind: label.kind.clone(),
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());

            if matches!(label.kind, LabelKind::Global) {
                global_label_groups_by_name
                    .entry(canonical_label_name(&label.name))
                    .or_default()
                    .push(root);
            }
        }

        for port in sheet.ports.values() {
            let node = NodeKey {
                sheet: sheet.uuid,
                point: port.position,
            };
            let root = uf.find(node);
            point_groups.entry(root).or_default().ports.push(PortRef {
                uuid: port.uuid,
                name: port.name.clone(),
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());
        }

        for symbol in sheet.symbols.values() {
            for pin in &symbol.pins {
                let node = NodeKey {
                    sheet: sheet.uuid,
                    point: pin.position,
                };
                let root = uf.find(node);
                point_groups.entry(root).or_default().pins.push(NetPinRef {
                    uuid: pin.uuid,
                    component: symbol.reference.clone(),
                    pin: pin.number.clone(),
                    electrical_type: pin.electrical_type.clone(),
                });
                point_groups
                    .entry(root)
                    .or_default()
                    .sheets
                    .insert(sheet.name.clone());
            }
        }
    }

    for (sheet_uuid, sheet) in &schematic.sheets {
        for wire in sheet.wires.values() {
            let root = uf.find(NodeKey {
                sheet: *sheet_uuid,
                point: wire.from,
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());
        }
        for junction in sheet.junctions.values() {
            let root = uf.find(NodeKey {
                sheet: *sheet_uuid,
                point: junction.position,
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());
        }
        for port in sheet.ports.values() {
            let root = uf.find(NodeKey {
                sheet: *sheet_uuid,
                point: port.position,
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());
        }
        for symbol in sheet.symbols.values() {
            for pin in &symbol.pins {
                let root = uf.find(NodeKey {
                    sheet: *sheet_uuid,
                    point: pin.position,
                });
                point_groups
                    .entry(root)
                    .or_default()
                    .sheets
                    .insert(sheet.name.clone());
            }
        }
    }

    let mut merged: BTreeMap<String, NetAggregate> = BTreeMap::new();

    for roots in global_label_groups_by_name.values() {
        let merge_key = format!("global:{}", roots.len());
        let entry = merged.entry(merge_key).or_default();
        for root in roots {
            if let Some(group) = point_groups.get(root) {
                entry.labels.extend(group.labels.clone());
                entry.ports.extend(group.ports.clone());
                entry.pins.extend(group.pins.clone());
                entry.sheets.extend(group.sheets.iter().cloned());
            }
        }
    }

    let mut consumed_global_roots = HashSet::new();
    for roots in global_label_groups_by_name.values() {
        for root in roots {
            consumed_global_roots.insert(*root);
        }
    }

    for (root, group) in point_groups {
        if consumed_global_roots.contains(&root)
            && group
                .labels
                .iter()
                .any(|label| matches!(label.kind, LabelKind::Global))
        {
            continue;
        }

        let key = if let Some(name) = preferred_name(&group.labels, &group.ports) {
            format!(
                "named:{name}:{}:{}",
                root.sheet,
                root.point.x ^ root.point.y
            )
        } else {
            format!("anon:{}:{}:{}", root.sheet, root.point.x, root.point.y)
        };
        merged.insert(key, group);
    }

    let mut nets = Vec::new();
    for (key, group) in merged {
        let name = preferred_name(&group.labels, &group.ports).unwrap_or_else(|| {
            let anon_id = crate::ir::ids::import_uuid(
                &crate::ir::ids::namespace_kicad(),
                &format!("schematic-anon/{key}"),
            );
            format!("N${}", &anon_id.as_simple().to_string()[..8])
        });
        let semantic_class = infer_semantic_class(&name);
        let sheets: Vec<_> = group.sheets.into_iter().collect();
        let mut pins = group.pins;
        pins.sort_by(|a, b| {
            a.component
                .cmp(&b.component)
                .then_with(|| a.pin.cmp(&b.pin))
        });
        let mut port_uuids: Vec<_> = group.ports.iter().map(|port| port.uuid).collect();
        port_uuids.sort();
        let uuid = crate::ir::ids::import_uuid(
            &crate::ir::ids::namespace_kicad(),
            &format!("schematic-net/{key}"),
        );

        nets.push(SchematicNetInfo {
            uuid,
            name,
            class: None,
            pins,
            labels: group.labels.len(),
            ports: group.ports.len(),
            port_uuids,
            sheets,
            semantic_class,
        });
    }

    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    nets
}

pub fn schematic_hierarchy_info(schematic: &Schematic) -> crate::schematic::HierarchyInfo {
    let mut base = schematic.hierarchy();
    let resolution = hierarchy_resolution_without_unions(schematic);
    let port_to_net: HashMap<Uuid, Uuid> = schematic_net_info(schematic)
        .into_iter()
        .flat_map(|net| {
            net.port_uuids
                .into_iter()
                .map(move |port_uuid| (port_uuid, net.uuid))
        })
        .collect();
    let mut child_to_net: HashMap<Uuid, Uuid> = HashMap::new();
    for link in &resolution.links {
        if let Some(net_uuid) = port_to_net.get(&link.parent_port) {
            child_to_net.insert(link.child_port, *net_uuid);
        }
    }
    let mut links: Vec<_> = resolution
        .links
        .into_iter()
        .filter_map(|link| {
            port_to_net
                .get(&link.parent_port)
                .or_else(|| child_to_net.get(&link.child_port))
                .copied()
                .map(|net| HierarchicalLinkInfo {
                    parent_sheet: link.parent_sheet,
                    child_sheet: link.child_sheet,
                    parent_port: link.parent_port,
                    child_port: link.child_port,
                    net,
                })
        })
        .collect();
    links.sort_by(|a, b| {
        a.parent_sheet
            .cmp(&b.parent_sheet)
            .then_with(|| a.child_sheet.cmp(&b.child_sheet))
            .then_with(|| a.parent_port.cmp(&b.parent_port))
            .then_with(|| a.child_port.cmp(&b.child_port))
    });
    base.links = links;
    base
}

fn infer_semantic_class(name: &str) -> Option<String> {
    if name.eq_ignore_ascii_case("gnd") {
        return Some("ground".to_string());
    }

    if name.eq_ignore_ascii_case("vcc")
        || name.eq_ignore_ascii_case("vdd")
        || name.eq_ignore_ascii_case("vee")
        || name.eq_ignore_ascii_case("vss")
        || name.starts_with('+')
        || name.starts_with('-')
    {
        return Some("power".to_string());
    }

    None
}

pub fn schematic_diagnostics(schematic: &Schematic) -> Vec<ConnectivityDiagnosticInfo> {
    let mut diagnostics = hierarchy_resolution_without_unions(schematic).diagnostics;

    for sheet in schematic.sheets.values() {
        for label in sheet.labels.values() {
            if has_bus_syntax(&label.name)
                && !is_bus_container_label(label)
                && parse_scalar_bus_member_name(&label.name).is_none()
            {
                diagnostics.push(ConnectivityDiagnosticInfo {
                    kind: "unsupported_bus_member_syntax".into(),
                    severity: "warning".into(),
                    message: format!(
                        "label {} uses unsupported bus/member syntax in the current KiCad subset",
                        label.name
                    ),
                    objects: vec![label.uuid],
                });
            }
        }
    }

    for net in schematic_net_info(schematic) {
        if net.pins.len() == 1 && net.labels == 0 && net.ports == 0 {
            diagnostics.push(ConnectivityDiagnosticInfo {
                kind: "dangling_component_pin".into(),
                severity: "warning".into(),
                message: format!(
                    "component pin {}.{} is on an isolated anonymous net",
                    net.pins[0].component, net.pins[0].pin
                ),
                objects: vec![net.pins[0].uuid],
            });
        }

        if net.ports == 1 && net.pins.is_empty() && net.labels == 0 {
            diagnostics.push(ConnectivityDiagnosticInfo {
                kind: "dangling_interface_port".into(),
                severity: "warning".into(),
                message: format!("interface port {} is isolated", net.name),
                objects: net.port_uuids.clone(),
            });
        }

        if net.name.starts_with("N$") && net.pins.len() > 1 {
            let mut objects: Vec<_> = net.pins.iter().map(|pin| pin.uuid).collect();
            objects.sort();
            diagnostics.push(ConnectivityDiagnosticInfo {
                kind: "anonymous_multi_pin_net".into(),
                severity: "info".into(),
                message: format!(
                    "anonymous net connects {} component pins without a label or port",
                    net.pins.len()
                ),
                objects,
            });
        }
    }

    diagnostics.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.message.cmp(&b.message))
            .then_with(|| a.objects.cmp(&b.objects))
    });
    diagnostics
}

fn hierarchy_resolution_without_unions(schematic: &Schematic) -> HierarchyResolution {
    let mut uf = UnionFind::default();
    for sheet in schematic.sheets.values() {
        for label in sheet.labels.values() {
            if is_bus_container_label(label) {
                continue;
            }
            uf.add(NodeKey {
                sheet: sheet.uuid,
                point: label.position,
            });
        }
        for port in sheet.ports.values() {
            uf.add(NodeKey {
                sheet: sheet.uuid,
                point: port.position,
            });
        }
    }
    apply_hierarchy_resolution(schematic, &mut uf)
}

fn apply_hierarchy_resolution(schematic: &Schematic, uf: &mut UnionFind) -> HierarchyResolution {
    let mut resolution = HierarchyResolution::default();

    for sheet in schematic.sheets.values() {
        let mut interface_nodes: BTreeMap<String, Vec<NodeKey>> = BTreeMap::new();
        for label in sheet.labels.values() {
            if matches!(label.kind, LabelKind::Hierarchical) && !is_bus_container_label(label) {
                interface_nodes
                    .entry(label.name.clone())
                    .or_default()
                    .push(NodeKey {
                        sheet: sheet.uuid,
                        point: label.position,
                    });
            }
        }
        for port in sheet.ports.values() {
            interface_nodes
                .entry(port.name.clone())
                .or_default()
                .push(NodeKey {
                    sheet: sheet.uuid,
                    point: port.position,
                });
        }
        for nodes in interface_nodes.values() {
            if let Some(first) = nodes.first().copied() {
                for node in nodes.iter().copied().skip(1) {
                    uf.union(first, node);
                }
            }
        }
    }

    let mut instances: Vec<_> = schematic.sheet_instances.values().collect();
    instances.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    for instance in instances {
        let Some(parent_sheet_uuid) = instance.parent_sheet else {
            continue;
        };
        let Some(parent_sheet) = schematic.sheets.get(&parent_sheet_uuid) else {
            continue;
        };
        let Some(definition) = schematic.sheet_definitions.get(&instance.definition) else {
            continue;
        };

        let parent_ports = parent_ports_for_instance(parent_sheet, instance);
        let child_labels = if definition.root_sheet.is_nil() {
            BTreeMap::new()
        } else {
            schematic
                .sheets
                .get(&definition.root_sheet)
                .map(child_hierarchical_labels_by_name)
                .unwrap_or_default()
        };

        for (name, parent_candidates) in &parent_ports {
            match child_labels.get(name).map(Vec::len).unwrap_or(0) {
                1 if parent_candidates.len() == 1 => {
                    let parent = parent_candidates[0];
                    let Some(children) = child_labels.get(name) else {
                        continue;
                    };
                    let child = children[0];
                    uf.union(
                        NodeKey {
                            sheet: parent_sheet_uuid,
                            point: parent.position,
                        },
                        NodeKey {
                            sheet: definition.root_sheet,
                            point: child.position,
                        },
                    );
                    resolution.links.push(ResolvedHierarchicalLink {
                        parent_sheet: parent_sheet_uuid,
                        child_sheet: definition.root_sheet,
                        parent_port: parent.uuid,
                        child_port: child.uuid,
                    });
                }
                0 => resolution.diagnostics.push(ConnectivityDiagnosticInfo {
                    kind: "missing_hierarchical_port_target".into(),
                    severity: "warning".into(),
                    message: format!(
                        "sheet instance {} has no matching child hierarchical target for {}",
                        instance.name, name
                    ),
                    objects: parent_candidates.iter().map(|port| port.uuid).collect(),
                }),
                _ => {
                    let mut objects: Vec<_> =
                        parent_candidates.iter().map(|port| port.uuid).collect();
                    if let Some(children) = child_labels.get(name) {
                        objects.extend(children.iter().map(|label| label.uuid));
                    }
                    objects.sort();
                    objects.dedup();
                    resolution.diagnostics.push(ConnectivityDiagnosticInfo {
                        kind: "multiply_mapped_hierarchical_port".into(),
                        severity: "warning".into(),
                        message: format!(
                            "sheet instance {} has multiple hierarchical targets for {}",
                            instance.name, name
                        ),
                        objects,
                    });
                }
            }
        }

        for (name, child_candidates) in child_labels {
            if parent_ports.contains_key(&name) {
                continue;
            }
            let mut objects: Vec<_> = child_candidates.iter().map(|label| label.uuid).collect();
            objects.sort();
            resolution.diagnostics.push(ConnectivityDiagnosticInfo {
                kind: "missing_hierarchical_port_target".into(),
                severity: "warning".into(),
                message: format!(
                    "sheet instance {} does not expose child hierarchical target {} on the parent sheet",
                    instance.name, name
                ),
                objects,
            });
        }
    }

    resolution.links.sort_by(|a, b| {
        a.parent_sheet
            .cmp(&b.parent_sheet)
            .then_with(|| a.child_sheet.cmp(&b.child_sheet))
            .then_with(|| a.parent_port.cmp(&b.parent_port))
            .then_with(|| a.child_port.cmp(&b.child_port))
    });
    resolution.diagnostics.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.message.cmp(&b.message))
            .then_with(|| a.objects.cmp(&b.objects))
    });
    resolution.diagnostics.dedup();
    resolution
}

fn parent_ports_for_instance<'a>(
    parent_sheet: &'a crate::schematic::Sheet,
    instance: &crate::schematic::SheetInstance,
) -> BTreeMap<String, Vec<&'a crate::schematic::HierarchicalPort>> {
    let mut grouped: BTreeMap<String, Vec<&crate::schematic::HierarchicalPort>> = BTreeMap::new();
    let mut port_ids = instance.ports.clone();
    port_ids.sort();
    for port_uuid in port_ids {
        if let Some(port) = parent_sheet.ports.get(&port_uuid) {
            grouped.entry(port.name.clone()).or_default().push(port);
        }
    }
    grouped
}

fn child_hierarchical_labels_by_name(
    child_sheet: &crate::schematic::Sheet,
) -> BTreeMap<String, Vec<&NetLabel>> {
    let mut grouped: BTreeMap<String, Vec<&NetLabel>> = BTreeMap::new();
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

fn point_on_wire_segment(point: Point, a: Point, b: Point) -> bool {
    if a == b {
        return point == a;
    }

    let cross = (point.y - a.y) * (b.x - a.x) - (point.x - a.x) * (b.y - a.y);
    if cross != 0 {
        return false;
    }

    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);

    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn preferred_name(labels: &[LabelRef], ports: &[PortRef]) -> Option<String> {
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

fn canonical_label_name(name: &str) -> String {
    parse_scalar_bus_member_name(name).unwrap_or_else(|| name.to_string())
}

fn is_bus_container_label(label: &NetLabel) -> bool {
    parse_bus_range_members(&label.name).is_some()
}

fn has_bus_syntax(name: &str) -> bool {
    name.contains('[') || name.contains(']')
}

fn parse_scalar_bus_member_name(name: &str) -> Option<String> {
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

fn parse_bus_range_members(name: &str) -> Option<Vec<String>> {
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
    let (start_text, end_text) = body.split_once("..")?;
    let start = start_text.trim().parse::<i32>().ok()?;
    let end = end_text.trim().parse::<i32>().ok()?;
    let step = if start <= end { 1 } else { -1 };
    let mut members = Vec::new();
    let mut index = start;
    loop {
        members.push(format!("{base}{index}"));
        if index == end {
            break;
        }
        index += step;
    }
    Some(members)
}

#[cfg(test)]
mod tests {
    #[path = "mod_tests_netinfo_basics.rs"]
    mod netinfo_basics;

    #[path = "mod_tests_diagnostics_hierarchical.rs"]
    mod diagnostics_hierarchical;
}
