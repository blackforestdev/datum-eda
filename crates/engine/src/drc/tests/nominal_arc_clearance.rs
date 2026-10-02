use super::empty_board;
use crate::{
    board::{Net, NetClass, Track},
    drc::{RuleType, run},
    ir::geometry::Point,
};
use uuid::Uuid;
#[test]
fn clearance_uses_arc_locus_and_keeps_equality_out_of_violations() {
    let mut board = empty_board();
    let class = Uuid::from_u128(10);
    board.net_classes.insert(
        class,
        NetClass {
            uuid: class,
            name: "test".into(),
            clearance: 0,
            track_width: 1,
            via_drill: 1,
            via_diameter: 1,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    );
    for id in [1, 2] {
        let uuid = Uuid::from_u128(id);
        board.nets.insert(
            uuid,
            Net {
                uuid,
                name: id.to_string(),
                class,
                controlled_impedance: None,
            },
        );
    }
    let mut arc = Track::straight(
        Uuid::from_u128(11),
        Uuid::from_u128(1),
        Point::new(-5, 0),
        Point::new(5, 0),
        1,
        1,
    );
    arc.midpoint = Some(Point::new(0, 5));
    board.tracks.insert(arc.uuid, arc.clone());
    let query = board.route_path_candidate(arc.net, Uuid::from_u128(30), Uuid::from_u128(31));
    assert!(
        matches!(query,Err(crate::board::RoutePathCandidateError::UnsupportedArcSources{track_ids}) if track_ids==vec![arc.uuid])
    );
    let connectivity = run(&board, &[RuleType::Connectivity]);
    assert!(!connectivity.passed);
    assert!(
        connectivity
            .violations
            .iter()
            .any(|finding| finding.code == "nominal_connectivity_unavailable"
                && finding.objects == vec![arc.uuid])
    );
    assert!(
        board
            .diagnostics()
            .iter()
            .any(|finding| finding.kind == "nominal_connectivity_unavailable"
                && finding.objects == vec![arc.uuid])
    );
    let mut line = Track::straight(
        Uuid::from_u128(12),
        Uuid::from_u128(2),
        Point::new(-1, 0),
        Point::new(1, 0),
        1,
        1,
    );
    board.tracks.insert(line.uuid, line.clone());
    assert!(run(&board, &[RuleType::ClearanceCopper]).passed); // chord would report a fault
    line.from = Point::new(-20, 6);
    line.to = Point::new(20, 6);
    board.tracks.insert(line.uuid, line.clone());
    assert!(run(&board, &[RuleType::ClearanceCopper]).passed); // exactly touching
    board.net_classes.get_mut(&class).unwrap().clearance = 1;
    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert!(!report.passed);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].objects, vec![arc.uuid, line.uuid]);
    assert_eq!(report.violations[0].code, "clearance_copper");
    assert!(report.violations[0].fingerprint.is_some());
    let via = crate::board::Via {
        uuid: Uuid::from_u128(40),
        net: line.net,
        position: Point::new(0, 5),
        drill: 1,
        diameter: 2,
        from_layer: 1,
        to_layer: 1,
    };
    board.vias.insert(via.uuid, via.clone());
    let classified = run(&board, &[RuleType::ClearanceCopper]);
    assert!(
        classified
            .violations
            .iter()
            .any(|finding| finding.code == "clearance_copper"
                && finding.objects == vec![arc.uuid, via.uuid])
    );
    board.vias.clear();
    arc.midpoint = Some(Point::new(0, 0));
    board.tracks.insert(arc.uuid, arc);
    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert!(!report.passed);
    assert_eq!(report.violations[0].code, "nominal_geometry_unavailable");
}

#[test]
fn certified_pad_and_via_clearance_uses_actual_copper_and_span() {
    use crate::board::{StackupLayer, StackupLayerType};
    let mut board = empty_board();
    board.stackup.layers = vec![
        StackupLayer::new(1, "A", StackupLayerType::Copper, 1),
        StackupLayer::new(2, "B", StackupLayerType::Copper, 1),
    ];
    let class = Uuid::from_u128(90);
    board.net_classes.insert(
        class,
        NetClass {
            uuid: class,
            name: "".into(),
            clearance: 0,
            track_width: 1,
            via_drill: 1,
            via_diameter: 3,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    );
    for id in [1, 2] {
        let uuid = Uuid::from_u128(id);
        board
            .nets
            .insert(uuid, Net::new(uuid, "same display", class));
    }
    let mut arc = Track::straight(
        Uuid::from_u128(11),
        Uuid::from_u128(1),
        Point::new(-5, 0),
        Point::new(5, 0),
        1,
        1,
    );
    arc.midpoint = Some(Point::new(0, 5));
    board.tracks.insert(arc.uuid, arc.clone());
    let pad:crate::board::PlacedPad=serde_json::from_value(serde_json::json!({"uuid":Uuid::from_u128(12),"package":Uuid::new_v4(),"name":"","net":Uuid::from_u128(2),"position":{"x":0,"y":7},"layer":1,"diameter":3})).unwrap();
    board.pads.insert(pad.uuid, pad.clone());
    assert!(run(&board, &[RuleType::ClearanceCopper]).passed);
    board.net_classes.get_mut(&class).unwrap().clearance = 1;
    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].code, "clearance_copper");
    assert_eq!(report.violations[0].objects, vec![arc.uuid, pad.uuid]);
    assert!(report.violations[0].fingerprint.is_some());
    board.pads.clear();
    let mut via = crate::board::Via {
        uuid: Uuid::from_u128(13),
        net: pad.net.unwrap(),
        position: pad.position,
        diameter: 3,
        drill: 1,
        from_layer: 1,
        to_layer: 2,
    };
    board.vias.insert(via.uuid, via.clone());
    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].objects, vec![arc.uuid, via.uuid]);
    assert_eq!(report.violations[0].code, "clearance_copper");
    via.from_layer = 2;
    board.vias.insert(via.uuid, via.clone());
    assert!(run(&board, &[RuleType::ClearanceCopper]).passed);
    via.from_layer = 999;
    board.vias.insert(via.uuid, via.clone());
    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].code, "nominal_geometry_unavailable");
    assert_eq!(report.violations[0].objects, vec![arc.uuid, via.uuid]);
}
