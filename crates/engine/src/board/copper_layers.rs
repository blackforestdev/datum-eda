//! Conductive layers and explicit pad barrel/span authority.
use super::*;
pub(super) fn layer_index(stackup: &Stackup, id: LayerId) -> Result<usize> {
    let indices: Vec<_> = stackup
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.id == id && l.layer_type == StackupLayerType::Copper)
        .map(|(i, _)| i)
        .collect();
    if indices.len() != 1 {
        return Err(GeometryError::UnknownConductiveLayer);
    }
    Ok(indices[0])
}
pub fn track_layers(stackup: &Stackup, track: &Track) -> Result<Vec<LayerId>> {
    layer_index(stackup, track.layer)?;
    Ok(vec![track.layer])
}
pub fn pad_layers(stackup: &Stackup, pad: &PlacedPad) -> Result<Vec<LayerId>> {
    let mut layers = if pad.copper_layers.is_empty() {
        vec![pad.layer]
    } else {
        pad.copper_layers.clone()
    };
    for id in &layers {
        layer_index(stackup, *id)?;
    }
    layers.sort();
    layers.dedup();
    Ok(layers)
}
pub fn via_layers(stackup: &Stackup, via: &Via) -> Result<Vec<LayerId>> {
    let a = layer_index(stackup, via.from_layer)?;
    let b = layer_index(stackup, via.to_layer)?;
    Ok(stackup.layers[a.min(b)..=a.max(b)]
        .iter()
        .filter(|l| l.layer_type == StackupLayerType::Copper)
        .map(|l| l.id)
        .collect())
}

/// Actual aperture groups joined only by explicit placed-pad process/span facts.
pub fn pad_layer_groups(stackup: &Stackup, pad: &PlacedPad) -> Result<Vec<Vec<LayerId>>> {
    use super::super::PadLayerConnection;
    let layers = pad_layers(stackup, pad)?;
    let span: Vec<_> = match pad.layer_connection {
        PadLayerConnection::Unknown if layers.len() == 1 => return Ok(vec![layers]),
        PadLayerConnection::Unknown => return Err(GeometryError::UnknownPadLayerConnection),
        PadLayerConnection::Separate => return Ok(layers.into_iter().map(|id| vec![id]).collect()),
        PadLayerConnection::PlatedThrough => stackup
            .layers
            .iter()
            .filter(|l| l.layer_type == StackupLayerType::Copper)
            .map(|l| l.id)
            .collect(),
        PadLayerConnection::PlatedSpan {
            start_layer,
            end_layer,
        } => {
            let a = layer_index(stackup, start_layer)?;
            let b = layer_index(stackup, end_layer)?;
            if a == b {
                return Err(GeometryError::UnknownPadLayerConnection);
            }
            stackup.layers[a.min(b)..=a.max(b)]
                .iter()
                .filter(|l| l.layer_type == StackupLayerType::Copper)
                .map(|l| l.id)
                .collect()
        }
    };
    if pad.drill <= 0
        || span.is_empty()
        || span.len() != layers.len()
        || !span.iter().all(|id| layers.contains(id))
    {
        return Err(GeometryError::UnknownPadLayerConnection);
    }
    Ok(vec![layers])
}
