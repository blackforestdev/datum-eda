use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::new_v4();
    let other_net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let foreign_track_uuid = Uuid::new_v4();
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "corridor".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer {
                        id: 1,
                        name: "Top".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 2,
                        name: "Core".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 3,
                        name: "Bottom".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                ],
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
                    Uuid::new_v4(),
                    PlacedPad {
                        uuid: Uuid::new_v4(),
                        package: pkg_a,
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
                    Uuid::new_v4(),
                    PlacedPad {
                        uuid: Uuid::new_v4(),
                        package: pkg_b,
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([(
                foreign_track_uuid,
                Track {
                    uuid: foreign_track_uuid,
                    net: other_net_uuid,
                    from: Point::new(500_000, 100_000),
                    to: Point::new(500_000, 300_000),
                    width: 150_000,
                    layer: 1,
                },
            )]),
            vias: HashMap::new(),
            zones: HashMap::new(),
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
        other_net_uuid,
        class_uuid,
        pkg_a,
        pkg_b,
    )
}

#[test]
fn route_corridor_reports_available_span_when_authored_geometry_is_clear() {
    let (board, net_uuid, _, class_uuid, _, _) = demo_board();
    let report = board.route_corridor(net_uuid).expect("net should exist");

    assert_eq!(report.status, RouteCorridorStatus::CorridorAvailable);
    assert_eq!(report.net_uuid, net_uuid);
    assert_eq!(report.net_class_uuid, class_uuid);
    assert_eq!(report.summary.anchor_count, 2);
    assert_eq!(report.summary.anchor_pair_count, 1);
    assert_eq!(report.summary.span_count, 2);
    assert_eq!(report.summary.available_span_count, 2);
    assert_eq!(report.summary.blocked_span_count, 0);
    assert!(report.authored_obstacle_geometry.is_empty());
}

#[test]
fn route_corridor_reports_blocked_when_authored_keepout_covers_all_candidate_layers() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board.route_corridor(net_uuid).expect("net should exist");

    assert_eq!(report.status, RouteCorridorStatus::CorridorBlocked);
    assert_eq!(report.summary.blocked_span_count, 2);
    assert_eq!(report.summary.available_span_count, 0);
    assert_eq!(report.authored_obstacle_geometry.len(), 2);
    assert!(report.corridor_spans.iter().all(|span| span.blocked));
}

#[test]
fn route_corridor_reports_insufficient_authored_inputs_when_only_one_anchor_exists() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    let single_pad_uuid = board.pads.keys().next().copied().expect("one pad exists");
    let remaining_pad_uuid = board
        .pads
        .keys()
        .copied()
        .find(|uuid| *uuid != single_pad_uuid)
        .expect("second pad exists");
    board.pads.remove(&remaining_pad_uuid);

    let report = board.route_corridor(net_uuid).expect("net should exist");

    assert_eq!(report.status, RouteCorridorStatus::InsufficientAuthoredInputs);
    assert_eq!(report.summary.anchor_count, 1);
    assert!(report.corridor_spans.is_empty());
}

#[test]
fn route_corridor_detects_foreign_zone_crossing_with_segment_polygon_truth_boundary() {
    let (mut board, net_uuid, other_net_uuid, class_uuid, _, _) = demo_board();
    let top_zone_uuid = Uuid::new_v4();
    let bottom_zone_uuid = Uuid::new_v4();
    board.zones.insert(
        top_zone_uuid,
        Zone {
            uuid: top_zone_uuid,
            net: other_net_uuid,
            polygon: Polygon::new(vec![
                Point::new(450_000, 200_000),
                Point::new(550_000, 200_000),
                Point::new(550_000, 800_000),
                Point::new(450_000, 800_000),
            ]),
            layer: 1,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 150_000,
            thermal_spoke_width: 120_000,
        },
    );
    board.zones.insert(
        bottom_zone_uuid,
        Zone {
            uuid: bottom_zone_uuid,
            net: other_net_uuid,
            polygon: Polygon::new(vec![
                Point::new(450_000, 200_000),
                Point::new(550_000, 200_000),
                Point::new(550_000, 800_000),
                Point::new(450_000, 800_000),
            ]),
            layer: 3,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 150_000,
            thermal_spoke_width: 120_000,
        },
    );
    board.net_classes.insert(
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
    );

    let report = board.route_corridor(net_uuid).expect("net should exist");

    assert_eq!(report.status, RouteCorridorStatus::CorridorBlocked);
    assert_eq!(report.summary.blocked_span_count, 2);
    assert_eq!(report.summary.obstacle_count, 2);
    assert!(report
        .corridor_spans
        .iter()
        .all(|span| span.blockages.iter().any(|entry| {
            matches!(entry.kind, RouteCorridorObstacleKind::ForeignZone)
        })));
}
