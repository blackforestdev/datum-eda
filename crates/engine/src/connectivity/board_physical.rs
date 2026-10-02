//! Complete source-referenced board components. No semantic identity allocation.
use crate::board::{
    Net, NetClass, PlacedPad, Stackup, Track, Via, Zone,
    nominal_geometry::{DistanceBoundary, GeometryError, Rational},
    occupied_copper as copper,
    track_contact::tracks_within,
};
use crate::ir::geometry::{LayerId, Point, Polygon};
use crate::substrate::{
    DesignModel, ElectricalOccurrence, ModelRevision, SelectionSourceBasis, ZoneFillState,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
mod zone_regions;
pub(crate) use zone_regions::successor;
pub use zone_regions::{ZoneClearReason, ZoneRegionQualifier, ZoneRegionSuccessor};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BoardCopperSource {
    pub uuid: Uuid,
    pub stackup: Stackup,
    pub pads: BTreeMap<Uuid, PlacedPad>,
    pub tracks: BTreeMap<Uuid, Track>,
    pub vias: BTreeMap<Uuid, Via>,
    pub zones: BTreeMap<Uuid, Zone>,
    pub nets: BTreeMap<Uuid, Net>,
    #[serde(default)]
    pub net_classes: BTreeMap<Uuid, NetClass>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalQueryFailure {
    InvalidQualifier,
    AbsentSource {
        source_id: Uuid,
    },
    UnavailableAssignment {
        source_id: Uuid,
    },
    UnavailableGeometry {
        sources: BTreeSet<Uuid>,
        reason: String,
    },
    UnavailableFill {
        source_id: Uuid,
        state: Option<ZoneFillState>,
    },
    AmbiguousOrigin {
        source_id: Uuid,
    },
    OriginOutsideCopper {
        source_id: Uuid,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossNetContact {
    pub left: ElectricalOccurrence,
    pub right: ElectricalOccurrence,
    pub left_net: Uuid,
    pub right_net: Uuid,
    pub layers: BTreeSet<LayerId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneCopperProjection {
    pub zone_id: Uuid,
    pub layer: LayerId,
    pub polygons: Vec<Polygon>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardRunMembership {
    pub project_id: Uuid,
    pub revision: ModelRevision,
    pub source_basis: Vec<SelectionSourceBasis>,
    pub origin: ElectricalOccurrence,
    pub board_net: Uuid,
    pub members: BTreeSet<ElectricalOccurrence>,
    pub copper_layers: BTreeMap<ElectricalOccurrence, BTreeSet<LayerId>>,
    pub zone_copper: Vec<ZoneCopperProjection>,
    pub zone_regions: Vec<ZoneRegionQualifier>,
    pub contact_evidence: Result<Vec<CrossNetContact>, PhysicalQueryFailure>,
}
#[derive(Clone)]
enum Shape {
    Track(Track),
    Pad(PlacedPad),
    Via(Via),
    Fill(Polygon),
}
#[derive(Clone)]
struct Node {
    reference: ElectricalOccurrence,
    net: Option<Uuid>,
    layers: Vec<LayerId>,
    shape: Shape,
}
#[derive(Clone)]
struct Unavailable {
    net: Option<Uuid>,
    layers: Option<Vec<LayerId>>,
    failure: PhysicalQueryFailure,
}
fn reference(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn geometry(
    sources: impl IntoIterator<Item = Uuid>,
    error: impl std::fmt::Debug,
) -> PhysicalQueryFailure {
    PhysicalQueryFailure::UnavailableGeometry {
        sources: sources.into_iter().collect(),
        reason: format!("{error:?}"),
    }
}
fn layers_match(a: &[LayerId], b: &[LayerId]) -> bool {
    a.iter().any(|id| b.contains(id))
}
fn contact(a: &Node, b: &Node) -> Result<bool, GeometryError> {
    if !layers_match(&a.layers, &b.layers) {
        return Ok(false);
    }
    let boundary = DistanceBoundary::Inclusive;
    match (&a.shape, &b.shape) {
        (Shape::Track(a), Shape::Track(b)) => tracks_within(a, b, 0, boundary),
        (Shape::Track(a), Shape::Pad(b)) | (Shape::Pad(b), Shape::Track(a)) => {
            copper::track_pad_within(a, b, 0, boundary)
        }
        (Shape::Track(a), Shape::Via(b)) | (Shape::Via(b), Shape::Track(a)) => {
            copper::track_via_within(a, b, 0, boundary)
        }
        (Shape::Track(a), Shape::Fill(b)) | (Shape::Fill(b), Shape::Track(a)) => {
            copper::track_polygon_within(a, b, 0, boundary)
        }
        (Shape::Pad(a), Shape::Pad(b)) => copper::pads_within(a, b, 0, boundary),
        (Shape::Pad(a), Shape::Via(b)) | (Shape::Via(b), Shape::Pad(a)) => {
            copper::pad_via_within(a, b, 0, boundary)
        }
        (Shape::Pad(a), Shape::Fill(b)) | (Shape::Fill(b), Shape::Pad(a)) => {
            copper::pad_polygon_within(a, b, 0, boundary)
        }
        (Shape::Via(a), Shape::Via(b)) => copper::vias_within(a, b, 0, boundary),
        (Shape::Via(a), Shape::Fill(b)) | (Shape::Fill(b), Shape::Via(a)) => {
            copper::via_polygon_within(a, b, 0, boundary)
        }
        (Shape::Fill(a), Shape::Fill(b)) => copper::polygons_within(a, b, 0, boundary),
    }
}
fn occupied(node: &Node, p: Point) -> Result<bool, GeometryError> {
    match &node.shape {
        Shape::Track(t) => {
            copper::track_point_within(t, p, Rational::integer(0), DistanceBoundary::Inclusive)
        }
        Shape::Pad(pad) => copper::pad_contains_point(pad, p),
        Shape::Via(via) => copper::via_contains_point(via, p),
        Shape::Fill(poly) => copper::polygon_contains_point(poly, p),
    }
}
struct Graph {
    nodes: Vec<Node>,
    unavailable: Vec<Unavailable>,
    groups: Vec<Vec<usize>>,
}
fn current_fill(model: &DesignModel, id: Uuid) -> Option<&crate::substrate::ZoneFill> {
    model.current_zone_fill(id)
}
impl Graph {
    fn build(
        source: &BoardCopperSource,
        model: &DesignModel,
        target: Uuid,
    ) -> Result<Self, PhysicalQueryFailure> {
        if target.is_nil() || !source.nets.get(&target).is_some_and(|n| n.uuid == target) {
            return Err(PhysicalQueryFailure::UnavailableAssignment { source_id: target });
        }
        let mut graph = Self {
            nodes: vec![],
            unavailable: vec![],
            groups: vec![],
        };
        for (id, track) in &source.tracks {
            if *id != track.uuid {
                return Err(geometry([*id], "Track source key mismatch"));
            }
            let layers = copper::track_layers(&source.stackup, track);
            let validation = copper::validate_track(track);
            graph.push(
                reference("tracks", *id),
                Some(track.net),
                Shape::Track(track.clone()),
                layers,
                validation,
            );
        }
        for (id, pad) in &source.pads {
            if *id != pad.uuid {
                return Err(geometry([*id], "Pad source key mismatch"));
            }
            match copper::pad_layer_groups(&source.stackup, pad) {
                Ok(groups) => {
                    for layers in groups {
                        graph.push(
                            reference("pads", *id),
                            pad.net,
                            Shape::Pad(pad.clone()),
                            Ok(layers),
                            copper::validate_pad(pad),
                        );
                    }
                }
                Err(reason) => graph.push(
                    reference("pads", *id),
                    pad.net,
                    Shape::Pad(pad.clone()),
                    copper::pad_layers(&source.stackup, pad),
                    Err(reason),
                ),
            }
        }
        for (id, via) in &source.vias {
            if *id != via.uuid {
                return Err(geometry([*id], "Via source key mismatch"));
            }
            graph.push(
                reference("vias", *id),
                Some(via.net),
                Shape::Via(via.clone()),
                copper::via_layers(&source.stackup, via),
                copper::validate_via(via),
            );
        }
        for (id, zone) in &source.zones {
            if *id != zone.uuid {
                return Err(geometry([*id], "Zone source key mismatch"));
            }
            let layers =
                copper::conductive_layer(&source.stackup, zone.layer).map(|_| vec![zone.layer]);
            let current = current_fill(model, *id);
            let Some(fill) = current else {
                graph.unavailable.push(Unavailable {
                    net: Some(zone.net),
                    layers: layers.ok(),
                    failure: PhysicalQueryFailure::UnavailableFill {
                        source_id: *id,
                        state: model.zone_fills.get(id).map(|f| f.state),
                    },
                });
                continue;
            };
            for polygon in &fill.islands {
                graph.push(
                    reference("zones", *id),
                    Some(zone.net),
                    Shape::Fill(polygon.clone()),
                    layers.clone(),
                    copper::validate_polygon(polygon),
                );
            }
        }
        for unavailable in &graph.unavailable {
            if unavailable.net == Some(target) {
                return Err(unavailable.failure.clone());
            }
        }
        let target_nodes: Vec<_> = graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.net == Some(target))
            .map(|(i, _)| i)
            .collect();
        let mut parent: Vec<_> = (0..graph.nodes.len()).collect();
        fn root(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        for (offset, a) in target_nodes.iter().enumerate() {
            for b in &target_nodes[offset + 1..] {
                if contact(&graph.nodes[*a], &graph.nodes[*b]).map_err(|e| {
                    geometry(
                        [
                            graph.nodes[*a].reference.source_id,
                            graph.nodes[*b].reference.source_id,
                        ],
                        e,
                    )
                })? {
                    let ar = root(&mut parent, *a);
                    let br = root(&mut parent, *b);
                    parent[br] = ar;
                }
            }
        }
        let mut groups = BTreeMap::<usize, Vec<usize>>::new();
        for i in target_nodes {
            groups.entry(root(&mut parent, i)).or_default().push(i);
        }
        graph.groups = groups.into_values().collect();
        Ok(graph)
    }
    fn push(
        &mut self,
        reference: ElectricalOccurrence,
        net: Option<Uuid>,
        shape: Shape,
        layers: Result<Vec<LayerId>, GeometryError>,
        validation: Result<(), GeometryError>,
    ) {
        match (layers, validation) {
            (Ok(layers), Ok(())) => self.nodes.push(Node {
                reference,
                net,
                layers,
                shape,
            }),
            (layers, error) => {
                let failure = geometry(
                    [reference.source_id],
                    error
                        .err()
                        .or_else(|| layers.as_ref().err().copied())
                        .expect("failed source"),
                );
                self.unavailable.push(Unavailable {
                    net,
                    layers: layers.ok(),
                    failure,
                });
            }
        }
    }
    fn contacts(
        &self,
        group: &[usize],
        target: Uuid,
        nets: &BTreeMap<Uuid, Net>,
    ) -> Result<Vec<CrossNetContact>, PhysicalQueryFailure> {
        let mut result = BTreeMap::new();
        let mut affected_layers = BTreeSet::<LayerId>::new();
        for i in group {
            affected_layers.extend(&self.nodes[*i].layers);
        }
        for u in &self.unavailable {
            if u.layers
                .as_ref()
                .is_none_or(|v| v.iter().any(|l| affected_layers.contains(l)))
            {
                return Err(u.failure.clone());
            }
        }
        for i in group {
            for peer in &self.nodes {
                if peer.net == Some(target) {
                    continue;
                }
                if !contact(&self.nodes[*i], peer).map_err(|e| {
                    geometry(
                        [self.nodes[*i].reference.source_id, peer.reference.source_id],
                        e,
                    )
                })? {
                    continue;
                }
                let Some(net) = peer
                    .net
                    .filter(|n| !n.is_nil() && nets.get(n).is_some_and(|v| v.uuid == *n))
                else {
                    return Err(PhysicalQueryFailure::UnavailableAssignment {
                        source_id: peer.reference.source_id,
                    });
                };
                let left = self.nodes[*i].reference.clone();
                let right = peer.reference.clone();
                let entry = result
                    .entry((left.clone(), right.clone()))
                    .or_insert_with(|| CrossNetContact {
                        left,
                        right,
                        left_net: target,
                        right_net: net,
                        layers: BTreeSet::new(),
                    });
                entry.layers.extend(
                    self.nodes[*i]
                        .layers
                        .iter()
                        .filter(|l| peer.layers.contains(l)),
                );
            }
        }
        Ok(result.into_values().collect())
    }
}
fn assignment(
    source: &BoardCopperSource,
    origin: &ElectricalOccurrence,
) -> Result<Uuid, PhysicalQueryFailure> {
    if !origin.instance_path.is_empty() {
        return Err(geometry(
            [origin.source_id],
            "board occurrence has schematic path",
        ));
    }
    let net = match origin.class.as_str() {
        "tracks" => source.tracks.get(&origin.source_id).map(|o| Some(o.net)),
        "pads" => source.pads.get(&origin.source_id).map(|o| o.net),
        "vias" => source.vias.get(&origin.source_id).map(|o| Some(o.net)),
        "zones" => source.zones.get(&origin.source_id).map(|o| Some(o.net)),
        _ => None,
    }
    .ok_or(PhysicalQueryFailure::AbsentSource {
        source_id: origin.source_id,
    })?;
    net.filter(|n| !n.is_nil() && source.nets.contains_key(n))
        .ok_or(PhysicalQueryFailure::UnavailableAssignment {
            source_id: origin.source_id,
        })
}
pub(crate) fn board_run(
    source: &BoardCopperSource,
    model: &DesignModel,
    origin: &ElectricalOccurrence,
    hit: Option<Point>,
    source_basis: &[SelectionSourceBasis],
) -> Result<BoardRunMembership, PhysicalQueryFailure> {
    let net = assignment(source, origin)?;
    let graph = Graph::build(source, model, net)?;
    let mut choices = vec![];
    for (index, group) in graph.groups.iter().enumerate() {
        let mut eligible = false;
        for i in group {
            let node = &graph.nodes[*i];
            if node.reference == *origin {
                eligible |= match hit {
                    Some(p) => occupied(node, p).map_err(|e| geometry([origin.source_id], e))?,
                    None => true,
                };
            }
        }
        if eligible {
            choices.push(index)
        }
    }
    if choices.len() > 1 {
        return Err(PhysicalQueryFailure::AmbiguousOrigin {
            source_id: origin.source_id,
        });
    }
    let Some(index) = choices.first() else {
        return Err(PhysicalQueryFailure::OriginOutsideCopper {
            source_id: origin.source_id,
        });
    };
    membership(source, model, origin, &graph, *index, source_basis)
}
fn membership(
    source: &BoardCopperSource,
    model: &DesignModel,
    origin: &ElectricalOccurrence,
    graph: &Graph,
    index: usize,
    source_basis: &[SelectionSourceBasis],
) -> Result<BoardRunMembership, PhysicalQueryFailure> {
    let net = assignment(source, origin)?;
    let group = &graph.groups[index];
    let mut members = BTreeSet::new();
    let mut copper_layers = BTreeMap::<ElectricalOccurrence, BTreeSet<LayerId>>::new();
    let mut regions = BTreeMap::<Uuid, ZoneCopperProjection>::new();
    for i in group {
        let node = &graph.nodes[*i];
        members.insert(node.reference.clone());
        copper_layers
            .entry(node.reference.clone())
            .or_default()
            .extend(node.layers.iter().copied());
        if let Shape::Fill(polygon) = &node.shape {
            regions
                .entry(node.reference.source_id)
                .or_insert_with(|| ZoneCopperProjection {
                    zone_id: node.reference.source_id,
                    layer: node.layers[0],
                    polygons: vec![],
                })
                .polygons
                .push(polygon.clone());
        }
    }
    let contacts = graph.contacts(group, net, &source.nets);
    if let Err(PhysicalQueryFailure::UnavailableAssignment { source_id }) = &contacts {
        return Err(PhysicalQueryFailure::UnavailableAssignment {
            source_id: *source_id,
        });
    }
    Ok(BoardRunMembership {
        project_id: model.project.project_id,
        revision: model.model_revision.clone(),
        source_basis: source_basis.to_vec(),
        origin: origin.clone(),
        board_net: net,
        members,
        copper_layers,
        zone_regions: regions
            .keys()
            .map(|id| zone_regions::qualifier(model, source_basis, *id, index))
            .collect(),
        zone_copper: regions.into_values().collect(),
        contact_evidence: contacts,
    })
}

/// Native checking consumes these same complete physical groups; no second graph.
pub(crate) struct BoardNetComponents {
    pub components: Vec<BTreeSet<ElectricalOccurrence>>,
    pub contacts: Result<Vec<CrossNetContact>, PhysicalQueryFailure>,
}
pub(crate) fn board_net_components(
    source: &BoardCopperSource,
    model: &DesignModel,
    net: Uuid,
) -> Result<BoardNetComponents, PhysicalQueryFailure> {
    let graph = Graph::build(source, model, net)?;
    let components = graph
        .groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|i| graph.nodes[*i].reference.clone())
                .collect()
        })
        .collect();
    let mut contacts = BTreeMap::new();
    for group in &graph.groups {
        match graph.contacts(group, net, &source.nets) {
            Ok(values) => {
                for value in values {
                    let key = (value.left.clone(), value.right.clone());
                    contacts
                        .entry(key)
                        .and_modify(|old: &mut CrossNetContact| old.layers.extend(&value.layers))
                        .or_insert(value);
                }
            }
            Err(failure) => {
                return Ok(BoardNetComponents {
                    components,
                    contacts: Err(failure),
                });
            }
        }
    }
    Ok(BoardNetComponents {
        components,
        contacts: Ok(contacts.into_values().collect()),
    })
}
