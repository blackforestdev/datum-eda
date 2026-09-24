use super::*;

pub(super) struct NativeCopperLayerContext {
    pub(super) pads: Vec<PlacedPad>,
    pub(super) tracks: Vec<Track>,
    pub(super) zones: Vec<Zone>,
    pub(super) unfilled_zone_count: usize,
    pub(super) unfilled_zone_ids: Vec<String>,
    pub(super) vias: Vec<Via>,
}
pub(super) fn resolve_native_project_copper_layer_context(
    project: &LoadedNativeProject,
    model: Option<&DesignModel>,
    layer: i32,
) -> Result<NativeCopperLayerContext> {
    let mut pads = project
        .board
        .pads
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board pad"))
        .collect::<Result<Vec<PlacedPad>>>()?;
    for (component_key, component_pads) in &project.board.component_pads {
        let component_uuid = Uuid::parse_str(component_key).with_context(|| {
            format!(
                "failed to parse component UUID in {}",
                project.board_path.display()
            )
        })?;
        pads.extend(component_pads.iter().filter_map(|pad| {
            pad.shape.map(|shape| PlacedPad {
                uuid: pad.uuid,
                package: component_uuid,
                name: pad.name.clone(),
                net: None,
                position: Point {
                    x: pad.position.x,
                    y: pad.position.y,
                },
                layer: pad.layer,
                copper_layers: vec![pad.layer],
                shape,
                diameter: pad.diameter_nm,
                width: pad.width_nm,
                height: pad.height_nm,
                drill: pad.drill_nm.unwrap_or(0),
                rotation: 0,
                mask_layers: Vec::new(),
                paste_layers: Vec::new(),
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
                roundrect_rratio_ppm: 250_000,
            })
        }));
    }
    pads.retain(|pad| pad.layer == layer);
    pads.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.package.cmp(&b.package))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    let mut tracks = project
        .board
        .tracks
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board track"))
        .collect::<Result<Vec<Track>>>()?;
    tracks.retain(|track| track.layer == layer);
    tracks.sort_by_key(|a| a.uuid);
    let mut authored_zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    authored_zones.retain(|zone| zone.layer == layer);
    authored_zones.sort_by_key(|a| a.uuid);
    let (zones, unfilled_zone_ids) = match model {
        Some(model) => zone_fill_copper_projection_zones(&authored_zones, &model.zone_fills),
        None => (
            Vec::new(),
            authored_zones
                .iter()
                .map(|zone| zone.uuid.to_string())
                .collect::<Vec<_>>(),
        ),
    };
    let unfilled_zone_count = unfilled_zone_ids.len();
    let mut vias = project
        .board
        .vias
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board via"))
        .collect::<Result<Vec<Via>>>()?;
    vias.retain(|via| {
        layer >= via.from_layer.min(via.to_layer) && layer <= via.from_layer.max(via.to_layer)
    });
    vias.sort_by_key(|a| a.uuid);
    Ok(NativeCopperLayerContext {
        pads,
        tracks,
        zones,
        unfilled_zone_count,
        unfilled_zone_ids,
        vias,
    })
}
