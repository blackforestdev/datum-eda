use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x3600);
    let other_net_uuid = Uuid::from_u128(0x3601);
    let class_uuid = Uuid::from_u128(0x3602);
    let anchor_a_uuid = Uuid::from_u128(0x3603);
    let anchor_b_uuid = Uuid::from_u128(0x3604);
    let track_a_uuid = Uuid::from_u128(0x3605);
    let track_b_uuid = Uuid::from_u128(0x3606);
    let via_uuid = Uuid::from_u128(0x3607);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-authored-copper-graph-obstacle-aware".into(),
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
                    anchor_a_uuid,
                    PlacedPad {
                        uuid: anchor_a_uuid,
                        package: Uuid::from_u128(0x3610),
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
                        package: Uuid::from_u128(0x3611),
                        name: "1".into(),
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
        track_a_uuid,
        track_b_uuid,
        via_uuid,
        other_net_uuid,
    )
}

#[test]
fn route_path_candidate_authored_copper_graph_obstacle_aware_reports_unblocked_existing_path() {
    let (board, net_uuid, anchor_a_uuid, anchor_b_uuid, track_a_uuid, track_b_uuid, via_uuid, _) =
        demo_board();

    let report = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        )
        .expect("obstacle-aware authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.blocked_track_count, 0);
    assert_eq!(report.summary.blocked_via_count, 0);
    assert_eq!(report.path.as_ref().map(|path| path.steps.len()), Some(3));
    assert_eq!(
        report.path.as_ref().map(|path| path.steps[0].object_uuid),
        Some(track_a_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.steps[1].object_uuid),
        Some(via_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.steps[2].object_uuid),
        Some(track_b_uuid)
    );
}

#[test]
fn route_path_candidate_authored_copper_graph_obstacle_aware_excludes_blocked_existing_track_edges()
{
    let (
        mut board,
        net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        _track_a_uuid,
        _track_b_uuid,
        _via_uuid,
        _,
    ) = demo_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x3620),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        )
        .expect("obstacle-aware authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert_eq!(report.summary.blocked_track_count, 2);
    assert_eq!(report.summary.blocked_via_count, 1);
    assert!(report.path.is_none());
}

#[test]
fn route_path_candidate_authored_copper_graph_obstacle_aware_prefers_unblocked_direct_track_when_via_path_is_blocked()
 {
    let (
        mut board,
        net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        _track_a_uuid,
        track_b_uuid,
        _via_uuid,
        _,
    ) = demo_board();
    let direct_track_uuid = Uuid::from_u128(0x3621);
    board.pads.get_mut(&anchor_b_uuid).expect("anchor b").layer = 1;
    board.tracks.insert(
        direct_track_uuid,
        Track {
            uuid: direct_track_uuid,
            net: net_uuid,
            from: Point::new(100_000, 100_000),
            to: Point::new(900_000, 900_000),
            width: 150_000,
            layer: 1,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x3622),
        polygon: Polygon::new(vec![
            Point::new(450_000, 450_000),
            Point::new(550_000, 450_000),
            Point::new(550_000, 550_000),
            Point::new(450_000, 550_000),
        ]),
        layers: vec![3],
        kind: "route".into(),
    });
    board.tracks.remove(&track_b_uuid);

    let report = board
        .route_path_candidate_authored_copper_graph_obstacle_aware(
            net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        )
        .expect("obstacle-aware authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.path.as_ref().map(|path| path.steps.len()), Some(1));
    assert_eq!(
        report.path.as_ref().map(|path| path.steps[0].object_uuid),
        Some(direct_track_uuid)
    );
}
