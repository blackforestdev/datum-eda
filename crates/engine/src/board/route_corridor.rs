use crate::ir::geometry::{LayerId, Point, Polygon};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePreflightAnchor, RoutePreflightConstraintFacts, RoutePreflightStatus, StackupLayer,
};

use super::route_segment_blockage::analyze_route_segment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCorridorStatus {
    CorridorAvailable,
    CorridorBlocked,
    InsufficientAuthoredInputs,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCorridorObstacleKind {
    Keepout,
    ForeignTrack,
    ForeignVia,
    ForeignZone,
    OutsideOutline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCorridorObstacleGeometry {
    pub kind: RouteCorridorObstacleKind,
    pub object_uuid: Option<Uuid>,
    pub layer: Option<LayerId>,
    pub net_uuid: Option<Uuid>,
    pub net_name: Option<String>,
    pub polygon: Option<Polygon>,
    pub from: Option<Point>,
    pub to: Option<Point>,
    pub position: Option<Point>,
    pub diameter_nm: Option<i64>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCorridorSpanBlockage {
    pub kind: RouteCorridorObstacleKind,
    pub object_uuid: Option<Uuid>,
    pub layer: Option<LayerId>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCorridorSpan {
    pub pair_index: usize,
    pub layer: LayerId,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub from: Point,
    pub to: Point,
    pub blocked: bool,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCorridorSummary {
    pub anchor_count: usize,
    pub candidate_copper_layer_count: usize,
    pub anchor_pair_count: usize,
    pub obstacle_count: usize,
    pub span_count: usize,
    pub available_span_count: usize,
    pub blocked_span_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCorridorReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RouteCorridorStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub net_class_uuid: Uuid,
    pub persisted_constraints: RoutePreflightConstraintFacts,
    pub anchors: Vec<RoutePreflightAnchor>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RouteCorridorSummary,
    pub authored_obstacle_geometry: Vec<RouteCorridorObstacleGeometry>,
    pub corridor_spans: Vec<RouteCorridorSpan>,
}

impl Board {
    pub fn route_corridor(&self, net_uuid: Uuid) -> Option<RouteCorridorReport> {
        let preflight = self.route_preflight(net_uuid)?;
        let anchor_pairs = anchor_pairs(&preflight.anchors);
        let (authored_obstacle_geometry, corridor_spans) = corridor_geometry(
            self,
            net_uuid,
            &preflight.anchors,
            &preflight.candidate_copper_layers,
        );

        let available_pair_count = pair_availability_count(anchor_pairs.len(), &corridor_spans);
        let available_span_count = corridor_spans.iter().filter(|span| !span.blocked).count();
        let blocked_span_count = corridor_spans.len().saturating_sub(available_span_count);

        let status = match preflight.status {
            RoutePreflightStatus::InsufficientAuthoredInputs => {
                RouteCorridorStatus::InsufficientAuthoredInputs
            }
            RoutePreflightStatus::BlockedByAuthoredObstacle => RouteCorridorStatus::CorridorBlocked,
            RoutePreflightStatus::PreflightReady if available_pair_count == anchor_pairs.len() => {
                RouteCorridorStatus::CorridorAvailable
            }
            RoutePreflightStatus::PreflightReady => RouteCorridorStatus::CorridorBlocked,
        };

        Some(RouteCorridorReport {
            contract: "m5_route_corridor_v1".to_string(),
            persisted_native_board_state_only: true,
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            net_class_uuid: preflight.net_class_uuid,
            persisted_constraints: preflight.persisted_constraints,
            anchors: preflight.anchors.clone(),
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RouteCorridorSummary {
                anchor_count: preflight.anchors.len(),
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                anchor_pair_count: anchor_pairs.len(),
                obstacle_count: authored_obstacle_geometry.len(),
                span_count: corridor_spans.len(),
                available_span_count,
                blocked_span_count,
            },
            authored_obstacle_geometry,
            corridor_spans,
        })
    }
}

fn anchor_pairs(
    anchors: &[RoutePreflightAnchor],
) -> Vec<(&RoutePreflightAnchor, &RoutePreflightAnchor)> {
    anchors
        .windows(2)
        .map(|window| (&window[0], &window[1]))
        .collect()
}

fn pair_availability_count(pair_count: usize, spans: &[RouteCorridorSpan]) -> usize {
    (0..pair_count)
        .filter(|pair_index| {
            spans
                .iter()
                .any(|span| span.pair_index == *pair_index && !span.blocked)
        })
        .count()
}

fn corridor_geometry(
    board: &Board,
    net_uuid: Uuid,
    anchors: &[RoutePreflightAnchor],
    candidate_copper_layers: &[StackupLayer],
) -> (Vec<RouteCorridorObstacleGeometry>, Vec<RouteCorridorSpan>) {
    let mut obstacles = Vec::new();
    let mut spans = Vec::new();
    let anchor_pairs = anchor_pairs(anchors);

    for layer in candidate_copper_layers {
        for (pair_index, (from_anchor, to_anchor)) in anchor_pairs.iter().enumerate() {
            let span_from = from_anchor.position;
            let span_to = to_anchor.position;
            let analysis = analyze_route_segment(
                board,
                net_uuid,
                layer.id,
                span_from,
                span_to,
                &format!("span {} on layer {}", pair_index, layer.id),
            );
            let blockages = analysis.blockages;
            obstacles.extend(analysis.obstacles);

            spans.push(RouteCorridorSpan {
                pair_index,
                layer: layer.id,
                from_anchor_pad_uuid: from_anchor.pad_uuid,
                to_anchor_pad_uuid: to_anchor.pad_uuid,
                from: span_from,
                to: span_to,
                blocked: !blockages.is_empty(),
                blockages,
            });
        }
    }

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
    spans.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.pair_index.cmp(&b.pair_index))
    });

    (obstacles, spans)
}
