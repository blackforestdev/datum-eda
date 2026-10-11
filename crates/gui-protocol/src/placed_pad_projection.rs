//! Ephemeral native source projection; no query allocates or persists Pad IDs.
use super::*;
pub(super) fn project(
    model: &eda_engine::substrate::DesignModel,
    mut board: Value,
) -> Result<Value> {
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(model, &board)?;
    for id in projection.query_pads.keys() {
        anyhow::ensure!(
            board
                .get("pads")
                .and_then(|m| m.get(id.to_string()))
                .is_none(),
            "placed Pad source collision"
        );
    }
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

pub(super) fn append_exact(
    scene: &mut BoardReviewSceneV1,
    projection: &eda_engine::substrate::component_pad_source::ComponentPadProjection,
) -> Result<()> {
    use eda_engine::board::{
        nominal_geometry::Rational as R, quadratic3::Quadratic3 as Q, query_pad::QueryPadGeometry,
    };
    for query in projection.query_pads.values() {
        let QueryPadGeometry::SourceRelative(exact) = query else {
            continue;
        };
        let pad = &exact.local;
        let radius = R::new(
            if pad.shape == eda_engine::board::PadShape::Circle {
                i128::from(pad.diameter)
            } else {
                i128::from(pad.width) + i128::from(pad.height)
            },
            2,
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        let bounded = |axis: usize| -> Result<(i64, i64)> {
            let min = exact.center[axis]
                .sub(Q::rational(radius))
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
                .floor_ceil_nm()
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
                .0;
            let max = exact.center[axis]
                .add(Q::rational(radius))
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
                .floor_ceil_nm()
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
                .1;
            Ok((min, max))
        };
        let x = bounded(0)?;
        let y = bounded(1)?;
        let bounds = RectNm {
            min_x: x.0,
            max_x: x.1,
            min_y: y.0,
            max_y: y.1,
        };
        scene.pads.push(PadPrimitive {
            native_process: None,
            object_id: format!("pad:{}", pad.uuid),
            object_kind: "pad".into(),
            source_object_uuid: pad.uuid.to_string(),
            pad_uuid: pad.uuid.to_string(),
            component_uuid: pad.package.to_string(),
            net_uuid: pad.net.map(|n| n.to_string()),
            layer_id: layer_id(pad.layer),
            copper_layer_ids: pad.copper_layers.iter().copied().map(layer_id).collect(),
            center: PadCenter::Exact {
                exact: exact.center,
                local_width_nm: if pad.shape == eda_engine::board::PadShape::Circle {
                    pad.diameter
                } else {
                    pad.width
                },
                local_height_nm: if pad.shape == eda_engine::board::PadShape::Circle {
                    pad.diameter
                } else {
                    pad.height
                },
                back: exact.back,
            },
            bounds,
            shape_kind: match pad.shape {
                eda_engine::board::PadShape::Circle => "circle",
                eda_engine::board::PadShape::Rect => "rect",
                _ => unreachable!("frozen source validator owns aperture support"),
            }
            .into(),
            roundrect_rratio_ppm: pad.roundrect_rratio_ppm,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: Some(pad.drill),
            rotation_degrees: 30.0,
        });
        for component in scene
            .components
            .iter_mut()
            .filter(|c| c.component_uuid == pad.package.to_string())
        {
            component.bounds.min_x = component.bounds.min_x.min(bounds.min_x);
            component.bounds.max_x = component.bounds.max_x.max(bounds.max_x);
            component.bounds.min_y = component.bounds.min_y.min(bounds.min_y);
            component.bounds.max_y = component.bounds.max_y.max(bounds.max_y);
        }
        scene.bounds.min_x = scene.bounds.min_x.min(bounds.min_x);
        scene.bounds.max_x = scene.bounds.max_x.max(bounds.max_x);
        scene.bounds.min_y = scene.bounds.min_y.min(bounds.min_y);
        scene.bounds.max_y = scene.bounds.max_y.max(bounds.max_y);
        scene.version = 2;
    }
    Ok(())
}

pub(super) fn integer_primitive(pad: BoardPadPayload) -> PadPrimitive {
    let bounds = pad_bounds(&pad);
    PadPrimitive {
        native_process: None,
        object_id: format!("pad:{}", pad.uuid),
        object_kind: "pad".to_string(),
        source_object_uuid: pad.uuid.clone(),
        pad_uuid: pad.uuid.clone(),
        component_uuid: pad.package.clone(),
        net_uuid: pad.net.clone(),
        layer_id: layer_id(pad.layer),
        copper_layer_ids: pad.copper_layers.into_iter().map(layer_id).collect(),
        center: pad.position.into(),
        bounds,
        shape_kind: pad.shape,
        roundrect_rratio_ppm: pad.roundrect_rratio_ppm,
        mask_layer_ids: pad.mask_layers.into_iter().map(layer_id).collect(),
        paste_layer_ids: pad.paste_layers.into_iter().map(layer_id).collect(),
        solder_mask_margin_nm: pad.solder_mask_margin_nm,
        solder_paste_margin_nm: pad.solder_paste_margin_nm,
        solder_paste_margin_ratio_ppm: pad.solder_paste_margin_ratio_ppm,
        drill_nm: pad.drill,
        rotation_degrees: pad.rotation as f32,
    }
}

#[cfg(test)]
mod exact30_tests;

#[cfg(test)]
mod process_tests;

pub(super) fn attach_native_process(
    scene: &mut BoardReviewSceneV1,
    processes: &std::collections::BTreeMap<
        uuid::Uuid,
        eda_engine::substrate::component_pad_source::process::PlacedProcessGeometry,
    >,
) -> Result<()> {
    for pad in &mut scene.pads {
        let Ok(id) = uuid::Uuid::parse_str(&pad.pad_uuid) else {
            continue;
        };
        if let Some(process) = processes.get(&id) {
            pad.mask_layer_ids = process
                .membership
                .mask_layers
                .iter()
                .copied()
                .map(layer_id)
                .collect();
            pad.paste_layer_ids = process
                .membership
                .paste_layers
                .iter()
                .copied()
                .map(layer_id)
                .collect();
            pad.native_process = Some(process.clone());
        }
    }
    Ok(())
}

/// Collect native authority before the scene's historical serde defaults are applied.
pub(super) fn native_processes(
    board: &Value,
    projection: &eda_engine::substrate::component_pad_source::ComponentPadProjection,
) -> Result<
    std::collections::BTreeMap<
        uuid::Uuid,
        eda_engine::substrate::component_pad_source::process::PlacedProcessGeometry,
    >,
> {
    use eda_engine::substrate::{
        component_pad_source::process::preserved_placed_process,
        process_operations::capability_from_board,
    };
    let Some(capability) = capability_from_board(board)? else {
        return Ok(projection.process_pads.clone());
    };
    anyhow::ensure!(
        projection.unavailable.is_empty() && projection.unadopted_packages.is_empty(),
        "native process scene has unresolved placed sources: {:?}; legacy placements {:?}",
        projection.unavailable,
        projection.unadopted_packages
    );
    for id in projection.query_pads.keys() {
        anyhow::ensure!(
            projection.process_pads.contains_key(id),
            "placed Pad {id} lacks frozen native process authority; explicit process materialization required"
        );
    }
    let mut processes = projection.process_pads.clone();
    for raw in board
        .get("pads")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|p| p.values())
    {
        let mut process = preserved_placed_process(raw)?;
        process.source_capability = Some(capability.clone());
        anyhow::ensure!(
            processes.insert(process.pad_id, process).is_none(),
            "native process scene has duplicate placed Pad authority"
        );
    }
    Ok(processes)
}
