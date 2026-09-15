use super::*;

#[test]
fn board_text_mesh_path_bypasses_legacy_fill_fragments() {
    let handle = GlyphMeshHandlePrimitive {
        font_id: 1,
        glyph_id: 42,
        tolerance_class: 1,
        epoch: 0,
    };
    let asset = GlyphMeshAssetPrimitive {
        handle,
        vertices: vec![
            datum_gui_protocol::MeshVertexEmPrimitive {
                x_em_nm: 0,
                y_em_nm: 0,
            },
            datum_gui_protocol::MeshVertexEmPrimitive {
                x_em_nm: 1_000_000,
                y_em_nm: 0,
            },
            datum_gui_protocol::MeshVertexEmPrimitive {
                x_em_nm: 0,
                y_em_nm: 1_000_000,
            },
        ],
        indices: vec![0, 1, 2],
        bbox_em_nm: datum_gui_protocol::MeshRectEmPrimitive {
            min_x_em_nm: 0,
            min_y_em_nm: 0,
            max_x_em_nm: 1_000_000,
            max_y_em_nm: 1_000_000,
        },
    };
    let text_geometry = BoardTextGeometryPrimitive {
        object_id: "board-text:test".to_string(),
        object_kind: "board_text".to_string(),
        text_uuid: "test".to_string(),
        layer_id: "L37".to_string(),
        world_transform_nm: Some(Affine2DFixedPrimitive {
            m11_ppm: 1_000_000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 1_000_000,
            tx_nm: 10,
            ty_nm: 20,
        }),
        block_bbox_em_nm: None,
        glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
            glyph_handle: handle,
            origin_em_nm_x: 0,
            origin_em_nm_y: 0,
        }],
        fills: vec![datum_gui_protocol::BoardTextFillPrimitive {
            outer: vec![
                PointNm { x: 0, y: 0 },
                PointNm { x: 10, y: 0 },
                PointNm { x: 10, y: 10 },
                PointNm { x: 0, y: 10 },
            ],
            holes: Vec::new(),
        }],
        strokes: Vec::new(),
    };
    let assets = BTreeMap::from([(handle, &asset)]);
    let projection = Projection::new(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        &datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 100,
            max_y: 100,
        },
        CameraState {
            center_x_nm: 50.0,
            center_y_nm: 50.0,
            zoom: 1.0,
        },
    );
    let mut out = Vec::new();

    push_board_text_geometry_world(
        &mut out,
        &text_geometry,
        &assets,
        [1.0, 1.0, 1.0],
        &projection,
    );

    assert_eq!(
        out.len(),
        1,
        "mesh-backed text must render from glyph mesh triangles, not legacy fill fragments"
    );
    assert_eq!(
        out[0].points,
        [
            (10.0, 20.0),
            (1_000_010.0, 20.0),
            (10.0, 1_000_020.0),
            (10.0, 1_000_020.0),
        ]
    );
}

#[test]
fn board_text_mesh_missing_asset_does_not_fall_back_to_legacy_fragments() {
    let handle = GlyphMeshHandlePrimitive {
        font_id: 1,
        glyph_id: 42,
        tolerance_class: 1,
        epoch: 0,
    };
    let text_geometry = BoardTextGeometryPrimitive {
        object_id: "board-text:test".to_string(),
        object_kind: "board_text".to_string(),
        text_uuid: "test".to_string(),
        layer_id: "L37".to_string(),
        world_transform_nm: Some(Affine2DFixedPrimitive {
            m11_ppm: 1_000_000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 1_000_000,
            tx_nm: 10,
            ty_nm: 20,
        }),
        block_bbox_em_nm: None,
        glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
            glyph_handle: handle,
            origin_em_nm_x: 0,
            origin_em_nm_y: 0,
        }],
        fills: vec![datum_gui_protocol::BoardTextFillPrimitive {
            outer: vec![
                PointNm { x: 0, y: 0 },
                PointNm { x: 10, y: 0 },
                PointNm { x: 10, y: 10 },
                PointNm { x: 0, y: 10 },
            ],
            holes: Vec::new(),
        }],
        strokes: Vec::new(),
    };
    let projection = Projection::new(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        &datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 100,
            max_y: 100,
        },
        CameraState {
            center_x_nm: 50.0,
            center_y_nm: 50.0,
            zoom: 1.0,
        },
    );
    let mut out = Vec::new();
    let assets = BTreeMap::new();

    push_board_text_geometry_world(
        &mut out,
        &text_geometry,
        &assets,
        [1.0, 1.0, 1.0],
        &projection,
    );

    assert!(
        out.is_empty(),
        "malformed mesh-backed text should skip the bad glyph, not render stale legacy fragments"
    );
}

#[test]
fn board_text_mesh_bad_indices_skip_bad_triangles_without_panic() {
    let handle = GlyphMeshHandlePrimitive {
        font_id: 1,
        glyph_id: 42,
        tolerance_class: 1,
        epoch: 0,
    };
    let asset = GlyphMeshAssetPrimitive {
        handle,
        vertices: vec![
            datum_gui_protocol::MeshVertexEmPrimitive {
                x_em_nm: 0,
                y_em_nm: 0,
            },
            datum_gui_protocol::MeshVertexEmPrimitive {
                x_em_nm: 1_000_000,
                y_em_nm: 0,
            },
        ],
        indices: vec![0, 1, 2],
        bbox_em_nm: datum_gui_protocol::MeshRectEmPrimitive {
            min_x_em_nm: 0,
            min_y_em_nm: 0,
            max_x_em_nm: 1_000_000,
            max_y_em_nm: 0,
        },
    };
    let text_geometry = BoardTextGeometryPrimitive {
        object_id: "board-text:test".to_string(),
        object_kind: "board_text".to_string(),
        text_uuid: "test".to_string(),
        layer_id: "L37".to_string(),
        world_transform_nm: Some(Affine2DFixedPrimitive {
            m11_ppm: 1_000_000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 1_000_000,
            tx_nm: 10,
            ty_nm: 20,
        }),
        block_bbox_em_nm: None,
        glyphs: vec![datum_gui_protocol::TextGlyphInstancePrimitive {
            glyph_handle: handle,
            origin_em_nm_x: 0,
            origin_em_nm_y: 0,
        }],
        fills: Vec::new(),
        strokes: Vec::new(),
    };
    let assets = BTreeMap::from([(handle, &asset)]);
    let projection = Projection::new(
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        &datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 100,
            max_y: 100,
        },
        CameraState {
            center_x_nm: 50.0,
            center_y_nm: 50.0,
            zoom: 1.0,
        },
    );
    let mut out = Vec::new();

    push_board_text_geometry_world(
        &mut out,
        &text_geometry,
        &assets,
        [1.0, 1.0, 1.0],
        &projection,
    );

    assert!(
        out.is_empty(),
        "bad mesh indices should skip only the invalid triangle"
    );
}
