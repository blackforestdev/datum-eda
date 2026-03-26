use super::empty_board;
use crate::board::{Net, NetClass, PlacedPackage, Track};
use crate::drc::{run, RuleType};
use crate::ir::geometry::Point;
use uuid::Uuid;

#[test]
fn connectivity_check_reports_no_copper_net_with_two_pins() {
    let mut board = empty_board();
    let class_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    board.net_classes.insert(
        class_uuid,
        NetClass {
            uuid: class_uuid,
            name: "default".into(),
            clearance: 100_000,
            track_width: 200_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    );
    board.nets.insert(
        net_uuid,
        Net {
            uuid: net_uuid,
            name: "SIG".into(),
            class: class_uuid,
        },
    );
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    board.packages.insert(
        pkg_a,
        PlacedPackage {
            uuid: pkg_a,
            part: Uuid::new_v4(),
            package: Uuid::nil(),
            reference: "R1".into(),
            value: "10k".into(),
            position: Point::new(10_000_000, 10_000_000),
            rotation: 0,
            layer: 1,
            locked: false,
        },
    );
    board.packages.insert(
        pkg_b,
        PlacedPackage {
            uuid: pkg_b,
            part: Uuid::new_v4(),
            package: Uuid::nil(),
            reference: "R2".into(),
            value: "10k".into(),
            position: Point::new(40_000_000, 10_000_000),
            rotation: 0,
            layer: 1,
            locked: false,
        },
    );
    board.pads.insert(
        Uuid::new_v4(),
        crate::board::PlacedPad {
            uuid: Uuid::new_v4(),
            package: pkg_a,
            name: "1".into(),
            net: Some(net_uuid),
            position: Point::new(10_000_000, 10_000_000),
            layer: 1,
        },
    );
    board.pads.insert(
        Uuid::new_v4(),
        crate::board::PlacedPad {
            uuid: Uuid::new_v4(),
            package: pkg_b,
            name: "1".into(),
            net: Some(net_uuid),
            position: Point::new(40_000_000, 10_000_000),
            layer: 1,
        },
    );

    let report = run(&board, &[RuleType::Connectivity]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 2);
    assert!(
        report
            .violations
            .iter()
            .any(|v| v.code == "connectivity_no_copper")
    );
    assert!(
        report
            .violations
            .iter()
            .any(|v| v.code == "connectivity_unrouted_net")
    );
}

#[test]
fn clearance_check_reports_overlapping_tracks_on_different_nets() {
    let mut board = empty_board();
    let class_uuid = Uuid::new_v4();
    board.net_classes.insert(
        class_uuid,
        NetClass {
            uuid: class_uuid,
            name: "default".into(),
            clearance: 200_000,
            track_width: 200_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    );
    let net_a = Uuid::new_v4();
    let net_b = Uuid::new_v4();
    board.nets.insert(
        net_a,
        Net {
            uuid: net_a,
            name: "A".into(),
            class: class_uuid,
        },
    );
    board.nets.insert(
        net_b,
        Net {
            uuid: net_b,
            name: "B".into(),
            class: class_uuid,
        },
    );

    let track_a = Uuid::new_v4();
    let track_b = Uuid::new_v4();
    board.tracks.insert(
        track_a,
        Track {
            uuid: track_a,
            net: net_a,
            from: Point::new(0, 0),
            to: Point::new(10_000_000, 0),
            width: 200_000,
            layer: 1,
        },
    );
    board.tracks.insert(
        track_b,
        Track {
            uuid: track_b,
            net: net_b,
            from: Point::new(0, 100_000),
            to: Point::new(10_000_000, 100_000),
            width: 200_000,
            layer: 1,
        },
    );

    let report = run(&board, &[RuleType::ClearanceCopper]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].code, "clearance_copper");
    let mut expected = vec![track_a, track_b];
    expected.sort();
    assert_eq!(report.violations[0].objects, expected);
}

#[test]
fn connectivity_reports_single_pin_unconnected_pin_violation() {
    let mut board = empty_board();
    let class_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    board.net_classes.insert(
        class_uuid,
        NetClass {
            uuid: class_uuid,
            name: "default".into(),
            clearance: 100_000,
            track_width: 200_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        },
    );
    board.nets.insert(
        net_uuid,
        Net {
            uuid: net_uuid,
            name: "SIG".into(),
            class: class_uuid,
        },
    );
    let pkg = Uuid::new_v4();
    board.packages.insert(
        pkg,
        PlacedPackage {
            uuid: pkg,
            part: Uuid::new_v4(),
            package: Uuid::nil(),
            reference: "TP1".into(),
            value: "TP".into(),
            position: Point::new(10_000_000, 10_000_000),
            rotation: 0,
            layer: 1,
            locked: false,
        },
    );
    board.pads.insert(
        Uuid::new_v4(),
        crate::board::PlacedPad {
            uuid: Uuid::new_v4(),
            package: pkg,
            name: "1".into(),
            net: Some(net_uuid),
            position: Point::new(10_000_000, 10_000_000),
            layer: 1,
        },
    );

    let report = run(&board, &[RuleType::Connectivity]);
    assert!(!report.passed);
    assert!(
        report
            .violations
            .iter()
            .any(|v| v.code == "connectivity_unconnected_pin")
    );
}
