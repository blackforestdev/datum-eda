use super::empty_board;
use crate::board::{Net, NetClass, PadExpansionSetup, PadShape, PlacedPad, Track, Via};
use crate::drc::{RuleType, run};
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
            controlled_impedance: None,
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
            controlled_impedance: None,
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
            controlled_impedance: None,
        },
    );

    let text_uuid = Uuid::new_v4();
    board.texts.push(crate::board::BoardText {
        uuid: text_uuid,
        text: "REF".into(),
        position: Point::new(10_000_000, 10_000_000),
        rotation: 0,
        render_intent: crate::text::TextRenderIntent::Manufacturing,
        family: crate::text::TextFamilyId::default(),
        family_source: crate::text::TextFamilySource::ImplicitDefault,
        style: crate::text::TextStyleId::default(),
        height_nm: 1_000_000,
        stroke_width_nm: 100_000,
        layer: 37,
        h_align: crate::text::TextHAlign::Left,
        v_align: crate::text::TextVAlign::Bottom,
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1_000_000,
        italic: false,
        bold: false,
        style_class: None,
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

#[test]
fn process_aperture_check_reports_inherited_mask_and_paste() {
    let mut board = empty_board();
    board.pad_expansion_setup = PadExpansionSetup {
        pad_to_mask_clearance_nm: 127_000,
        pad_to_paste_clearance_nm: -127_000,
        pad_to_paste_ratio_ppm: 0,
        solder_mask_min_width_nm: 0,
    };

    let package_uuid = Uuid::new_v4();
    let pad_uuid = Uuid::new_v4();
    board.pads.insert(
        pad_uuid,
        PlacedPad {
            uuid: pad_uuid,
            package: package_uuid,
            name: "1".into(),
            net: None,
            position: Point::new(1_000_000, 2_000_000),
            layer: 0,
            copper_layers: vec![0],
            shape: PadShape::Rect,
            diameter: 0,
            width: 1_000_000,
            height: 500_000,
            drill: 0,
            rotation: 0,
            roundrect_rratio_ppm: 250_000,
            mask_layers: vec![2],
            paste_layers: vec![4],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
        },
    );

    let report = run(&board, &[RuleType::ProcessAperture]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 3);
    assert_eq!(
        report
            .violations
            .iter()
            .map(|violation| violation.code.as_str())
            .collect::<Vec<_>>(),
        vec![
            "pad_mask_expansion_missing",
            "pad_paste_reduction_missing",
            "pad_process_aperture_inherited_from_copper"
        ]
    );
    assert!(
        report
            .violations
            .iter()
            .all(|violation| violation.objects == vec![pad_uuid])
    );

    let repeat = run(&board, &[RuleType::ProcessAperture]);
    assert_eq!(
        report
            .violations
            .iter()
            .map(|violation| violation.id)
            .collect::<Vec<_>>(),
        repeat
            .violations
            .iter()
            .map(|violation| violation.id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn process_aperture_check_reports_peer_footprint_policy_inconsistency() {
    let mut board = empty_board();
    let package_uuid = Uuid::new_v4();
    let pad_1_uuid = Uuid::new_v4();
    let pad_2_uuid = Uuid::new_v4();
    let pad_3_uuid = Uuid::new_v4();

    for (uuid, name, x_nm, mask_margin_nm) in [
        (pad_1_uuid, "1", 1_000_000, 75_000),
        (pad_2_uuid, "2", 2_000_000, 75_000),
        (pad_3_uuid, "3", 3_000_000, 100_000),
    ] {
        board.pads.insert(
            uuid,
            PlacedPad {
                uuid,
                package: package_uuid,
                name: name.into(),
                net: None,
                position: Point::new(x_nm, 2_000_000),
                layer: 0,
                copper_layers: vec![0],
                shape: PadShape::Rect,
                diameter: 0,
                width: 1_000_000,
                height: 500_000,
                drill: 0,
                rotation: 0,
                roundrect_rratio_ppm: 250_000,
                mask_layers: vec![2],
                paste_layers: vec![4],
                solder_mask_margin_nm: mask_margin_nm,
                solder_paste_margin_nm: -75_000,
                solder_paste_margin_ratio_ppm: 0,
            },
        );
    }

    let report = run(&board, &[RuleType::ProcessAperture]);
    assert!(!report.passed);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(
        report.violations[0].code,
        "pad_process_aperture_inconsistent_with_peer_footprint"
    );
    assert_eq!(report.violations[0].objects, vec![pad_3_uuid]);

    let repeat = run(&board, &[RuleType::ProcessAperture]);
    assert_eq!(report.violations[0].id, repeat.violations[0].id);
}
