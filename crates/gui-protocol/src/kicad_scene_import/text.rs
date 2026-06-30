fn board_text_primitive(text: &BoardText) -> BoardTextPrimitive {
    BoardTextPrimitive {
        object_id: format!("board-text:{}", text.uuid),
        object_kind: "board_text".to_string(),
        text_uuid: text.uuid.to_string(),
        text: text.text.clone(),
        layer_id: layer_id(text.layer),
        position: PointNm {
            x: text.position.x,
            y: text.position.y,
        },
        rotation_degrees: text.rotation,
        height_nm: text.height_nm,
        stroke_width_nm: text.stroke_width_nm,
        render_intent: render_intent_to_string(text.render_intent).to_string(),
        family: text.family.0.clone(),
        style: text.style.0.clone(),
        style_class: text.style_class.clone(),
        h_align: h_align_to_string(text.h_align).to_string(),
        v_align: v_align_to_string(text.v_align).to_string(),
        mirrored: text.mirrored,
        keep_upright: text.keep_upright,
        line_spacing_ratio_ppm: text.line_spacing_ratio_ppm,
        bold: text.bold,
        italic: text.italic,
    }
}

fn render_intent_to_string(intent: TextRenderIntent) -> &'static str {
    match intent {
        TextRenderIntent::Manufacturing => "manufacturing",
        TextRenderIntent::Annotation => "annotation",
        TextRenderIntent::Branding => "branding",
        TextRenderIntent::Documentation => "documentation",
        TextRenderIntent::UiPreview => "ui_preview",
    }
}

fn h_align_to_string(align: TextHAlign) -> &'static str {
    match align {
        TextHAlign::Left => "left",
        TextHAlign::Center => "center",
        TextHAlign::Right => "right",
    }
}

fn v_align_to_string(align: TextVAlign) -> &'static str {
    match align {
        TextVAlign::Top => "top",
        TextVAlign::Center => "center",
        TextVAlign::Bottom => "bottom",
    }
}

fn board_text_geometry(
    text: &BoardText,
) -> (BoardTextGeometryPrimitive, Vec<GlyphMeshAssetPrimitive>) {
    let mesh_scene = match layout_text_mesh_from_board_text(text) {
        Ok(scene) => Some(scene),
        Err(error) => {
            trace_board_text_geometry_fallback(text, "mesh", &error);
            None
        }
    };
    let glyph_mesh_assets = mesh_scene
        .as_ref()
        .map(|scene| {
            scene
                .glyph_mesh_assets
                .iter()
                .map(glyph_mesh_asset_primitive)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let world_transform_nm = mesh_scene
        .as_ref()
        .map(|scene| affine_2d_fixed_primitive(scene.batch.world_transform));
    let block_bbox_em_nm = mesh_scene
        .as_ref()
        .map(|scene| mesh_rect_em_primitive(scene.batch.block_bbox_em_nm));
    let glyphs = mesh_scene
        .as_ref()
        .map(|scene| {
            scene
                .batch
                .glyphs
                .iter()
                .map(text_glyph_instance_primitive)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut fills = Vec::new();
    let mut strokes = Vec::new();

    // The renderer consumes glyph meshes when available. Legacy fill/stroke
    // fragments are only a fallback; generating both paths doubles import-time
    // text tessellation cost on real KiCad boards.
    if glyphs.is_empty() {
        match layout_text_geometry(&text.text, &TextAttributes::from_board_text(text)) {
            Ok(primitives) => {
                for primitive in primitives {
                    match primitive {
                        TextGeometryPrimitive::Stroke(stroke) => {
                            strokes.push(BoardTextStrokePrimitive {
                                from: PointNm {
                                    x: stroke.from.x,
                                    y: stroke.from.y,
                                },
                                to: PointNm {
                                    x: stroke.to.x,
                                    y: stroke.to.y,
                                },
                                width_nm: stroke.width_nm,
                            });
                        }
                        TextGeometryPrimitive::FilledPolygon(polygon) => {
                            fills.push(BoardTextFillPrimitive {
                                outer: polygon
                                    .outer
                                    .into_iter()
                                    .map(|point| PointNm {
                                        x: point.x,
                                        y: point.y,
                                    })
                                    .collect(),
                                holes: polygon
                                    .holes
                                    .into_iter()
                                    .map(|ring| {
                                        ring.into_iter()
                                            .map(|point| PointNm {
                                                x: point.x,
                                                y: point.y,
                                            })
                                            .collect()
                                    })
                                    .collect(),
                            });
                        }
                    }
                }
            }
            Err(error) => trace_board_text_geometry_fallback(text, "legacy", &error),
        }
    }

    (
        BoardTextGeometryPrimitive {
            object_id: format!("board-text:{}", text.uuid),
            object_kind: "board_text_geometry".to_string(),
            text_uuid: text.uuid.to_string(),
            layer_id: layer_id(text.layer),
            world_transform_nm,
            block_bbox_em_nm,
            glyphs,
            fills,
            strokes,
        },
        glyph_mesh_assets,
    )
}

fn trace_board_text_geometry_fallback(
    text: &BoardText,
    stage: &str,
    error: &dyn std::fmt::Display,
) {
    if !kicad_import_text_trace_enabled() {
        return;
    }
    eprintln!(
        "[datum-import-text] board_text_geometry_fallback stage={stage} text_uuid={} layer={} family={} intent={} chars={} error={error}",
        text.uuid,
        layer_id(text.layer),
        text.family.0,
        render_intent_to_string(text.render_intent),
        text.text.chars().count(),
    );
}

pub(super) fn push_board_text_scene_primitives(
    board_text: &BoardText,
    primitives: &mut Vec<BoardTextPrimitive>,
    geometries: &mut Vec<BoardTextGeometryPrimitive>,
    mesh_assets_by_handle: &mut BTreeMap<GlyphMeshHandlePrimitive, GlyphMeshAssetPrimitive>,
) {
    primitives.push(board_text_primitive(board_text));
    let (geometry, mesh_assets) = board_text_geometry(board_text);
    geometries.push(geometry);
    for asset in mesh_assets {
        mesh_assets_by_handle.entry(asset.handle).or_insert(asset);
    }
}

pub(super) fn kicad_import_text_uuid(kind: &str, key: &str) -> uuid::Uuid {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("datum:kicad-import-text:{kind}:{key}").as_bytes(),
    )
}

pub(super) fn kicad_board_text(
    uuid: uuid::Uuid,
    text: String,
    layer: i32,
    position: PointNm,
    rotation_degrees: i32,
    height_nm: i64,
    stroke_width_nm: Option<i64>,
    justify: KicadTextJustify,
    style_class: Option<String>,
) -> BoardText {
    let attrs = kicad_text_attributes(
        position,
        rotation_degrees,
        height_nm,
        stroke_width_nm,
        justify,
    );
    BoardText {
        uuid,
        text,
        position: attrs.position,
        rotation: attrs.rotation_degrees,
        layer,
        render_intent: attrs.render_intent,
        family: attrs.family,
        family_source: attrs.family_source,
        style: attrs.style,
        height_nm: attrs.height_nm,
        stroke_width_nm: attrs.stroke_width_nm,
        h_align: attrs.h_align,
        v_align: attrs.v_align,
        mirrored: attrs.mirrored,
        keep_upright: attrs.keep_upright,
        line_spacing_ratio_ppm: attrs.line_spacing_ratio_ppm,
        italic: attrs.italic,
        bold: attrs.bold,
        style_class,
    }
}

fn merge_glyph_mesh_assets(
    target: &mut Vec<GlyphMeshAssetPrimitive>,
    incoming: Vec<GlyphMeshAssetPrimitive>,
) {
    let mut seen: BTreeSet<GlyphMeshHandlePrimitive> =
        target.iter().map(|asset| asset.handle).collect();
    for asset in incoming {
        if seen.insert(asset.handle) {
            target.push(asset);
        }
    }
}

fn glyph_mesh_asset_primitive(asset: &eda_engine::text::GlyphMeshAsset) -> GlyphMeshAssetPrimitive {
    GlyphMeshAssetPrimitive {
        handle: glyph_mesh_handle_primitive(asset.handle),
        vertices: asset
            .vertices
            .iter()
            .map(|vertex| MeshVertexEmPrimitive {
                x_em_nm: vertex.x_em_nm,
                y_em_nm: vertex.y_em_nm,
            })
            .collect(),
        indices: asset.indices.clone(),
        bbox_em_nm: mesh_rect_em_primitive(asset.bbox_em_nm),
    }
}

fn text_glyph_instance_primitive(
    glyph: &eda_engine::text::TextGlyphInstance,
) -> TextGlyphInstancePrimitive {
    TextGlyphInstancePrimitive {
        glyph_handle: glyph_mesh_handle_primitive(glyph.glyph_handle),
        origin_em_nm_x: glyph.origin_em_nm_x,
        origin_em_nm_y: glyph.origin_em_nm_y,
    }
}

fn glyph_mesh_handle_primitive(
    handle: eda_engine::text::GlyphMeshHandle,
) -> GlyphMeshHandlePrimitive {
    GlyphMeshHandlePrimitive {
        font_id: handle.font_id,
        glyph_id: handle.glyph_id,
        tolerance_class: handle.tolerance_class,
        epoch: handle.epoch,
    }
}

fn mesh_rect_em_primitive(rect: eda_engine::text::MeshRectEm) -> MeshRectEmPrimitive {
    MeshRectEmPrimitive {
        min_x_em_nm: rect.min_x_em_nm,
        min_y_em_nm: rect.min_y_em_nm,
        max_x_em_nm: rect.max_x_em_nm,
        max_y_em_nm: rect.max_y_em_nm,
    }
}

fn affine_2d_fixed_primitive(transform: eda_engine::text::Affine2DFixed) -> Affine2DFixedPrimitive {
    Affine2DFixedPrimitive {
        m11_ppm: transform.m11_ppm,
        m12_ppm: transform.m12_ppm,
        m21_ppm: transform.m21_ppm,
        m22_ppm: transform.m22_ppm,
        tx_nm: transform.tx_nm,
        ty_nm: transform.ty_nm,
    }
}
