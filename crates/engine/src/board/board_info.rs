use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Board, nearest_pin_pair, segment_length_nm};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardNetInfo {
    pub uuid: Uuid,
    pub name: String,
    pub class: String,
    pub pins: Vec<NetPinRef>,
    pub tracks: usize,
    pub vias: usize,
    pub zones: usize,
    pub routed_length_nm: i64,
    pub routed_pct: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub uuid: Uuid,
    pub package_uuid: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetPinRef {
    pub component: String,
    pub pin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Airwire {
    pub net: Uuid,
    pub net_name: String,
    pub from: NetPinRef,
    pub to: NetPinRef,
    pub distance_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSummary {
    pub name: String,
    pub layer_count: usize,
    pub component_count: usize,
    pub net_count: usize,
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
}
