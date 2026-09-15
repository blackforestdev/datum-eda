use anyhow::{Context, Result};
use eda_engine::board::{Keepout, Net, NetClass, PadShape, PlacedPad, Track, Via};
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::ZoneFillCopperContext;
use uuid::Uuid;

pub(crate) fn zone_fill_copper_context(
    board: &crate::NativeBoardRoot,
) -> Result<ZoneFillCopperContext> {
    let nets = board
        .nets
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board net"))
        .collect::<Result<Vec<Net>>>()?;
    let net_classes = board
        .net_classes
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board net class"))
        .collect::<Result<Vec<NetClass>>>()?;
    let net_class_clearance = net_classes
        .into_iter()
        .map(|class| (class.uuid, class.clearance))
        .collect::<std::collections::BTreeMap<_, _>>();
    let net_clearance_nm = nets
        .into_iter()
        .filter_map(|net| {
            net_class_clearance
                .get(&net.class)
                .copied()
                .map(|clearance| (net.uuid, clearance))
        })
        .collect();
    let mut pads = board
        .pads
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board pad"))
        .collect::<Result<Vec<PlacedPad>>>()?;
    for (component_key, component_pads) in &board.component_pads {
        let package = Uuid::parse_str(component_key).unwrap_or_else(|_| Uuid::nil());
        for pad in component_pads {
            pads.push(PlacedPad {
                uuid: pad.uuid,
                package,
                name: pad.name.clone(),
                net: None,
                position: Point::new(pad.position.x, pad.position.y),
                layer: pad.layer,
                copper_layers: vec![pad.layer],
                shape: pad.shape.unwrap_or(PadShape::Circle),
                diameter: pad.diameter_nm,
                width: pad.width_nm,
                height: pad.height_nm,
                drill: pad.drill_nm.unwrap_or(0),
                rotation: 0,
                roundrect_rratio_ppm: 250_000,
                mask_layers: Vec::new(),
                paste_layers: Vec::new(),
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
            });
        }
    }
    Ok(ZoneFillCopperContext {
        pads,
        tracks: board
            .tracks
            .values()
            .cloned()
            .map(|value| serde_json::from_value(value).context("failed to parse board track"))
            .collect::<Result<Vec<Track>>>()?,
        vias: board
            .vias
            .values()
            .cloned()
            .map(|value| serde_json::from_value(value).context("failed to parse board via"))
            .collect::<Result<Vec<Via>>>()?,
        keepouts: board
            .keepouts
            .iter()
            .cloned()
            .map(|value| serde_json::from_value(value).context("failed to parse board keepout"))
            .collect::<Result<Vec<Keepout>>>()?,
        net_clearance_nm,
    })
}
