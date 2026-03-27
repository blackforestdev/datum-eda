use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::{LayerId, Point, Polygon};
use crate::rules::ast::Rule;
use crate::schematic::ConnectivityDiagnosticInfo;

pub type RuleSet = Vec<Rule>;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPackage {
    pub uuid: Uuid,
    pub part: Uuid,
    pub package: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPad {
    pub uuid: Uuid,
    pub package: Uuid,
    pub name: String,
    pub net: Option<Uuid>,
    pub position: Point,
    pub layer: LayerId,
    #[serde(default)]
    pub diameter: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub uuid: Uuid,
    pub net: Uuid,
    pub from: Point,
    pub to: Point,
    pub width: i64,
    pub layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Via {
    pub uuid: Uuid,
    pub net: Uuid,
    pub position: Point,
    pub drill: i64,
    pub diameter: i64,
    pub from_layer: LayerId,
    pub to_layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub uuid: Uuid,
    pub net: Uuid,
    pub polygon: Polygon,
    pub layer: LayerId,
    pub priority: u32,
    pub thermal_relief: bool,
    pub thermal_gap: i64,
    pub thermal_spoke_width: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub class: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetClass {
    pub uuid: Uuid,
    pub name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stackup {
    pub layers: Vec<StackupLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackupLayer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: StackupLayerType,
    pub thickness_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackupLayerType {
    Copper,
    Dielectric,
    SolderMask,
    Silkscreen,
    Paste,
    Mechanical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keepout {
    pub uuid: Uuid,
    pub polygon: Polygon,
    pub layers: Vec<LayerId>,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub uuid: Uuid,
    pub from: Point,
    pub to: Point,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackupInfo {
    pub layers: Vec<StackupLayer>,
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

fn segment_length_nm(from: Point, to: Point) -> i64 {
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    (dx.hypot(dy).round()) as i64
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PadPoint {
    component: String,
    pin: String,
    position: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Anchor {
    point: Point,
    layer: LayerId,
}

struct BoardNetGraph {
    parents: Vec<usize>,
}

impl BoardNetGraph {
    fn build(board: &Board, net: Uuid, pads: &[PadPoint]) -> Self {
        let mut anchors: Vec<Anchor> = pads
            .iter()
            .map(|pad| Anchor {
                point: pad.position,
                layer: pad.layer,
            })
            .collect();
        let pad_count = anchors.len();

        for track in board.tracks.values().filter(|track| track.net == net) {
            anchors.push(Anchor {
                point: track.from,
                layer: track.layer,
            });
            anchors.push(Anchor {
                point: track.to,
                layer: track.layer,
            });
        }

        let via_start = anchors.len();
        for via in board.vias.values().filter(|via| via.net == net) {
            anchors.push(Anchor {
                point: via.position,
                layer: via.from_layer,
            });
            anchors.push(Anchor {
                point: via.position,
                layer: via.to_layer,
            });
        }

        let mut uf = UnionFind::new(anchors.len());

        let mut index_by_anchor: HashMap<Anchor, Vec<usize>> = HashMap::new();
        for (idx, anchor) in anchors.iter().copied().enumerate() {
            index_by_anchor.entry(anchor).or_default().push(idx);
        }
        for indices in index_by_anchor.values() {
            if let Some((&first, rest)) = indices.split_first() {
                for &idx in rest {
                    uf.union(first, idx);
                }
            }
        }

        let mut cursor = pad_count;
        for _track in board.tracks.values().filter(|track| track.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        cursor = via_start;
        for _via in board.vias.values().filter(|via| via.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        for zone in board.zones.values().filter(|zone| zone.net == net) {
            let contained: Vec<_> = anchors
                .iter()
                .enumerate()
                .filter(|(_, anchor)| {
                    anchor.layer == zone.layer && point_in_polygon(anchor.point, &zone.polygon)
                })
                .map(|(idx, _)| idx)
                .collect();
            if let Some((&first, rest)) = contained.split_first() {
                for &idx in rest {
                    uf.union(first, idx);
                }
            }
        }

        Self {
            parents: (0..pad_count).map(|idx| uf.find(idx)).collect(),
        }
    }

    fn root_of_pad(&self, pad_idx: usize) -> usize {
        self.parents[pad_idx]
    }
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, idx: usize) -> usize {
        if self.parent[idx] != idx {
            let root = self.find(self.parent[idx]);
            self.parent[idx] = root;
        }
        self.parent[idx]
    }

    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent[root_b] = root_a;
        }
    }
}

fn point_in_polygon(point: Point, polygon: &Polygon) -> bool {
    let Some(bounds) = polygon.bounding_box() else {
        return false;
    };
    if !bounds.contains(&point) {
        return false;
    }

    let vertices = &polygon.vertices;
    if vertices.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let xi = vertices[i].x as f64;
        let yi = vertices[i].y as f64;
        let xj = vertices[j].x as f64;
        let yj = vertices[j].y as f64;
        let px = point.x as f64;
        let py = point.y as f64;

        let intersects =
            ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / ((yj - yi).max(1.0)) + xi);
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn nearest_pin_pair(from: &[PadPoint], to: &[PadPoint]) -> (usize, usize, i64) {
    let mut best = (0usize, 0usize, i64::MAX);
    for (i, from_pin) in from.iter().enumerate() {
        for (j, to_pin) in to.iter().enumerate() {
            let distance_nm = segment_length_nm(from_pin.position, to_pin.position);
            if distance_nm < best.2 {
                best = (i, j, distance_nm);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    #[path = "mod_tests_queries_and_netinfo.rs"]
    mod queries_and_netinfo;

    #[path = "mod_tests_diagnostics_and_unrouted.rs"]
    mod diagnostics_and_unrouted;
}
