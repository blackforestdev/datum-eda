//! Ephemeral native source projection; no query allocates or persists Pad IDs.
use super::*;
pub(super) fn project(
    model: &eda_engine::substrate::DesignModel,
    mut board: Value,
) -> Result<Value> {
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(model, &board)?;
    if !projection.pads.is_empty() {
        if board.get("pads").is_none() {
            board["pads"] = serde_json::json!({});
        }
        let map = board["pads"]
            .as_object_mut()
            .context("invalid Pad source map")?;
        for (id, pad) in projection.pads {
            anyhow::ensure!(
                !map.contains_key(&id.to_string()),
                "placed Pad source collision"
            );
            map.insert(id.to_string(), serde_json::to_value(pad)?);
        }
    }
    Ok(board)
}
pub(super) fn extract_pads(board: &Value) -> Result<Vec<BoardPadPayload>> {
    let pads_map = board
        .get("pads")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut pads = Vec::with_capacity(pads_map.len());
    for (_key, value) in pads_map {
        let pad: EnginePadPayload =
            serde_json::from_value(value).context("failed to parse board pad")?;
        pads.push(BoardPadPayload {
            uuid: pad.uuid.to_string(),
            package: pad.package.to_string(),
            name: pad.name,
            net: pad.net.map(|u| u.to_string()),
            position: PointNm {
                x: pad.position.x,
                y: pad.position.y,
            },
            layer: pad.layer,
            copper_layers: pad.copper_layers,
            shape: pad.shape.to_string(),
            diameter: pad.diameter,
            width: pad.width,
            height: pad.height,
            roundrect_rratio_ppm: pad.roundrect_rratio_ppm,
            mask_layers: pad.mask_layers,
            paste_layers: pad.paste_layers,
            solder_mask_margin_nm: pad.solder_mask_margin_nm,
            solder_paste_margin_nm: pad.solder_paste_margin_nm,
            solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
            drill: Some(pad.drill),
            rotation: pad.rotation.unwrap_or(0),
        });
    }
    pads.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

#[cfg(test)]
mod tests;
