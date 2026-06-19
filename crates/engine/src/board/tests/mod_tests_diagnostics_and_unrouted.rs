use std::collections::HashMap;

use uuid::Uuid;

use crate::board::*;
use crate::ir::geometry::Point;

#[test]
fn board_diagnostics_reports_empty_and_via_only_nets() {
    let gnd = Uuid::new_v4();
    let vcc = Uuid::new_v4();
    let avcc = Uuid::new_v4();
    let class = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup { layers: Vec::new() },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::new(),
        pads: HashMap::new(),
        tracks: HashMap::from([(
            Uuid::new_v4(),
            Track {
                uuid: Uuid::new_v4(),
                net: gnd,
                from: Point::new(0, 0),
                to: Point::new(10, 0),
                width: 200_000,
                layer: 1,
            },
        )]),
        vias: HashMap::from([(
            Uuid::new_v4(),
            Via {
                uuid: Uuid::new_v4(),
                net: avcc,
                position: Point::new(5, 5),
                drill: 300_000,
                diameter: 600_000,
                from_layer: 1,
                to_layer: 2,
            },
        )]),
        zones: HashMap::new(),
        nets: HashMap::from([
            (
                gnd,
                Net {
                    uuid: gnd,
                    name: "GND".into(),
                    class,
                },
            ),
            (
                vcc,
                Net {
                    uuid: vcc,
                    name: "VCC".into(),
                    class,
                },
            ),
            (
                avcc,
                Net {
                    uuid: avcc,
                    name: "AVCC".into(),
                    class,
                },
            ),
        ]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let diagnostics = board.diagnostics();
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.kind == "net_without_copper" && d.objects == vec![vcc])
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.kind == "via_only_net" && d.objects == vec![avcc])
    );
}

#[test]
fn board_diagnostics_report_partially_routed_net() {
    let net_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup { layers: Vec::new() },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                pkg_a,
                PlacedPackage {
                    uuid: pkg_a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R1".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
            (
                pkg_b,
                PlacedPackage {
                    uuid: pkg_b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R2".into(),
                    value: "10k".into(),
                    position: Point::new(10_000_000, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([
            (
                Uuid::new_v4(),
                PlacedPad {
                    uuid: Uuid::new_v4(),
                    package: pkg_a,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(0, 0),
                    layer: 0,
                    copper_layers: Vec::new(),
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 0,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
            (
                Uuid::new_v4(),
                PlacedPad {
                    uuid: Uuid::new_v4(),
                    package: pkg_b,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(10_000_000, 0),
                    layer: 0,
                    copper_layers: Vec::new(),
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 0,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
        ]),
        tracks: HashMap::from([(
            Uuid::new_v4(),
            Track {
                uuid: Uuid::new_v4(),
                net: net_uuid,
                from: Point::new(0, 0),
                to: Point::new(4_000_000, 0),
                width: 200_000,
                layer: 0,
            },
        )]),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: Uuid::nil(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let diagnostics = board.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, "partially_routed_net");
    assert_eq!(diagnostics[0].severity, "warning");
}

#[test]
fn board_net_info_counts_zones_as_copper_coverage() {
    let net_uuid = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup { layers: Vec::new() },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::new(),
        pads: HashMap::new(),
        tracks: HashMap::new(),
        vias: HashMap::new(),
        zones: HashMap::from([(
            Uuid::new_v4(),
            Zone {
                uuid: Uuid::new_v4(),
                net: net_uuid,
                polygon: Polygon::new(vec![
                    Point::new(0, 0),
                    Point::new(5, 0),
                    Point::new(5, 5),
                    Point::new(0, 5),
                ]),
                layer: 1,
                priority: 1,
                thermal_relief: true,
                thermal_gap: 200_000,
                thermal_spoke_width: 200_000,
            },
        )]),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "GND".into(),
                class: Uuid::new_v4(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let infos = board.net_info();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].zones, 1);
    assert_eq!(infos[0].routed_pct, 1.0);
    assert!(board.diagnostics().is_empty());
}

#[test]
fn board_unrouted_computes_airwires_from_pad_endpoints() {
    let net_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let pad_a = Uuid::new_v4();
    let pad_b = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "demo".into(),
        stackup: Stackup { layers: Vec::new() },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                pkg_a,
                PlacedPackage {
                    uuid: pkg_a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R1".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
            (
                pkg_b,
                PlacedPackage {
                    uuid: pkg_b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R2".into(),
                    value: "10k".into(),
                    position: Point::new(10_000_000, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([
            (
                pad_a,
                PlacedPad {
                    uuid: pad_a,
                    package: pkg_a,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(0, 0),
                    layer: 0,
                    copper_layers: Vec::new(),
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 0,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
            (
                pad_b,
                PlacedPad {
                    uuid: pad_b,
                    package: pkg_b,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(10_000_000, 0),
                    layer: 0,
                    copper_layers: Vec::new(),
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 0,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
        ]),
        tracks: HashMap::new(),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: Uuid::nil(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let airwires = board.unrouted();
    assert_eq!(airwires.len(), 1);
    assert_eq!(airwires[0].net_name, "SIG");
    assert_eq!(airwires[0].from.component, "R1");
    assert_eq!(airwires[0].to.component, "R2");
    assert_eq!(airwires[0].distance_nm, 10_000_000);
}

#[test]
fn board_unrouted_treats_multilayer_pads_as_connected_on_bottom_copper() {
    let net_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let pad_a = Uuid::new_v4();
    let pad_b = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "bottom-copper-complete".into(),
        stackup: Stackup {
            layers: vec![
                StackupLayer {
                    id: 0,
                    name: "F.Cu".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                },
                StackupLayer {
                    id: 31,
                    name: "B.Cu".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                },
            ],
        },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                pkg_a,
                PlacedPackage {
                    uuid: pkg_a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "J1".into(),
                    value: "A".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
            (
                pkg_b,
                PlacedPackage {
                    uuid: pkg_b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "J2".into(),
                    value: "B".into(),
                    position: Point::new(10_000_000, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([
            (
                pad_a,
                PlacedPad {
                    uuid: pad_a,
                    package: pkg_a,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(0, 0),
                    layer: 0,
                    copper_layers: vec![0, 31],
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 1_000_000,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
            (
                pad_b,
                PlacedPad {
                    uuid: pad_b,
                    package: pkg_b,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(10_000_000, 0),
                    layer: 0,
                    copper_layers: vec![0, 31],
                    shape: crate::board::PadShape::Circle,
                    diameter: 0,
                    width: 0,
                    height: 0,
                    drill: 1_000_000,
                    rotation: 0,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
        ]),
        tracks: HashMap::from([(
            track_uuid,
            Track {
                uuid: track_uuid,
                net: net_uuid,
                from: Point::new(0, 0),
                to: Point::new(10_000_000, 0),
                width: 200_000,
                layer: 31,
            },
        )]),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: Uuid::nil(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let airwires = board.unrouted();
    assert!(
        airwires.is_empty(),
        "multilayer pads connected only on B.Cu must not be reported as unrouted"
    );
}

#[test]
fn board_unrouted_treats_track_endpoint_inside_rotated_rect_pad_as_connected() {
    let net_uuid = Uuid::new_v4();
    let pkg_a = Uuid::new_v4();
    let pkg_b = Uuid::new_v4();
    let pad_a = Uuid::new_v4();
    let pad_b = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let board = Board {
        uuid: Uuid::new_v4(),
        name: "pad-shape-complete".into(),
        stackup: Stackup {
            layers: vec![StackupLayer {
                id: 0,
                name: "F.Cu".into(),
                layer_type: StackupLayerType::Copper,
                thickness_nm: 35_000,
            }],
        },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        packages: HashMap::from([
            (
                pkg_a,
                PlacedPackage {
                    uuid: pkg_a,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R1".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
            (
                pkg_b,
                PlacedPackage {
                    uuid: pkg_b,
                    part: Uuid::nil(),
                    package: Uuid::nil(),
                    reference: "R2".into(),
                    value: "10k".into(),
                    position: Point::new(10_000_000, 0),
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            ),
        ]),
        pads: HashMap::from([
            (
                pad_a,
                PlacedPad {
                    uuid: pad_a,
                    package: pkg_a,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(0, 0),
                    layer: 0,
                    copper_layers: vec![0],
                    shape: crate::board::PadShape::Rect,
                    diameter: 0,
                    width: 2_000_000,
                    height: 1_000_000,
                    drill: 0,
                    rotation: 90,
                    mask_layers: Vec::new(),
                    paste_layers: Vec::new(),
                    solder_mask_margin_nm: 0,
                    solder_paste_margin_nm: 0,
                    solder_paste_margin_ratio_ppm: 0,
                    roundrect_rratio_ppm: 250_000,
                },
            ),
            (
                pad_b,
                PlacedPad {
                    uuid: pad_b,
                    package: pkg_b,
                    name: "1".into(),
                    net: Some(net_uuid),
                    position: Point::new(10_000_000, 0),
                    layer: 0,
                    copper_layers: vec![0],
                    shape: crate::board::PadShape::Circle,
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
            ),
        ]),
        tracks: HashMap::from([(
            track_uuid,
            Track {
                uuid: track_uuid,
                net: net_uuid,
                from: Point::new(400_000, 0),
                to: Point::new(10_000_000, 0),
                width: 200_000,
                layer: 0,
            },
        )]),
        vias: HashMap::new(),
        zones: HashMap::new(),
        nets: HashMap::from([(
            net_uuid,
            Net {
                uuid: net_uuid,
                name: "SIG".into(),
                class: Uuid::nil(),
            },
        )]),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };

    let airwires = board.unrouted();
    assert!(
        airwires.is_empty(),
        "track endpoint landing inside rotated rect pad copper must count as connected"
    );
}
