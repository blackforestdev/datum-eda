use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::{Point, Polygon};
use uuid::Uuid;

fn path_ids(report: &RoutePathCandidateAuthoredCopperGraphPolicyReport) -> Vec<Uuid> {
    report
        .path
        .as_ref()
        .map(|path| path.steps.iter().map(|step| step.object_uuid).collect())
        .unwrap_or_default()
}

pub(super) fn plain_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9100);
    let class_uuid = Uuid::from_u128(0x9101);
    let from_pad_uuid = Uuid::from_u128(0x9102);
    let to_pad_uuid = Uuid::from_u128(0x9103);
    let track_uuid = Uuid::from_u128(0x9104);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-plain".into(),
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
                Point::new(5_000_000, 0),
                Point::new(5_000_000, 5_000_000),
                Point::new(0, 5_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(500_000, 500_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
                (
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(4_500_000, 500_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([(
                track_uuid,
                Track {
                    uuid: track_uuid,
                    net: net_uuid,
                    from: Point::new(500_000, 500_000),
                    to: Point::new(4_500_000, 500_000),
                    width: 120_000,
                    layer: 1,
                },
            )]),
            vias: HashMap::new(),
            zones: HashMap::new(),
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        from_pad_uuid,
        to_pad_uuid,
        track_uuid,
    )
}

pub(super) fn zone_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9200);
    let class_uuid = Uuid::from_u128(0x9201);
    let from_pad_uuid = Uuid::from_u128(0x9202);
    let to_pad_uuid = Uuid::from_u128(0x9203);
    let zone_uuid = Uuid::from_u128(0x9204);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-zone".into(),
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
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
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
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
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
                zone_uuid,
                Zone {
                    uuid: zone_uuid,
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
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        from_pad_uuid,
        to_pad_uuid,
        zone_uuid,
    )
}

pub(super) fn obstacle_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9300);
    let class_uuid = Uuid::from_u128(0x9301);
    let from_pad_uuid = Uuid::from_u128(0x9302);
    let to_pad_uuid = Uuid::from_u128(0x9303);
    let track_a_uuid = Uuid::from_u128(0x9304);
    let via_uuid = Uuid::from_u128(0x9305);
    let track_b_uuid = Uuid::from_u128(0x9306);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-obstacle".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer { id: 1, name: "Top".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 2, name: "Core".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 3, name: "Bottom".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
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
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
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
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 3,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    track_a_uuid,
                    Track {
                        uuid: track_a_uuid,
                        net: net_uuid,
                        from: Point::new(100_000, 100_000),
                        to: Point::new(500_000, 500_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    track_b_uuid,
                    Track {
                        uuid: track_b_uuid,
                        net: net_uuid,
                        from: Point::new(500_000, 500_000),
                        to: Point::new(900_000, 900_000),
                        width: 150_000,
                        layer: 3,
                    },
                ),
            ]),
            vias: HashMap::from([(
                via_uuid,
                Via {
                    uuid: via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    drill: 300_000,
                    diameter: 600_000,
                    from_layer: 1,
                    to_layer: 3,
                },
            )]),
            zones: HashMap::new(),
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
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
        from_pad_uuid,
        to_pad_uuid,
        track_a_uuid,
        via_uuid,
        track_b_uuid,
    )
}

fn zone_obstacle_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9400);
    let class_uuid = Uuid::from_u128(0x9401);
    let from_pad_uuid = Uuid::from_u128(0x9402);
    let to_pad_uuid = Uuid::from_u128(0x9403);
    let zone_uuid = Uuid::from_u128(0x9404);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-zone-obstacle".into(),
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
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
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
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
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
                zone_uuid,
                Zone {
                    uuid: zone_uuid,
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
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        from_pad_uuid,
        to_pad_uuid,
        zone_uuid,
    )
}

fn topology_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9500);
    let class_uuid = Uuid::from_u128(0x9501);
    let from_pad_uuid = Uuid::from_u128(0x9502);
    let to_pad_uuid = Uuid::from_u128(0x9503);
    let via_uuid = Uuid::from_u128(0x9504);
    let track_a_uuid = Uuid::from_u128(0x9505);
    let track_b_uuid = Uuid::from_u128(0x9506);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-topology".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer { id: 1, name: "Top".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 2, name: "Inner".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                ],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(5_000_000, 0),
                Point::new(5_000_000, 5_000_000),
                Point::new(0, 5_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(500_000, 500_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
                (
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(3_500_000, 500_000),
                        layer: 2,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    track_a_uuid,
                    Track {
                        uuid: track_a_uuid,
                        net: net_uuid,
                        from: Point::new(500_000, 500_000),
                        to: Point::new(2_000_000, 500_000),
                        width: 120_000,
                        layer: 2,
                    },
                ),
                (
                    track_b_uuid,
                    Track {
                        uuid: track_b_uuid,
                        net: net_uuid,
                        from: Point::new(2_000_000, 500_000),
                        to: Point::new(3_500_000, 500_000),
                        width: 120_000,
                        layer: 2,
                    },
                ),
            ]),
            vias: HashMap::from([(
                via_uuid,
                Via {
                    uuid: via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    from_layer: 1,
                    to_layer: 2,
                    diameter: 300_000,
                    drill: 150_000,
                },
            )]),
            zones: HashMap::new(),
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        from_pad_uuid,
        to_pad_uuid,
        via_uuid,
        track_a_uuid,
        track_b_uuid,
    )
}

fn layer_balance_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9600);
    let class_uuid = Uuid::from_u128(0x9601);
    let from_pad_uuid = Uuid::from_u128(0x9602);
    let to_pad_uuid = Uuid::from_u128(0x9603);
    let via_uuid = Uuid::from_u128(0x9604);
    let track_uuid = Uuid::from_u128(0x9605);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-layer-balance".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer { id: 1, name: "Top".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 2, name: "Inner".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                ],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(5_000_000, 0),
                Point::new(5_000_000, 5_000_000),
                Point::new(0, 5_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(500_000, 500_000),
                        layer: 1,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
                (
                    to_pad_uuid,
                    PlacedPad {
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(2_000_000, 500_000),
                        layer: 2,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::from([
                (
                    Uuid::from_u128(0x9606),
                    Track {
                        uuid: Uuid::from_u128(0x9606),
                        net: net_uuid,
                        from: Point::new(500_000, 500_000),
                        to: Point::new(2_000_000, 500_000),
                        width: 120_000,
                        layer: 1,
                    },
                ),
                (
                    track_uuid,
                    Track {
                        uuid: track_uuid,
                        net: net_uuid,
                        from: Point::new(500_000, 500_000),
                        to: Point::new(2_000_000, 500_000),
                        width: 120_000,
                        layer: 2,
                    },
                ),
            ]),
            vias: HashMap::from([
                (
                    Uuid::from_u128(0x9607),
                    Via {
                        uuid: Uuid::from_u128(0x9607),
                        net: net_uuid,
                        position: Point::new(2_000_000, 500_000),
                        from_layer: 1,
                        to_layer: 2,
                        diameter: 300_000,
                        drill: 150_000,
                    },
                ),
                (
                    via_uuid,
                    Via {
                        uuid: via_uuid,
                        net: net_uuid,
                        position: Point::new(500_000, 500_000),
                        from_layer: 1,
                        to_layer: 2,
                        diameter: 300_000,
                        drill: 150_000,
                    },
                ),
            ]),
            zones: HashMap::new(),
            nets: HashMap::from([(
                net_uuid,
                Net {
                    uuid: net_uuid,
                    name: "SIG".into(),
                    class: class_uuid,
                },
            )]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        from_pad_uuid,
        to_pad_uuid,
        via_uuid,
        track_uuid,
    )
}

#[test]
fn authored_copper_graph_policy_preserves_plain_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_uuid) = plain_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::Plain,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(report.summary.candidate_track_count, direct.summary.candidate_track_count);
    assert_eq!(path_ids(&report), vec![track_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_zone_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, zone_uuid) = zone_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_aware(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(report.summary.candidate_zone_count, direct.summary.candidate_zone_count);
    assert_eq!(path_ids(&report), vec![zone_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_obstacle_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_a_uuid, via_uuid, track_b_uuid) =
        obstacle_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ObstacleAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(report.summary.blocked_track_count, direct.summary.blocked_track_count);
    assert_eq!(path_ids(&report), vec![track_a_uuid, via_uuid, track_b_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_zone_obstacle_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, zone_uuid) = zone_obstacle_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.blocked_zone_connection_count,
        direct.summary.blocked_zone_connection_count
    );
    assert_eq!(path_ids(&report), vec![zone_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_topology_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, via_uuid, track_a_uuid, track_b_uuid) =
        topology_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(
        report.summary.topology_transition_count,
        direct.summary.topology_transition_count
    );
    assert_eq!(path_ids(&report), vec![via_uuid, track_a_uuid, track_b_uuid]);
}

#[test]
fn authored_copper_graph_policy_preserves_layer_balance_aware_behavior() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, via_uuid, track_uuid) = layer_balance_board();
    let report = board
        .route_path_candidate_authored_copper_graph_by_policy(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
            RoutePathCandidateAuthoredCopperGraphPolicy::ZoneObstacleTopologyLayerBalanceAware,
        )
        .expect("policy query should succeed");
    let direct = board
        .route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_layer_balance_aware(
            net_uuid,
            from_pad_uuid,
            to_pad_uuid,
        )
        .expect("direct query should succeed");

    assert_eq!(report.status, direct.status);
    assert_eq!(report.selection_rule, direct.selection_rule);
    assert_eq!(report.summary.layer_balance_score, direct.summary.layer_balance_score);
    assert_eq!(path_ids(&report), vec![via_uuid, track_uuid]);
}
