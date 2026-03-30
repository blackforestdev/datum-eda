use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::Point;
use uuid::Uuid;

pub(super) fn demo_board() -> (
    Board,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
) {
    let net_uuid = Uuid::from_u128(0x1030);
    let other_net_uuid = Uuid::from_u128(0x1031);
    let class_uuid = Uuid::from_u128(0x1032);
    let anchor_top_uuid = Uuid::from_u128(0x1033);
    let anchor_bottom_uuid = Uuid::from_u128(0x1034);
    let via_a_uuid = Uuid::from_u128(0x1035);
    let via_b_uuid = Uuid::from_u128(0x1036);
    let via_c_uuid = Uuid::from_u128(0x1037);
    let via_d_uuid = Uuid::from_u128(0x1038);
    let via_e_uuid = Uuid::from_u128(0x1039);
    let via_f_uuid = Uuid::from_u128(0x103a);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "path-candidate-six-via".into(),
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
                        name: "Core A".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 3,
                        name: "Inner 1".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 4,
                        name: "Core B".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 5,
                        name: "Inner 2".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 6,
                        name: "Core C".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 7,
                        name: "Inner 3".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 8,
                        name: "Core D".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 9,
                        name: "Inner 4".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 10,
                        name: "Core E".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 11,
                        name: "Inner 5".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    },
                    StackupLayer {
                        id: 12,
                        name: "Core F".into(),
                        layer_type: StackupLayerType::Dielectric,
                        thickness_nm: 1_000_000,
                    },
                    StackupLayer {
                        id: 13,
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
                        package: Uuid::from_u128(0x1040),
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
                        package: Uuid::from_u128(0x1041),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 13,
                        shape: PadShape::Circle,
                        diameter: 300_000,
                        width: 0,
                        height: 0,
                    },
                ),
            ]),
            tracks: HashMap::new(),
            vias: HashMap::from([
                (
                    via_a_uuid,
                    Via {
                        uuid: via_a_uuid,
                        net: net_uuid,
                        position: Point::new(160_000, 160_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 1,
                        to_layer: 3,
                    },
                ),
                (
                    via_b_uuid,
                    Via {
                        uuid: via_b_uuid,
                        net: net_uuid,
                        position: Point::new(260_000, 260_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 3,
                        to_layer: 5,
                    },
                ),
                (
                    via_c_uuid,
                    Via {
                        uuid: via_c_uuid,
                        net: net_uuid,
                        position: Point::new(400_000, 400_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 5,
                        to_layer: 7,
                    },
                ),
                (
                    via_d_uuid,
                    Via {
                        uuid: via_d_uuid,
                        net: net_uuid,
                        position: Point::new(560_000, 560_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 7,
                        to_layer: 9,
                    },
                ),
                (
                    via_e_uuid,
                    Via {
                        uuid: via_e_uuid,
                        net: net_uuid,
                        position: Point::new(700_000, 700_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 9,
                        to_layer: 11,
                    },
                ),
                (
                    via_f_uuid,
                    Via {
                        uuid: via_f_uuid,
                        net: net_uuid,
                        position: Point::new(840_000, 840_000),
                        drill: 300_000,
                        diameter: 600_000,
                        from_layer: 11,
                        to_layer: 13,
                    },
                ),
            ]),
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
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    )
}

#[test]
fn route_path_candidate_six_via_reports_deterministic_path_using_authored_via_sextuple() {
    let (
        board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    ) = demo_board();

    let report = board
        .route_path_candidate_six_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("six-via path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_via_count, 6);
    assert_eq!(report.summary.candidate_via_sextuple_count, 720);
    assert_eq!(report.summary.matching_via_sextuple_count, 1);
    assert_eq!(report.summary.available_via_sextuple_count, 1);
    assert_eq!(
        report.path.as_ref().map(|path| path.via_a_uuid),
        Some(via_a_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_c_uuid),
        Some(via_c_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_d_uuid),
        Some(via_d_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_e_uuid),
        Some(via_e_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_f_uuid),
        Some(via_f_uuid)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.first_intermediate_layer),
        Some(3)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.second_intermediate_layer),
        Some(5)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.third_intermediate_layer),
        Some(7)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.fourth_intermediate_layer),
        Some(9)
    );
    assert_eq!(
        report
            .path
            .as_ref()
            .map(|path| path.fifth_intermediate_layer),
        Some(11)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.segments.len()),
        Some(7)
    );
}

#[test]
fn route_path_candidate_six_via_selects_next_matching_sextuple_when_earlier_sextuple_is_blocked() {
    let (
        mut board,
        net_uuid,
        _,
        anchor_top_uuid,
        anchor_bottom_uuid,
        first_via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    ) = demo_board();
    let second_via_a_uuid = Uuid::from_u128(0x103b);
    board.vias.insert(
        second_via_a_uuid,
        Via {
            uuid: second_via_a_uuid,
            net: net_uuid,
            position: Point::new(120_000, 240_000),
            drill: 300_000,
            diameter: 600_000,
            from_layer: 1,
            to_layer: 3,
        },
    );
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0x103c),
        polygon: Polygon::new(vec![
            Point::new(140_000, 140_000),
            Point::new(210_000, 140_000),
            Point::new(210_000, 210_000),
            Point::new(140_000, 210_000),
        ]),
        layers: vec![1, 3],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_six_via(net_uuid, anchor_top_uuid, anchor_bottom_uuid)
        .expect("six-via path candidate should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.matching_via_sextuple_count, 2);
    assert_eq!(report.summary.blocked_via_sextuple_count, 1);
    assert_eq!(report.summary.available_via_sextuple_count, 1);
    assert_ne!(
        report.path.as_ref().map(|path| path.via_a_uuid),
        Some(first_via_a_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_a_uuid),
        Some(second_via_a_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_b_uuid),
        Some(via_b_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_c_uuid),
        Some(via_c_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_d_uuid),
        Some(via_d_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_e_uuid),
        Some(via_e_uuid)
    );
    assert_eq!(
        report.path.as_ref().map(|path| path.via_f_uuid),
        Some(via_f_uuid)
    );
}
