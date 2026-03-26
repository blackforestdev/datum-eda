use super::empty_board;
use crate::board::{Net, NetClass, Track, Via};
use crate::drc::{run, RuleType};
use crate::ir::geometry::Point;
use uuid::Uuid;

#[test]
fn track_width_check_reports_below_minimum_width() {
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
    let track_uuid = Uuid::new_v4();
    board.tracks.insert(
        track_uuid,
        Track {
            uuid: track_uuid,
            net: net_uuid,
            from: Point::new(0, 0),
            to: Point::new(10_000_000, 0),
            width: 100_000,
            layer: 1,
        },
    );

    let report = run(&board, &[RuleType::TrackWidth]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.violations[0].code, "track_width_below_min");
    assert_eq!(report.violations[0].objects, vec![track_uuid]);
}

#[test]
fn via_checks_report_small_hole_and_annular_ring() {
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
            via_drill: 200_000,
            via_diameter: 500_000,
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
    let via_uuid = Uuid::new_v4();
    board.vias.insert(
        via_uuid,
        Via {
            uuid: via_uuid,
            net: net_uuid,
            position: Point::new(1_000_000, 2_000_000),
            drill: 100_000,
            diameter: 200_000,
            from_layer: 1,
            to_layer: 2,
        },
    );

    let hole_report = run(&board, &[RuleType::ViaHole]);
    assert!(!hole_report.passed);
    assert_eq!(hole_report.summary.errors, 1);
    assert_eq!(hole_report.violations[0].code, "via_hole_out_of_range");

    let annular_report = run(&board, &[RuleType::ViaAnnularRing]);
    assert!(!annular_report.passed);
    assert_eq!(annular_report.summary.errors, 1);
    assert_eq!(annular_report.violations[0].code, "via_annular_below_min");
}

#[test]
fn silk_clearance_reports_text_too_close_to_track() {
    let mut board = empty_board();
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
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

    let text_uuid = Uuid::new_v4();
    board.texts.push(crate::board::BoardText {
        uuid: text_uuid,
        text: "REF".into(),
        position: Point::new(10_000_000, 10_000_000),
        rotation: 0,
        layer: 37,
    });
    let track_uuid = Uuid::new_v4();
    board.tracks.insert(
        track_uuid,
        Track {
            uuid: track_uuid,
            net: net_uuid,
            from: Point::new(9_800_000, 10_000_000),
            to: Point::new(10_200_000, 10_000_000),
            width: 100_000,
            layer: 0,
        },
    );

    let report = run(&board, &[RuleType::SilkClearance]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.violations[0].code, "silk_clearance_copper");
    let mut expected = vec![text_uuid, track_uuid];
    expected.sort();
    assert_eq!(report.violations[0].objects, expected);
}
