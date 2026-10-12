//! Board visibility and ordered hit-shape emission over borrowed source data.
use super::hit_construction::{Region, Shape};
use super::*;
#[path = "board_hit_construction/text_layouts.rs"]
mod text_layouts;

pub(super) fn build(
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    mut admit: impl FnMut(usize) -> anyhow::Result<()>,
) -> anyhow::Result<Vec<WorldHitRegion>> {
    let needed = authored_visible(state)
        && scene
            .board_texts
            .iter()
            .any(|text| layer_visible(state, &text.layer_id));
    let layouts = text_layouts::Lookup::new(
        if needed {
            &scene.board_text_geometries
        } else {
            &[]
        },
        &mut admit,
    )?;
    hit_construction::build(|emit| visit(emit, scene, state, &layouts), admit)
}

fn visit<'a>(
    emit: &mut dyn FnMut(Region<'a>) -> anyhow::Result<()>,
    scene: &'a BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    layouts: &text_layouts::Lookup<'a>,
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
        if !layer_visible(state, &zone.layer_id) {
            continue;
        }
        let legacy = (zone.fill_projection.is_legacy() && zone.polygon.len() >= 3)
            .then_some(zone.polygon.as_slice());
        let current = (!zone.fill_projection.is_legacy()).then_some(zone);
        for polygon in legacy
            .into_iter()
            .chain(current.into_iter().flat_map(|z| z.occupied_polygons()))
        {
            emit(Region {
                target: &zone.object_id,
                layer_id: Some(&zone.layer_id),
                shape: Shape::Polygon(polygon),
            })?;
        }
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
            shape: layouts
                .get(&text.object_id)
                .filter(|g| {
                    datum_gui_protocol::selection_geometry_catalog::layout_polygon(g).is_some()
                })
                .map(Shape::Layout)
                .unwrap_or_else(|| Shape::Rect(board_text_hit_rect(text))),
        })?;
    }
    for gfx in &scene.board_graphics {
        if gfx.object_id.starts_with("board-text:") {
            continue;
        }
        if !layer_visible(state, &gfx.layer_id) {
            continue;
        }
        if gfx.primitive_kind == "polygon" {
            emit(Region {
                target: &gfx.object_id,
                layer_id: Some(&gfx.layer_id),
                shape: Shape::Contours {
                    outer: &gfx.path,
                    holes: &gfx.holes,
                },
            })?;
        }
        let width = gfx.width_nm.unwrap_or(100_000);
        if gfx.primitive_kind == "polygon" && gfx.width_nm.is_none() {
            continue;
        }
        for hole in &gfx.holes {
            emit(Region {
                target: &gfx.object_id,
                layer_id: Some(&gfx.layer_id),
                shape: Shape::Polyline {
                    path: hole,
                    half_width_nm: (width as f32 * 0.5).max(150_000.0),
                },
            })?;
        }
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
    fn filled_board_graphic_hits_material_and_preserves_voids_with_exact_preflight() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        state.scene.tracks.clear();
        state.scene.vias.clear();
        state.scene.components.clear();
        state.scene.pads.clear();
        state.scene.zones.clear();
        state.scene.component_graphics.clear();
        state.scene.board_texts.clear();
        state.scene.outline.clear();
        let p = |x, y| PointNm { x, y };
        state.scene.board_graphics = vec![BoardGraphicPrimitive {
            object_id: "board-graphic:recorded".into(),
            object_kind: "board_graphic".into(),
            primitive_kind: "polygon".into(),
            source_object_uuid: "recorded".into(),
            layer_id: "L37".into(),
            path: vec![
                p(0, 0),
                p(10_000_000, 0),
                p(10_000_000, 10_000_000),
                p(8_000_000, 10_000_000),
                p(8_000_000, 8_000_000),
                p(0, 8_000_000),
                p(0, 0),
            ],
            holes: vec![vec![
                p(3_000_000, 3_000_000),
                p(7_000_000, 3_000_000),
                p(7_000_000, 7_000_000),
                p(3_000_000, 7_000_000),
                p(3_000_000, 3_000_000),
            ]],
            width_nm: None,
        }];
        state.ui.filters.layer_visibility.insert("L37".into(), true);
        let scope = crate::cpu_alloc::Scope::new("filled-graphic-hit-proof");
        let mut admitted = 0;
        let regions = scope
            .with(|| {
                build(&state.scene, &state, |bytes| {
                    admitted = bytes;
                    Ok(())
                })
            })
            .unwrap();
        let usage = scope.usage();
        assert_eq!(admitted as u64, usage.payload_bytes + usage.tracking_bytes);
        assert_eq!(regions.len(), 1);
        let index = datum_gui_viewport::SpatialHitIndex::new(regions);
        assert_eq!(
            index
                .hit_test(p(1_000_000, 5_000_000), |_| true)
                .target
                .map(String::as_str),
            Some("board-graphic:recorded")
        );
        assert!(
            index
                .hit_test(p(5_000_000, 5_000_000), |_| true)
                .target
                .is_none(),
            "hole is not material"
        );
        assert!(
            index
                .hit_test(p(11_000_000, 5_000_000), |_| true)
                .target
                .is_none()
        );
        assert!(
            index
                .hit_test(p(5_000_000, 9_000_000), |_| true)
                .target
                .is_none(),
            "bounding-box-only point is not material"
        );
        state
            .ui
            .filters
            .layer_visibility
            .insert("L37".into(), false);
        assert!(build(&state.scene, &state, |_| Ok(())).unwrap().is_empty());
    }

    #[test]
    fn real_board_hit_preflight_matches_owned_bytes_and_hidden_scene_allocates_nothing() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        let scope = crate::cpu_alloc::Scope::new("board-hit-construction-proof");
        assert!(!state.scene.board_text_geometries.is_empty());
        let index_payload = state.scene.board_text_geometries.len() * std::mem::size_of::<usize>();
        let index_bytes = crate::cpu_alloc::heap::allocation_bytes(
            std::alloc::Layout::array::<usize>(state.scene.board_text_geometries.len()).unwrap(),
        );
        let mut required = 0;
        let mut calls = 0;
        let regions = scope
            .with(|| {
                build(&state.scene, &state, |bytes| {
                    calls += 1;
                    let usage = scope.usage();
                    if calls == 1 {
                        assert_eq!(bytes, index_bytes);
                        assert_eq!(usage.peak_payload_bytes, 0);
                    } else {
                        assert_eq!(usage.peak_payload_bytes, index_payload as u64);
                        assert_eq!(
                            usage.payload_bytes + usage.tracking_bytes,
                            index_bytes as u64
                        );
                        required = bytes;
                    }
                    Ok(())
                })
            })
            .unwrap();
        assert_eq!(calls, 2);
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
                .find(|region| region.target == track.object_id)
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
        for refused_stage in [1, 2] {
            let mut calls = 0;
            let refusal = build(&state.scene, &state, |bytes| {
                calls += 1;
                assert_eq!(bytes, if calls == 1 { index_bytes } else { required });
                anyhow::ensure!(calls != refused_stage, "test admission refusal");
                Ok(())
            });
            assert!(refusal.is_err());
            assert_eq!(calls, refused_stage);
        }
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
