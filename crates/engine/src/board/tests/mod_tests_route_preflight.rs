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
            name: "preflight".into(),
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
fn route_preflight_is_ready_with_two_authored_anchors_and_no_conflict() {
    let (board, net_uuid, other_net_uuid, class_uuid, pkg_a, pkg_b) = demo_board();
    let report = board.route_preflight(net_uuid).expect("net should exist");

    assert_eq!(report.status, RoutePreflightStatus::PreflightReady);
    assert_eq!(report.net_uuid, net_uuid);
    assert_eq!(report.net_class_uuid, class_uuid);
    assert_eq!(report.anchors.len(), 2);
    let anchor_owners = report
        .anchors
        .iter()
        .map(|anchor| anchor.owner_uuid)
        .collect::<Vec<_>>();
    assert!(anchor_owners.contains(&pkg_a));
    assert!(anchor_owners.contains(&pkg_b));
    assert_eq!(report.candidate_copper_layers.len(), 2);
    assert_eq!(report.candidate_copper_layers[0].id, 1);
    assert_eq!(report.candidate_copper_layers[1].id, 3);
    assert_eq!(report.summary.foreign_track_count, 1);
    assert_eq!(report.summary.keepout_conflict_count, 0);
    assert_eq!(report.summary.outside_outline_count, 0);
    assert_eq!(report.foreign_obstacles[0].net_uuid, Some(other_net_uuid));
    assert!(matches!(
        report.foreign_obstacles[0].kind,
        RoutePreflightObstacleKind::ForeignTrack
    ));
}

#[test]
fn route_preflight_reports_blocked_by_keepout_conflict() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(50_000, 50_000),
            Point::new(150_000, 50_000),
            Point::new(150_000, 150_000),
            Point::new(50_000, 150_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.keepout_conflict_count, 1);
    assert!(matches!(
        report.keepout_conflicts[0].kind,
        RoutePreflightObstacleKind::KeepoutConflict
    ));
}

#[test]
fn route_preflight_detects_keepout_crossing_when_track_endpoints_stay_outside_keepout() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.tracks.insert(
        Uuid::new_v4(),
        Track {
            uuid: Uuid::new_v4(),
            net: net_uuid,
            from: Point::new(100_000, 500_000),
            to: Point::new(900_000, 500_000),
            width: 120_000,
            layer: 1,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.keepout_conflict_count, 1);
}

#[test]
fn route_preflight_detects_non_convex_outline_escape_when_track_endpoints_remain_inside() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.outline = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(1_000_000, 0),
        Point::new(1_000_000, 1_000_000),
        Point::new(600_000, 1_000_000),
        Point::new(600_000, 400_000),
        Point::new(400_000, 400_000),
        Point::new(400_000, 1_000_000),
        Point::new(0, 1_000_000),
    ]);
    board.tracks.insert(
        Uuid::new_v4(),
        Track {
            uuid: Uuid::new_v4(),
            net: net_uuid,
            from: Point::new(200_000, 800_000),
            to: Point::new(800_000, 800_000),
            width: 120_000,
            layer: 1,
        },
    );

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.outside_outline_count, 1);
    assert!(matches!(
        report.outside_outline_conflicts[0].kind,
        RoutePreflightObstacleKind::OutsideOutline
    ));
}

#[test]
fn route_preflight_detects_non_convex_outline_escape_when_segment_leaves_and_reenters_with_midpoint_inside() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.outline = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(1_000_000, 0),
        Point::new(1_000_000, 1_000_000),
        Point::new(700_000, 1_000_000),
        Point::new(700_000, 300_000),
        Point::new(600_000, 300_000),
        Point::new(600_000, 700_000),
        Point::new(400_000, 700_000),
        Point::new(400_000, 300_000),
        Point::new(300_000, 300_000),
        Point::new(300_000, 1_000_000),
        Point::new(0, 1_000_000),
    ]);
    board.tracks.insert(
        Uuid::new_v4(),
        Track {
            uuid: Uuid::new_v4(),
            net: net_uuid,
            from: Point::new(100_000, 500_000),
            to: Point::new(900_000, 500_000),
            width: 120_000,
            layer: 1,
        },
    );

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.outside_outline_count, 1);
}

#[test]
fn route_preflight_detects_concave_keepout_crossing() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.tracks.insert(
        Uuid::new_v4(),
        Track {
            uuid: Uuid::new_v4(),
            net: net_uuid,
            from: Point::new(100_000, 500_000),
            to: Point::new(900_000, 500_000),
            width: 120_000,
            layer: 1,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(400_000, 200_000),
            Point::new(800_000, 200_000),
            Point::new(800_000, 400_000),
            Point::new(600_000, 400_000),
            Point::new(600_000, 800_000),
            Point::new(400_000, 800_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.keepout_conflict_count, 1);
}

#[test]
fn route_preflight_detects_zone_crossing_when_zone_vertices_do_not_enter_keepout() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    board.zones.insert(
        Uuid::new_v4(),
        Zone {
            uuid: Uuid::new_v4(),
            net: net_uuid,
            polygon: Polygon::new(vec![
                Point::new(200_000, 450_000),
                Point::new(800_000, 450_000),
                Point::new(800_000, 550_000),
                Point::new(200_000, 550_000),
            ]),
            layer: 1,
            priority: 1,
            thermal_relief: true,
            thermal_gap: 150_000,
            thermal_spoke_width: 120_000,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::new_v4(),
        polygon: Polygon::new(vec![
            Point::new(450_000, 200_000),
            Point::new(550_000, 200_000),
            Point::new(550_000, 800_000),
            Point::new(450_000, 800_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::BlockedByAuthoredObstacle
    );
    assert_eq!(report.summary.keepout_conflict_count, 1);
}

#[test]
fn route_preflight_sorts_authored_anchors_deterministically_before_pairing() {
    let (mut board, net_uuid, _, _, pkg_b, _) = demo_board();
    let early_pad_uuid = Uuid::from_u128(0x10);
    board.pads.insert(
        Uuid::new_v4(),
        PlacedPad {
            uuid: early_pad_uuid,
            package: pkg_b,
            name: "1".into(),
            net: Some(net_uuid),
            position: Point::new(850_000, 850_000),
            layer: 1,
            shape: PadShape::Circle,
            diameter: 300_000,
            width: 0,
            height: 0,
        },
    );

    let report = board.route_preflight(net_uuid).expect("net should exist");
    let owner_b_pad_uuids = report
        .anchors
        .iter()
        .filter(|anchor| anchor.owner_uuid == pkg_b)
        .map(|anchor| anchor.pad_uuid)
        .collect::<Vec<_>>();

    assert_eq!(owner_b_pad_uuids.len(), 2);
    assert_eq!(owner_b_pad_uuids[0], early_pad_uuid);
    assert!(owner_b_pad_uuids[0] < owner_b_pad_uuids[1]);
}

#[test]
fn route_preflight_reports_insufficient_authored_inputs_when_only_one_anchor_exists() {
    let (mut board, net_uuid, _, _, _, _) = demo_board();
    let single_pad_uuid = board.pads.keys().next().copied().expect("one pad exists");
    let remaining_pad_uuid = board
        .pads
        .keys()
        .copied()
        .find(|uuid| *uuid != single_pad_uuid)
        .expect("second pad exists");
    board.pads.remove(&remaining_pad_uuid);

    let report = board.route_preflight(net_uuid).expect("net should exist");
    assert_eq!(
        report.status,
        RoutePreflightStatus::InsufficientAuthoredInputs
    );
    assert_eq!(report.summary.anchor_count, 1);
}
