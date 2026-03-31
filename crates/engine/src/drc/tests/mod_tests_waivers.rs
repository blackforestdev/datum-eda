use super::empty_board;
use crate::board::{Net, NetClass, PlacedPackage, Track};
use crate::drc::run_with_waivers;
use crate::ir::geometry::Point;
use crate::rules::ast::RuleType;
use crate::schematic::{CheckDomain, CheckWaiver, WaiverTarget};
use uuid::Uuid;

fn default_net_class() -> (Uuid, NetClass) {
    let class_uuid = Uuid::new_v4();
    (
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
    )
}

#[test]
fn authored_drc_object_waiver_marks_matching_violation_as_waived() {
    let mut board = empty_board();
    let (class_uuid, net_class) = default_net_class();
    let net_uuid = Uuid::new_v4();
    board.net_classes.insert(class_uuid, net_class);
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
            shape: crate::board::PadShape::Circle,
            diameter: 0,
            width: 0,
            height: 0,
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
            shape: crate::board::PadShape::Circle,
            diameter: 0,
            width: 0,
            height: 0,
        },
    );

    let waiver = CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::DRC,
        target: WaiverTarget::Object(net_uuid),
        rationale: "Intentional incomplete route in fixture".into(),
        created_by: Some("drc-test".into()),
    };

    let report = run_with_waivers(&board, &[RuleType::Connectivity], &[waiver]);
    assert!(report.passed);
    assert_eq!(report.summary.errors, 0);
    assert_eq!(report.summary.waived, 2);
    assert_eq!(report.violations.len(), 2);
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_no_copper" && violation.waived)
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net" && violation.waived)
    );
}

#[test]
fn authored_drc_rule_objects_waiver_matches_independent_of_object_order() {
    let mut board = empty_board();
    let (class_uuid, net_class) = default_net_class();
    board.net_classes.insert(class_uuid, net_class);

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

    let waiver = CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::DRC,
        target: WaiverTarget::RuleObjects {
            rule: "clearance_copper".into(),
            objects: vec![track_b, track_a],
        },
        rationale: "Known fixture overlap".into(),
        created_by: Some("drc-test".into()),
    };

    let report = run_with_waivers(&board, &[RuleType::ClearanceCopper], &[waiver]);
    assert!(report.passed);
    assert_eq!(report.summary.errors, 0);
    assert_eq!(report.summary.waived, 1);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].code, "clearance_copper");
    assert!(report.violations[0].waived);
}
