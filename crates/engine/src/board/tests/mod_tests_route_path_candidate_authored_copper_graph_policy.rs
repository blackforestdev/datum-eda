#[path = "route_policy_preservation_tests.rs"]
mod route_policy_preservation_tests;

use std::collections::HashMap;

use crate::board::*;
use crate::ir::geometry::{Point, Polygon};
use uuid::Uuid;

fn path_ids(report: &RoutePathCandidateAuthoredCopperGraphPolicyReport) -> Vec<Uuid> {
    report
        .path
        .as_ref()
        .map(|path| path.steps.iter().map(|step| step.object_uuid).collect())
        .unwrap_or_default()
}

pub(super) fn plain_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9100);
    let class_uuid = Uuid::from_u128(0x9101);
    let from_pad_uuid = Uuid::from_u128(0x9102);
    let to_pad_uuid = Uuid::from_u128(0x9103);
    let track_uuid = Uuid::from_u128(0x9104);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-plain".into(),
            stackup: Stackup {
                layers: vec![StackupLayer::new(
                    1,
                    "Top",
                    StackupLayerType::Copper,
                    35_000,
                )],
            },
            pad_expansion_setup: crate::board::PadExpansionSetup::default(),
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(5_000_000, 0),
                Point::new(5_000_000, 5_000_000),
                Point::new(0, 5_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(500_000, 500_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                    to_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(4_500_000, 500_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                track_uuid,
                Track::straight(
                    track_uuid,
                    net_uuid,
                    Point::new(500_000, 500_000),
                    Point::new(4_500_000, 500_000),
                    120_000,
                    1,
                ),
            )]),
            vias: HashMap::new(),
            zones: HashMap::new(),
            nets: HashMap::from([(net_uuid, Net::new(net_uuid, "SIG", class_uuid))]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        track_uuid,
    )
}

pub(super) fn zone_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9200);
    let class_uuid = Uuid::from_u128(0x9201);
    let from_pad_uuid = Uuid::from_u128(0x9202);
    let to_pad_uuid = Uuid::from_u128(0x9203);
    let zone_uuid = Uuid::from_u128(0x9204);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-zone".into(),
            stackup: Stackup {
                layers: vec![StackupLayer::new(
                    1,
                    "Top",
                    StackupLayerType::Copper,
                    35_000,
                )],
            },
            pad_expansion_setup: crate::board::PadExpansionSetup::default(),
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 100_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                    to_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 100_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
            zones: HashMap::from([(
                zone_uuid,
                Zone {
                    uuid: zone_uuid,
                    net: net_uuid,
                    polygon: Polygon::new(vec![
                        Point::new(50_000, 50_000),
                        Point::new(950_000, 50_000),
                        Point::new(950_000, 150_000),
                        Point::new(50_000, 150_000),
                    ]),
                    layer: 1,
                    priority: 1,
                    thermal_relief: true,
                    thermal_gap: 150_000,
                    thermal_spoke_width: 120_000,
                },
            )]),
            nets: HashMap::from([(net_uuid, Net::new(net_uuid, "SIG", class_uuid))]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        zone_uuid,
    )
}

pub(super) fn obstacle_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9300);
    let class_uuid = Uuid::from_u128(0x9301);
    let from_pad_uuid = Uuid::from_u128(0x9302);
    let to_pad_uuid = Uuid::from_u128(0x9303);
    let track_a_uuid = Uuid::from_u128(0x9304);
    let via_uuid = Uuid::from_u128(0x9305);
    let track_b_uuid = Uuid::from_u128(0x9306);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-obstacle".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer::new(1, "Top", StackupLayerType::Copper, 35_000),
                    StackupLayer::new(2, "Core", StackupLayerType::Dielectric, 1_000_000),
                    StackupLayer::new(3, "Bottom", StackupLayerType::Copper, 35_000),
                ],
            },
            pad_expansion_setup: crate::board::PadExpansionSetup::default(),
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 100_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                    to_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 900_000),
                        layer: 3,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
            tracks: HashMap::from([
                (
                    track_a_uuid,
                    Track::straight(
                        track_a_uuid,
                        net_uuid,
                        Point::new(100_000, 100_000),
                        Point::new(500_000, 500_000),
                        150_000,
                        1,
                    ),
                ),
                (
                    track_b_uuid,
                    Track::straight(
                        track_b_uuid,
                        net_uuid,
                        Point::new(500_000, 500_000),
                        Point::new(900_000, 900_000),
                        150_000,
                        3,
                    ),
                ),
            ]),
            vias: HashMap::from([(
                via_uuid,
                Via {
                    uuid: via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    drill: 300_000,
                    diameter: 600_000,
                    from_layer: 1,
                    to_layer: 3,
                },
            )]),
            zones: HashMap::new(),
            nets: HashMap::from([(net_uuid, Net::new(net_uuid, "SIG", class_uuid))]),
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
        from_pad_uuid,
        to_pad_uuid,
        track_a_uuid,
        via_uuid,
        track_b_uuid,
    )
}

fn zone_obstacle_board() -> (Board, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9400);
    let class_uuid = Uuid::from_u128(0x9401);
    let from_pad_uuid = Uuid::from_u128(0x9402);
    let to_pad_uuid = Uuid::from_u128(0x9403);
    let zone_uuid = Uuid::from_u128(0x9404);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-zone-obstacle".into(),
            stackup: Stackup {
                layers: vec![StackupLayer::new(
                    1,
                    "Top",
                    StackupLayerType::Copper,
                    35_000,
                )],
            },
            pad_expansion_setup: crate::board::PadExpansionSetup::default(),
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(1_000_000, 0),
                Point::new(1_000_000, 1_000_000),
                Point::new(0, 1_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(100_000, 100_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                    to_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(900_000, 100_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
            zones: HashMap::from([(
                zone_uuid,
                Zone {
                    uuid: zone_uuid,
                    net: net_uuid,
                    polygon: Polygon::new(vec![
                        Point::new(50_000, 50_000),
                        Point::new(950_000, 50_000),
                        Point::new(950_000, 150_000),
                        Point::new(50_000, 150_000),
                    ]),
                    layer: 1,
                    priority: 1,
                    thermal_relief: true,
                    thermal_gap: 150_000,
                    thermal_spoke_width: 120_000,
                },
            )]),
            nets: HashMap::from([(net_uuid, Net::new(net_uuid, "SIG", class_uuid))]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        zone_uuid,
    )
}

fn topology_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9500);
    let class_uuid = Uuid::from_u128(0x9501);
    let from_pad_uuid = Uuid::from_u128(0x9502);
    let to_pad_uuid = Uuid::from_u128(0x9503);
    let via_uuid = Uuid::from_u128(0x9504);
    let track_a_uuid = Uuid::from_u128(0x9505);
    let track_b_uuid = Uuid::from_u128(0x9506);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-topology".into(),
            stackup: Stackup {
                layers: vec![
                    StackupLayer::new(1, "Top", StackupLayerType::Copper, 35_000),
                    StackupLayer::new(2, "Inner", StackupLayerType::Copper, 35_000),
                ],
            },
            pad_expansion_setup: crate::board::PadExpansionSetup::default(),
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(5_000_000, 0),
                Point::new(5_000_000, 5_000_000),
                Point::new(0, 5_000_000),
            ]),
            packages: HashMap::new(),
            pads: HashMap::from([
                (
                    from_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: from_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "1".into(),
                        net: Some(net_uuid),
                        position: Point::new(500_000, 500_000),
                        layer: 1,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
                    to_pad_uuid,
                    PlacedPad {
                        layer_connection: Default::default(),
                        uuid: to_pad_uuid,
                        package: Uuid::new_v4(),
                        name: "2".into(),
                        net: Some(net_uuid),
                        position: Point::new(3_500_000, 500_000),
                        layer: 2,
                        copper_layers: Vec::new(),
                        shape: PadShape::Circle,
                        diameter: 300_000,
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
            tracks: HashMap::from([
                (
                    track_a_uuid,
                    Track::straight(
                        track_a_uuid,
                        net_uuid,
                        Point::new(500_000, 500_000),
                        Point::new(2_000_000, 500_000),
                        120_000,
                        2,
                    ),
                ),
                (
                    track_b_uuid,
                    Track::straight(
                        track_b_uuid,
                        net_uuid,
                        Point::new(2_000_000, 500_000),
                        Point::new(3_500_000, 500_000),
                        120_000,
                        2,
                    ),
                ),
            ]),
            vias: HashMap::from([(
                via_uuid,
                Via {
                    uuid: via_uuid,
                    net: net_uuid,
                    position: Point::new(500_000, 500_000),
                    from_layer: 1,
                    to_layer: 2,
                    diameter: 300_000,
                    drill: 150_000,
                },
            )]),
            zones: HashMap::new(),
            nets: HashMap::from([(net_uuid, Net::new(net_uuid, "SIG", class_uuid))]),
            net_classes: HashMap::from([(
                class_uuid,
                NetClass {
                    uuid: class_uuid,
                    name: "Default".into(),
                    clearance: 100_000,
                    track_width: 120_000,
                    via_drill: 150_000,
                    via_diameter: 300_000,
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
        via_uuid,
        track_a_uuid,
        track_b_uuid,
    )
}

#[path = "route_policy_layer_balance_fixture.rs"]
mod route_policy_layer_balance_fixture;
use route_policy_layer_balance_fixture::layer_balance_board;
