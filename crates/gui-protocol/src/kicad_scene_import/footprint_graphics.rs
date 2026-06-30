pub(super) fn extract_kicad_footprint_graphics(
    contents: &str,
    components: &[BoardComponentPayload],
    layer_table: &std::collections::HashMap<String, i32>,
) -> (
    Vec<ComponentGraphicPrimitive>,
    Vec<ComponentTextPrimitive>,
    Vec<BoardTextPrimitive>,
    Vec<BoardTextGeometryPrimitive>,
    Vec<GlyphMeshAssetPrimitive>,
) {
    let mut all_graphics = Vec::new();
    let mut board_texts = Vec::new();
    let mut board_text_geometries = Vec::new();
    let mut glyph_mesh_assets_by_handle = BTreeMap::new();
    let mut text_trace = KicadImportTextTrace::default();

    // Build a lookup from UUID string to component.
    let comp_by_uuid: std::collections::HashMap<&str, &BoardComponentPayload> =
        components.iter().map(|c| (c.uuid.as_str(), c)).collect();

    for fp_block in kicad_nested_blocks(contents, "footprint") {
        // Find the footprint UUID and match to a known component.
        let fp_uuid = match kicad_parse_uuid(&fp_block) {
            Some(u) => u,
            None => continue,
        };
        let component = match comp_by_uuid.get(fp_uuid.as_str()) {
            Some(c) => *c,
            None => continue,
        };

        let mut graphic_index = 0usize;
        let mut text_index = 0usize;
        let fp_blocks = kicad_nested_blocks_by_form(
            &fp_block,
            &[
                "fp_line",
                "fp_rect",
                "fp_circle",
                "fp_arc",
                "fp_poly",
                "fp_text",
                "property",
            ],
        );

        // --- fp_line ---
        for block in fp_blocks.get("fp_line").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let start = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let end = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-line:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: false,
                path: vec![
                    transform_component_local_point(component, start),
                    transform_component_local_point(component, end),
                ],
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_rect ---
        for block in fp_blocks.get("fp_rect").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let s = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let e = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let min_x = s.x.min(e.x);
            let min_y = s.y.min(e.y);
            let max_x = s.x.max(e.x);
            let max_y = s.y.max(e.y);
            let corners = [
                PointNm { x: min_x, y: min_y },
                PointNm { x: max_x, y: min_y },
                PointNm { x: max_x, y: max_y },
                PointNm { x: min_x, y: max_y },
                PointNm { x: min_x, y: min_y },
            ];
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-rect:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: corners
                    .iter()
                    .map(|p| transform_component_local_point(component, *p))
                    .collect(),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_circle ---
        for block in fp_blocks.get("fp_circle").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let center = match kicad_parse_xy_anywhere(&block, "center") {
                Some(p) => p,
                None => continue,
            };
            let end_pt = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let dx = end_pt.x - center.x;
            let dy = end_pt.y - center.y;
            let radius = ((dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt()).round() as i64;
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-circle:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: approximate_circle_path(component, center, radius),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_arc ---
        for block in fp_blocks.get("fp_arc").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let start = match kicad_parse_xy_anywhere(&block, "start") {
                Some(p) => p,
                None => continue,
            };
            let mid = match kicad_parse_xy_anywhere(&block, "mid") {
                Some(p) => p,
                None => continue,
            };
            let end = match kicad_parse_xy_anywhere(&block, "end") {
                Some(p) => p,
                None => continue,
            };
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let path = if let Some((center, radius, start_angle, end_angle)) =
                kicad_arc_from_three_points(&start, &mid, &end)
            {
                approximate_arc_path(component, center, radius, start_angle, end_angle)
            } else {
                // Collinear fallback — just draw start→mid→end.
                vec![
                    transform_component_local_point(component, start),
                    transform_component_local_point(component, mid),
                    transform_component_local_point(component, end),
                ]
            };
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-arc:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polyline".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: false,
                path,
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_poly ---
        for block in fp_blocks.get("fp_poly").into_iter().flatten() {
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let vertices = kicad_parse_xy_points(&block);
            if vertices.is_empty() {
                continue;
            }
            let width = kicad_parse_width_nm(&block);
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            all_graphics.push(ComponentGraphicPrimitive {
                graphic_id: format!("component-graphic:{}:kicad-poly:{graphic_index}", fp_uuid),
                component_uuid: fp_uuid.clone(),
                layer_id: Some(layer_id(lid)),
                primitive_kind: "polygon".to_string(),
                render_role: role.to_string(),
                width_nm: Some(width),
                closed: true,
                path: vertices
                    .into_iter()
                    .map(|p| transform_component_local_point(component, p))
                    .collect(),
                holes: Vec::new(),
            });
            graphic_index += 1;
        }

        // --- fp_text (literal text only, skip ${REFERENCE} and ${VALUE}) ---
        for block in fp_blocks.get("fp_text").into_iter().flatten() {
            text_trace.fp_text_total += 1;
            let first_line = match block.lines().next() {
                Some(l) => l.trim(),
                None => continue,
            };
            // Extract the text content — it is the second quoted token.
            // Format: (fp_text TYPE "text" (at ...) ...)
            let text = match kicad_extract_fp_text_content(first_line) {
                Some(t) => t,
                None => continue,
            };
            // Skip template references handled by the label system.
            if text.contains("${REFERENCE}")
                || text.contains("${VALUE}")
                || text == "%R"
                || text == "%V"
            {
                text_trace.fp_text_template_skipped += 1;
                continue;
            }
            if kicad_block_hidden(&block) {
                text_trace.fp_text_hidden_skipped += 1;
                continue;
            }
            let layer_name = match kicad_parse_layer_anywhere(&block) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let (local_pos, local_rot) = match kicad_parse_at(&block) {
                Some(v) => v,
                None => continue,
            };
            let lid = kicad_resolve_layer_id(&layer_name, layer_table);
            let height = kicad_parse_font_height_nm(&block);
            let stroke_width_nm = kicad_parse_font_thickness_nm(&block);
            let board_pos = transform_component_local_point(component, local_pos);
            let board_rot = kicad_text_rotation_degrees(component.rotation + local_rot);
            let mut justify = kicad_parse_text_justify(&block);
            justify.keep_upright = true;
            let source_uuid = kicad_parse_uuid(&block).unwrap_or_else(|| {
                kicad_import_text_uuid("fp_text", &format!("{fp_uuid}/{text_index}/{text}/{lid}"))
                    .to_string()
            });
            let Ok(text_uuid) = uuid::Uuid::parse_str(&source_uuid) else {
                continue;
            };
            let board_text = kicad_board_text(
                text_uuid,
                text,
                lid,
                board_pos,
                board_rot,
                height,
                stroke_width_nm,
                justify,
                Some(format!("imported_kicad_fp_text:{fp_uuid}:{role}")),
            );
            push_board_text_scene_primitives(
                &board_text,
                &mut board_texts,
                &mut board_text_geometries,
                &mut glyph_mesh_assets_by_handle,
            );
            text_trace.fp_text_imported += 1;
            text_trace.record_import("fp_text", &layer_name, lid, &board_text.text);
            text_index += 1;
        }

        // --- property blocks (Reference/Value on silkscreen/fab layers) ---
        for prop_section in fp_blocks.get("property").into_iter().flatten() {
            text_trace.property_total += 1;
            let first_line = match prop_section.lines().next() {
                Some(line) => line.trim(),
                None => continue,
            };
            let mut quoted = Vec::new();
            let mut rest = first_line;
            while let Some(start) = rest.find('"') {
                let after = &rest[start + 1..];
                if let Some(end) = after.find('"') {
                    quoted.push(after[..end].to_string());
                    rest = &after[end + 1..];
                } else {
                    break;
                }
            }
            if quoted.len() < 2 {
                continue;
            }
            let key = &quoted[0];
            if key != "Reference" && key != "Value" {
                text_trace.property_metadata_skipped += 1;
                continue;
            }
            let text = quoted[1].clone();
            if text.is_empty() || text.starts_with('~') {
                text_trace.property_empty_skipped += 1;
                continue;
            }
            if kicad_block_hidden(&prop_section) {
                text_trace.property_hidden_skipped += 1;
                continue;
            }
            let layer_name = match kicad_parse_layer_anywhere(&prop_section) {
                Some(n) => n,
                None => continue,
            };
            let role = match kicad_render_role(&layer_name) {
                Some(r) => r,
                None => continue,
            };
            let layer_id = kicad_resolve_layer_id(&layer_name, layer_table);
            let (local_pos, local_rot) = match kicad_parse_at(&prop_section) {
                Some(v) => v,
                None => continue,
            };
            let board_pos = transform_component_local_point(component, local_pos);
            let height_nm = kicad_parse_font_height_nm(&prop_section);
            let stroke_width_nm = kicad_parse_font_thickness_nm(&prop_section);
            let board_rot = kicad_text_rotation_degrees(component.rotation + local_rot);
            let mut justify = kicad_parse_text_justify(&prop_section);
            justify.keep_upright = true;
            let text_uuid = kicad_import_text_uuid(
                "property_text",
                &format!(
                    "{}/{}/{text}/{layer_id}",
                    component.uuid,
                    key.to_lowercase()
                ),
            );
            let board_text = kicad_board_text(
                text_uuid,
                text,
                layer_id,
                board_pos,
                board_rot,
                height_nm,
                stroke_width_nm,
                justify,
                Some(format!(
                    "imported_kicad_property_text:{}:{}:{role}",
                    component.uuid,
                    key.to_lowercase()
                )),
            );
            push_board_text_scene_primitives(
                &board_text,
                &mut board_texts,
                &mut board_text_geometries,
                &mut glyph_mesh_assets_by_handle,
            );
            if key == "Reference" {
                text_trace.property_reference_imported += 1;
                text_trace.record_import(
                    "property_reference",
                    &layer_name,
                    layer_id,
                    &board_text.text,
                );
            } else {
                text_trace.property_value_imported += 1;
                text_trace.record_import("property_value", &layer_name, layer_id, &board_text.text);
            }
        }
    }

    text_trace.emit(
        "footprints",
        board_texts.len(),
        board_text_geometries.len(),
        glyph_mesh_assets_by_handle.len(),
    );

    (
        all_graphics,
        Vec::new(),
        board_texts,
        board_text_geometries,
        glyph_mesh_assets_by_handle.into_values().collect(),
    )
}
