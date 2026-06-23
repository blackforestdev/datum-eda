use std::collections::BTreeMap;

use super::empty_board;
use crate::board::{Net, PadShape, PlacedPackage, PlacedPad, Zone};
use crate::drc::{RuleType, run_with_zone_fills};
use crate::ir::geometry::{Point, Polygon};
use crate::substrate::{ModelRevision, ObjectRevision, ZoneFill, ZoneFillState};
use uuid::Uuid;

#[test]
fn drc_connectivity_requires_filled_zone_evidence() {
    let net = Uuid::new_v4();
    let class = Uuid::new_v4();
    let package = Uuid::new_v4();
    let zone = Uuid::new_v4();
    let mut board = empty_board();
    board.nets.insert(net, Net::new(net, "GND", class));
    board.packages.insert(
        package,
        PlacedPackage {
            uuid: package,
            part: Uuid::nil(),
            package: Uuid::nil(),
            reference: "J1".into(),
            value: "CONN".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        },
    );
    board.pads.insert(
        Uuid::new_v4(),
        PlacedPad {
            uuid: Uuid::new_v4(),
            package,
            name: "1".into(),
            net: Some(net),
            position: Point::new(10_000_000, 10_000_000),
            layer: 1,
            copper_layers: vec![1],
            shape: PadShape::Circle,
            diameter: 1_000_000,
            width: 1_000_000,
            height: 1_000_000,
            drill: 0,
            rotation: 0,
            mask_layers: Vec::new(),
            paste_layers: Vec::new(),
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            roundrect_rratio_ppm: 250_000,
        },
    );
    board.zones.insert(
        zone,
        Zone {
            uuid: zone,
            net,
            polygon: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(20_000_000, 0),
                Point::new(20_000_000, 20_000_000),
                Point::new(0, 20_000_000),
            ]),
            layer: 1,
            priority: 1,
            thermal_relief: false,
            thermal_gap: 0,
            thermal_spoke_width: 0,
        },
    );

    let unfilled_report = run_with_zone_fills(&board, &[RuleType::Connectivity], &BTreeMap::new());
    assert!(
        unfilled_report
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unconnected_pin"),
        "authored zone boundaries must not count as routed copper without filled evidence"
    );

    let mut fills = BTreeMap::new();
    fills.insert(
        zone,
        ZoneFill {
            zone_id: zone,
            state: ZoneFillState::Filled,
            source_zone_revision: ObjectRevision(0),
            model_revision: ModelRevision("test-revision".into()),
            islands: vec![board.zones[&zone].polygon.clone()],
            provenance: Some("test filled zone".into()),
        },
    );
    let filled_report = run_with_zone_fills(&board, &[RuleType::Connectivity], &fills);
    assert!(
        filled_report.violations.iter().all(|violation| {
            violation.code != "connectivity_unconnected_pin"
                && violation.code != "connectivity_no_copper"
        }),
        "filled ZoneFill evidence should project zone copper into DRC connectivity"
    );
}

/// Build a two-pin board on a single net whose ONLY copper is a zone covering
/// both pads. Connectivity therefore depends entirely on whether the zone's
/// fill state is trusted as copper.
fn board_with_two_pin_zone_only_net() -> (crate::board::Board, Uuid) {
    let net = Uuid::new_v4();
    let class = Uuid::new_v4();
    let package = Uuid::new_v4();
    let zone = Uuid::new_v4();
    let mut board = empty_board();
    board.nets.insert(net, Net::new(net, "GND", class));
    board.packages.insert(
        package,
        PlacedPackage {
            uuid: package,
            part: Uuid::nil(),
            package: Uuid::nil(),
            reference: "J1".into(),
            value: "CONN".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        },
    );
    for (name, x, y) in [("1", 4_000_000, 4_000_000), ("2", 16_000_000, 16_000_000)] {
        board.pads.insert(
            Uuid::new_v4(),
            PlacedPad {
                uuid: Uuid::new_v4(),
                package,
                name: name.into(),
                net: Some(net),
                position: Point::new(x, y),
                layer: 1,
                copper_layers: vec![1],
                shape: PadShape::Circle,
                diameter: 1_000_000,
                width: 1_000_000,
                height: 1_000_000,
                drill: 0,
                rotation: 0,
                mask_layers: Vec::new(),
                paste_layers: Vec::new(),
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                roundrect_rratio_ppm: 250_000,
            },
        );
    }
    board.zones.insert(
        zone,
        Zone {
            uuid: zone,
            net,
            polygon: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(20_000_000, 0),
                Point::new(20_000_000, 20_000_000),
                Point::new(0, 20_000_000),
            ]),
            layer: 1,
            priority: 1,
            thermal_relief: false,
            thermal_gap: 0,
            thermal_spoke_width: 0,
        },
    );
    (board, zone)
}

fn fill_with_state(board: &crate::board::Board, zone: Uuid, state: ZoneFillState) -> ZoneFill {
    // Stale/Unfilled/Unsupported carry no trustworthy islands; only Filled does.
    let islands = if state == ZoneFillState::Filled {
        vec![board.zones[&zone].polygon.clone()]
    } else {
        Vec::new()
    };
    ZoneFill {
        zone_id: zone,
        state,
        source_zone_revision: ObjectRevision(0),
        model_revision: ModelRevision("test-revision".into()),
        islands,
        provenance: Some(format!("test zone fill: {state:?}")),
    }
}

fn has_connectivity_violation(report: &crate::drc::DrcReport) -> bool {
    report.violations.iter().any(|violation| {
        violation.code == "connectivity_no_copper"
            || violation.code == "connectivity_unrouted_net"
            || violation.code == "connectivity_unconnected_pin"
    })
}

/// A multi-pin net whose only copper is a non-Filled zone must FAIL DRC
/// connectivity for every untrusted fill state. A Filled zone must PASS.
#[test]
fn drc_connectivity_rejects_every_non_filled_zone_state() {
    let (board, zone) = board_with_two_pin_zone_only_net();

    for untrusted in [
        ZoneFillState::Unfilled,
        ZoneFillState::Stale,
        ZoneFillState::Unsupported,
    ] {
        let mut fills = BTreeMap::new();
        fills.insert(zone, fill_with_state(&board, zone, untrusted));
        let report = run_with_zone_fills(&board, &[RuleType::Connectivity], &fills);
        assert!(
            !report.passed && has_connectivity_violation(&report),
            "a two-pin net poured only by a {untrusted:?} zone must NOT pass DRC connectivity"
        );
    }

    // Filled evidence is the only state that contributes copper.
    let mut fills = BTreeMap::new();
    fills.insert(zone, fill_with_state(&board, zone, ZoneFillState::Filled));
    let report = run_with_zone_fills(&board, &[RuleType::Connectivity], &fills);
    assert!(
        report.passed && !has_connectivity_violation(&report),
        "a two-pin net poured by a Filled zone must pass DRC connectivity"
    );
}
