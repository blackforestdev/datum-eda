use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use uuid::Uuid;

use crate::ir::geometry::Point;
use crate::schematic::{
    ConnectivityDiagnosticInfo, LabelKind, NetPinRef, Schematic, SchematicNetInfo,
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

    let mut point_groups: HashMap<NodeKey, NetAggregate> = HashMap::new();
    let mut global_label_groups_by_name: BTreeMap<String, Vec<NodeKey>> = BTreeMap::new();
    let mut interface_groups_by_name: BTreeMap<String, Vec<NodeKey>> = BTreeMap::new();

    for sheet in schematic.sheets.values() {
        for label in sheet.labels.values() {
            let node = NodeKey {
                sheet: sheet.uuid,
                point: label.position,
            };
            let root = uf.find(node);
            point_groups.entry(root).or_default().labels.push(LabelRef {
                name: label.name.clone(),
                kind: label.kind.clone(),
            });
            point_groups
                .entry(root)
                .or_default()
                .sheets
                .insert(sheet.name.clone());

            if matches!(label.kind, LabelKind::Global) {
                global_label_groups_by_name
                    .entry(label.name.clone())
                    .or_default()
                    .push(root);
            } else if matches!(label.kind, LabelKind::Hierarchical) {
                interface_groups_by_name
                    .entry(label.name.clone())
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
            interface_groups_by_name
                .entry(port.name.clone())
                .or_default()
                .push(root);
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

    // Full instance-aware hierarchical resolution is deferred. For M1, merge
    // hierarchical labels and sheet ports by matching interface name.
    for (name, roots) in &interface_groups_by_name {
        let merge_key = format!("interface:{name}");
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
    let mut consumed_interface_roots = HashSet::new();
    for roots in interface_groups_by_name.values() {
        for root in roots {
            consumed_interface_roots.insert(*root);
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
        if consumed_interface_roots.contains(&root)
            && (group
                .labels
                .iter()
                .any(|label| matches!(label.kind, LabelKind::Hierarchical))
                || !group.ports.is_empty())
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
    let mut diagnostics = Vec::new();

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

#[cfg(test)]
mod tests {
    #[path = "mod_tests_netinfo_basics.rs"]
    mod netinfo_basics;

    #[path = "mod_tests_diagnostics_hierarchical.rs"]
    mod diagnostics_hierarchical;
}
