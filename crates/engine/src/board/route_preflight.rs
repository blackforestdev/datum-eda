use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, NetClass, PlacedPad, StackupLayer, Track, Via, Zone,
    polygon::{
        point_in_polygon, polygon_escapes_polygon, polygons_intersect, segment_escapes_polygon,
        segment_intersects_polygon,
    },
    StackupLayerType,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePreflightStatus {
    PreflightReady,
    BlockedByAuthoredObstacle,
    InsufficientAuthoredInputs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePreflightObstacleKind {
    KeepoutConflict,
    ForeignTrack,
    ForeignVia,
    ForeignZone,
    OutsideOutline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightNetClassFacts {
    pub uuid: Uuid,
    pub name: String,
    pub clearance_nm: i64,
    pub track_width_nm: i64,
    pub via_drill_nm: i64,
    pub via_diameter_nm: i64,
    pub diffpair_width_nm: i64,
    pub diffpair_gap_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightAnchor {
    pub pad_uuid: Uuid,
    pub owner_uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightObstacle {
    pub kind: RoutePreflightObstacleKind,
    pub object_uuid: Uuid,
    pub layer: Option<LayerId>,
    pub net_uuid: Option<Uuid>,
    pub net_name: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightConstraintFacts {
    pub net_class_present: bool,
    pub layer_filter_present: bool,
    pub net_class: Option<RoutePreflightNetClassFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightSummary {
    pub anchor_count: usize,
    pub candidate_copper_layer_count: usize,
    pub target_track_count: usize,
    pub target_via_count: usize,
    pub target_zone_count: usize,
    pub keepout_conflict_count: usize,
    pub foreign_track_count: usize,
    pub foreign_via_count: usize,
    pub foreign_zone_count: usize,
    pub outside_outline_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePreflightReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePreflightStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub net_class_uuid: Uuid,
    pub persisted_constraints: RoutePreflightConstraintFacts,
    pub anchors: Vec<RoutePreflightAnchor>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePreflightSummary,
    pub keepout_conflicts: Vec<RoutePreflightObstacle>,
    pub foreign_obstacles: Vec<RoutePreflightObstacle>,
    pub outside_outline_conflicts: Vec<RoutePreflightObstacle>,
}

impl Board {
    pub fn route_preflight(&self, net_uuid: Uuid) -> Option<RoutePreflightReport> {
        let net = self.nets.get(&net_uuid)?.clone();

        let net_class = self.net_classes.get(&net.class).map(net_class_facts);
        let persisted_constraints = RoutePreflightConstraintFacts {
            net_class_present: net_class.is_some(),
            layer_filter_present: false,
            net_class,
        };

        let anchors = anchors_for_net(self, net_uuid);
        let candidate_copper_layers = candidate_copper_layers(self);
        let target_tracks = tracks_for_net(self, net_uuid);
        let target_vias = vias_for_net(self, net_uuid);
        let target_zones = zones_for_net(self, net_uuid);
        let keepout_conflicts =
            keepout_conflicts(self, &anchors, &target_tracks, &target_vias, &target_zones);
        let foreign_obstacles = foreign_obstacles(self, net_uuid);
        let outside_outline_conflicts =
            outside_outline_conflicts(self, &anchors, &target_tracks, &target_vias, &target_zones);

        let status = if anchors.len() < 2 || candidate_copper_layers.is_empty() {
            RoutePreflightStatus::InsufficientAuthoredInputs
        } else if !keepout_conflicts.is_empty() || !outside_outline_conflicts.is_empty() {
            RoutePreflightStatus::BlockedByAuthoredObstacle
        } else {
            RoutePreflightStatus::PreflightReady
        };

        let summary = RoutePreflightSummary {
            anchor_count: anchors.len(),
            candidate_copper_layer_count: candidate_copper_layers.len(),
            target_track_count: target_tracks.len(),
            target_via_count: target_vias.len(),
            target_zone_count: target_zones.len(),
            keepout_conflict_count: keepout_conflicts.len(),
            foreign_track_count: foreign_obstacles
                .iter()
                .filter(|entry| matches!(entry.kind, RoutePreflightObstacleKind::ForeignTrack))
                .count(),
            foreign_via_count: foreign_obstacles
                .iter()
                .filter(|entry| matches!(entry.kind, RoutePreflightObstacleKind::ForeignVia))
                .count(),
            foreign_zone_count: foreign_obstacles
                .iter()
                .filter(|entry| matches!(entry.kind, RoutePreflightObstacleKind::ForeignZone))
                .count(),
            outside_outline_count: outside_outline_conflicts.len(),
        };

        Some(RoutePreflightReport {
            contract: "m5_route_preflight_v1".to_string(),
            persisted_native_board_state_only: true,
            status,
            net_uuid: net.uuid,
            net_name: net.name,
            net_class_uuid: net.class,
            persisted_constraints,
            anchors,
            candidate_copper_layers,
            summary,
            keepout_conflicts,
            foreign_obstacles,
            outside_outline_conflicts,
        })
    }
}

fn net_class_facts(net_class: &NetClass) -> RoutePreflightNetClassFacts {
    RoutePreflightNetClassFacts {
        uuid: net_class.uuid,
        name: net_class.name.clone(),
        clearance_nm: net_class.clearance,
        track_width_nm: net_class.track_width,
        via_drill_nm: net_class.via_drill,
        via_diameter_nm: net_class.via_diameter,
        diffpair_width_nm: net_class.diffpair_width,
        diffpair_gap_nm: net_class.diffpair_gap,
    }
}

fn anchors_for_net(board: &Board, net_uuid: Uuid) -> Vec<RoutePreflightAnchor> {
    let mut anchors = board
        .pads
        .values()
        .filter(|pad| pad.net == Some(net_uuid))
        .map(anchor_from_pad)
        .collect::<Vec<_>>();
    anchors.sort_by(|a, b| {
        a.owner_uuid
            .cmp(&b.owner_uuid)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.pad_uuid.cmp(&b.pad_uuid))
    });
    anchors
}

fn anchor_from_pad(pad: &PlacedPad) -> RoutePreflightAnchor {
    RoutePreflightAnchor {
        pad_uuid: pad.uuid,
        owner_uuid: pad.package,
        name: pad.name.clone(),
        position: pad.position,
        layer: pad.layer,
    }
}

fn candidate_copper_layers(board: &Board) -> Vec<StackupLayer> {
    let mut layers = board
        .stackup
        .layers
        .iter()
        .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
        .cloned()
        .collect::<Vec<_>>();
    layers.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));
    layers
}

fn tracks_for_net(board: &Board, net_uuid: Uuid) -> Vec<Track> {
    let mut tracks = board
        .tracks
        .values()
        .filter(|track| track.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    tracks
}

fn vias_for_net(board: &Board, net_uuid: Uuid) -> Vec<Via> {
    let mut vias = board
        .vias
        .values()
        .filter(|via| via.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    vias
}

fn zones_for_net(board: &Board, net_uuid: Uuid) -> Vec<Zone> {
    let mut zones = board
        .zones
        .values()
        .filter(|zone| zone.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    zones
}

fn keepout_conflicts(
    board: &Board,
    anchors: &[RoutePreflightAnchor],
    target_tracks: &[Track],
    target_vias: &[Via],
    target_zones: &[Zone],
) -> Vec<RoutePreflightObstacle> {
    let mut conflicts = Vec::new();
    let mut keepouts = board.keepouts.clone();
    keepouts.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.uuid.cmp(&b.uuid)));

    for keepout in keepouts {
        for anchor in anchors {
            if keepout.layers.contains(&anchor.layer)
                && point_in_polygon(anchor.position, &keepout.polygon)
            {
                conflicts.push(RoutePreflightObstacle {
                    kind: RoutePreflightObstacleKind::KeepoutConflict,
                    object_uuid: keepout.uuid,
                    layer: Some(anchor.layer),
                    net_uuid: None,
                    net_name: None,
                    reason: format!(
                        "anchor {} on layer {} lies inside keepout {}",
                        anchor.name, anchor.layer, keepout.kind
                    ),
                });
            }
        }

        for track in target_tracks {
            if keepout.layers.contains(&track.layer)
                && segment_intersects_polygon(track.from, track.to, &keepout.polygon)
            {
                conflicts.push(RoutePreflightObstacle {
                    kind: RoutePreflightObstacleKind::KeepoutConflict,
                    object_uuid: keepout.uuid,
                    layer: Some(track.layer),
                    net_uuid: None,
                    net_name: None,
                    reason: format!(
                        "target track {} has an endpoint inside keepout {}",
                        track.uuid, keepout.kind
                    ),
                });
            }
        }

        for via in target_vias {
            if (keepout.layers.contains(&via.from_layer) || keepout.layers.contains(&via.to_layer))
                && point_in_polygon(via.position, &keepout.polygon)
            {
                conflicts.push(RoutePreflightObstacle {
                    kind: RoutePreflightObstacleKind::KeepoutConflict,
                    object_uuid: keepout.uuid,
                    layer: None,
                    net_uuid: None,
                    net_name: None,
                    reason: format!(
                        "target via {} lies inside keepout {}",
                        via.uuid, keepout.kind
                    ),
                });
            }
        }

        for zone in target_zones {
            if keepout.layers.contains(&zone.layer)
                && polygons_intersect(&zone.polygon, &keepout.polygon)
            {
                conflicts.push(RoutePreflightObstacle {
                    kind: RoutePreflightObstacleKind::KeepoutConflict,
                    object_uuid: keepout.uuid,
                    layer: Some(zone.layer),
                    net_uuid: None,
                    net_name: None,
                    reason: format!(
                        "target zone {} has a vertex inside keepout {}",
                        zone.uuid, keepout.kind
                    ),
                });
            }
        }
    }

    conflicts.sort_by(|a, b| {
        a.object_uuid
            .cmp(&b.object_uuid)
            .then_with(|| a.reason.cmp(&b.reason))
    });
    conflicts
}

fn foreign_obstacles(board: &Board, target_net_uuid: Uuid) -> Vec<RoutePreflightObstacle> {
    let mut obstacles = Vec::new();

    let mut tracks = board
        .tracks
        .values()
        .filter(|track| track.net != target_net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for track in tracks {
        let net_name = board.nets.get(&track.net).map(|net| net.name.clone());
        obstacles.push(RoutePreflightObstacle {
            kind: RoutePreflightObstacleKind::ForeignTrack,
            object_uuid: track.uuid,
            layer: Some(track.layer),
            net_uuid: Some(track.net),
            net_name,
            reason: format!("foreign track {} on layer {}", track.uuid, track.layer),
        });
    }

    let mut vias = board
        .vias
        .values()
        .filter(|via| via.net != target_net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for via in vias {
        let net_name = board.nets.get(&via.net).map(|net| net.name.clone());
        obstacles.push(RoutePreflightObstacle {
            kind: RoutePreflightObstacleKind::ForeignVia,
            object_uuid: via.uuid,
            layer: None,
            net_uuid: Some(via.net),
            net_name,
            reason: format!(
                "foreign via {} spans layers {}..{}",
                via.uuid, via.from_layer, via.to_layer
            ),
        });
    }

    let mut zones = board
        .zones
        .values()
        .filter(|zone| zone.net != target_net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for zone in zones {
        let net_name = board.nets.get(&zone.net).map(|net| net.name.clone());
        obstacles.push(RoutePreflightObstacle {
            kind: RoutePreflightObstacleKind::ForeignZone,
            object_uuid: zone.uuid,
            layer: Some(zone.layer),
            net_uuid: Some(zone.net),
            net_name,
            reason: format!("foreign zone {} on layer {}", zone.uuid, zone.layer),
        });
    }

    obstacles
}

fn outside_outline_conflicts(
    board: &Board,
    anchors: &[RoutePreflightAnchor],
    target_tracks: &[Track],
    target_vias: &[Via],
    target_zones: &[Zone],
) -> Vec<RoutePreflightObstacle> {
    let mut conflicts = Vec::new();

    for anchor in anchors {
        if !point_in_polygon(anchor.position, &board.outline) {
            conflicts.push(RoutePreflightObstacle {
                kind: RoutePreflightObstacleKind::OutsideOutline,
                object_uuid: anchor.pad_uuid,
                layer: Some(anchor.layer),
                net_uuid: None,
                net_name: None,
                reason: format!("anchor {} lies outside the board outline", anchor.name),
            });
        }
    }

    for track in target_tracks {
        if !point_in_polygon(track.from, &board.outline)
            || !point_in_polygon(track.to, &board.outline)
            || segment_escapes_polygon(track.from, track.to, &board.outline)
        {
            conflicts.push(RoutePreflightObstacle {
                kind: RoutePreflightObstacleKind::OutsideOutline,
                object_uuid: track.uuid,
                layer: Some(track.layer),
                net_uuid: None,
                net_name: None,
                reason: format!(
                    "target track {} has an endpoint outside the board outline",
                    track.uuid
                ),
            });
        }
    }

    for via in target_vias {
        if !point_in_polygon(via.position, &board.outline) {
            conflicts.push(RoutePreflightObstacle {
                kind: RoutePreflightObstacleKind::OutsideOutline,
                object_uuid: via.uuid,
                layer: None,
                net_uuid: None,
                net_name: None,
                reason: format!("target via {} lies outside the board outline", via.uuid),
            });
        }
    }

    for zone in target_zones {
        if zone
            .polygon
            .vertices
            .iter()
            .copied()
            .any(|point| !point_in_polygon(point, &board.outline))
            || polygon_escapes_polygon(&zone.polygon, &board.outline)
        {
            conflicts.push(RoutePreflightObstacle {
                kind: RoutePreflightObstacleKind::OutsideOutline,
                object_uuid: zone.uuid,
                layer: Some(zone.layer),
                net_uuid: None,
                net_name: None,
                reason: format!(
                    "target zone {} has a vertex outside the board outline",
                    zone.uuid
                ),
            });
        }
    }

    conflicts.sort_by(|a, b| {
        a.object_uuid
            .cmp(&b.object_uuid)
            .then_with(|| a.reason.cmp(&b.reason))
    });
    conflicts
}
