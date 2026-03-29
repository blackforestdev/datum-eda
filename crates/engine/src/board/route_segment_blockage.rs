use crate::ir::geometry::{LayerId, Point};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorObstacleGeometry, RouteCorridorObstacleKind, RouteCorridorSpanBlockage,
    polygon::{
        point_in_polygon, point_to_segment_distance_nm, segment_escapes_polygon,
        segment_intersects_polygon, segment_intersects_segment,
    },
};

#[derive(Debug, Clone)]
pub(super) struct RouteSegmentBlockageAnalysis {
    pub blockages: Vec<RouteCorridorSpanBlockage>,
    pub obstacles: Vec<RouteCorridorObstacleGeometry>,
}

pub(super) fn analyze_route_segment(
    board: &Board,
    target_net_uuid: Uuid,
    layer: LayerId,
    from: Point,
    to: Point,
    subject: &str,
) -> RouteSegmentBlockageAnalysis {
    let mut blockages = Vec::new();
    let mut obstacles = Vec::new();

    for keepout in board
        .keepouts
        .iter()
        .filter(|keepout| keepout.layers.contains(&layer))
    {
        if segment_intersects_polygon(from, to, &keepout.polygon) {
            let reason = format!("{subject} crosses keepout {}", keepout.kind);
            blockages.push(RouteCorridorSpanBlockage {
                kind: RouteCorridorObstacleKind::Keepout,
                object_uuid: Some(keepout.uuid),
                layer: Some(layer),
                reason: reason.clone(),
            });
            obstacles.push(RouteCorridorObstacleGeometry {
                kind: RouteCorridorObstacleKind::Keepout,
                object_uuid: Some(keepout.uuid),
                layer: Some(layer),
                net_uuid: None,
                net_name: None,
                polygon: Some(keepout.polygon.clone()),
                from: None,
                to: None,
                position: None,
                diameter_nm: None,
                reason,
            });
        }
    }

    let mut foreign_tracks = board
        .tracks
        .values()
        .filter(|track| track.net != target_net_uuid && track.layer == layer)
        .cloned()
        .collect::<Vec<_>>();
    foreign_tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for track in foreign_tracks {
        if segment_intersects_segment(from, to, track.from, track.to) {
            let net_name = board.nets.get(&track.net).map(|net| net.name.clone());
            let reason = format!("{subject} crosses foreign track {}", track.uuid);
            blockages.push(RouteCorridorSpanBlockage {
                kind: RouteCorridorObstacleKind::ForeignTrack,
                object_uuid: Some(track.uuid),
                layer: Some(layer),
                reason: reason.clone(),
            });
            obstacles.push(RouteCorridorObstacleGeometry {
                kind: RouteCorridorObstacleKind::ForeignTrack,
                object_uuid: Some(track.uuid),
                layer: Some(layer),
                net_uuid: Some(track.net),
                net_name,
                polygon: None,
                from: Some(track.from),
                to: Some(track.to),
                position: None,
                diameter_nm: Some(track.width),
                reason,
            });
        }
    }

    let mut foreign_vias = board
        .vias
        .values()
        .filter(|via| {
            via.net != target_net_uuid && layer >= via.from_layer && layer <= via.to_layer
        })
        .cloned()
        .collect::<Vec<_>>();
    foreign_vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for via in foreign_vias {
        if point_to_segment_distance_nm(via.position, from, to) <= via.diameter / 2 {
            let net_name = board.nets.get(&via.net).map(|net| net.name.clone());
            let reason = format!("{subject} intersects foreign via {}", via.uuid);
            blockages.push(RouteCorridorSpanBlockage {
                kind: RouteCorridorObstacleKind::ForeignVia,
                object_uuid: Some(via.uuid),
                layer: Some(layer),
                reason: reason.clone(),
            });
            obstacles.push(RouteCorridorObstacleGeometry {
                kind: RouteCorridorObstacleKind::ForeignVia,
                object_uuid: Some(via.uuid),
                layer: Some(layer),
                net_uuid: Some(via.net),
                net_name,
                polygon: None,
                from: None,
                to: None,
                position: Some(via.position),
                diameter_nm: Some(via.diameter),
                reason,
            });
        }
    }

    let mut foreign_zones = board
        .zones
        .values()
        .filter(|zone| zone.net != target_net_uuid && zone.layer == layer)
        .cloned()
        .collect::<Vec<_>>();
    foreign_zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    for zone in foreign_zones {
        if segment_intersects_polygon(from, to, &zone.polygon) {
            let net_name = board.nets.get(&zone.net).map(|net| net.name.clone());
            let reason = format!("{subject} crosses foreign zone {}", zone.uuid);
            blockages.push(RouteCorridorSpanBlockage {
                kind: RouteCorridorObstacleKind::ForeignZone,
                object_uuid: Some(zone.uuid),
                layer: Some(layer),
                reason: reason.clone(),
            });
            obstacles.push(RouteCorridorObstacleGeometry {
                kind: RouteCorridorObstacleKind::ForeignZone,
                object_uuid: Some(zone.uuid),
                layer: Some(layer),
                net_uuid: Some(zone.net),
                net_name,
                polygon: Some(zone.polygon.clone()),
                from: None,
                to: None,
                position: None,
                diameter_nm: None,
                reason,
            });
        }
    }

    if !point_in_polygon(from, &board.outline)
        || !point_in_polygon(to, &board.outline)
        || segment_escapes_polygon(from, to, &board.outline)
    {
        let reason = format!("{subject} leaves the board outline");
        blockages.push(RouteCorridorSpanBlockage {
            kind: RouteCorridorObstacleKind::OutsideOutline,
            object_uuid: None,
            layer: Some(layer),
            reason: reason.clone(),
        });
        obstacles.push(RouteCorridorObstacleGeometry {
            kind: RouteCorridorObstacleKind::OutsideOutline,
            object_uuid: None,
            layer: Some(layer),
            net_uuid: None,
            net_name: None,
            polygon: Some(board.outline.clone()),
            from: None,
            to: None,
            position: None,
            diameter_nm: None,
            reason,
        });
    }

    blockages.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.object_uuid.cmp(&b.object_uuid))
            .then_with(|| a.reason.cmp(&b.reason))
    });
    blockages.dedup_by(|a, b| {
        a.kind == b.kind
            && a.object_uuid == b.object_uuid
            && a.layer == b.layer
            && a.reason == b.reason
    });

    obstacles.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.layer.cmp(&b.layer))
            .then_with(|| a.object_uuid.cmp(&b.object_uuid))
            .then_with(|| a.reason.cmp(&b.reason))
    });
    obstacles.dedup_by(|a, b| {
        a.kind == b.kind
            && a.object_uuid == b.object_uuid
            && a.layer == b.layer
            && a.reason == b.reason
    });

    RouteSegmentBlockageAnalysis {
        blockages,
        obstacles,
    }
}
