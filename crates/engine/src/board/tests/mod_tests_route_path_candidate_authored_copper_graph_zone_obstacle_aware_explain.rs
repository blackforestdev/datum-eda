use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x3900);
    let other_net_uuid = Uuid::from_u128(0x3901);
    let class_uuid = Uuid::from_u128(0x3902);
    let anchor_a_uuid = Uuid::from_u128(0x3903);
    let anchor_b_uuid = Uuid::from_u128(0x3904);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-authored-copper-graph-zone-obstacle-aware-explain".into(),
            stackup: Stackup {
                layers: vec![StackupLayer {
                    id: 1,
                    name: "Top".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                }],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    anchor_a_uuid,
                    PlacedPad {
                        uuid: anchor_a_uuid,
                        package: Uuid::from_u128(0x3910),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 100_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
                (
                    anchor_b_uuid,
                    PlacedPad {
                        uuid: anchor_b_uuid,
                        package: Uuid::from_u128(0x3911),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 100_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::new(),
            zones: HashMap::from([(
                Uuid::from_u128(0x3912),
                Zone {
                    uuid: Uuid::from_u128(0x3912),
                    net: net_uuid,
                    polygon: Polygon::new(vec![
                        Point::new(50_000, 50_000),
                        Point::new(950_000, 50_000),
                        Point::new(950_000, 150_000),
                        Point::new(50_000, 150_000),
                    ]),
                    layer: 1,
                    priority: 1,
                    thermal_relief: true,
                    thermal_gap: 150_000,
                    thermal_spoke_width: 120_000,
                },
            )]),
            nets: HashMap::from([
                (
                    net_uuid,
                    Net {
                        uuid: net_uuid,
                        name: "SIG".into(),
                        class: class_uuid,
                    },
                ),
                (
                    other_net_uuid,
                    Net {
                        uuid: other_net_uuid,
                        name: "OTHER".into(),
                        class: class_uuid,
                    },
                ),
            ]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 150_000,
                    track_width: 200_000,
                    via_drill: 300_000,
                    via_diameter: 600_000,
                    diffpair_width: 0,
                    diffpair_gap: 0,
                },
            )]),
            rules: Vec::new(),
            keepouts: Vec::new(),
            dimensions: Vec::new(),
            texts: Vec::new(),
        },
        net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        Uuid::from_u128(0x3912),
    )
}

#[test]
fn route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain_reports_selected_path_reasoning()
 {
    let (board, net_uuid, anchor_a_uuid, anchor_b_uuid, zone_uuid) = demo_board();

    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain(
            net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        )
        .expect("zone obstacle-aware authored copper graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareExplainKind::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_track_count, 0);
    assert_eq!(report.summary.candidate_via_count, 0);
    assert_eq!(report.summary.candidate_zone_count, 1);
    assert_eq!(report.summary.blocked_zone_connection_count, 0);
    assert_eq!(
        report.selected_path.as_ref().map(|path| path.steps.len()),
        Some(1)
    );
    assert_eq!(
        report
            .selected_path
            .as_ref()
            .map(|path| path.steps[0].object_uuid),
        Some(zone_uuid)
    );
}

#[test]
fn route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain_reports_no_existing_path()
{
    let (mut board, net_uuid, anchor_a_uuid, anchor_b_uuid, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x3913),
        polygon: Polygon::new(vec![
            Point::new(450_000, 50_000),
            Point::new(550_000, 50_000),
            Point::new(550_000, 150_000),
            Point::new(450_000, 150_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_explain(
            net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        )
        .expect("zone obstacle-aware authored copper graph explain should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(
        report.explanation_kind,
        RoutePathCandidateAuthoredCopperGraphZoneObstacleAwareExplainKind::NoExistingAuthoredCopperPath
    );
    assert_eq!(report.summary.blocked_zone_connection_count, 1);
    assert!(report.selected_path.is_none());
}
