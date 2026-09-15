use super::*;

pub(super) type ComponentSideSnapshot = (
    crate::board::PlacedPackage,
    crate::board::PlacedPackage,
    Vec<crate::board::PlacedPad>,
    Vec<crate::board::PlacedPad>,
);

pub(super) fn apply_component_side_transform(
    board: &mut crate::board::Board,
    component_uuid: uuid::Uuid,
    target_layer: crate::ir::geometry::LayerId,
) -> Result<ComponentSideSnapshot, EngineError> {
    let before = board
        .packages
        .get(&component_uuid)
        .cloned()
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: component_uuid,
        })?;
    let before_pads = component_pads(board, component_uuid);
    board
        .packages
        .get_mut(&component_uuid)
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: component_uuid,
        })?
        .layer = target_layer;

    if before.layer != target_layer {
        mirror_owned_pads_for_side(
            board,
            component_uuid,
            before.position.x,
            before.layer,
            target_layer,
        );
    }

    let after = board
        .packages
        .get(&component_uuid)
        .cloned()
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: component_uuid,
        })?;
    Ok((
        before,
        after,
        before_pads,
        component_pads(board, component_uuid),
    ))
}

fn mirror_owned_pads_for_side(
    board: &mut crate::board::Board,
    component_uuid: uuid::Uuid,
    origin_x: i64,
    previous_layer: crate::ir::geometry::LayerId,
    target_layer: crate::ir::geometry::LayerId,
) {
    for pad in board
        .pads
        .values_mut()
        .filter(|pad| pad.package == component_uuid)
    {
        pad.position.x = origin_x * 2 - pad.position.x;
        pad.layer = target_layer;
        swap_layer_membership(&mut pad.copper_layers, previous_layer, target_layer);
        swap_layer_membership(&mut pad.mask_layers, previous_layer, target_layer);
        swap_layer_membership(&mut pad.paste_layers, previous_layer, target_layer);
        pad.rotation = (180 - pad.rotation).rem_euclid(360);
    }
}

fn swap_layer_membership(
    layers: &mut [crate::ir::geometry::LayerId],
    previous_layer: crate::ir::geometry::LayerId,
    target_layer: crate::ir::geometry::LayerId,
) {
    for layer in layers {
        if *layer == previous_layer {
            *layer = target_layer;
        } else if *layer == target_layer {
            *layer = previous_layer;
        }
    }
}
