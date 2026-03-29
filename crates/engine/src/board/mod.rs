use std::collections::HashMap;

use crate::ir::geometry::{LayerId, Point, Polygon};
use crate::schematic::ConnectivityDiagnosticInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

include!("board_module_surface.rs");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub uuid: Uuid,
    pub name: String,
    pub stackup: Stackup,
    pub outline: Polygon,
    pub packages: HashMap<Uuid, PlacedPackage>,
    pub pads: HashMap<Uuid, PlacedPad>,
    pub tracks: HashMap<Uuid, Track>,
    pub vias: HashMap<Uuid, Via>,
    pub zones: HashMap<Uuid, Zone>,
    pub nets: HashMap<Uuid, Net>,
    pub net_classes: HashMap<Uuid, NetClass>,
    pub rules: RuleSet,
    pub keepouts: Vec<Keepout>,
    pub dimensions: Vec<Dimension>,
    pub texts: Vec<BoardText>,
}

impl Board {
    pub fn summary(&self) -> BoardSummary {
        BoardSummary {
            name: self.name.clone(),
            layer_count: self.stackup.layers.len(),
            component_count: self.packages.len(),
            net_count: self.nets.len(),
        }
    }

    pub fn components(&self) -> Vec<ComponentInfo> {
        let mut components: Vec<_> = self
            .packages
            .values()
            .map(|package| ComponentInfo {
                uuid: package.uuid,
                package_uuid: package.package,
                reference: package.reference.clone(),
                value: package.value.clone(),
                position: package.position,
                rotation: package.rotation,
                layer: package.layer,
                locked: package.locked,
            })
            .collect();
        components.sort_by(|a, b| {
            a.reference
                .cmp(&b.reference)
                .then_with(|| a.uuid.cmp(&b.uuid))
        });
        components
    }

    pub fn net_info(&self) -> Vec<BoardNetInfo> {
        let mut infos: Vec<_> = self
            .nets
            .values()
            .map(|net| {
                let class_name = self
                    .net_classes
                    .get(&net.class)
                    .map(|class| class.name.clone())
                    .unwrap_or_else(|| "Default".to_string());
                let tracks: Vec<_> = self
                    .tracks
                    .values()
                    .filter(|track| track.net == net.uuid)
                    .collect();
                let vias = self.vias.values().filter(|via| via.net == net.uuid).count();
                let zones = self
                    .zones
                    .values()
                    .filter(|zone| zone.net == net.uuid)
                    .count();
                let pins = self.net_pins(net.uuid);
                let routed_length_nm = tracks
                    .iter()
                    .map(|track| segment_length_nm(track.from, track.to))
                    .sum();

                BoardNetInfo {
                    uuid: net.uuid,
                    name: net.name.clone(),
                    class: class_name,
                    pins,
                    tracks: tracks.len(),
                    vias,
                    zones,
                    routed_length_nm,
                    routed_pct: if tracks.is_empty() && vias == 0 && zones == 0 {
                        0.0
                    } else {
                        1.0
                    },
                }
            })
            .collect();
        infos.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
        infos
    }

    pub fn unrouted(&self) -> Vec<Airwire> {
        let mut airwires = Vec::new();

        for net in self.nets.values() {
            let pad_points = self.net_pad_points(net.uuid);
            if pad_points.len() < 2 {
                continue;
            }

            let components = self.net_connected_pin_groups(net.uuid, &pad_points);
            if components.len() < 2 {
                continue;
            }

            let mut connected = vec![false; components.len()];
            connected[0] = true;

            for _ in 1..components.len() {
                let mut best: Option<(usize, usize, usize, usize, i64)> = None;
                for (i, from_group) in components.iter().enumerate().filter(|(i, _)| connected[*i])
                {
                    for (j, to_group) in components
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| !connected[*j])
                    {
                        let (from_idx, to_idx, distance_nm) =
                            nearest_pin_pair(from_group, to_group);
                        match best {
                            Some((_, _, _, _, best_distance)) if distance_nm >= best_distance => {}
                            _ => best = Some((i, j, from_idx, to_idx, distance_nm)),
                        }
                    }
                }

                let Some((i, j, from_idx, to_idx, distance_nm)) = best else {
                    break;
                };
                connected[j] = true;
                airwires.push(Airwire {
                    net: net.uuid,
                    net_name: net.name.clone(),
                    from: NetPinRef {
                        component: components[i][from_idx].component.clone(),
                        pin: components[i][from_idx].pin.clone(),
                    },
                    to: NetPinRef {
                        component: components[j][to_idx].component.clone(),
                        pin: components[j][to_idx].pin.clone(),
                    },
                    distance_nm,
                });
            }
        }

        airwires.sort_by(|a, b| {
            a.net_name
                .cmp(&b.net_name)
                .then_with(|| a.from.component.cmp(&b.from.component))
                .then_with(|| a.from.pin.cmp(&b.from.pin))
                .then_with(|| a.to.component.cmp(&b.to.component))
                .then_with(|| a.to.pin.cmp(&b.to.pin))
        });
        airwires
    }

    pub fn stackup_info(&self) -> StackupInfo {
        let mut layers = self.stackup.layers.clone();
        layers.sort_by_key(|layer| layer.id);
        StackupInfo { layers }
    }

    pub fn diagnostics(&self) -> Vec<ConnectivityDiagnosticInfo> {
        let mut diagnostics = Vec::new();
        let unrouted_by_net: HashMap<Uuid, usize> = {
            let mut counts = HashMap::new();
            for airwire in self.unrouted() {
                *counts.entry(airwire.net).or_insert(0) += 1;
            }
            counts
        };

        for net in self.net_info() {
            if net.tracks == 0 && net.vias == 0 && net.zones == 0 {
                diagnostics.push(ConnectivityDiagnosticInfo {
                    kind: "net_without_copper".into(),
                    severity: "info".into(),
                    message: format!("board net {} has no imported copper geometry", net.name),
                    objects: vec![net.uuid],
                });
            } else if net.vias > 0 && net.tracks == 0 && net.zones == 0 {
                diagnostics.push(ConnectivityDiagnosticInfo {
                    kind: "via_only_net".into(),
                    severity: "warning".into(),
                    message: format!("board net {} is represented only by vias", net.name),
                    objects: vec![net.uuid],
                });
            } else if let Some(airwires) = unrouted_by_net.get(&net.uuid) {
                diagnostics.push(ConnectivityDiagnosticInfo {
                    kind: "partially_routed_net".into(),
                    severity: "warning".into(),
                    message: format!(
                        "board net {} still has {} unrouted connection(s)",
                        net.name, airwires
                    ),
                    objects: vec![net.uuid],
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

    fn net_pins(&self, net: Uuid) -> Vec<NetPinRef> {
        let mut pins: Vec<_> = self
            .pads
            .values()
            .filter(|pad| pad.net == Some(net))
            .filter_map(|pad| {
                let package = self.packages.get(&pad.package)?;
                Some(NetPinRef {
                    component: package.reference.clone(),
                    pin: pad.name.clone(),
                })
            })
            .collect();
        pins.sort_by(|a, b| {
            a.component
                .cmp(&b.component)
                .then_with(|| a.pin.cmp(&b.pin))
        });
        pins
    }

    fn net_pad_points(&self, net: Uuid) -> Vec<PadPoint> {
        let mut pins: Vec<_> = self
            .pads
            .values()
            .filter(|pad| pad.net == Some(net))
            .filter_map(|pad| {
                let package = self.packages.get(&pad.package)?;
                Some(PadPoint {
                    component: package.reference.clone(),
                    pin: pad.name.clone(),
                    position: pad.position,
                    layer: pad.layer,
                })
            })
            .collect();
        pins.sort_by(|a, b| {
            a.component
                .cmp(&b.component)
                .then_with(|| a.pin.cmp(&b.pin))
                .then_with(|| a.position.x.cmp(&b.position.x))
                .then_with(|| a.position.y.cmp(&b.position.y))
        });
        pins
    }

    fn net_connected_pin_groups(&self, net: Uuid, pads: &[PadPoint]) -> Vec<Vec<PadPoint>> {
        let graph = BoardNetGraph::build(self, net, pads);
        let mut pad_groups: HashMap<usize, Vec<PadPoint>> = HashMap::new();
        for (pad_idx, pad) in pads.iter().enumerate() {
            let root = graph.root_of_pad(pad_idx);
            pad_groups.entry(root).or_default().push(pad.clone());
        }

        let mut groups: Vec<_> = pad_groups.into_values().collect();
        for group in &mut groups {
            group.sort_by(|a, b| {
                a.component
                    .cmp(&b.component)
                    .then_with(|| a.pin.cmp(&b.pin))
                    .then_with(|| a.position.x.cmp(&b.position.x))
                    .then_with(|| a.position.y.cmp(&b.position.y))
            });
        }
        groups.sort_by(|a, b| {
            a[0].component
                .cmp(&b[0].component)
                .then_with(|| a[0].pin.cmp(&b[0].pin))
        });
        groups
    }
}

#[cfg(test)]
mod tests;
