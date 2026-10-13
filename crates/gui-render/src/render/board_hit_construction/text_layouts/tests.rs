use super::*;
use datum_gui_protocol::{Affine2DFixedPrimitive, BoardTextPrimitive, MeshRectEmPrimitive};

fn geometry(id: &str, x: i64) -> BoardTextGeometryPrimitive {
    BoardTextGeometryPrimitive {
        object_id: id.into(),
        object_kind: "board_text".into(),
        text_uuid: id.into(),
        layer_id: "L1".into(),
        world_transform_nm: Some(Affine2DFixedPrimitive {
            m11_ppm: 640000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 640000,
            tx_nm: x,
            ty_nm: 0,
        }),
        block_bbox_em_nm: Some(MeshRectEmPrimitive {
            min_x_em_nm: 53000,
            min_y_em_nm: -698000,
            max_x_em_nm: 309000,
            max_y_em_nm: 0,
        }),
        glyphs: vec![],
        fills: vec![],
        strokes: vec![],
    }
}
fn text(id: &str) -> BoardTextPrimitive {
    BoardTextPrimitive {
        object_id: id.into(),
        object_kind: "board_text".into(),
        text_uuid: id.into(),
        source_object_uuid: None,
        text: "I".into(),
        layer_id: "L1".into(),
        position: datum_gui_protocol::PointNm { x: 0, y: 0 },
        rotation_degrees: 0,
        height_nm: 640000,
        stroke_width_nm: 100000,
        render_intent: "annotation".into(),
        family: "ibm_plex_sans_condensed".into(),
        style: "regular".into(),
        style_class: None,
        h_align: "left".into(),
        v_align: "bottom".into(),
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1000000,
        bold: false,
        italic: false,
    }
}

#[test]
fn first_raw_match_parity_including_invalid_duplicates_and_full_population() {
    let mut layouts = vec![
        geometry("z", 1),
        geometry("A", 2),
        geometry("a", 3),
        geometry("z", 4),
    ];
    layouts[0].world_transform_nm = None;
    let lookup = Lookup::new(&layouts, &mut |_| Ok(())).unwrap();
    for key in ["z", "A", "a", "missing", ""] {
        let old = layouts.iter().find(|g| g.object_id == key);
        assert_eq!(lookup.get(key), old);
    }
    assert!(
        datum_gui_protocol::selection_geometry_catalog::layout_polygon(lookup.get("z").unwrap())
            .is_none()
    );
    assert!(datum_gui_protocol::selection_geometry_catalog::layout_polygon(&layouts[3]).is_some());
    let population: Vec<_> = (0..100000)
        .rev()
        .map(|i| geometry(&format!("text-{i:06}"), i))
        .collect();
    let lookup = Lookup::new(&population, &mut |_| Ok(())).unwrap();
    for i in 0..100000 {
        let g = lookup.get(&format!("text-{i:06}")).unwrap();
        assert_eq!(g.world_transform_nm.unwrap().tx_nm, i);
    }
}

#[test]
fn actual_hits_preserve_source_order_first_invalid_fallback_and_visibility() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.ui.filters.layer_visibility.insert("L1".into(), true);
    state.scene.board_texts = vec![text("z"), text("A"), text("missing"), text("a")];
    state.scene.board_text_geometries = vec![
        geometry("a", 3),
        geometry("z", 1),
        geometry("A", 2),
        geometry("z", 4),
    ];
    state.scene.board_text_geometries[1].world_transform_nm = None;
    let hits = super::super::build(&state.scene, &state, |_| Ok(())).unwrap();
    let texts: Vec<_> = hits
        .iter()
        .filter(|h| {
            state
                .scene
                .board_texts
                .iter()
                .any(|t| t.object_id == h.target)
        })
        .collect();
    assert_eq!(
        texts.iter().map(|h| h.target.as_str()).collect::<Vec<_>>(),
        ["z", "A", "missing", "a"]
    );
    for (hit, text) in texts.iter().zip(&state.scene.board_texts) {
        let old = state
            .scene
            .board_text_geometries
            .iter()
            .find(|g| g.object_id == text.object_id)
            .filter(|g| {
                datum_gui_protocol::selection_geometry_catalog::layout_polygon(g).is_some()
            });
        let expected = old
            .map(|g| crate::WorldHitShape::LayoutBasis {
                transform: g.world_transform_nm.unwrap(),
                bbox: g.block_bbox_em_nm.unwrap(),
            })
            .unwrap_or_else(|| crate::WorldHitShape::Rect(super::super::board_text_hit_rect(text)));
        assert_eq!(hit.shape, expected);
    }
    // Isolate the text visibility control from the populated parity scene:
    // unrelated copper/pad hits remain lawful when only text's layer is hidden.
    state.scene.tracks.clear();
    state.scene.vias.clear();
    state.scene.components.clear();
    state.scene.pads.clear();
    state.scene.zones.clear();
    state.scene.component_graphics.clear();
    state.scene.board_graphics.clear();
    state.scene.outline.clear();
    for authored in [true, false] {
        state.ui.filters.show_authored = authored;
        state.ui.filters.layer_visibility.insert("L1".into(), false);
        let scope = crate::cpu_alloc::Scope::new("hidden-text-index");
        let hidden = scope
            .with(|| {
                super::super::build(&state.scene, &state, |bytes| {
                    assert_eq!(scope.usage().peak_payload_bytes, 0);
                    assert_eq!(bytes, 0);
                    Ok(())
                })
            })
            .unwrap();
        assert!(hidden.is_empty());
        assert_eq!(scope.usage().peak_payload_bytes, 0);
    }
}

#[test]
fn index_and_hit_output_admission_refuse_before_publication_and_release_overlap() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.ui.filters.layer_visibility.insert("L1".into(), true);
    state.scene.board_texts = vec![text("indexed")];
    state.scene.board_text_geometries = vec![geometry("indexed", 7)];
    let index_bytes = crate::cpu_alloc::heap::allocation_bytes(Layout::array::<usize>(1).unwrap());
    for refuse in [1, 2] {
        let scope = crate::cpu_alloc::Scope::new("refused-text-hit-construction");
        let mut calls = 0;
        scope.with(|| {
            let result = super::super::build(&state.scene, &state, |bytes| {
                calls += 1;
                let live = scope.usage();
                if calls == 1 {
                    assert_eq!(bytes, index_bytes);
                    assert_eq!(live.peak_payload_bytes, 0);
                } else {
                    assert_eq!(live.payload_bytes + live.tracking_bytes, index_bytes as u64);
                    assert!(bytes > index_bytes);
                }
                anyhow::ensure!(calls != refuse, "controlled constructor refusal");
                Ok(())
            });
            assert!(result.is_err());
        });
        assert_eq!(calls, refuse);
        assert_eq!(scope.usage().allocations, 0);
    }
    let scope = crate::cpu_alloc::Scope::new("admitted-text-hit-construction");
    let mut calls = 0;
    let hits = scope
        .with(|| {
            super::super::build(&state.scene, &state, |_| {
                calls += 1;
                if calls == 2 {
                    assert_eq!(
                        scope.usage().payload_bytes + scope.usage().tracking_bytes,
                        index_bytes as u64
                    );
                }
                Ok(())
            })
        })
        .unwrap();
    assert_eq!(calls, 2);
    assert!(hits.iter().any(|h| h.target == "indexed"));
    drop(hits);
    assert_eq!(scope.usage().allocations, 0);
}

#[test]
fn aligned_engine_text_basis_preserves_visual_pick_region_and_hover() {
    use datum_gui_protocol::PointNm;
    use datum_gui_viewport::selection_region::SelectionRegion;
    use eda_engine::text::{
        TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign,
    };
    let source = eda_engine::board::BoardText {
        uuid: "50000000-0000-4000-8000-000000001080".parse().unwrap(),
        text: "I".into(),
        position: eda_engine::ir::geometry::Point { x: 256000000, y: 0 },
        rotation: 0,
        layer: 214,
        render_intent: TextRenderIntent::Annotation,
        family: TextFamilyId("ibm_plex_sans_condensed".into()),
        family_source: TextFamilySource::Explicit,
        style: TextStyleId::default(),
        height_nm: 640000,
        stroke_width_nm: 120000,
        h_align: TextHAlign::Left,
        v_align: TextVAlign::Center,
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1000000,
        italic: false,
        bold: false,
        style_class: None,
    };
    let mesh = eda_engine::text::layout_text_mesh_from_board_text(&source).unwrap();
    let b = mesh.batch.block_bbox_em_nm;
    let t = mesh.batch.world_transform;
    let mut layout = geometry("aligned-text", 0);
    layout.world_transform_nm = Some(Affine2DFixedPrimitive {
        m11_ppm: t.m11_ppm,
        m12_ppm: t.m12_ppm,
        m21_ppm: t.m21_ppm,
        m22_ppm: t.m22_ppm,
        tx_nm: t.tx_nm,
        ty_nm: t.ty_nm,
    });
    layout.block_bbox_em_nm = Some(MeshRectEmPrimitive {
        min_x_em_nm: b.min_x_em_nm,
        min_y_em_nm: b.min_y_em_nm,
        max_x_em_nm: b.max_x_em_nm,
        max_y_em_nm: b.max_y_em_nm,
    });
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.ui.filters.layer_visibility.insert("L1".into(), true);
    state.scene.board_texts = vec![text("aligned-text")];
    state.scene.board_text_geometries = vec![layout.clone()];
    let hits = super::super::build(&state.scene, &state, |_| Ok(())).unwrap();
    let hit = hits
        .iter()
        .find(|hit| hit.target == "aligned-text")
        .unwrap();
    // Original visible target and actual failed integer-pixel inverse (rounded
    // here to nm) both sit in the aligned layout, above the former raw box.
    for point in [
        PointNm { x: 256081920, y: 0 },
        PointNm {
            x: 256120981,
            y: 40711,
        },
    ] {
        assert!(hit.shape.contains(point));
    }
    let raw_only = PointNm {
        x: 256180000,
        y: -400000,
    };
    assert!(!hit.shape.contains(raw_only));
    let expected = datum_gui_protocol::RectNm {
        min_x: 256000000,
        max_x: 256163840,
        min_y: -223360,
        max_y: 223360,
    };
    assert_eq!(hit.shape.bounds(), Some(expected));
    assert_eq!(
        crate::interaction_overlay::schematic_hover_bounds(&state.scene, "aligned-text"),
        Some(expected)
    );
    let corners = datum_gui_protocol::selection_geometry_catalog::layout_polygon(&layout).unwrap();
    let aligned_region = SelectionRegion::new(&[
        PointNm {
            x: 255999999,
            y: -223361,
        },
        PointNm {
            x: 256163841,
            y: -223361,
        },
        PointNm {
            x: 256163841,
            y: 223361,
        },
        PointNm {
            x: 255999999,
            y: 223361,
        },
    ])
    .unwrap();
    assert!(aligned_region.encloses(&corners).unwrap());
    let former_only_region = SelectionRegion::new(&[
        PointNm {
            x: 256164000,
            y: -446720,
        },
        PointNm {
            x: 256197760,
            y: -446720,
        },
        PointNm {
            x: 256197760,
            y: -223361,
        },
        PointNm {
            x: 256164000,
            y: -223361,
        },
    ])
    .unwrap();
    assert!(!former_only_region.majority_layout(&corners).unwrap());
}
