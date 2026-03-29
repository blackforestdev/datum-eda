use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x3200);
    let other_net_uuid = Uuid::from_u128(0x3201);
    let class_uuid = Uuid::from_u128(0x3202);
    let anchor_top_uuid = Uuid::from_u128(0x3203);
    let anchor_bottom_uuid = Uuid::from_u128(0x3204);
    let track_a_uuid = Uuid::from_u128(0x3205);
    let track_b_uuid = Uuid::from_u128(0x3206);
    let via_uuid = Uuid::from_u128(0x3207);
    let long_track_uuid = Uuid::from_u128(0x3208);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-authored-copper-graph".into(),
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
                    anchor_top_uuid,
                    PlacedPad {
                        uuid: anchor_top_uuid,
                        package: Uuid::from_u128(0x3210),
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
                    anchor_bottom_uuid,
                    PlacedPad {
                        uuid: anchor_bottom_uuid,
                        package: Uuid::from_u128(0x3211),
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
                (
                    long_track_uuid,
                    Track {
                        uuid: long_track_uuid,
                        net: net_uuid,
                        from: Point::new(100_000, 100_000),
                        to: Point::new(900_000, 900_000),
                        width: 150_000,
                        layer: 1,
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
        other_net_uuid,
        anchor_top_uuid,
        anchor_bottom_uuid,
        track_a_uuid,
        track_b_uuid,
        via_uuid,
        long_track_uuid,
    )
}

#[test]
fn route_path_candidate_authored_copper_graph_reports_existing_track_via_path() {
    let (
        mut board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        track_a_uuid,
        track_b_uuid,
        via_uuid,
        long_track_uuid,
    ) = demo_board();
    board.tracks.remove(&long_track_uuid);

    let report = board
        .route_path_candidate_authored_copper_graph(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_track_count, 2);
    assert_eq!(report.summary.candidate_via_count, 1);
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
fn route_path_candidate_authored_copper_graph_prefers_shorter_existing_copper_path() {
    let net_uuid = Uuid::from_u128(0x3290);
    let class_uuid = Uuid::from_u128(0x3291);
    let anchor_left_uuid = Uuid::from_u128(0x3292);
    let anchor_right_uuid = Uuid::from_u128(0x3293);
    let direct_track_uuid = Uuid::from_u128(0x3294);
    let first_track_uuid = Uuid::from_u128(0x3295);
    let second_track_uuid = Uuid::from_u128(0x3296);
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "path-candidate-authored-copper-graph-shortest".into(),
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
                anchor_left_uuid,
                PlacedPad {
                    uuid: anchor_left_uuid,
                    package: Uuid::from_u128(0x3297),
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
                anchor_right_uuid,
                PlacedPad {
                    uuid: anchor_right_uuid,
                    package: Uuid::from_u128(0x3298),
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
        tracks: HashMap::from([
            (
                direct_track_uuid,
                Track {
                    uuid: direct_track_uuid,
                    net: net_uuid,
                    from: Point::new(100_000, 100_000),
                    to: Point::new(900_000, 100_000),
                    width: 150_000,
                    layer: 1,
                },
            ),
            (
                first_track_uuid,
                Track {
                    uuid: first_track_uuid,
                    net: net_uuid,
                    from: Point::new(100_000, 100_000),
                    to: Point::new(500_000, 100_000),
                    width: 150_000,
                    layer: 1,
                },
            ),
            (
                second_track_uuid,
                Track {
                    uuid: second_track_uuid,
                    net: net_uuid,
                    from: Point::new(500_000, 100_000),
                    to: Point::new(900_000, 100_000),
                    width: 150_000,
                    layer: 1,
                },
            ),
        ]),
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
    };

    let report = board
        .route_path_candidate_authored_copper_graph(net_uuid, anchor_left_uuid, anchor_right_uuid)
        .expect("authored copper graph path candidate should succeed");

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

#[test]
fn route_path_candidate_authored_copper_graph_breaks_equal_length_ties_by_object_uuid_sequence() {
    let (
        mut board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        _,
        track_b_uuid,
        via_uuid,
        long_track_uuid,
    ) = demo_board();
    board.tracks.remove(&long_track_uuid);
    let alt_track_a_uuid = Uuid::from_u128(0x3204f);
    board.tracks.insert(
        alt_track_a_uuid,
        Track {
            uuid: alt_track_a_uuid,
            net: net_uuid,
            from: Point::new(100_000, 100_000),
            to: Point::new(500_000, 500_000),
            width: 150_000,
            layer: 1,
        },
    );

    let report = board
        .route_path_candidate_authored_copper_graph(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.path.as_ref().map(|path| path.steps.len()), Some(3));
    assert_eq!(
        report.path.as_ref().map(|path| path.steps[0].object_uuid),
        Some(alt_track_a_uuid.min(Uuid::from_u128(0x3205)))
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
fn route_path_candidate_authored_copper_graph_reports_no_path_when_existing_copper_is_disconnected()
{
    let (
        mut board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        _,
        track_b_uuid,
        via_uuid,
        long_track_uuid,
    ) = demo_board();
    board.tracks.remove(&track_b_uuid);
    board.vias.remove(&via_uuid);
    board.tracks.remove(&long_track_uuid);

    let report = board
        .route_path_candidate_authored_copper_graph(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored copper graph path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert!(report.path.is_none());
}
