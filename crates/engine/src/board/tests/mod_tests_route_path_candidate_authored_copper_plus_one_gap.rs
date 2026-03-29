use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::{Point, Polygon};
use uuid::Uuid;

fn plus_one_gap_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0xa100);
    let class_uuid = Uuid::from_u128(0xa101);
    let from_pad_uuid = Uuid::from_u128(0xa102);
    let to_pad_uuid = Uuid::from_u128(0xa103);
    let track_a_uuid = Uuid::from_u128(0xa104);
    let track_b_uuid = Uuid::from_u128(0xa105);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "plus-one-gap".into(),
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
                Point::new(2_000_000, 0),
                Point::new(2_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        uuid: from_pad_uuid,
                        package: Uuid::from_u128(0xa110),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 500_000),
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
                        package: Uuid::from_u128(0xa111),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(1_900_000, 500_000),
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
                    track_a_uuid,
                    Track {
                        uuid: track_a_uuid,
                        net: net_uuid,
                        from: Point::new(100_000, 500_000),
                        to: Point::new(700_000, 500_000),
                        width: 150_000,
                        layer: 1,
                    },
                ),
                (
                    track_b_uuid,
                    Track {
                        uuid: track_b_uuid,
                        net: net_uuid,
                        from: Point::new(1_300_000, 500_000),
                        to: Point::new(1_900_000, 500_000),
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
                    track_width: 150_000,
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
        track_b_uuid,
    )
}

#[test]
fn route_path_candidate_authored_copper_plus_one_gap_reports_deterministic_path() {
    let (board, net_uuid, from_pad_uuid, to_pad_uuid, track_a_uuid, track_b_uuid) =
        plus_one_gap_board();
    let report = board
        .route_path_candidate_authored_copper_plus_one_gap(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::DeterministicPathFound
    );
    assert_eq!(report.summary.candidate_track_count, 2);
    assert_eq!(report.summary.candidate_via_count, 0);
    assert_eq!(report.summary.candidate_gap_count, 1);
    assert_eq!(report.summary.path_step_count, 3);
    assert_eq!(report.summary.path_gap_step_count, 1);

    let path = report.path.expect("path should exist");
    assert_eq!(path.steps.len(), 3);
    assert_eq!(
        path.steps
            .iter()
            .map(|step| step.object_uuid)
            .collect::<Vec<_>>(),
        vec![Some(track_a_uuid), None, Some(track_b_uuid)]
    );
    assert!(matches!(
        path.steps[1].kind,
        RoutePathCandidateAuthoredCopperPlusOneGapStepKindView::Gap
    ));
}

#[test]
fn route_path_candidate_authored_copper_plus_one_gap_reports_no_path_when_gap_blocked() {
    let (mut board, net_uuid, from_pad_uuid, to_pad_uuid, _, _) = plus_one_gap_board();
    board.keepouts.push(Keepout {
        uuid: Uuid::from_u128(0xa120),
        polygon: Polygon::new(vec![
            Point::new(750_000, 400_000),
            Point::new(1_250_000, 400_000),
            Point::new(1_250_000, 600_000),
            Point::new(750_000, 600_000),
        ]),
        layers: vec![1],
        kind: "route".into(),
    });

    let report = board
        .route_path_candidate_authored_copper_plus_one_gap(net_uuid, from_pad_uuid, to_pad_uuid)
        .expect("query should succeed");

    assert_eq!(
        report.status,
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    );
    assert!(report.path.is_none());
}
