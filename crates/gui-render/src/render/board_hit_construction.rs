//! Board visibility and ordered hit-shape emission over borrowed source data.
use super::hit_construction::{Region, Shape};
use super::*;

pub(super) fn build(
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    admit: impl FnOnce(usize) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<WorldHitRegion>> {
    hit_construction::build(|emit| visit(emit, scene, state), admit)
}

fn visit<'a>(
    emit: &mut dyn FnMut(Region<'a>) -> anyhow::Result<()>,
    scene: &'a BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> anyhow::Result<()> {
    if !authored_visible(state) {
        return Ok(());
    }
    for track in &scene.tracks {
        if !layer_visible(state, &track.layer_id) {
            continue;
        }
        emit(Region {
            target: &track.object_id,
            layer_id: Some(&track.layer_id),
            shape: Shape::Polyline {
                path: &track.path,
                half_width_nm: (track.width_nm as f32 * 0.5).max(150_000.0),
            },
        })?;
    }
    for via in &scene.vias {
        if !via_visible(state, &via.start_layer_id, &via.end_layer_id) {
            continue;
        }
        emit(Region {
            target: &via.object_id,
            layer_id: None,
            shape: Shape::Circle {
                center: via.position,
                radius_nm: (via.diameter_nm as f32 * 0.5).max(250_000.0),
            },
        })?;
    }
    for component in &scene.components {
        if !layer_visible(state, &component.placement_layer) {
            continue;
        }
        let component_pads = scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid);
        let has_non_edge_graphics = scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component.component_uuid
                && !graphic.layer_id.as_deref().is_some_and(|layer_id| {
                    scene
                        .layers
                        .iter()
                        .find(|layer| layer.layer_id == layer_id)
                        .is_some_and(|layer| layer.name == "Edge.Cuts")
                })
        });
        let has_text = scene
            .component_texts
            .iter()
            .any(|text| text.component_uuid == component.component_uuid);
        let body = component_body::inferred_component_body_bounds_iter(component_pads);
        if let Some(hit_rect) = body.filter(|body| {
            let width = body.max_x - body.min_x;
            let height = body.max_y - body.min_y;
            width > 0 && height > 0 && width <= 4_500_000 && height <= 4_500_000
        }) && !has_non_edge_graphics
            && !has_text
        {
            emit(Region {
                target: &component.object_id,
                layer_id: Some(&component.placement_layer),
                shape: Shape::Rect(hit_rect),
            })?;
            continue;
        }
        if has_non_edge_graphics || has_text {
            continue;
        }
        let hit_rect = body.unwrap_or(component.bounds);
        emit(Region {
            target: &component.object_id,
            layer_id: Some(&component.placement_layer),
            shape: Shape::Rect(hit_rect),
        })?;
    }
    for pad in &scene.pads {
        let pad_visible = pad_visible_on_any_copper_layer(state, pad);
        if !pad_visible {
            continue;
        }
        emit(Region {
            target: &pad.object_id,
            layer_id: None,
            shape: Shape::Rect(pad.bounds),
        })?;
    }
    for zone in &scene.zones {
        if !layer_visible(state, &zone.layer_id) || zone.polygon.len() < 3 {
            continue;
        }
        emit(Region {
            target: &zone.object_id,
            layer_id: Some(&zone.layer_id),
            shape: Shape::Polygon(&zone.polygon),
        })?;
    }
    for graphic in &scene.component_graphics {
        let Some(target_id) = component_object_id_for_uuid(scene, &graphic.component_uuid) else {
            continue;
        };
        if let Some(layer_id) = graphic.layer_id.as_deref()
            && !layer_visible(state, layer_id)
        {
            continue;
        }
        if graphic.layer_id.as_deref().is_some_and(|layer_id| {
            scene
                .layers
                .iter()
                .find(|layer| layer.layer_id == layer_id)
                .is_some_and(|layer| layer.name == "Edge.Cuts")
        }) {
            continue;
        }
        let width = graphic.width_nm.unwrap_or(100_000);
        match graphic.primitive_kind.as_str() {
            "polygon" => {
                let (min_x, min_y, max_x, max_y) = graphic.path.iter().fold(
                    (i64::MAX, i64::MAX, i64::MIN, i64::MIN),
                    |(min_x, min_y, max_x, max_y), point| {
                        (
                            min_x.min(point.x),
                            min_y.min(point.y),
                            max_x.max(point.x),
                            max_y.max(point.y),
                        )
                    },
                );
                if min_x <= max_x && min_y <= max_y {
                    emit(Region {
                        target: target_id,
                        layer_id: graphic.layer_id.as_deref(),
                        shape: Shape::Rect(datum_gui_protocol::RectNm {
                            min_x,
                            min_y,
                            max_x,
                            max_y,
                        }),
                    })?;
                }
            }
            _ => {
                emit(Region {
                    target: target_id,
                    layer_id: graphic.layer_id.as_deref(),
                    shape: Shape::Polyline {
                        path: &graphic.path,
                        half_width_nm: (width as f32 * 0.5).max(180_000.0),
                    },
                })?;
            }
        }
    }
    for text in &scene.board_texts {
        if !layer_visible(state, &text.layer_id) {
            continue;
        }
        emit(Region {
            target: &text.object_id,
            layer_id: Some(&text.layer_id),
            shape: Shape::Rect(board_text_hit_rect(text)),
        })?;
    }
    for gfx in &scene.board_graphics {
        if gfx.object_id.starts_with("board-text:") {
            continue;
        }
        if !layer_visible(state, &gfx.layer_id) {
            continue;
        }
        let width = gfx.width_nm.unwrap_or(100_000);
        emit(Region {
            target: &gfx.object_id,
            layer_id: Some(&gfx.layer_id),
            shape: Shape::Polyline {
                path: &gfx.path,
                half_width_nm: (width as f32 * 0.5).max(150_000.0),
            },
        })?;
    }
    for outline in &scene.outline {
        if !layer_visible(state, &outline.layer_id) {
            continue;
        }
        emit(Region {
            target: &outline.object_id,
            layer_id: Some(&outline.layer_id),
            shape: Shape::Polyline {
                path: &outline.path,
                half_width_nm: 300_000.0,
            },
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_board_hit_preflight_matches_owned_bytes_and_hidden_scene_allocates_nothing() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        let scope = crate::cpu_alloc::Scope::new("board-hit-construction-proof");
        let mut required = 0;
        let regions = scope
            .with(|| {
                build(&state.scene, &state, |bytes| {
                    required = bytes;
                    assert_eq!(
                        scope.usage().peak_payload_bytes,
                        0,
                        "preflight is allocation-free"
                    );
                    Ok(())
                })
            })
            .unwrap();
        assert!(!regions.is_empty());
        assert_eq!(regions.len(), regions.capacity());
        let usage = scope.usage();
        assert_eq!(required as u64, usage.payload_bytes + usage.tracking_bytes);
        for track in &state.scene.tracks {
            if !layer_visible(&state, &track.layer_id) {
                continue;
            }
            let region = regions
                .iter()
                .find(|region| region.target == HitTarget::AuthoredObject(track.object_id.clone()))
                .expect("visible track hit");
            assert_eq!(region.layer_id.as_deref(), Some(track.layer_id.as_str()));
            assert_eq!(
                region.shape,
                WorldHitShape::Polyline {
                    path: track.path.clone(),
                    half_width_nm: (track.width_nm as f32 * 0.5).max(150_000.0),
                }
            );
        }
        drop(regions);
        assert_eq!(scope.usage().allocations, 0);
        let refusal = build(&state.scene, &state, |bytes| {
            assert_eq!(bytes, required);
            anyhow::bail!("test admission refusal")
        });
        assert!(refusal.is_err());
        state.ui.filters.show_authored = false;
        let hidden = crate::cpu_alloc::Scope::new("hidden-board-hit-proof");
        let regions = hidden
            .with(|| {
                build(&state.scene, &state, |bytes| {
                    assert_eq!(bytes, 0);
                    Ok(())
                })
            })
            .unwrap();
        assert!(regions.is_empty());
        assert_eq!(hidden.usage().peak_payload_bytes, 0);
    }
}
