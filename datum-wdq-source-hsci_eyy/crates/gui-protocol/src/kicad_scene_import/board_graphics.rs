pub(super) fn kicad_interpolate_arc_world(
    start: PointNm,
    mid: PointNm,
    end: PointNm,
) -> Vec<PointNm> {
    let Some((center, radius, start_tenths, end_tenths)) =
        kicad_arc_from_three_points(&start, &mid, &end)
    else {
        return vec![start, mid, end];
    };
    let mut sweep_tenths = end_tenths - start_tenths;
    // Pick the sweep direction that includes the mid-angle.
    let mid_tenths = (((mid.y as f64 - center.y as f64)
        .atan2(mid.x as f64 - center.x as f64)
        .to_degrees()
        * 10.0)
        .round() as i32)
        .rem_euclid(3600);
    let includes_mid = |s_t: i32, sweep: i32, m_t: i32| -> bool {
        let mut rel = (m_t - s_t).rem_euclid(3600);
        if sweep >= 0 {
            rel <= sweep
        } else {
            rel -= 3600;
            rel >= sweep
        }
    };
    if !includes_mid(start_tenths, sweep_tenths, mid_tenths) {
        if sweep_tenths > 0 {
            sweep_tenths -= 3600;
        } else {
            sweep_tenths += 3600;
        }
    }
    const SEGMENT_ANGLE_TENTHS: i32 = 100; // ~10 deg → ≈36 segments for a full circle
    let segment_count = (sweep_tenths.abs() / SEGMENT_ANGLE_TENTHS).max(1);
    let mut out: Vec<PointNm> = (0..=segment_count)
        .map(|idx| {
            let t = start_tenths + sweep_tenths * idx / segment_count;
            let rad = (f64::from(t) / 10.0).to_radians();
            PointNm {
                x: (center.x as f64 + radius as f64 * rad.cos()).round() as i64,
                y: (center.y as f64 + radius as f64 * rad.sin()).round() as i64,
            }
        })
        .collect();
    // Force first/last to exact source endpoints so chaining against adjacent
    // contributors remains precise.
    if let Some(first) = out.first_mut() {
        *first = start;
    }
    if let Some(last) = out.last_mut() {
        *last = end;
    }
    out
}

/// Extract imported Edge.Cuts contributors as authored board-level graphics.
/// One walk produces primitives for top-level `gr_line` / `gr_arc` and
/// footprint-embedded `fp_line` / `fp_arc` on Edge.Cuts, under the footprint
/// `(at x y rot)` transform where applicable. See M7-SCN-007 brief.
///
/// `edge_cuts_layer_key` is the scene-level layer-id key under which the
/// Edge.Cuts layer is indexed (the `"L{n}"` form used by `scene.layers` and
/// the layer-visibility map). This must match the rest of the scene's
/// layer-id convention so visibility toggles actually gate these primitives.
pub(super) fn extract_kicad_board_graphics(
    contents: &str,
    board_uuid: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> Vec<BoardGraphicPrimitive> {
    let mut out: Vec<BoardGraphicPrimitive> = Vec::new();
    let mut ordinal: usize = 0;

    let mut stable_id = |kind: &str, src_uuid: &str| -> (String, String) {
        let src = if src_uuid.is_empty() {
            format!("{board_uuid}:edge-cuts:{kind}:{ordinal}")
        } else {
            src_uuid.to_string()
        };
        let oid = format!("board-graphic:{src}");
        ordinal += 1;
        (oid, src)
    };

    // Top-level contributors (no transform).
    for block in kicad_nested_blocks(contents, "gr_line") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(start), Some(end)) = (
            kicad_parse_xy_anywhere_block(&block, "start"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("line", &uuid);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: "polyline".to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path: vec![start, end],
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_arc") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(start), Some(mid), Some(end)) = (
            kicad_parse_xy_anywhere_block(&block, "start"),
            kicad_parse_xy_anywhere_block(&block, "mid"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("arc", &uuid);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: "polyline".to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path: kicad_interpolate_arc_world(start, mid, end),
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_poly") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let mut path = kicad_parse_xy_points(&block);
        if path.len() < 2 {
            continue;
        }
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("poly", &uuid);
        let filled = block.contains("(fill yes)");
        if !filled
            && path
                .first()
                .zip(path.last())
                .is_some_and(|(first, last)| first != last)
            && let Some(first) = path.first().copied()
        {
            path.push(first);
        }
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: if filled { "polygon" } else { "polyline" }.to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path,
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    for block in kicad_nested_blocks(contents, "gr_circle") {
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let Some(layer_key) = kicad_board_graphic_layer_key(&layer_name, layer_table) else {
            continue;
        };
        let (Some(center), Some(end_pt)) = (
            kicad_parse_xy_anywhere_block(&block, "center"),
            kicad_parse_xy_anywhere_block(&block, "end"),
        ) else {
            continue;
        };
        let dx = end_pt.x - center.x;
        let dy = end_pt.y - center.y;
        let radius = ((dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt()).round() as i64;
        let width = kicad_parse_width_nm(&block);
        let uuid = kicad_parse_uuid(&block).unwrap_or_default();
        let (object_id, source) = stable_id("circle", &uuid);
        let filled = block.contains("(fill yes)");
        let path = approximate_world_circle_path(center, radius);
        out.push(BoardGraphicPrimitive {
            object_id,
            object_kind: "board_graphic".to_string(),
            primitive_kind: if filled { "polygon" } else { "polyline" }.to_string(),
            source_object_uuid: source,
            layer_id: layer_key,
            path,
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    out
}

pub(super) fn extract_kicad_board_texts(
    contents: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> (
    Vec<BoardTextPrimitive>,
    Vec<BoardTextGeometryPrimitive>,
    Vec<GlyphMeshAssetPrimitive>,
) {
    let mut board_texts = Vec::new();
    let mut board_text_geometries = Vec::new();
    let mut glyph_mesh_assets_by_handle = BTreeMap::new();
    let mut text_trace = KicadImportTextTrace::default();
    for (index, block) in kicad_nested_blocks(contents, "gr_text")
        .into_iter()
        .enumerate()
    {
        text_trace.gr_text_total += 1;
        if kicad_block_hidden(&block) {
            text_trace.gr_text_hidden_skipped += 1;
            continue;
        }
        let Some(layer_name) = kicad_parse_layer_anywhere(&block) else {
            continue;
        };
        let layer = kicad_resolve_layer_id(&layer_name, layer_table);
        let Some((position, rotation)) = kicad_parse_at(&block) else {
            continue;
        };
        let Some(first_line) = block.lines().next().map(str::trim) else {
            continue;
        };
        let Some(start) = first_line.find('"') else {
            continue;
        };
        let rest = &first_line[start + 1..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let text = rest[..end].to_string();
        let uuid = kicad_parse_uuid(&block)
            .and_then(|value| uuid::Uuid::parse_str(&value).ok())
            .unwrap_or_else(|| {
                kicad_import_text_uuid(
                    "gr_text",
                    &format!("{index}/{text}/{}/{}/{}", position.x, position.y, layer),
                )
            });
        let board_text = kicad_board_text(
            uuid,
            text,
            layer,
            position,
            kicad_text_rotation_degrees(rotation),
            kicad_parse_font_height_nm(&block),
            kicad_parse_font_thickness_nm(&block),
            kicad_parse_text_justify(&block),
            Some("imported_kicad_gr_text".to_string()),
        );
        push_board_text_scene_primitives(
            &board_text,
            &mut board_texts,
            &mut board_text_geometries,
            &mut glyph_mesh_assets_by_handle,
        );
        text_trace.gr_text_imported += 1;
        text_trace.record_import("gr_text", &layer_name, layer, &board_text.text);
    }

    text_trace.emit(
        "board",
        board_texts.len(),
        board_text_geometries.len(),
        glyph_mesh_assets_by_handle.len(),
    );

    (
        board_texts,
        board_text_geometries,
        glyph_mesh_assets_by_handle.into_values().collect(),
    )
}

pub(super) fn kicad_board_graphic_layer_key(
    layer_name: &str,
    layer_table: &std::collections::HashMap<String, i32>,
) -> Option<String> {
    match layer_name {
        "F.SilkS" | "B.SilkS" | "F.Fab" | "B.Fab" | "F.CrtYd" | "B.CrtYd" | "Edge.Cuts" => {
            Some(layer_id(kicad_resolve_layer_id(layer_name, layer_table)))
        }
        _ => None,
    }
}

pub(super) fn approximate_world_circle_path(center: PointNm, radius: i64) -> Vec<PointNm> {
    let segments = 32usize;
    (0..=segments)
        .map(|i| {
            let angle = std::f64::consts::TAU * (i as f64) / (segments as f64);
            PointNm {
                x: center.x + (radius as f64 * angle.cos()).round() as i64,
                y: center.y + (radius as f64 * angle.sin()).round() as i64,
            }
        })
        .collect()
}

/// Native-project parity helper for M7-SCN-007 Option B.
///
/// Native board JSON persists the assembled board outline polygon but does not
/// currently preserve the original per-contributor Edge.Cuts primitives or
/// their source identities. For native projects, derive stable board-scoped
/// Edge.Cuts line primitives from the persisted outline so authored-layer
/// visibility, stacking, and picking behave consistently with imported boards.
pub(super) fn outline_board_graphics_from_outline(
    outline: &OutlinePayload,
    board_uuid: &str,
    edge_cuts_layer_key: &str,
) -> Vec<BoardGraphicPrimitive> {
    let mut vertices = outline.vertices.clone();
    if vertices.len() < 2 {
        return Vec::new();
    }
    if outline.closed && vertices.first() != vertices.last()
        && let Some(first) = vertices.first().copied() {
            vertices.push(first);
        }
    vertices
        .windows(2)
        .enumerate()
        .map(|(index, segment)| {
            let source = format!("{board_uuid}:outline-segment:{index}");
            BoardGraphicPrimitive {
                object_id: format!("board-graphic:{source}"),
                object_kind: "board_graphic".to_string(),
                primitive_kind: "line".to_string(),
                source_object_uuid: source,
                layer_id: edge_cuts_layer_key.to_string(),
                path: vec![segment[0], segment[1]],
                holes: Vec::new(),
                width_nm: None,
            }
        })
        .collect()
}

pub(super) fn unrouted_primitives_from_airwires(
    airwires: &[eda_engine::board::Airwire],
) -> Vec<UnroutedPrimitive> {
    airwires
        .iter()
        .map(|airwire| {
            let source = format!(
                "{}:{}:{}:{}:{}",
                airwire.net,
                airwire.from.component,
                airwire.from.pin,
                airwire.to.component,
                airwire.to.pin
            );
            UnroutedPrimitive {
                object_id: format!("unrouted:{source}"),
                object_kind: "unrouted".to_string(),
                source_object_uuid: source,
                net_uuid: airwire.net.to_string(),
                from_component: airwire.from.component.clone(),
                from_pin: airwire.from.pin.clone(),
                to_component: airwire.to.component.clone(),
                to_pin: airwire.to.pin.clone(),
                path: vec![
                    PointNm {
                        x: airwire.from_position.x,
                        y: airwire.from_position.y,
                    },
                    PointNm {
                        x: airwire.to_position.x,
                        y: airwire.to_position.y,
                    },
                ],
            }
        })
        .collect()
}

pub(super) fn net_display_from_imported_board(
    board: &eda_engine::board::Board,
) -> Vec<NetDisplayEntry> {
    let mut nets: Vec<_> = board.nets.values().collect();
    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    nets.into_iter()
        .map(|net| NetDisplayEntry {
            net_uuid: net.uuid.to_string(),
            net_name: net.name.clone(),
            airwire_color_rgb: deterministic_airwire_color(net.uuid.as_bytes()),
        })
        .collect()
}

/// Block-level variant of `kicad_parse_xy_anywhere`: scan every line of the
/// block to locate the first `(form x y ...)` occurrence.
pub(super) fn kicad_parse_xy_anywhere_block(block: &str, form: &str) -> Option<PointNm> {
    block
        .lines()
        .find_map(|line| kicad_parse_xy_anywhere(line.trim_start(), form))
}

/// Extract the text content from an `fp_text` first line.
/// Format: `(fp_text TYPE "text content" (at ...`
pub(super) fn kicad_extract_fp_text_content(first_line: &str) -> Option<String> {
    let trimmed = first_line.trim();
    if !trimmed.starts_with("(fp_text ") {
        return None;
    }
    let after = &trimmed["(fp_text ".len()..];
    // Skip the type token (reference, value, user).
    let rest = after.trim_start();
    let rest = if let Some(stripped) = rest.strip_prefix('"') {
        // Type is quoted (rare).
        let end = stripped.find('"')?;
        rest[end + 2..].trim_start()
    } else {
        let end = rest.find(|c: char| c.is_whitespace())?;
        rest[end..].trim_start()
    };
    // Now the text content should be quoted.
    if !rest.starts_with('"') {
        return None;
    }
    let inner = &rest[1..];
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}
