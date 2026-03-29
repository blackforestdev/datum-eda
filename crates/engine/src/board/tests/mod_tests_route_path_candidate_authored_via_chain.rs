use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

fn demo_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x3000);
    let other_net_uuid = Uuid::from_u128(0x3001);
    let class_uuid = Uuid::from_u128(0x3002);
    let anchor_top_uuid = Uuid::from_u128(0x3003);
    let anchor_bottom_uuid = Uuid::from_u128(0x3004);
    let via_a_uuid = Uuid::from_u128(0x3005);
    let via_b_uuid = Uuid::from_u128(0x3006);
    let via_c_uuid = Uuid::from_u128(0x3007);
    let via_d_uuid = Uuid::from_u128(0x3008);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-authored-via-chain".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer { id: 1, name: "Top".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 2, name: "Core A".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 3, name: "Inner 1".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 4, name: "Core B".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 5, name: "Inner 2".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
                    StackupLayer { id: 6, name: "Core C".into(), layer_type: StackupLayerType::Dielectric, thickness_nm: 1_000_000 },
                    StackupLayer { id: 7, name: "Bottom".into(), layer_type: StackupLayerType::Copper, thickness_nm: 35_000 },
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
                        package: Uuid::from_u128(0x3010),
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
                        package: Uuid::from_u128(0x3011),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 7,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::from([
                (via_a_uuid, Via { uuid: via_a_uuid, net: net_uuid, position: Point::new(250_000, 250_000), drill: 300_000, diameter: 600_000, from_layer: 1, to_layer: 3 }),
                (via_b_uuid, Via { uuid: via_b_uuid, net: net_uuid, position: Point::new(500_000, 500_000), drill: 300_000, diameter: 600_000, from_layer: 3, to_layer: 7 }),
                (via_c_uuid, Via { uuid: via_c_uuid, net: net_uuid, position: Point::new(420_000, 420_000), drill: 300_000, diameter: 600_000, from_layer: 3, to_layer: 5 }),
                (via_d_uuid, Via { uuid: via_d_uuid, net: net_uuid, position: Point::new(680_000, 680_000), drill: 300_000, diameter: 600_000, from_layer: 5, to_layer: 7 }),
            ]),
            zones: HashMap::new(),
            nets: HashMap::from([
                (net_uuid, Net { uuid: net_uuid, name: "SIG".into(), class: class_uuid }),
                (other_net_uuid, Net { uuid: other_net_uuid, name: "OTHER".into(), class: class_uuid }),
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
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
    )
}

#[test]
fn route_path_candidate_authored_via_chain_prefers_shorter_matching_chain_before_uuid_order() {
    let (board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid, _, _) =
        demo_board();

    let report = board
        .route_path_candidate_authored_via_chain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored via chain path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert_eq!(report.summary.candidate_via_count, 4);
    assert_eq!(report.summary.matching_via_chain_count, 2);
    assert_eq!(report.summary.available_via_chain_count, 2);
    assert_eq!(report.path.as_ref().map(|path| path.via_chain.len()), Some(2));
    assert_eq!(
        report.path.as_ref().map(|path| {
            path.via_chain
                .iter()
                .map(|via| via.via_uuid)
                .collect::<Vec<_>>()
        }),
        Some(vec![via_a_uuid, via_b_uuid])
    );
    assert_eq!(report.path.as_ref().map(|path| path.segments.len()), Some(3));
}

#[test]
fn route_path_candidate_authored_via_chain_prefers_smaller_uuid_sequence_with_same_via_count() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, via_a_uuid, via_b_uuid, _, _) =
        demo_board();
    let via_alt_a_uuid = Uuid::from_u128(0x3009);
    let via_alt_b_uuid = Uuid::from_u128(0x300a);
    board.vias.insert(
        via_alt_a_uuid,
        Via {
            uuid: via_alt_a_uuid,
            net: net_uuid,
            position: Point::new(200_000, 180_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );
    board.vias.insert(
        via_alt_b_uuid,
        Via {
            uuid: via_alt_b_uuid,
            net: net_uuid,
            position: Point::new(760_000, 760_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 3,
            to_layer: 7,
        },
    );

    let report = board
        .route_path_candidate_authored_via_chain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored via chain path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert!(report.summary.matching_via_chain_count >= 4);
    assert_eq!(
        report.path.as_ref().map(|path| {
            path.via_chain
                .iter()
                .map(|via| via.via_uuid)
                .collect::<Vec<_>>()
        }),
        Some(vec![via_a_uuid, via_b_uuid])
    );
}

#[test]
fn route_path_candidate_authored_via_chain_selects_next_unblocked_chain_after_blocked_earlier_chain() {
    let (mut board, net_uuid, _, anchor_top_uuid, anchor_bottom_uuid, first_via_a_uuid, _, _, _) =
        demo_board();
    let second_via_a_uuid = Uuid::from_u128(0x300b);
    let second_via_b_uuid = Uuid::from_u128(0x300c);
    board.vias.insert(
        second_via_a_uuid,
        Via {
            uuid: second_via_a_uuid,
            net: net_uuid,
            position: Point::new(120_000, 420_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );
    board.vias.insert(
        second_via_b_uuid,
        Via {
            uuid: second_via_b_uuid,
            net: net_uuid,
            position: Point::new(820_000, 860_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 3,
            to_layer: 7,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x300d),
        polygon: Polygon::new(vec![
            Point::new(200_000, 200_000),
            Point::new(320_000, 200_000),
            Point::new(320_000, 320_000),
            Point::new(200_000, 320_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_authored_via_chain(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("authored via chain path candidate should succeed");

    assert_eq!(report.status, RoutePathCandidateStatus::DeterministicPathFound);
    assert!(report.summary.matching_via_chain_count >= 4);
    assert!(report.summary.blocked_via_chain_count >= 1);
    assert_eq!(
        report.path.as_ref().map(|path| path.via_chain[0].via_uuid),
        Some(second_via_a_uuid)
    );
    assert_ne!(
        report.path.as_ref().map(|path| path.via_chain[0].via_uuid),
        Some(first_via_a_uuid)
    );
    assert_eq!(report.path.as_ref().map(|path| path.via_chain.len()), Some(2));
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.via_chain.iter().any(|via| via.via_uuid == second_via_b_uuid)),
        Some(false)
    );
}
