//! Native process capability consumes frozen engine geometry, never numeric side guesses.
use crate::*;
use eda_engine::substrate::{
    ProjectResolver, SourceShardKind, process_operations::capability_from_board,
};

pub(super) fn resolve(
    project: &LoadedNativeProject,
    layer: i32,
    paste: bool,
) -> Result<Option<(StackupLayer, i32, Vec<PlacedPad>)>> {
    let model = ProjectResolver::new(&project.root).resolve()?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let Some(capability) = capability_from_board(&board)? else {
        return Ok(None);
    };
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    capability.validate(&stackup)?;
    let kind = if paste {
        StackupLayerType::Paste
    } else {
        StackupLayerType::SolderMask
    };
    let entry = stackup
        .layers
        .iter()
        .find(|l| l.id == layer && l.layer_type == kind)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("missing or incorrectly typed process layer {layer}"))?;
    let source_copper = [&capability.front, &capability.back]
        .into_iter()
        .find(|face| {
            if paste {
                face.paste_layer == Some(layer)
            } else {
                face.mask_layer == Some(layer)
            }
        })
        .map(|face| face.copper_layer)
        .ok_or_else(|| {
            anyhow::anyhow!("process layer {layer} has no explicit physical face association")
        })?;
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(&model, &board)?;
    if !projection.unavailable.is_empty() || !projection.unadopted_packages.is_empty() {
        bail!(
            "native process output has unresolved placed sources: {:?}; legacy placements {:?}",
            projection.unavailable,
            projection.unadopted_packages
        );
    }
    let mut pads = Vec::new();
    for (id, physical) in &projection.query_pads {
        let process = projection.process_pads.get(id).ok_or_else(|| {
            anyhow::anyhow!("placed Pad {id} has legacy geometry but no frozen process authority")
        })?;
        if let Some(pad) =
            eda_engine::export::process_pad_for_layer(process, physical.terminal(), layer, paste)?
        {
            pads.push(pad);
        }
    }
    if let Some(authored) = board.get("pads").and_then(serde_json::Value::as_object) {
        for raw in authored.values() {
            let process =
                eda_engine::substrate::component_pad_source::process::preserved_placed_process(
                    raw,
                )?;
            let copper: PlacedPad = serde_json::from_value(raw.clone())?;
            if let Some(pad) =
                eda_engine::export::process_pad_for_layer(&process, &copper, layer, paste)?
            {
                pads.push(pad);
            }
        }
    }
    pads.sort_by_key(|p| p.uuid);
    Ok(Some((entry, source_copper, pads)))
}
