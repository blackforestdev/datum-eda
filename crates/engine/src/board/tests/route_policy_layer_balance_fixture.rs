use super::*;
pub(super) fn layer_balance_board() -> (Board, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let net_uuid = Uuid::from_u128(0x9600);
    let class_uuid = Uuid::from_u128(0x9601);
    let from_pad_uuid = Uuid::from_u128(0x9602);
    let to_pad_uuid = Uuid::from_u128(0x9603);
    let via_uuid = Uuid::from_u128(0x9604);
    let track_uuid = Uuid::from_u128(0x9605);
    (
        Board {
            uuid: Uuid::new_v4(),
            name: "policy-layer-balance".into(),
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
                        position: Point::new(2_000_000, 500_000),
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
                    Uuid::from_u128(0x9606),
                    Track::straight(
                        Uuid::from_u128(0x9606),
                        net_uuid,
                        Point::new(500_000, 500_000),
                        Point::new(2_000_000, 500_000),
                        120_000,
                        1,
                    ),
                ),
                (
                    track_uuid,
                    Track::straight(
                        track_uuid,
                        net_uuid,
                        Point::new(500_000, 500_000),
                        Point::new(2_000_000, 500_000),
                        120_000,
                        2,
                    ),
                ),
            ]),
            vias: HashMap::from([
                (
                    Uuid::from_u128(0x9607),
                    Via {
                        uuid: Uuid::from_u128(0x9607),
                        net: net_uuid,
                        position: Point::new(2_000_000, 500_000),
                        from_layer: 1,
                        to_layer: 2,
                        diameter: 300_000,
                        drill: 150_000,
                    },
                ),
                (
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
                ),
            ]),
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
        track_uuid,
    )
}
