use std::collections::HashMap;

use crate::ir::geometry::{Point, Polygon};
use crate::schematic::ConnectivityDiagnosticInfo;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod board_info;
mod board_root_exports;
mod board_types;
mod dimension;
mod net_graph;
mod pad;
mod polygon;
mod route_surface;
mod rule_set;
mod stackup;
mod text;
use net_graph::{BoardNetGraph, PadPoint, nearest_pin_pair, segment_length_nm};

pub use board_root_exports::*;

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
