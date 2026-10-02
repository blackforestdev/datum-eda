use super::*;
/// One circular anchor pad (parses back to the same `PlacedPad` facts the
/// historical hand-written fixture pads produced).
pub(super) fn route_strategy_fixture_pad(
    uuid: Uuid,
    package: Uuid,
    net: Uuid,
    position: Point,
    layer: i32,
    diameter: i64,
) -> PlacedPad {
    PlacedPad {
        layer_connection: Default::default(),
        uuid,
        package,
        name: "1".to_string(),
        net: Some(net),
        position,
        layer,
        copper_layers: Vec::new(),
        shape: PadShape::Circle,
        diameter,
        width: 0,
        height: 0,
        drill: 0,
        rotation: 0,
        roundrect_rratio_ppm: 250_000,
        mask_layers: Vec::new(),
        paste_layers: Vec::new(),
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
    }
}
